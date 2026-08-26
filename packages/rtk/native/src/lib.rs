//! Native RTK plugin — shares `../rules/commands.json` with the OpenCode2 entry.

use regex::Regex;
use serde::Deserialize;
use serde_json::Value;

const RULES_JSON: &str = include_str!("../../rules/commands.json");

#[derive(Debug, Deserialize)]
struct RewriteRules {
    prefix: String,
    rewrites: Vec<RewriteRule>,
}

#[derive(Debug, Deserialize)]
struct RewriteRule {
    #[serde(rename = "match")]
    pattern: String,
    mode: String,
}

fn load_rules() -> RewriteRules {
    serde_json::from_str(RULES_JSON).expect("embedded commands.json must parse")
}

fn already_rtk(command: &str) -> bool {
    let trimmed = command.trim_start();
    trimmed == "rtk" || trimmed.starts_with("rtk ")
}

/// Table rewrite — CI source of truth (does not require `rtk` on PATH).
pub fn rewrite_with_table(command: &str) -> String {
    let trimmed = command.trim();
    if trimmed.is_empty() || already_rtk(trimmed) {
        return command.to_string();
    }
    let rules = load_rules();
    for rule in &rules.rewrites {
        if rule.mode != "prefix" {
            continue;
        }
        if let Ok(re) = Regex::new(&rule.pattern)
            && re.is_match(trimmed)
        {
            return format!("{} {}", rules.prefix, trimmed);
        }
    }
    command.to_string()
}

fn extract_command(payload: &Value) -> Option<String> {
    if let Some(c) = payload.get("command").and_then(|v| v.as_str()) {
        return Some(c.to_string());
    }
    payload
        .get("args")
        .and_then(|a| a.get("command"))
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn is_bash_tool(payload: &Value) -> bool {
    let tool = payload
        .get("tool")
        .or_else(|| payload.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    matches!(tool.to_ascii_lowercase().as_str(), "bash" | "shell")
}

fn handle_tool_execute_before(payload: Value) -> Value {
    if !is_bash_tool(&payload) {
        let mut out = payload;
        if let Some(obj) = out.as_object_mut() {
            obj.insert("rewritten".into(), Value::Bool(false));
            obj.insert("host".into(), Value::String("native".into()));
            obj.insert("hook".into(), Value::String("tool.execute.before".into()));
            obj.insert("status".into(), Value::String("ok".into()));
            obj.insert("stub".into(), Value::Bool(false));
        }
        return out;
    }
    let command = extract_command(&payload).unwrap_or_default();
    let next = rewrite_with_table(&command);
    let rewritten = next != command;
    let mut out = payload;
    if let Some(obj) = out.as_object_mut() {
        obj.insert("command".into(), Value::String(next.clone()));
        obj.insert("rewritten".into(), Value::Bool(rewritten));
        obj.insert("host".into(), Value::String("native".into()));
        obj.insert("hook".into(), Value::String("tool.execute.before".into()));
        obj.insert("status".into(), Value::String("ok".into()));
        obj.insert("stub".into(), Value::Bool(false));
        if let Some(args) = obj.get_mut("args").and_then(|a| a.as_object_mut()) {
            args.insert("command".into(), Value::String(next));
        }
    }
    out
}

fn handle_hook(hook: &str, payload: Value) -> Result<Value, i32> {
    match hook {
        "tool.execute.before" => Ok(handle_tool_execute_before(payload)),
        "before_transform" => {
            let input = payload
                .get("input")
                .or_else(|| payload.get("command"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let next = rewrite_with_table(input);
            Ok(serde_json::json!({
                "host": "native",
                "hook": "before_transform",
                "transformed": next,
                "stub": false,
                "status": "ok"
            }))
        }
        _ => Ok(serde_json::json!({
            "host": "native",
            "hook": hook,
            "stub": false,
            "status": "ok",
            "skipped": true
        })),
    }
}

jereko_plugin_sdk::export_plugin!("jereko-rtk", "0.0.1", handle_hook);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewrites_git_status() {
        assert_eq!(rewrite_with_table("git status"), "rtk git status");
    }

    #[test]
    fn passthrough_echo() {
        assert_eq!(rewrite_with_table("echo hi"), "echo hi");
    }

    #[test]
    fn tool_execute_before_mutates_bash() {
        let out = handle_tool_execute_before(serde_json::json!({
            "tool": "bash",
            "command": "git diff"
        }));
        assert_eq!(out["command"], "rtk git diff");
        assert_eq!(out["rewritten"], true);
        assert_eq!(out["host"], "native");
    }
}
