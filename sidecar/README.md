# Bun Plugin Host Sidecar

The Jereko sidecar is a Bun/TypeScript process that hosts TUI and server plugins with full JavaScript ecosystem fidelity. The Rust core (`jereko-cli`, `jereko-server`) owns HTTP, sessions, config, and provider routing; the sidecar owns plugin lifecycle and UI rendering.

## Why a Sidecar?

- **Plugin fidelity**: Existing OpenCode-compatible plugins expect Bun/Node APIs.
- **Isolation**: Plugin crashes do not take down the Rust runtime.
- **Default path**: Bun sidecar is the default TUI strategy. A native Rust TUI remains a documented future option only.

## IPC Contract

Transport: **JSON-lines over stdio** (one JSON object per line). Message `type` tags and field names use **snake_case** (canonical Rust serde contract).

Requires **Bun >= 1.1** (CI pins `1.2.5`).

### Rust → Sidecar

| Message | Fields | Description |
|---------|--------|-------------|
| `init` | `config`, `plugins[]` | Bootstrap with merged config and plugin list |
| `session_start` | `session_id` | Begin interactive session |
| `session_message` | `session_id`, `content` | User input |
| `tui_render` | `frame` | Terminal frame update |
| `shutdown` | — | Graceful teardown |

### Sidecar → Rust

| Message | Fields | Description |
|---------|--------|-------------|
| `ready` | — | Sidecar initialized |
| `tui_render` | `frame` | Terminal frame update |
| `plugin_event` | `plugin`, `event` | Plugin lifecycle/event hook |
| `error` | `message` | Fatal or recoverable error |
| `log` | `level`, `message` | Diagnostic log line |

## Running

```bash
cd sidecar
bun install
bun run start
bun test
bun run check
```

## Integration

`jereko run`:

1. Load merged config from Rust (`jereko-config`).
2. Spawn `bun run sidecar/src/index.ts` via `BunProcessSidecarPort`.
3. Exchange JSON-line messages over stdio.
4. Forward session events between HTTP core and TUI plugins.
5. Send `shutdown` for graceful exit.
