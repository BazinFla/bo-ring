use super::{ButtonActionConfig, NamedActionConfig};
use crate::config::presets::{build_translation_map, find_texts_for_lang};

#[derive(Debug, Clone, Copy)]
pub struct MediaActionTexts {
    pub play_pause: &'static str,
    pub next_track: &'static str,
    pub prev_track: &'static str,
    pub mute_sound: &'static str,
}

pub const MEDIA_TRANSLATIONS: &[(&str, MediaActionTexts)] = &[
    (
        "en",
        MediaActionTexts {
            play_pause: "Play / Pause",
            next_track: "Next Track",
            prev_track: "Previous Track",
            mute_sound: "Mute Sound",
        },
    ),
    (
        "fr",
        MediaActionTexts {
            play_pause: "Lecture / Pause",
            next_track: "Piste Suivante",
            prev_track: "Piste Précédente",
            mute_sound: "Couper le Son",
        },
    ),
    (
        "es",
        MediaActionTexts {
            play_pause: "Reproducir / Pausa",
            next_track: "Pista Siguiente",
            prev_track: "Pista Anterior",
            mute_sound: "Silenciar",
        },
    ),
];

pub fn media_action_texts(lang: &str) -> &'static MediaActionTexts {
    find_texts_for_lang(MEDIA_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn media_actions() -> Vec<NamedActionConfig> {
    media_actions_for_lang("en")
}

pub fn media_actions_for_lang(lang: &str) -> Vec<NamedActionConfig> {
    let t = media_action_texts(lang);
    vec![
        NamedActionConfig {
            id: "media_play_pause".to_string(),
            label: t.play_pause.to_string(),
            translations: build_translation_map(MEDIA_TRANSLATIONS, |x| x.play_pause),
            icon: "⏯️".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["XF86AudioPlay".to_string()] },
        },
        NamedActionConfig {
            id: "media_next".to_string(),
            label: t.next_track.to_string(),
            translations: build_translation_map(MEDIA_TRANSLATIONS, |x| x.next_track),
            icon: "⏭️".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["XF86AudioNext".to_string()] },
        },
        NamedActionConfig {
            id: "media_prev".to_string(),
            label: t.prev_track.to_string(),
            translations: build_translation_map(MEDIA_TRANSLATIONS, |x| x.prev_track),
            icon: "⏮️".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["XF86AudioPrev".to_string()] },
        },
        NamedActionConfig {
            id: "media_mute".to_string(),
            label: t.mute_sound.to_string(),
            translations: build_translation_map(MEDIA_TRANSLATIONS, |x| x.mute_sound),
            icon: "🔇".to_string(),
            action: ButtonActionConfig::KeyCombo { keys: vec!["XF86AudioMute".to_string()] },
        },
    ]
}
