use crate::config::actions::ButtonActionConfig;
use crate::config::rings::*;

#[derive(Debug, Clone, Copy)]
pub struct CodeTexts {
    pub profile_label: &'static str,
    pub command_palette: &'static str,
    pub global_search: &'static str,
    pub terminal: &'static str,
    pub file_explorer: &'static str,
    pub secondary_sidebar: &'static str,
}

pub const CODE_TRANSLATIONS: &[(&str, CodeTexts)] = &[
    (
        "en",
        CodeTexts {
            profile_label: "Code Editors & IDE (VS Code / Antigravity)",
            command_palette: "Command Palette",
            global_search: "Global Search",
            terminal: "Integrated Terminal",
            file_explorer: "File Explorer / Sidebar",
            secondary_sidebar: "Secondary Sidebar",
        },
    ),
    (
        "fr",
        CodeTexts {
            profile_label: "Éditeurs & IDE (VS Code / Antigravity)",
            command_palette: "Palette de commandes",
            global_search: "Recherche Globale",
            terminal: "Terminal Intégré",
            file_explorer: "Explorateur / Barre latérale",
            secondary_sidebar: "Barre latérale secondaire",
        },
    ),
    (
        "es",
        CodeTexts {
            profile_label: "Editores & IDE (VS Code / Antigravity)",
            command_palette: "Paleta de Comandos",
            global_search: "Búsqueda Global",
            terminal: "Terminal Integrado",
            file_explorer: "Explorador / Barra Lateral",
            secondary_sidebar: "Barra Lateral Secundaria",
        },
    ),
];

pub fn code_texts(lang: &str) -> &'static CodeTexts {
    super::find_texts_for_lang(CODE_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn code_profile() -> AppProfileConfig {
    code_profile_for_lang("en")
}

pub fn code_profile_for_lang(lang: &str) -> AppProfileConfig {
    let t = code_texts(lang);
    let code_ring = RingMenuConfig {
        default_color: default_ring_color(),
        default_active_color: default_ring_active_color(),
        animation: RingAnimation::default(),
        trigger_mode: RingTriggerMode::default(),
        wheel_navigation: true,
        max_slots: 16,
        glow_effect: default_true(),
        pulse_effect: default_true(),
        items: vec![
            RingMenuItemConfig {
                slot: 0,
                label: t.command_palette.to_string(),
                translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.command_palette),
                icon: "💻".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "p".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 1,
                label: t.secondary_sidebar.to_string(),
                translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.secondary_sidebar),
                icon: "📑".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "ALT".to_string(), "b".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 2,
                label: t.terminal.to_string(),
                translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.terminal),
                icon: "🖥️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "j".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 3,
                label: t.file_explorer.to_string(),
                translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.file_explorer),
                icon: "📂".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "b".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 4,
                label: t.global_search.to_string(),
                translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.global_search),
                icon: "🔍".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "f".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
        ],
    };

    AppProfileConfig {
        app_id: "code; vscode; codium; antigravity; antigravity-ide".to_string(),
        label: t.profile_label.to_string(),
        translations: super::build_translation_map(CODE_TRANSLATIONS, |x| x.profile_label),
        logo_path: "assets/apps/vscode.webp".to_string(),
        ring_menu: Some(code_ring),
    }
}
