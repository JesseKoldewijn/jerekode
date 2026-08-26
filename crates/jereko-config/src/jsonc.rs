//! Parse JSONC (JSON with comments and trailing commas) into strict JSON for serde.
use crate::error::{ConfigError, ConfigResult};
use jsonc_parser::{ParseOptions, parse_to_serde_value};

/// Strip JSONC comments/trailing commas and parse as `T`.
pub fn parse_jsonc<T: serde::de::DeserializeOwned>(raw: &str, path: &str) -> ConfigResult<T> {
    // Empty / whitespace-only JSONC deserializes as null for Value; treat as validation error.
    if raw.trim().is_empty() {
        return Err(ConfigError::Validation(format!("empty config: {path}")));
    }

    parse_to_serde_value(raw, &ParseOptions::default()).map_err(|e| ConfigError::Parse {
        path: path.to_string(),
        source: serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            e.to_string(),
        )),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OpenCodeConfig;

    #[test]
    fn parses_comments_and_trailing_commas() {
        let raw = r#"{
            // provider default
            "provider": "anthropic",
            "port": 4096,
        }"#;
        let parsed: OpenCodeConfig = parse_jsonc(raw, "test.jsonc").unwrap();
        assert_eq!(parsed.provider.as_deref(), Some("anthropic"));
        assert_eq!(parsed.port, Some(4096));
    }
}
