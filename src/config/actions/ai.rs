use super::{ButtonActionConfig, NamedActionConfig};
use crate::config::presets::{build_translation_map, find_texts_for_lang};

#[derive(Debug, Clone, Copy)]
pub struct AiActionTexts {
    pub chatgpt: &'static str,
    pub perplexity: &'static str,
    pub claude: &'static str,
    pub deepseek: &'static str,
}

pub const AI_TRANSLATIONS: &[(&str, AiActionTexts)] = &[
    (
        "en",
        AiActionTexts {
            chatgpt: "Open ChatGPT",
            perplexity: "Open Perplexity",
            claude: "Open Claude AI",
            deepseek: "Open DeepSeek",
        },
    ),
    (
        "fr",
        AiActionTexts {
            chatgpt: "Ouvrir ChatGPT",
            perplexity: "Ouvrir Perplexity",
            claude: "Ouvrir Claude AI",
            deepseek: "Ouvrir DeepSeek",
        },
    ),
    (
        "es",
        AiActionTexts {
            chatgpt: "Abrir ChatGPT",
            perplexity: "Abrir Perplexity",
            claude: "Abrir Claude AI",
            deepseek: "Abrir DeepSeek",
        },
    ),
];

pub fn ai_action_texts(lang: &str) -> &'static AiActionTexts {
    find_texts_for_lang(AI_TRANSLATIONS, lang)
}

#[allow(dead_code)]
pub fn ai_actions() -> Vec<NamedActionConfig> {
    ai_actions_for_lang("en")
}

pub fn ai_actions_for_lang(lang: &str) -> Vec<NamedActionConfig> {
    let t = ai_action_texts(lang);
    vec![
        NamedActionConfig {
            id: "ai_chatgpt".to_string(),
            label: t.chatgpt.to_string(),
            translations: build_translation_map(AI_TRANSLATIONS, |x| x.chatgpt),
            icon: "🤖".to_string(),
            action: ButtonActionConfig::Command { cmd: "xdg-open https://chatgpt.com".to_string() },
        },
        NamedActionConfig {
            id: "ai_perplexity".to_string(),
            label: t.perplexity.to_string(),
            translations: build_translation_map(AI_TRANSLATIONS, |x| x.perplexity),
            icon: "✳️".to_string(),
            action: ButtonActionConfig::Command { cmd: "xdg-open https://perplexity.ai".to_string() },
        },
        NamedActionConfig {
            id: "ai_claude".to_string(),
            label: t.claude.to_string(),
            translations: build_translation_map(AI_TRANSLATIONS, |x| x.claude),
            icon: "🟣".to_string(),
            action: ButtonActionConfig::Command { cmd: "xdg-open https://claude.ai".to_string() },
        },
        NamedActionConfig {
            id: "ai_deepseek".to_string(),
            label: t.deepseek.to_string(),
            translations: build_translation_map(AI_TRANSLATIONS, |x| x.deepseek),
            icon: "⚡".to_string(),
            action: ButtonActionConfig::Command { cmd: "xdg-open https://chat.deepseek.com".to_string() },
        },
    ]
}
