/**
 * Jereko Bun Sidecar — Plugin Host Entry Point (Phase 0 stub)
 *
 * The sidecar runs as a child process of the `jereko` CLI. It hosts TUI and
 * server plugins with full Bun/TypeScript fidelity while the Rust core handles
 * HTTP, sessions, and provider routing.
 *
 * ## IPC Contract (Phase 1)
 *
 * Transport: stdio JSON-lines (one message per line) or Unix domain socket.
 *
 * ### Rust → Sidecar messages
 * - `{ "type": "init", "config": { ... }, "plugins": [ ... ] }`
 * - `{ "type": "session.start", "sessionId": "..." }`
 * - `{ "type": "session.message", "sessionId": "...", "content": "..." }`
 * - `{ "type": "shutdown" }`
 *
 * ### Sidecar → Rust messages
 * - `{ "type": "ready" }`
 * - `{ "type": "tui.render", "frame": { ... } }`
 * - `{ "type": "plugin.event", "plugin": "...", "event": { ... } }`
 * - `{ "type": "error", "message": "..." }`
 *
 * See `sidecar/README.md` for full contract documentation.
 */

export type SidecarMessage =
  | { type: "ready" }
  | { type: "error"; message: string }
  | { type: "log"; level: "info" | "warn" | "error"; message: string };

export interface SidecarOptions {
  /** Path to opencode.json-derived config passed from Rust */
  configPath?: string;
}

/**
 * Start the plugin host sidecar (stub).
 */
export function startSidecar(options: SidecarOptions = {}): void {
  const msg: SidecarMessage = {
    type: "log",
    level: "info",
    message: `jereko sidecar stub started (config: ${options.configPath ?? "none"})`,
  };
  console.log(JSON.stringify(msg));
  console.log(JSON.stringify({ type: "ready" } satisfies SidecarMessage));
}

if (import.meta.main) {
  startSidecar({ configPath: process.env.JEREKO_CONFIG_PATH });
}
