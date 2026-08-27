//! SSE / NDJSON stream parsing helpers for provider adapters.

use crate::error::{ProviderError, ProviderResult};
use crate::provider::CompletionChunk;

/// Parse OpenAI-compatible `text/event-stream` chat completion chunks.
pub fn parse_openai_sse(body: &str, model: &str) -> ProviderResult<Vec<CompletionChunk>> {
    let mut chunks = Vec::new();
    for block in body.split("\n\n") {
        for line in block.lines() {
            let line = line.trim();
            if !line.starts_with("data:") {
                continue;
            }
            let data = line.trim_start_matches("data:").trim();
            if data.is_empty() || data == "[DONE]" {
                continue;
            }
            let value: serde_json::Value =
                serde_json::from_str(data).map_err(|e| ProviderError::ProviderFailure {
                    provider: "openai".into(),
                    message: format!("invalid SSE JSON: {e}"),
                })?;
            let delta = value
                .pointer("/choices/0/delta/content")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let finish = value
                .pointer("/choices/0/finish_reason")
                .and_then(|v| v.as_str())
                .map(str::to_string);
            if delta.is_empty() && finish.is_none() {
                continue;
            }
            chunks.push(CompletionChunk {
                delta,
                finish_reason: finish,
                model: model.to_string(),
            });
        }
    }
    Ok(chunks)
}

/// Parse Anthropic Messages API SSE stream.
pub fn parse_anthropic_sse(body: &str, model: &str) -> ProviderResult<Vec<CompletionChunk>> {
    let mut chunks = Vec::new();
    for block in body.split("\n\n") {
        let mut data_line = None;
        for line in block.lines() {
            let line = line.trim();
            if let Some(rest) = line.strip_prefix("data:") {
                data_line = Some(rest.trim());
            }
        }
        let Some(data) = data_line else {
            continue;
        };
        if data.is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(data).map_err(|e| ProviderError::ProviderFailure {
                provider: "anthropic".into(),
                message: format!("invalid SSE JSON: {e}"),
            })?;
        let event_type = value.get("type").and_then(|v| v.as_str()).unwrap_or("");
        match event_type {
            "content_block_delta" => {
                let delta = value
                    .pointer("/delta/text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                if !delta.is_empty() {
                    chunks.push(CompletionChunk {
                        delta,
                        finish_reason: None,
                        model: model.to_string(),
                    });
                }
            }
            "message_delta" => {
                let finish = value
                    .pointer("/delta/stop_reason")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                if finish.is_some() {
                    chunks.push(CompletionChunk {
                        delta: String::new(),
                        finish_reason: finish,
                        model: model.to_string(),
                    });
                }
            }
            _ => {}
        }
    }
    Ok(chunks)
}

/// Parse Ollama NDJSON chat stream (`stream: true`).
pub fn parse_ollama_ndjson(body: &str, model: &str) -> ProviderResult<Vec<CompletionChunk>> {
    let mut chunks = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value =
            serde_json::from_str(line).map_err(|e| ProviderError::ProviderFailure {
                provider: "ollama".into(),
                message: format!("invalid NDJSON: {e}"),
            })?;
        let delta = value
            .pointer("/message/content")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let done = value.get("done").and_then(|v| v.as_bool()).unwrap_or(false);
        let finish = if done { Some("stop".into()) } else { None };
        if delta.is_empty() && finish.is_none() {
            continue;
        }
        chunks.push(CompletionChunk {
            delta,
            finish_reason: finish,
            model: model.to_string(),
        });
    }
    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_openai_sse() {
        let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n\
data: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\n\
data: [DONE]\n\n";
        let chunks = parse_openai_sse(body, "gpt").unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].delta, "Hi");
        assert_eq!(chunks[1].finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn parses_anthropic_sse() {
        let body = "event: content_block_delta\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Yo\"}}\n\n\
event: message_delta\ndata: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n";
        let chunks = parse_anthropic_sse(body, "claude").unwrap();
        assert_eq!(chunks[0].delta, "Yo");
        assert_eq!(chunks[1].finish_reason.as_deref(), Some("end_turn"));
    }

    #[test]
    fn parses_ollama_ndjson() {
        let body = "{\"message\":{\"content\":\"a\"},\"done\":false}\n{\"message\":{\"content\":\"b\"},\"done\":true}\n";
        let chunks = parse_ollama_ndjson(body, "llama").unwrap();
        assert_eq!(chunks[0].delta, "a");
        assert_eq!(chunks[1].delta, "b");
        assert_eq!(chunks[1].finish_reason.as_deref(), Some("stop"));
    }
}
