use crate::config::actions::ButtonActionConfig;
use crate::config::rings::*;

#[derive(Debug, Clone, Copy)]
pub struct BlenderTexts {
    pub profile_label: &'static str,
    pub object_edit_mode: &'static str,
    pub shading_render_menu: &'static str,
    pub camera_view: &'static str,
    pub search_function: &'static str,
    pub undo: &'static str,
    pub redo: &'static str,
}

pub const BLENDER_TRANSLATIONS: &[(&str, BlenderTexts)] = &[
    (
        "en",
        BlenderTexts {
            profile_label: "Blender 3D",
            object_edit_mode: "Object / Edit Mode",
            shading_render_menu: "Shading / Render Menu",
            camera_view: "Camera View",
            search_function: "Search Function",
            undo: "Undo",
            redo: "Redo",
        },
    ),
    (
        "fr",
        BlenderTexts {
            profile_label: "Blender 3D",
            object_edit_mode: "Mode Objet / Édition",
            shading_render_menu: "Menu Ombrage / Rendu",
            camera_view: "Vue Caméra",
            search_function: "Rechercher une fonction",
            undo: "Annuler",
            redo: "Rétablir",
        },
    ),
    (
        "es",
        BlenderTexts {
            profile_label: "Blender 3D",
            object_edit_mode: "Modo Objeto / Edición",
            shading_render_menu: "Menú Sombreado / Render",
            camera_view: "Vista de Cámara",
            search_function: "Buscar una función",
            undo: "Deshacer",
            redo: "Rehacer",
        },
    ),
];

pub fn blender_texts(lang: &str) -> &'static BlenderTexts {
    super::find_texts_for_lang(BLENDER_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn blender_profile() -> AppProfileConfig {
    blender_profile_for_lang("en")
}

pub fn blender_profile_for_lang(lang: &str) -> AppProfileConfig {
    let t = blender_texts(lang);
    let blender_ring = RingMenuConfig {
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
                label: t.object_edit_mode.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.object_edit_mode),
                icon: "🧊".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["Tab".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 1,
                label: t.shading_render_menu.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.shading_render_menu),
                icon: "💡".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["z".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 2,
                label: t.camera_view.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.camera_view),
                icon: "📷".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["0".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 3,
                label: t.search_function.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.search_function),
                icon: "🔍".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["F3".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 4,
                label: t.undo.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.undo),
                icon: "↩️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "z".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
            RingMenuItemConfig {
                slot: 5,
                label: t.redo.to_string(),
                translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.redo),
                icon: "↪️".to_string(),
                action: Some(ButtonActionConfig::KeyCombo { keys: vec!["CTRL".to_string(), "SHIFT".to_string(), "z".to_string()] }),
                items: vec![],
                auto_close: true,
                color: None,
                active_color: None,
            },
        ],
    };

    AppProfileConfig {
        app_id: "blender".to_string(),
        label: t.profile_label.to_string(),
        translations: super::build_translation_map(BLENDER_TRANSLATIONS, |x| x.profile_label),
        logo_path: "assets/apps/blender.webp".to_string(),
        ring_menu: Some(blender_ring),
    }
}
