//! Server-Sent Events helpers for streaming completions.

use jerekode_core::Message;
use jerekode_providers::CompletionChunk;
use serde_json::json;

/// Build a `text/event-stream` body from completion chunks + final assistant message.
pub fn format_completion_sse(chunks: &[CompletionChunk], assistant: &Message) -> String {
    let mut out = String::new();
    for chunk in chunks {
        let data = serde_json::to_string(chunk).unwrap_or_else(|_| "{}".into());
        out.push_str("event: chunk\ndata: ");
        out.push_str(&data);
        out.push_str("\n\n");
    }
    let done = json!({
        "content": assistant.content,
        "role": "assistant",
    });
    out.push_str("event: done\ndata: ");
    out.push_str(&done.to_string());
    out.push_str("\n\n");
    out
}
