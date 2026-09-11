use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::i18n::tr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonCalloutSide {
    Left,
    Right,
}

fn default_btn_icon() -> String {
    "🔘".to_string()
}

fn default_btn_side() -> ButtonCalloutSide {
    ButtonCalloutSide::Left
}

fn default_anchor() -> [f32; 2] {
    [0.5, 0.5]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceButtonConfig {
    pub code: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name_key: Option<String>,
    #[serde(default)]
    pub default_name: String,
    #[serde(default = "default_btn_icon")]
    pub icon: String,
    #[serde(default = "default_btn_side")]
    pub side: ButtonCalloutSide,
    #[serde(default = "default_anchor")]
    pub anchor: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_y_ratio: Option<f32>,
}

impl DeviceButtonConfig {
    pub fn parsed_cid(&self) -> Option<u16> {
        let cid_str = self.cid.as_deref()?.trim();
        if cid_str.starts_with("0x") || cid_str.starts_with("0X") {
            u16::from_str_radix(&cid_str[2..], 16).ok()
        } else {
            cid_str.parse::<u16>().ok()
        }
    }
    pub fn display_name(&self, lang: &str) -> String {
        if let Some(ref key) = self.name_key {
            let translated = tr(lang, key);
            if !translated.is_empty() && translated != key {
                return translated.to_string();
            }
        }
        if !self.default_name.is_empty() {
            self.default_name.clone()
        } else {
            crate::devices::get_button_name_lang(self.code, lang)
        }
    }

    pub fn anchor_vec2(&self) -> egui::Vec2 {
        egui::vec2(self.anchor[0], self.anchor[1])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModelConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub match_ids: Vec<String>,
    #[serde(default)]
    pub match_names: Vec<String>,
    pub image: String,
    pub buttons: Vec<DeviceButtonConfig>,
}

impl DeviceModelConfig {
    pub fn find_button(&self, code: u16) -> Option<&DeviceButtonConfig> {
        self.buttons.iter().find(|b| b.code == code)
    }

    /// Returns all declared (Control ID, button_code) diversion pairs for this device model.
    pub fn diverted_cids(&self) -> Vec<(u16, u16)> {
        self.buttons
            .iter()
            .filter_map(|btn| btn.parsed_cid().map(|cid| (cid, btn.code)))
            .collect()
    }

    pub fn default_anchor_positions(&self) -> HashMap<u16, egui::Vec2> {
        let mut map = HashMap::new();
        for btn in &self.buttons {
            map.insert(btn.code, btn.anchor_vec2());
        }
        map
    }

    pub fn matches_hardware_id(&self, vid_pid: &str) -> bool {
        let clean = vid_pid.trim().to_lowercase();
        self.match_ids
            .iter()
            .any(|id| id.trim().to_lowercase() == clean)
    }

    pub fn matches_device_name(&self, dev_name: &str) -> bool {
        let name_lower = dev_name.to_lowercase();
        for pattern in &self.match_names {
            if name_lower.contains(&pattern.to_lowercase()) {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_device_model() {
        let json_str = r#"{
            "id": "test_mouse",
            "name": "Test Mouse",
            "author": "Tester",
            "match_ids": ["046d:c548"],
            "match_names": ["Test Mouse", "Tester 1000"],
            "image": "mouse.webp",
            "buttons": [
                {
                    "code": 274,
                    "name_key": "btn_middle_wheel",
                    "default_name": "Middle Click",
                    "icon": "🖱️",
                    "side": "right",
                    "anchor": [0.5, 0.3],
                    "badge_y_ratio": 0.25
                },
                {
                    "code": 275,
                    "default_name": "Back",
                    "icon": "◀️",
                    "side": "left",
                    "anchor": [0.4, 0.5]
                }
            ]
        }"#;

        let model: DeviceModelConfig = serde_json::from_str(json_str).expect("Valid JSON");
        assert_eq!(model.id, "test_mouse");
        assert_eq!(model.buttons.len(), 2);
        assert!(model.matches_hardware_id("046d:c548"));
        assert!(model.matches_hardware_id("046D:C548"));
        assert!(!model.matches_hardware_id("046d:b034"));
        assert!(model.matches_device_name("Logitech Tester 1000 Wireless"));
        assert!(!model.matches_device_name("Keyboard"));

        let anchors = model.default_anchor_positions();
        assert_eq!(anchors.get(&274), Some(&egui::vec2(0.5, 0.3)));
    }

    #[test]
    fn test_device_model_button_add_and_remove() {
        let mut model = DeviceModelConfig {
            id: "custom".to_string(),
            name: "Custom Mouse".to_string(),
            author: None,
            match_ids: vec![],
            match_names: vec![],
            image: "mouse.png".to_string(),
            buttons: vec![
                DeviceButtonConfig {
                    code: 274,
                    cid: None,
                    name_key: None,
                    default_name: "Middle".to_string(),
                    icon: "🖱️".to_string(),
                    side: ButtonCalloutSide::Right,
                    anchor: [0.5, 0.3],
                    badge_y_ratio: None,
                },
            ],
        };

        // Add a new button with CID
        let new_btn = DeviceButtonConfig {
            code: 280,
            cid: Some("0x00C3".to_string()),
            name_key: None,
            default_name: "Top Button".to_string(),
            icon: "🔝".to_string(),
            side: ButtonCalloutSide::Left,
            anchor: [0.5, 0.5],
            badge_y_ratio: None,
        };
        model.buttons.push(new_btn);
        assert_eq!(model.buttons.len(), 2);
        assert!(model.find_button(280).is_some());

        let anchors = model.default_anchor_positions();
        assert_eq!(anchors.get(&280), Some(&egui::vec2(0.5, 0.5)));

        // Remove button
        model.buttons.retain(|b| b.code != 274);
        assert_eq!(model.buttons.len(), 1);
        assert_eq!(model.buttons[0].code, 280);
        assert_eq!(model.diverted_cids(), vec![(0x00C3, 280)]);

        // Serialize roundtrip
        let serialized = serde_json::to_string(&model).expect("Serialization failed");
        let deserialized: DeviceModelConfig = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.buttons.len(), 1);
        assert_eq!(deserialized.buttons[0].code, 280);
        assert_eq!(deserialized.diverted_cids(), vec![(0x00C3, 280)]);
    }
}
