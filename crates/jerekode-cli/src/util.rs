//! Shared CLI helpers (model flag parsing, provider registry selection).

use jerekode_providers::ProviderRegistry;

/// Parse OpenCode-style `--model provider/model`.
///
/// Returns `(provider, model)` when a `/` is present; otherwise `None`
/// (caller treats the value as a bare model id).
pub fn parse_provider_model(spec: &str) -> Option<(String, String)> {
    let (provider, model) = spec.split_once('/')?;
    if provider.is_empty() || model.is_empty() {
        return None;
    }
    Some((provider.to_string(), model.to_string()))
}

/// Resolve provider + model from split flags and optional `provider/model` form.
pub fn resolve_provider_model(
    provider_flag: Option<String>,
    model_flag: Option<String>,
) -> (Option<String>, Option<String>) {
    match model_flag {
        Some(m) => {
            if let Some((p, model)) = parse_provider_model(&m) {
                (provider_flag.or(Some(p)), Some(model))
            } else {
                (provider_flag, Some(m))
            }
        }
        None => (provider_flag, None),
    }
}

/// Prefer stub registry when `JEREKO_USE_STUB_PROVIDERS` is set (tests / offline).
pub fn provider_registry() -> ProviderRegistry {
    if std::env::var("JEREKO_USE_STUB_PROVIDERS").is_ok() {
        ProviderRegistry::with_stubs()
    } else {
        ProviderRegistry::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_provider_model_splits() {
        assert_eq!(
            parse_provider_model("anthropic/claude-3-5-sonnet"),
            Some(("anthropic".into(), "claude-3-5-sonnet".into()))
        );
        assert_eq!(parse_provider_model("bare-model"), None);
        assert_eq!(parse_provider_model("/nope"), None);
        assert_eq!(parse_provider_model("nope/"), None);
    }

    #[test]
    fn resolve_prefers_model_slash_form() {
        let (p, m) = resolve_provider_model(None, Some("openai/gpt-4o".into()));
        assert_eq!(p.as_deref(), Some("openai"));
        assert_eq!(m.as_deref(), Some("gpt-4o"));
    }
}
