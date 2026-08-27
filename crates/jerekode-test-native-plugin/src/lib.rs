fn handle_before_transform(
    hook: &str,
    payload: serde_json::Value,
) -> Result<serde_json::Value, i32> {
    if hook == "before_transform" {
        let input = payload.get("input").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "host": "native",
            "hook": "before_transform",
            "transformed": input,
            "stub": false
        }))
    } else {
        Ok(serde_json::json!({
            "host": "native",
            "hook": hook,
            "stub": false
        }))
    }
}

jerekode_plugin_sdk::export_plugin!("native-tools", "0.1.0", handle_before_transform);
