use crate::config::actions::ButtonActionConfig;
use crate::config::rings::*;

#[derive(Debug, Clone, Copy)]
pub struct GimpTexts {
    pub profile_label: &'static str,
    pub brush_tool: &'static str,
    pub select_tool: &'static str,
    pub color_picker: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
    pub export_as: &'static str,
}

pub const GIMP_TRANSLATIONS: &[(&str, GimpTexts)] = &[
    (
        "en",
        GimpTexts {
            profile_label: "GIMP",
            brush_tool: "Brush Tool",
            select_tool: "Select Tool",
            color_picker: "Color Picker",
            undo: "Undo",
            redo: "Redo",
            export_as: "Export As...",
        },
    ),
    (
        "fr",
        GimpTexts {
            profile_label: "GIMP",
            brush_tool: "Outil Pinceau",
            select_tool: "Outil Sélection",
            color_picker: "Pipette de couleur",
            undo: "Annuler",
            redo: "Rétablir",
            export_as: "Exporter sous...",
        },
    ),
    (
        "es",
        GimpTexts {
            profile_label: "GIMP",
            brush_tool: "Herramienta Pincel",
            select_tool: "Herramienta Selección",
            color_picker: "Selector de Color",
            undo: "Deshacer",
            redo: "Rehacer",
            export_as: "Exportar como...",
        },
    ),
];

pub fn gimp_texts(lang: &str) -> &'static GimpTexts {
    super::find_texts_for_lang(GIMP_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn gimp_profile() -> AppProfileConfig {
    gimp_profile_for_lang("en")
}

pub fn gimp_profile_for_lang(lang: &str) -> AppProfileConfig {
    let t = gimp_texts(lang);
    let gimp_ring = RingMenuConfig {
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
                label: t.brush_tool.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.brush_tool),
                icon: "🖌️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["p".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 1,
                label: t.select_tool.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.select_tool),
                icon: "🔲".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["r".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 2,
                label: t.color_picker.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.color_picker),
                icon: "💧".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["o".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 3,
                label: t.undo.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.undo),
                icon: "↩️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "z".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 4,
                label: t.redo.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.redo),
                icon: "↪️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "y".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 5,
                label: t.export_as.to_string(),
                translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.export_as),
                icon: "💾".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "e".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
        ],
    };

    AppProfileConfig {
        app_id: "gimp".to_string(),
        label: t.profile_label.to_string(),
        translations: super::build_translation_map(GIMP_TRANSLATIONS, |x| x.profile_label),
        logo_path: "assets/apps/gimp.webp".to_string(),
        ring_menu: Some(gimp_ring),
    }
}
