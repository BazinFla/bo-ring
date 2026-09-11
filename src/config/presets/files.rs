use crate::config::actions::ButtonActionConfig;
use crate::config::rings::*;

#[derive(Debug, Clone, Copy)]
pub struct FilesTexts {
    pub profile_label: &'static str,
    pub new_folder: &'static str,
    pub properties: &'static str,
    pub search: &'static str,
    pub back_page: &'static str,
    pub forward_page: &'static str,
    pub parent_folder: &'static str,
}

pub const FILES_TRANSLATIONS: &[(&str, FilesTexts)] = &[
    (
        "en",
        FilesTexts {
            profile_label: "File Managers",
            new_folder: "New Folder",
            properties: "Properties",
            search: "Search",
            back_page: "Previous Page",
            forward_page: "Next Page",
            parent_folder: "Parent Folder",
        },
    ),
    (
        "fr",
        FilesTexts {
            profile_label: "Gestionnaires de Fichiers",
            new_folder: "Nouveau Dossier",
            properties: "Propriétés",
            search: "Rechercher",
            back_page: "Page Précédente",
            forward_page: "Page Suivante",
            parent_folder: "Dossier Parent",
        },
    ),
    (
        "es",
        FilesTexts {
            profile_label: "Administradores de Archivos",
            new_folder: "Nueva Carpeta",
            properties: "Propiedades",
            search: "Buscar",
            back_page: "Página Anterior",
            forward_page: "Página Siguiente",
            parent_folder: "Carpeta Superior",
        },
    ),
];

pub fn files_texts(lang: &str) -> &'static FilesTexts {
    super::find_texts_for_lang(FILES_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn files_profile() -> AppProfileConfig {
    files_profile_for_lang("en")
}

pub fn files_profile_for_lang(lang: &str) -> AppProfileConfig {
    let t = files_texts(lang);
    let files_ring = RingMenuConfig {
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
                label: t.new_folder.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.new_folder),
                icon: "📁".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "n".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 1,
                label: t.properties.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.properties),
                icon: "ℹ️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["ALT".to_string(), "Return".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 2,
                label: t.search.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.search),
                icon: "🔍".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "f".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 3,
                label: t.back_page.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.back_page),
                icon: "⬅️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["BTN_BACK".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 4,
                label: t.forward_page.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.forward_page),
                icon: "➡️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["BTN_FORWARD".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 5,
                label: t.parent_folder.to_string(),
                translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.parent_folder),
                icon: "⬆️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["ALT".to_string(), "Up".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
        ],
    };

    AppProfileConfig {
        app_id: "files; nautilus; dolphin; thunar; nemo; pcmanfm; krusader".to_string(),
        label: t.profile_label.to_string(),
        translations: super::build_translation_map(FILES_TRANSLATIONS, |x| x.profile_label),
        logo_path: "assets/apps/files.webp".to_string(),
        ring_menu: Some(files_ring),
    }
}
