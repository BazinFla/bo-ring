use std::collections::HashMap;
use std::fs;
use serde::{Deserialize, Serialize};
use super::slugify;

pub mod ai;
pub mod developer;
pub mod media;
pub mod system;

pub use ai::*;
pub use developer::*;
pub use media::*;
pub use system::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ButtonActionConfig {
    ShowRingMenu,
    Command { cmd: String },
    KeyCombo { keys: Vec<String> },
    ActionRef { id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NamedActionConfig {
    pub id: String,
    pub label: String,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub translations: HashMap<String, String>,
    #[serde(default = "super::rings::default_icon")]
    pub icon: String,
    pub action: ButtonActionConfig,
}

impl NamedActionConfig {
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

pub fn default_actions_catalog() -> Vec<NamedActionConfig> {
    default_actions_catalog_for_lang("en")
}

pub fn default_actions_catalog_for_lang(lang: &str) -> Vec<NamedActionConfig> {
    let mut catalog = Vec::new();
    catalog.extend(media_actions_for_lang(lang));
    catalog.extend(system_actions_for_lang(lang));
    catalog.extend(developer_actions_for_lang(lang));
    catalog.extend(ai_actions_for_lang(lang));
    catalog
}

pub fn resolve_action(action: &ButtonActionConfig, catalog: &[NamedActionConfig]) -> ButtonActionConfig {
    match action {
        ButtonActionConfig::ActionRef { id } => {
            if let Some(named) = catalog.iter().find(|a| a.id == *id) {
                resolve_action(&named.action, catalog)
            } else {
                let path = super::Config::actions_dir().join(format!("{}.action.toml", slugify(id)));
                if path.exists() {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(named) = toml::from_str::<NamedActionConfig>(&content) {
                            return resolve_action(&named.action, catalog);
                        }
                    }
                }
                eprintln!("⚠️ ActionRef '{}' could not be resolved.", id);
                ButtonActionConfig::ShowRingMenu
            }
        }
        other => other.clone(),
    }
}
