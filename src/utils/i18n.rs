use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleFile {
    pub code: String,
    pub display_name: String,
    pub translations: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub code: String,
    pub display_name: String,
}

static I18N_MANAGER: OnceLock<I18nManager> = OnceLock::new();

pub struct I18nManager {
    locales: HashMap<String, LocaleFile>,
    languages: Vec<LanguageInfo>,
}

impl I18nManager {
    pub fn global() -> &'static I18nManager {
        I18N_MANAGER.get_or_init(Self::load_all)
    }

    fn load_all() -> Self {
        let mut locales = HashMap::new();

        // 1. Embedded fallback locales compiled into the binary
        let embedded_locales = [
            include_str!("../../locales/en.json"),
            include_str!("../../locales/fr.json"),
            include_str!("../../locales/es.json"),
        ];

        for content in embedded_locales {
            if let Ok(file) = serde_json::from_str::<LocaleFile>(content) {
                locales.insert(file.code.clone(), file);
            }
        }

        // 2. Scan external locale directories for user-created JSON translations
        let mut locale_dirs: Vec<PathBuf> = vec![
            PathBuf::from("./locales"),
            PathBuf::from("../locales"),
            PathBuf::from("/etc/bo-ring/locales"),
            PathBuf::from("/usr/local/share/bo-ring/locales"),
            PathBuf::from("/usr/share/bo-ring/locales"),
        ];

        // XDG standard path for user custom locales
        if let Ok(home) = std::env::var("HOME") {
            locale_dirs.push(PathBuf::from(&home).join(".config/bo-ring/locales"));
            locale_dirs.push(PathBuf::from(&home).join(".local/share/bo-ring/locales"));
        }

        for dir in &locale_dirs {
            if dir.is_dir() {
                if let Ok(entries) = fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.extension().map_or(false, |ext| ext == "json") {
                            if let Ok(content) = fs::read_to_string(&p) {
                                if let Ok(file) = serde_json::from_str::<LocaleFile>(&content) {
                                    locales.insert(file.code.clone(), file);
                                }
                            }
                        }
                    }
                }
            }
        }

        let mut languages: Vec<LanguageInfo> = locales
            .values()
            .map(|l| LanguageInfo {
                code: l.code.clone(),
                display_name: l.display_name.clone(),
            })
            .collect();
        languages.sort_by(|a, b| a.code.cmp(&b.code));

        Self { locales, languages }
    }

    pub fn available_languages(&self) -> &[LanguageInfo] {
        &self.languages
    }

    pub fn tr<'a>(&'a self, lang_code: &str, key: &'a str) -> &'a str {
        let resolved = resolve_language_code(lang_code);
        if let Some(locale) = self.locales.get(&resolved) {
            if let Some(val) = locale.translations.get(key) {
                return val.as_str();
            }
        }
        // Fallback to English if missing in target language
        if resolved != "en" {
            if let Some(locale) = self.locales.get("en") {
                if let Some(val) = locale.translations.get(key) {
                    return val.as_str();
                }
            }
        }
        // Fallback to the key name itself
        key
    }

    /// Checks whether the provided text matches the translation of `key` in ANY loaded locale (case-insensitive).
    pub fn matches_key_any_locale(&self, key: &str, text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }
        for locale in self.locales.values() {
            if let Some(val) = locale.translations.get(key) {
                if val.eq_ignore_ascii_case(trimmed) {
                    return true;
                }
            }
        }
        false
    }
}

/// Detects the system language (e.g. "fr", "en", "es")
pub fn detect_system_language() -> String {
    let lang_env = std::env::var("LC_ALL")
        .or_else(|_| std::env::var("LC_MESSAGES"))
        .or_else(|_| std::env::var("LANG"))
        .ok();

    if let Some(ref val) = lang_env {
        let code = val.split(&['_', '.'][..]).next().unwrap_or("").to_lowercase();
        if !code.is_empty() {
            return code;
        }
    }

    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if let Ok(out) = std::process::Command::new("sudo")
            .args(["-u", &sudo_user, "locale"])
            .output()
        {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    if line.starts_with("LANG=") || line.starts_with("LC_MESSAGES=") {
                        let val = line.split('=').nth(1).unwrap_or("").trim_matches('"');
                        let code = val.split(&['_', '.'][..]).next().unwrap_or("").to_lowercase();
                        if !code.is_empty() {
                            return code;
                        }
                    }
                }
            }
        }
    }

    "en".to_string()
}

/// Resolves the actual language code: if "system", "default" or empty, uses system language
pub fn resolve_language_code(lang_code: &str) -> String {
    if lang_code == "system" || lang_code == "default" || lang_code.is_empty() {
        let detected = detect_system_language();
        if I18nManager::global().locales.contains_key(&detected) {
            detected
        } else {
            "en".to_string()
        }
    } else {
        lang_code.to_string()
    }
}

pub fn tr<'a>(lang_code: &str, key: &'a str) -> &'a str {
    I18nManager::global().tr(lang_code, key)
}

pub fn matches_key_any_locale(key: &str, text: &str) -> bool {
    I18nManager::global().matches_key_any_locale(key, text)
}

pub fn available_languages() -> &'static [LanguageInfo] {
    I18nManager::global().available_languages()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i18n_translation_and_fallback() {
        let mgr = I18nManager::global();
        assert_eq!(mgr.tr("fr", "save_success"), "✅ Configuration enregistrée dans config.toml !");
        assert_eq!(mgr.tr("en", "save_success"), "✅ Configuration saved to config.toml !");
        assert_eq!(mgr.tr("es", "save_success"), "✅ Configuración guardada en config.toml !");

        // Missing key in non-existent language falls back to English, then to key itself
        assert_eq!(mgr.tr("de", "save_success"), "✅ Configuration saved to config.toml !");
        assert_eq!(mgr.tr("en", "non_existent_key_123"), "non_existent_key_123");
    }

    #[test]
    fn test_matches_key_any_locale() {
        assert!(matches_key_any_locale("desktop", "Bureau"));
        assert!(matches_key_any_locale("desktop", "Desktop"));
        assert!(matches_key_any_locale("desktop", "Escritorio"));
        assert!(matches_key_any_locale("desktop", "  bureau  "));
        assert!(!matches_key_any_locale("desktop", "Firefox"));

        assert!(matches_key_any_locale("detecting", "Détection..."));
        assert!(matches_key_any_locale("detecting", "Detecting..."));
        assert!(matches_key_any_locale("detecting", "Detectando..."));
        assert!(!matches_key_any_locale("detecting", "Ready"));
    }

    #[test]
    fn test_available_languages() {
        let langs = available_languages();
        assert!(langs.iter().any(|l| l.code == "en"));
        assert!(langs.iter().any(|l| l.code == "fr"));
        assert!(langs.iter().any(|l| l.code == "es"));
    }
}
