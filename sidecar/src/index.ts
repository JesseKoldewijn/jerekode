/**
 * Jereko Bun Sidecar — Plugin Host Entry Point
 *
 * JSON-line IPC over stdio (one message per line).
 * Field names use snake_case to match Rust serde (`rename_all = "snake_case"`).
 */

export type SidecarOutbound =
  | { type: "init"; config: Record<string, unknown>; plugins: string[] }
  | { type: "session_start"; session_id: string }
  | { type: "session_message"; session_id: string; content: string }
  | { type: "tui_render"; frame: Record<string, unknown> }
  | { type: "shutdown" };

export type SidecarInbound =
  | { type: "ready" }
  | { type: "tui_render"; frame: Record<string, unknown> }
  | { type: "plugin_event"; plugin: string; event: Record<string, unknown> }
  | { type: "error"; message: string }
  | { type: "log"; level: "info" | "warn" | "error"; message: string };

export interface SidecarOptions {
  configPath?: string;
  onMessage?: (msg: SidecarOutbound) => void;
}

function emit(msg: SidecarInbound): void {
  console.log(JSON.stringify(msg));
}

function handleOutbound(msg: SidecarOutbound): void {
  switch (msg.type) {
    case "init":
      emit({ type: "log", level: "info", message: `loaded ${msg.plugins.length} plugins` });
      emit({ type: "ready" });
      break;
    case "session_start":
      emit({ type: "log", level: "info", message: `session ${msg.session_id} started` });
      break;
    case "session_message":
      emit({
        type: "plugin_event",
        plugin: "sidecar",
        event: { session_id: msg.session_id, content: msg.content },
      });
      break;
    case "tui_render":
      emit({ type: "tui_render", frame: msg.frame });
      break;
    case "shutdown":
      emit({ type: "log", level: "info", message: "sidecar shutting down" });
      process.exit(0);
      break;
  }
}

/** Start the plugin host sidecar — reads JSON-line commands from stdin. */
export function startSidecar(options: SidecarOptions = {}): void {
  emit({
    type: "log",
    level: "info",
    message: `jereko sidecar started (config: ${options.configPath ?? "none"})`,
  });
  emit({ type: "ready" });

  if (import.meta.main) {
    const decoder = new TextDecoder();
    let buffer = "";

    Bun.stdin.stream().pipeTo(
      new WritableStream({
        write(chunk) {
          buffer += decoder.decode(chunk, { stream: true });
          const lines = buffer.split("\n");
          buffer = lines.pop() ?? "";
          for (const line of lines) {
            if (!line.trim()) continue;
            try {
              const msg = JSON.parse(line) as SidecarOutbound;
              options.onMessage?.(msg);
              handleOutbound(msg);
            } catch (err) {
              emit({
                type: "error",
                message: `invalid JSON-line: ${String(err)}`,
              });
            }
          }
        },
      }),
    );
  }
}

if (import.meta.main) {
  startSidecar({ configPath: process.env.JEREKO_CONFIG_PATH });
}
