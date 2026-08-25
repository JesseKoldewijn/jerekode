# Bun Plugin Host Sidecar

The Jereko sidecar is a Bun/TypeScript process that hosts TUI and server plugins with full JavaScript ecosystem fidelity. The Rust core (`jereko-cli`, `jereko-server`) owns HTTP, sessions, config, and provider routing; the sidecar owns plugin lifecycle and UI rendering.

## Why a Sidecar?

- **Plugin fidelity**: Existing OpenCode-compatible plugins expect Bun/Node APIs.
- **Isolation**: Plugin crashes do not take down the Rust runtime.
- **Default path**: Bun sidecar is the default TUI strategy. A native Rust TUI remains a documented future option only.

## IPC Contract (Phase 1)

Transport: **JSON-lines over stdio** (one JSON object per line). Unix domain socket is an alternative for local dev.

### Rust → Sidecar

| Message | Fields | Description |
|---------|--------|-------------|
| `init` | `config`, `plugins[]` | Bootstrap with merged config and plugin list |
| `session.start` | `sessionId` | Begin interactive session |
| `session.message` | `sessionId`, `content` | User input |
| `shutdown` | — | Graceful teardown |

### Sidecar → Rust

| Message | Fields | Description |
|---------|--------|-------------|
| `ready` | — | Sidecar initialized |
| `tui.render` | `frame` | Terminal frame update |
| `plugin.event` | `plugin`, `event` | Plugin lifecycle/event hook |
| `error` | `message` | Fatal or recoverable error |

## Running (stub)

```bash
cd sidecar
bun install
bun run start
```

## Phase 2 Integration

`jereko run` will:

1. Load merged config from Rust (`jereko-config`).
2. Spawn `bun run sidecar/src/index.ts` as a child process.
3. Exchange JSON-line messages over stdio.
4. Forward session events between HTTP core and TUI plugins.
