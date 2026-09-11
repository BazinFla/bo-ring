use super::{ButtonActionConfig, NamedActionConfig};
use crate::config::presets::{build_translation_map, find_texts_for_lang};

#[derive(Debug, Clone, Copy)]
pub struct DeveloperActionTexts {
    pub terminal: &'static str,
    pub vscode_palette: &'static str,
    pub global_search: &'static str,
    pub secondary_sidebar: &'static str,
}

pub const DEVELOPER_TRANSLATIONS: &[(&str, DeveloperActionTexts)] = &[
    (
        "en",
        DeveloperActionTexts {
            terminal: "Open Terminal",
            vscode_palette: "Command Palette",
            global_search: "Global Search",
            secondary_sidebar: "Toggle Secondary Sidebar",
        },
    ),
    (
        "fr",
        DeveloperActionTexts {
            terminal: "Ouvrir le Terminal",
            vscode_palette: "Palette de commandes",
            global_search: "Recherche Globale",
            secondary_sidebar: "Basculer la barre latérale secondaire",
        },
    ),
    (
        "es",
        DeveloperActionTexts {
            terminal: "Abrir Terminal",
            vscode_palette: "Paleta de Comandos",
            global_search: "Búsqueda Global",
            secondary_sidebar: "Alternar barra lateral secundaria",
        },
    ),
];

pub fn developer_action_texts(lang: &str) -> &'static DeveloperActionTexts {
    find_texts_for_lang(DEVELOPER_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn developer_actions() -> Vec<NamedActionConfig> {
    developer_actions_for_lang("en")
}

pub fn developer_actions_for_lang(lang: &str) -> Vec<NamedActionConfig> {
    let t = developer_action_texts(lang);
    vec![
        NamedActionConfig {
            id: "dev_terminal".to_string(),
            label: t.terminal.to_string(),
            translations: build_translation_map(DEVELOPER_TRANSLATIONS, |x| x.terminal),
            icon: "💻".to_string(),
            action: ButtonActionConfig::Command { cmd: "ptyxis || gnome-terminal || kgx || konsole || alacritty || kitty || x-terminal-emulator || xterm".to_string() },
        },
        NamedActionConfig {
            id: "dev_vscode_palette".to_string(),
            label: t.vscode_palette.to_string(),
            translations: build_translation_map(DEVELOPER_TRANSLATIONS, |x| x.vscode_palette),
            icon: "💻".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "p".to_string()] },
        },
        NamedActionConfig {
            id: "dev_global_search".to_string(),
            label: t.global_search.to_string(),
            translations: build_translation_map(DEVELOPER_TRANSLATIONS, |x| x.global_search),
            icon: "🔍".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "f".to_string()] },
        },
        NamedActionConfig {
            id: "dev_secondary_sidebar".to_string(),
            label: t.secondary_sidebar.to_string(),
            translations: build_translation_map(DEVELOPER_TRANSLATIONS, |x| x.secondary_sidebar),
            icon: "📑".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "ALT".to_string(), "b".to_string()] },
        },
    ]
}
