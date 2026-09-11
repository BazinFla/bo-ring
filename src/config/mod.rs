pub mod actions;
pub mod presets;
pub mod rings;

pub use actions::*;
pub use presets::*;
pub use rings::*;
pub use crate::utils::color::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn slugify(text: &str) -> String {
    let first_token = text.split(';').next().unwrap_or(text).trim();
    let s: String = first_token
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect();
    let s = s.trim_matches('_');
    if s.is_empty() {
        "profile".to_string()
    } else {
        s.to_lowercase()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    #[serde(with = "serde_u16_map")]
    pub buttons: HashMap<u16, ButtonActionConfig>,
    #[serde(default)]
    pub ring_menu: RingMenuConfig,
    #[serde(default = "default_app_profiles")]
    pub app_profiles: Vec<AppProfileConfig>,
    #[serde(default)]
    pub actions: Vec<NamedActionConfig>,
}

mod serde_u16_map {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S, V>(
        map: &HashMap<u16, V>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        V: Serialize,
    {
        use serde::ser::SerializeMap;
        let mut entries: Vec<(&u16, &V)> = map.iter().collect();
        entries.sort_by_key(|(k, _)| *k);
        let mut map_ser = serializer.serialize_map(Some(entries.len()))?;
        for (k, v) in entries {
            map_ser.serialize_entry(&k.to_string(), v)?;
        }
        map_ser.end()
    }

    pub fn deserialize<'de, D, V>(
        deserializer: D,
    ) -> Result<HashMap<u16, V>, D::Error>
    where
        D: Deserializer<'de>,
        V: Deserialize<'de>,
    {
        let string_map: HashMap<String, V> = HashMap::deserialize(deserializer)?;
        let mut u16_map = HashMap::with_capacity(string_map.len());
        for (k, v) in string_map {
            let code = k.parse::<u16>().map_err(serde::de::Error::custom)?;
            u16_map.insert(code, v);
        }
        Ok(u16_map)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct GeneralConfig {
    #[serde(default = "default_device_name")]
    pub device_name: String,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default)]
    pub autostart_daemon: bool,
    #[serde(default, alias = "dev_mode_anchors")]
    pub dev_mode: bool,
    #[serde(default, alias = "editor_mode")]
    pub mode_editor: bool,
    #[serde(default = "default_device_profile")]
    pub device_profile: String,
}

fn default_device_name() -> String {
    "Logitech USB Receiver Mouse".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_language() -> String {
    "system".to_string()
}

fn default_device_profile() -> String {
    "auto".to_string()
}

impl Config {
    pub fn default_config() -> Self {
        presets::default_config()
    }

    /// Returns the XDG-compliant config root directory: ~/.config/bo-ring
    pub fn config_dir() -> PathBuf {
        let base_dir = std::env::var("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                PathBuf::from(home).join(".config")
            });
        base_dir.join("bo-ring")
    }

    /// Returns the primary config file path: ~/.config/bo-ring/config.toml
    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    /// Returns the modular rings directory path: ~/.config/bo-ring/rings
    pub fn rings_dir() -> PathBuf {
        Self::config_dir().join("rings")
    }

    /// Returns the modular actions directory path: ~/.config/bo-ring/actions
    pub fn actions_dir() -> PathBuf {
        Self::config_dir().join("actions")
    }

    pub fn sanitize(&mut self) {
        // Primary left and right clicks must NEVER be remapped
        self.buttons.remove(&crate::devices::BTN_LEFT_CODE);
        self.buttons.remove(&crate::devices::BTN_RIGHT_CODE);
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.sanitize();
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        self.sanitize();
        let content = toml::to_string_pretty(self)?;
        if let Some(parent) = path.as_ref().parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("⚠️ Could not create config parent directory {:?}: {}", parent, e);
            }
        }
        fs::write(&path, content)?;

        // Synchronize modular rings/ directory
        let rings_dir = Self::rings_dir();
        if let Err(e) = fs::create_dir_all(&rings_dir) {
            eprintln!("⚠️ Could not create rings directory {:?}: {}", rings_dir, e);
        } else {
            let mut active_files = std::collections::HashSet::new();

            // 1. Save default main ring menu
            let default_profile = AppProfileConfig {
                app_id: "default".to_string(),
                label: "Main Ring".to_string(),
                translations: std::collections::HashMap::new(),
                logo_path: "".to_string(),
                ring_menu: Some(self.ring_menu.clone()),
            };
            let default_filename = "default.ring.toml".to_string();
            active_files.insert(default_filename.clone());
            if let Ok(toml_str) = toml::to_string_pretty(&default_profile) {
                let p = rings_dir.join(&default_filename);
                if let Err(e) = fs::write(&p, toml_str) {
                    eprintln!("⚠️ Failed to write default ring configuration to {:?}: {}", p, e);
                }
            }

            // 2. Save each app profile ring
            for profile in &self.app_profiles {
                let slug = slugify(&profile.app_id);
                let filename = format!("{}.ring.toml", slug);
                active_files.insert(filename.clone());
                if let Ok(toml_str) = toml::to_string_pretty(profile) {
                    let p = rings_dir.join(&filename);
                    if let Err(e) = fs::write(&p, toml_str) {
                        eprintln!("⚠️ Failed to write app profile ring to {:?}: {}", p, e);
                    }
                }
            }

            // 3. Clean up deleted ring profile files
            if let Ok(entries) = fs::read_dir(&rings_dir) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_file() {
                        if let Some(name_str) = p.file_name().and_then(|n| n.to_str()) {
                            if name_str.ends_with(".ring.toml") && !active_files.contains(name_str) {
                                let _ = fs::remove_file(p);
                            }
                        }
                    }
                }
            }
        }

        // Synchronize modular actions/ directory
        let actions_dir = Self::actions_dir();
        if let Err(e) = fs::create_dir_all(&actions_dir) {
            eprintln!("⚠️ Could not create actions directory {:?}: {}", actions_dir, e);
        } else {
            for action in &self.actions {
                let filename = format!("{}.action.toml", slugify(&action.id));
                if let Ok(toml_str) = toml::to_string_pretty(action) {
                    let p = actions_dir.join(filename);
                    if let Err(e) = fs::write(&p, toml_str) {
                        eprintln!("⚠️ Failed to write action configuration to {:?}: {}", p, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn load_or_default() -> Self {
        let rings_dir = Self::rings_dir();
        let actions_dir = Self::actions_dir();

        if let Err(e) = fs::create_dir_all(&rings_dir) {
            eprintln!("⚠️ Could not create rings directory {:?}: {}", rings_dir, e);
        }
        if let Err(e) = fs::create_dir_all(&actions_dir) {
            eprintln!("⚠️ Could not create actions directory {:?}: {}", actions_dir, e);
        }

        let mut has_ring_files = false;
        if let Ok(entries) = fs::read_dir(&rings_dir) {
            for entry in entries.flatten() {
                if entry.path().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "toml" || ext == "json" {
                            has_ring_files = true;
                            break;
                        }
                    }
                }
            }
        }

        if has_ring_files {
            let config_path = Self::config_path();
            let mut config = if config_path.exists() {
                Self::load_from_file(&config_path).unwrap_or_else(|_| Self::default_config())
            } else {
                Self::default_config()
            };

            config.app_profiles.clear();

            if let Ok(entries) = fs::read_dir(&rings_dir) {
                let mut loaded_profiles = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(profile) = toml::from_str::<AppProfileConfig>(&content) {
                                if profile.app_id == "default" || path.file_name().and_then(|n| n.to_str()) == Some("default.ring.toml") {
                                    if let Some(menu) = profile.ring_menu {
                                        config.ring_menu = menu;
                                    }
                                } else {
                                    loaded_profiles.push(profile);
                                }
                            }
                        }
                    }
                }
                if !loaded_profiles.is_empty() {
                    config.app_profiles = loaded_profiles;
                }
            }

            if let Ok(entries) = fs::read_dir(&actions_dir) {
                let mut loaded_actions = Vec::new();
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(action) = toml::from_str::<NamedActionConfig>(&content) {
                                loaded_actions.push(action);
                            }
                        }
                    }
                }
                if !loaded_actions.is_empty() {
                    config.actions = loaded_actions;
                } else {
                    config.actions = default_actions_catalog();
                }
            }

            println!("📖 Configuration and rings loaded from {:?}", rings_dir);
            return config;
        }

        let mut candidates = vec![
            Self::config_path(),
            PathBuf::from("config.toml"),
            PathBuf::from("../config.toml"),
        ];

        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                candidates.push(dir.join("config.toml"));
                if let Some(parent) = dir.parent() {
                    candidates.push(parent.join("config.toml"));
                }
            }
        }

        for path in &candidates {
            if path.exists() {
                if let Ok(mut cfg) = Self::load_from_file(path) {
                    if !cfg.ring_menu.items.is_empty() {
                        println!("📖 Legacy configuration loaded from {:?}. Migrating to modular rings directory...", path);
                        if let Err(e) = cfg.save_to_file(Self::config_path()) {
                            eprintln!("⚠️ Failed to save migrated configuration to {:?}: {}", Self::config_path(), e);
                        }
                        return cfg;
                    }
                }
            }
        }

        println!("⚠️ Loading default configuration.");
        let mut cfg = Self::default_config();
        if let Err(e) = cfg.save_to_file(Self::config_path()) {
            eprintln!("⚠️ Failed to save default configuration to {:?}: {}", Self::config_path(), e);
        }
        cfg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let cfg = Config::default_config();
        let toml_str = toml::to_string_pretty(&cfg).expect("Serialization failed");
        println!("TOML:\n{}", toml_str);
        let reloaded: Config = toml::from_str(&toml_str).expect("Deserialization failed");
        assert_eq!(reloaded.buttons.len(), cfg.buttons.len());
        assert_eq!(reloaded.ring_menu.animation, RingAnimation::ScaleFade);
        assert_eq!(reloaded.ring_menu.trigger_mode, RingTriggerMode::Hybrid);
        assert!(reloaded.ring_menu.wheel_navigation);
        assert_eq!(reloaded.ring_menu.max_slots, 16);
    }

    #[test]
    fn test_ring_trigger_mode_serialization() {
        let mut cfg = Config::default_config();
        cfg.ring_menu.trigger_mode = RingTriggerMode::HoldToRelease;
        cfg.ring_menu.wheel_navigation = false;
        cfg.ring_menu.max_slots = 12;

        let toml_str = toml::to_string(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.ring_menu.trigger_mode, RingTriggerMode::HoldToRelease);
        assert!(!loaded.ring_menu.wheel_navigation);
        assert_eq!(loaded.ring_menu.max_slots, 12);
    }

    #[test]
    fn test_matches_app_id() {
        let app_ids = "firefox; brave; brave-browser";
        assert!(matches_app_id(app_ids, "firefox"));
        assert!(matches_app_id(app_ids, "Brave"));
        assert!(matches_app_id(app_ids, "brave-browser"));
        assert!(matches_app_id(app_ids, "org.mozilla.firefox"));
        assert!(matches_app_id(app_ids, "org.kde.firefox"));
        assert!(matches_app_id("org.kde.dolphin; dolphin", "dolphin"));
        assert!(matches_app_id("org.kde.dolphin", "org.kde.dolphin"));
        assert!(!matches_app_id(app_ids, "chrome"));
        assert!(!matches_app_id(app_ids, ""));
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("firefox; brave"), "firefox");
        assert_eq!(slugify("Éditeurs & IDE (VS Code / Antigravity)"), "éditeurs___ide__vs_code___antigravity");
        assert_eq!(slugify("default"), "default");
        assert_eq!(slugify("  code  "), "code");
    }

    #[test]
    fn test_resolve_action() {
        let catalog = vec![
            NamedActionConfig {
                id: "test_cmd".to_string(),
                label: "Test Command".to_string(),
                translations: std::collections::HashMap::new(),
                icon: "🚀".to_string(),
                action: ButtonActionConfig::Command { cmd: "echo hello".to_string() },
            }
        ];

        let action_ref = ButtonActionConfig::ActionRef { id: "test_cmd".to_string() };
        let resolved = resolve_action(&action_ref, &catalog);
        assert_eq!(resolved, ButtonActionConfig::Command { cmd: "echo hello".to_string() });

        let direct_cmd = ButtonActionConfig::Command { cmd: "ls -la".to_string() };
        assert_eq!(resolve_action(&direct_cmd, &catalog), direct_cmd);
    }

    #[test]
    fn test_default_actions_catalog() {
        let catalog = default_actions_catalog();
        assert!(!catalog.is_empty());
        assert!(catalog.iter().any(|a| a.id == "media_play_pause"));
        assert!(catalog.iter().any(|a| a.id == "system_screenshot"));
        assert!(catalog.iter().any(|a| a.id == "dev_terminal"));
        assert!(catalog.iter().any(|a| a.id == "dev_secondary_sidebar"));
        assert!(catalog.iter().any(|a| a.id == "ai_chatgpt"));
    }

    #[test]
    fn test_actions_embedded_translations() {
        let en_catalog = default_actions_catalog_for_lang("en");
        let fr_catalog = default_actions_catalog_for_lang("fr");
        let es_catalog = default_actions_catalog_for_lang("es");

        let play_en = en_catalog.iter().find(|a| a.id == "media_play_pause").unwrap();
        let play_fr = fr_catalog.iter().find(|a| a.id == "media_play_pause").unwrap();
        let play_es = es_catalog.iter().find(|a| a.id == "media_play_pause").unwrap();

        assert_eq!(play_en.label, "Play / Pause");
        assert_eq!(play_fr.label, "Lecture / Pause");
        assert_eq!(play_es.label, "Reproducir / Pausa");

        // Test label_lang resolution
        assert_eq!(play_en.label_lang("fr"), "Lecture / Pause");
        assert_eq!(play_en.label_lang("es"), "Reproducir / Pausa");
        assert_eq!(play_en.label_lang("de"), "Play / Pause"); // fallback

        // Test action TOML serialization roundtrip (.action.toml)
        let toml_str = toml::to_string_pretty(play_en).expect("Action serialization failed");
        assert!(toml_str.contains("[translations]"));
        assert!(toml_str.contains("fr = \"Lecture / Pause\""));
        let reloaded: NamedActionConfig = toml::from_str(&toml_str).expect("Action deserialization failed");
        assert_eq!(reloaded.label_lang("fr"), "Lecture / Pause");
    }

    #[test]
    fn test_app_profile_ring_loading() {
        let cfg = Config::default_config();
        assert!(!cfg.app_profiles.is_empty());
        for p in &cfg.app_profiles {
            assert!(p.ring_menu.is_some(), "Profile {} has no ring_menu", p.label);
            let ring = p.ring_menu.as_ref().unwrap();
            assert!(!ring.items.is_empty(), "Profile {} ring menu has no items", p.label);
        }

        // Test serialization and deserialization of AppProfileConfig
        for p in &cfg.app_profiles {
            let toml_str = toml::to_string_pretty(p).expect("AppProfileConfig serialization failed");
            let reloaded: AppProfileConfig = toml::from_str(&toml_str).expect("AppProfileConfig deserialization failed");
            assert_eq!(reloaded.app_id, p.app_id);
            assert!(reloaded.ring_menu.is_some());
            assert_eq!(reloaded.ring_menu.unwrap().items.len(), p.ring_menu.as_ref().unwrap().items.len());
        }
    }

    #[test]
    fn test_presets_embedded_translations() {
        // 1. Blender
        let en_blender = blender_profile_for_lang("en");
        let fr_blender = blender_profile_for_lang("fr");
        let es_blender = blender_profile_for_lang("es");
        let fallback_blender = blender_profile_for_lang("de");
        assert_eq!(en_blender.ring_menu.as_ref().unwrap().items[0].label, "Object / Edit Mode");
        assert_eq!(fr_blender.ring_menu.as_ref().unwrap().items[0].label, "Mode Objet / Édition");
        assert_eq!(es_blender.ring_menu.as_ref().unwrap().items[0].label, "Modo Objeto / Edición");
        assert_eq!(fallback_blender.ring_menu.as_ref().unwrap().items[0].label, "Object / Edit Mode");

        // 2. Browser
        let en_browser = browser_profile_for_lang("en");
        let fr_browser = browser_profile_for_lang("fr");
        let es_browser = browser_profile_for_lang("es");
        assert_eq!(en_browser.label, "Web Browsers");
        assert_eq!(fr_browser.label, "Navigateurs Web");
        assert_eq!(es_browser.label, "Navegadores Web");
        assert_eq!(en_browser.ring_menu.as_ref().unwrap().items[0].label, "New Tab");
        assert_eq!(fr_browser.ring_menu.as_ref().unwrap().items[0].label, "Nouvel Onglet");
        assert_eq!(es_browser.ring_menu.as_ref().unwrap().items[0].label, "Nueva Pestaña");

        // 3. Code
        let en_code = code_profile_for_lang("en");
        let fr_code = code_profile_for_lang("fr");
        let es_code = code_profile_for_lang("es");
        assert_eq!(en_code.ring_menu.as_ref().unwrap().items[0].label, "Command Palette");
        assert_eq!(fr_code.ring_menu.as_ref().unwrap().items[0].label, "Palette de commandes");
        assert_eq!(es_code.ring_menu.as_ref().unwrap().items[0].label, "Paleta de Comandos");
        assert_eq!(en_code.ring_menu.as_ref().unwrap().items[1].label, "Secondary Sidebar");
        assert_eq!(fr_code.ring_menu.as_ref().unwrap().items[1].label, "Barre latérale secondaire");
        assert_eq!(es_code.ring_menu.as_ref().unwrap().items[1].label, "Barra Lateral Secundaria");

        // 4. Files
        let en_files = files_profile_for_lang("en");
        let fr_files = files_profile_for_lang("fr");
        let es_files = files_profile_for_lang("es");
        assert_eq!(en_files.label, "File Managers");
        assert_eq!(fr_files.label, "Gestionnaires de Fichiers");
        assert_eq!(es_files.label, "Administradores de Archivos");

        // 5. GIMP
        let en_gimp = gimp_profile_for_lang("en");
        let fr_gimp = gimp_profile_for_lang("fr");
        let es_gimp = gimp_profile_for_lang("es");
        assert_eq!(en_gimp.ring_menu.as_ref().unwrap().items[0].label, "Brush Tool");
        assert_eq!(fr_gimp.ring_menu.as_ref().unwrap().items[0].label, "Outil Pinceau");
        assert_eq!(es_gimp.ring_menu.as_ref().unwrap().items[0].label, "Herramienta Pincel");

        // 6. Default config
        let en_demo = default_config_for_lang("en");
        let fr_demo = default_config_for_lang("fr");
        let es_demo = default_config_for_lang("es");
        assert_eq!(en_demo.ring_menu.items[0].label, "Screenshot");
        assert_eq!(fr_demo.ring_menu.items[0].label, "Capture d'écran");
        assert_eq!(es_demo.ring_menu.items[0].label, "Captura de Pantalla");
    }

    #[test]
    fn test_locales_key_parity() {
        let en_file: crate::utils::i18n::LocaleFile = serde_json::from_str(include_str!("../../locales/en.json")).unwrap();
        let fr_file: crate::utils::i18n::LocaleFile = serde_json::from_str(include_str!("../../locales/fr.json")).unwrap();
        let es_file: crate::utils::i18n::LocaleFile = serde_json::from_str(include_str!("../../locales/es.json")).unwrap();

        for key in en_file.translations.keys() {
            assert!(fr_file.translations.contains_key(key), "fr.json is missing key: {}", key);
            assert!(es_file.translations.contains_key(key), "es.json is missing key: {}", key);
        }

        for key in fr_file.translations.keys() {
            assert!(en_file.translations.contains_key(key), "en.json is missing key from fr: {}", key);
        }

        for key in es_file.translations.keys() {
            assert!(en_file.translations.contains_key(key), "en.json is missing key from es: {}", key);
        }
    }

    #[test]
    fn test_app_profile_embedded_translations_roundtrip() {
        let browser = browser_profile_for_lang("en");
        
        // Check profile label translation resolution
        assert_eq!(browser.label_lang("en"), "Web Browsers");
        assert_eq!(browser.label_lang("fr"), "Navigateurs Web");
        assert_eq!(browser.label_lang("es"), "Navegadores Web");
        assert_eq!(browser.label_lang("de"), "Web Browsers"); // fallback

        // Check slot 0 item translation resolution
        let slot0 = &browser.ring_menu.as_ref().unwrap().items[0];
        assert_eq!(slot0.label_lang("en"), "New Tab");
        assert_eq!(slot0.label_lang("fr"), "Nouvel Onglet");
        assert_eq!(slot0.label_lang("es"), "Nueva Pestaña");

        // Check slot 1 sub-item 0 translation resolution
        let slot1_sub0 = &browser.ring_menu.as_ref().unwrap().items[1].items[0];
        assert_eq!(slot1_sub0.label_lang("en"), "Reopen Closed Tab");
        assert_eq!(slot1_sub0.label_lang("fr"), "Rouvrir Onglet Fermé");
        assert_eq!(slot1_sub0.label_lang("es"), "Reabrir Pestaña Cerrada");

        // 1. Test TOML serialization/deserialization roundtrip (.boring-app.toml)
        let toml_str = toml::to_string_pretty(&browser).expect("Failed to serialize to TOML");
        assert!(toml_str.contains("[translations]"));
        assert!(toml_str.contains("fr = \"Navigateurs Web\""));
        let toml_reloaded: AppProfileConfig = toml::from_str(&toml_str).expect("Failed to deserialize from TOML");
        assert_eq!(toml_reloaded.label_lang("fr"), "Navigateurs Web");
        assert_eq!(toml_reloaded.ring_menu.as_ref().unwrap().items[0].label_lang("fr"), "Nouvel Onglet");

        // 2. Test JSON serialization/deserialization roundtrip (.boring-app.json)
        let json_str = serde_json::to_string_pretty(&browser).expect("Failed to serialize to JSON");
        assert!(json_str.contains("\"translations\""));
        assert!(json_str.contains("\"Navigateurs Web\""));
        let json_reloaded: AppProfileConfig = serde_json::from_str(&json_str).expect("Failed to deserialize from JSON");
        assert_eq!(json_reloaded.label_lang("es"), "Navegadores Web");
        assert_eq!(json_reloaded.ring_menu.as_ref().unwrap().items[0].label_lang("es"), "Nueva Pestaña");

        // 3. Test Backward compatibility: older profile without translations field
        let legacy_toml = r#"
            app_id = "custom_app"
            label = "Custom App"
            logo_path = ""
        "#;
        let legacy: AppProfileConfig = toml::from_str(legacy_toml).expect("Failed to deserialize legacy profile");
        assert_eq!(legacy.label_lang("fr"), "Custom App");
        assert_eq!(legacy.label_lang("en"), "Custom App");
    }
}
