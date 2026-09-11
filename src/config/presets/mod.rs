use std::collections::HashMap;
use crate::config::rings::AppProfileConfig;

pub mod blender;
pub mod browser;
pub mod code;
pub mod default_config;
pub mod files;
pub mod gimp;

pub use blender::*;
pub use browser::*;
pub use code::*;
pub use default_config::*;
pub use files::*;
pub use gimp::*;

/// Looks up the translation struct for a given language code in a slice of `(lang_code, texts)`, falling back to the first entry (English).
pub fn find_texts_for_lang<'a, T>(translations: &'a [(&str, T)], lang: &str) -> &'a T {
    let code = lang.split(&['_', '.'][..]).next().unwrap_or(lang).to_lowercase();
    translations
        .iter()
        .find(|(k, _)| *k == code)
        .map(|(_, v)| v)
        .unwrap_or(&translations[0].1)
}

/// Constructs a `HashMap<String, String>` containing each language's localized string extracted via accessor `f`.
pub fn build_translation_map<T, F>(translations: &[(&str, T)], f: F) -> HashMap<String, String>
where
    F: Fn(&T) -> &str,
{
    let mut map = HashMap::new();
    for &(lang, ref texts) in translations {
        map.insert(lang.to_string(), f(texts).to_string());
    }
    map
}

pub fn default_app_profiles() -> Vec<AppProfileConfig> {
    default_app_profiles_for_lang("en")
}

pub fn default_app_profiles_for_lang(lang: &str) -> Vec<AppProfileConfig> {
    vec![
        browser_profile_for_lang(lang),
        code_profile_for_lang(lang),
        files_profile_for_lang(lang),
        blender_profile_for_lang(lang),
        gimp_profile_for_lang(lang),
    ]
}
