use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::utils::color::detect_os_accent_color;
use super::actions::ButtonActionConfig;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppProfileConfig {
    pub app_id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<String, String>,
    #[serde(default)]
    pub logo_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ring_menu: Option<RingMenuConfig>,
}

impl AppProfileConfig {
    pub fn label_lang<'a>(&'a self, lang: &str) -> &'a str {
        let code = lang.split(&['_', '.'][..]).next().unwrap_or(lang).to_lowercase();
        if let Some(t) = self.translations.get(&code) {
            if !t.is_empty() {
                return t.as_str();
            }
        }
        &self.label
    }
}

/// Returns true if `active_app` matches any of the `;`-separated IDs in `app_id_field`.
/// Handles Reverse-DNS desktop IDs like `org.kde.dolphin` -> `dolphin`, `org.mozilla.firefox` -> `firefox`.
pub fn matches_app_id(app_id_field: &str, active_app: &str) -> bool {
    let target = active_app.trim();
    if target.is_empty() {
        return false;
    }

    let short_target = target.rsplit('.').next().unwrap_or(target);

    app_id_field
        .split(';')
        .map(|s| s.trim())
        .any(|id| {
            if id.is_empty() {
                return false;
            }
            let short_id = id.rsplit('.').next().unwrap_or(id);
            id.eq_ignore_ascii_case(target)
                || short_id.eq_ignore_ascii_case(short_target)
                || id.eq_ignore_ascii_case(short_target)
                || short_id.eq_ignore_ascii_case(target)
        })
}

/// Animation style for the ring menu appearance/disappearance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RingAnimation {
    /// No animation — ring appears/disappears instantly.
    None,
    /// Scale from 0→1 with simultaneous opacity fade-in (200ms ease-out).
    ScaleFade,
}

impl Default for RingAnimation {
    fn default() -> Self {
        RingAnimation::ScaleFade
    }
}

impl RingAnimation {

    pub fn label_lang<'a>(&self, lang: &str) -> &'a str {
        match self {
            RingAnimation::None => crate::utils::i18n::tr(lang, "anim_instant"),
            RingAnimation::ScaleFade => crate::utils::i18n::tr(lang, "anim_scale_fade"),
        }
    }

    pub fn all() -> &'static [RingAnimation] {
        &[RingAnimation::None, RingAnimation::ScaleFade]
    }
}

/// Triggering behavior for opening/executing actions in the ring menu.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RingTriggerMode {
    Click,
    HoldToRelease,
    Hybrid,
}

impl Default for RingTriggerMode {
    fn default() -> Self {
        RingTriggerMode::Hybrid
    }
}

pub fn default_true() -> bool {
    true
}

pub fn default_max_slots() -> usize {
    16
}

pub fn default_ring_color() -> String {
    "#282C37".to_string()
}

pub fn default_ring_active_color() -> String {
    if let Some(color) = detect_os_accent_color() {
        println!("🎨 OS accent color detected: {}", color);
        color
    } else {
        "#00D7AF".to_string()
    }
}

pub fn default_auto_close() -> bool {
    true
}

pub fn default_icon() -> String {
    "⚡".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RingMenuConfig {
    #[serde(default = "default_ring_color")]
    pub default_color: String,
    #[serde(default = "default_ring_active_color")]
    pub default_active_color: String,
    #[serde(default)]
    pub items: Vec<RingMenuItemConfig>,
    #[serde(default)]
    pub animation: RingAnimation,
    #[serde(default)]
    pub trigger_mode: RingTriggerMode,
    #[serde(default = "default_true")]
    pub wheel_navigation: bool,
    #[serde(default = "default_max_slots")]
    pub max_slots: usize,
    #[serde(default = "default_true")]
    pub glow_effect: bool,
    #[serde(default = "default_true")]
    pub pulse_effect: bool,
}

impl Default for RingMenuConfig {
    fn default() -> Self {
        Self {
            default_color: default_ring_color(),
            default_active_color: default_ring_active_color(),
            items: vec![],
            animation: RingAnimation::default(),
            trigger_mode: RingTriggerMode::default(),
            wheel_navigation: default_true(),
            max_slots: default_max_slots(),
            glow_effect: default_true(),
            pulse_effect: default_true(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RingMenuItemConfig {
    pub slot: u8,
    pub label: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<String, String>,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub action: Option<ButtonActionConfig>,
    #[serde(default)]
    pub items: Vec<SubMenuItemConfig>,
    #[serde(default = "default_auto_close")]
    pub auto_close: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub active_color: Option<String>,
}

impl RingMenuItemConfig {
    pub fn label_lang<'a>(&'a self, lang: &str) -> &'a str {
        let code = lang.split(&['_', '.'][..]).next().unwrap_or(lang).to_lowercase();
        if let Some(t) = self.translations.get(&code) {
            if !t.is_empty() {
                return t.as_str();
            }
        }
        &self.label
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SubMenuItemConfig {
    pub label: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<String, String>,
    #[serde(default = "default_icon")]
    pub icon: String,
    #[serde(default)]
    pub action: Option<ButtonActionConfig>,
    #[serde(default)]
    pub items: Vec<SubMenuItemConfig>,
    #[serde(default = "default_auto_close")]
    pub auto_close: bool,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub active_color: Option<String>,
}

impl SubMenuItemConfig {
    pub fn label_lang<'a>(&'a self, lang: &str) -> &'a str {
        let code = lang.split(&['_', '.'][..]).next().unwrap_or(lang).to_lowercase();
        if let Some(t) = self.translations.get(&code) {
            if !t.is_empty() {
                return t.as_str();
            }
        }
        &self.label
    }
}
