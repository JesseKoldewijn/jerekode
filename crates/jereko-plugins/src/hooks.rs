//! OpenCode-compatible tool hook helpers.

use crate::types::{HookCall, HookResult};
use serde_json::{Value, json};

/// OpenCode / Jereko hook invoked before a tool runs.
pub const TOOL_EXECUTE_BEFORE: &str = "tool.execute.before";

/// Build a `tool.execute.before` hook call for a bash/shell command.
pub fn bash_before_hook(command: impl Into<String>) -> HookCall {
    let command = command.into();
    HookCall {
        hook: TOOL_EXECUTE_BEFORE.into(),
        payload: json!({
            "tool": "bash",
            "name": "bash",
            "command": command,
            "args": { "command": command },
        }),
    }
}

/// Apply the last non-skipped command mutation from hook results.
///
/// Plugins may return `command` at the top level and/or under `args.command`.
pub fn apply_command_mutations(mut command: String, results: &[HookResult]) -> String {
    for result in results {
        let out = &result.output;
        if out.get("skipped").and_then(|v| v.as_bool()) == Some(true) {
            continue;
        }
        if let Some(next) = out.get("command").and_then(|v| v.as_str()) {
            command = next.to_string();
            continue;
        }
        if let Some(next) = out
            .get("args")
            .and_then(|a| a.get("command"))
            .and_then(|v| v.as_str())
        {
            command = next.to_string();
        }
    }
    command
}

/// Merge a rewritten command back into a tool-call style JSON args object.
pub fn set_command_arg(args: &mut Value, command: &str) {
    if let Some(obj) = args.as_object_mut() {
        obj.insert("command".into(), Value::String(command.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HookResult;

    #[test]
    fn applies_last_command_mutation() {
        let results = vec![
            HookResult {
                plugin: "a".into(),
                output: json!({"command": "rtk git status"}),
            },
            HookResult {
                plugin: "b".into(),
                output: json!({"skipped": true}),
            },
        ];
        assert_eq!(
            apply_command_mutations("git status".into(), &results),
            "rtk git status"
        );
    }
}
