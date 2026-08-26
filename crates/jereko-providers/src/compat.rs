//! OpenAI-compatible provider clones (Groq, OpenRouter, etc.).

use crate::openai::OpenAiProvider;
use crate::provider::SharedHttpClient;

/// Groq — OpenAI-compatible Chat Completions at `https://api.groq.com/openai/v1`.
pub fn groq_provider(http: SharedHttpClient) -> OpenAiProvider {
    OpenAiProvider::new(http)
        .with_base_url("https://api.groq.com/openai/v1")
        .with_api_key_env("GROQ_API_KEY")
        .with_id("groq")
        .with_display_name("Groq")
}

/// OpenRouter — OpenAI-compatible gateway at `https://openrouter.ai/api/v1`.
pub fn openrouter_provider(http: SharedHttpClient) -> OpenAiProvider {
    OpenAiProvider::new(http)
        .with_base_url("https://openrouter.ai/api/v1")
        .with_api_key_env("OPENROUTER_API_KEY")
        .with_id("openrouter")
        .with_display_name("OpenRouter")
}
