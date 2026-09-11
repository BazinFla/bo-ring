use super::{ButtonActionConfig, NamedActionConfig};
use crate::config::presets::{build_translation_map, find_texts_for_lang};

#[derive(Debug, Clone, Copy)]
pub struct SystemActionTexts {
    pub screenshot: &'static str,
    pub overview: &'static str,
    pub settings: &'static str,
    pub lock_session: &'static str,
    pub power_off: &'static str,
}

pub const SYSTEM_TRANSLATIONS: &[(&str, SystemActionTexts)] = &[
    (
        "en",
        SystemActionTexts {
            screenshot: "Screenshot",
            overview: "Window Overview",
            settings: "System Settings",
            lock_session: "Lock Session",
            power_off: "Power Off",
        },
    ),
    (
        "fr",
        SystemActionTexts {
            screenshot: "Capture d'écran",
            overview: "Aperçu des Fenêtres",
            settings: "Paramètres Système",
            lock_session: "Verrouiller la Session",
            power_off: "Éteindre",
        },
    ),
    (
        "es",
        SystemActionTexts {
            screenshot: "Captura de Pantalla",
            overview: "Vista General de Ventanas",
            settings: "Configuración del Sistema",
            lock_session: "Bloquear Sesión",
            power_off: "Apagar",
        },
    ),
];

pub fn system_action_texts(lang: &str) -> &'static SystemActionTexts {
    find_texts_for_lang(SYSTEM_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn system_actions() -> Vec<NamedActionConfig> {
    system_actions_for_lang("en")
}

pub fn system_actions_for_lang(lang: &str) -> Vec<NamedActionConfig> {
    let t = system_action_texts(lang);
    vec![
        NamedActionConfig {
            id: "system_screenshot".to_string(),
            label: t.screenshot.to_string(),
            translations: build_translation_map(SYSTEM_TRANSLATIONS, |x| x.screenshot),
            icon: "📷".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["Print".to_string()] },
        },
        NamedActionConfig {
            id: "system_overview".to_string(),
            label: t.overview.to_string(),
            translations: build_translation_map(SYSTEM_TRANSLATIONS, |x| x.overview),
            icon: "🪟".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["Super_L".to_string()] },
        },
        NamedActionConfig {
            id: "system_settings".to_string(),
            label: t.settings.to_string(),
            translations: build_translation_map(SYSTEM_TRANSLATIONS, |x| x.settings),
            icon: "⚙️".to_string(),
            action: ButtonActionConfig::Command { cmd: "gnome-control-center".to_string() },
        },
        NamedActionConfig {
            id: "system_lock".to_string(),
            label: t.lock_session.to_string(),
            translations: build_translation_map(SYSTEM_TRANSLATIONS, |x| x.lock_session),
            icon: "🔒".to_string(),
            action: ButtonActionConfig::Command { cmd: "loginctl lock-session".to_string() },
        },
        NamedActionConfig {
            id: "system_poweroff".to_string(),
            label: t.power_off.to_string(),
            translations: build_translation_map(SYSTEM_TRANSLATIONS, |x| x.power_off),
            icon: "⏻".to_string(),
            action: ButtonActionConfig::Command { cmd: "systemctl poweroff".to_string() },
        },
    ]
}
