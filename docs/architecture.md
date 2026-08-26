# Jereko Architecture

Jereko is an AI coding agent runtime built as a **Rust core + Bun sidecar** architecture. It targets OpenCode API compatibility without vendoring upstream source code.

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                        jereko CLI                           │
│  (primary binary: `jereko`; aliases: opencode, opencode2)   │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌──────────────────────────┐    ┌────────────────────────────┐
│     jereko-server        │    │   Bun Sidecar (sidecar/)   │
│  HTTP + adapter layer    │◄──►│  TUI + server plugins      │
│  v1 / v2 normalization   │    │  JSON-line IPC             │
└──────────────┬───────────┘    └────────────────────────────┘
               │
               ▼
┌──────────────────────────┐    ┌────────────────────────────┐
│     jereko-core          │    │   jereko-providers         │
│  sessions, messages      │    │  75+ provider registry     │
└──────────────────────────┘    └────────────────────────────┘
               │
               ▼
┌──────────────────────────┐
│     jereko-config        │
│  JSONC config + merge    │
└──────────────────────────┘
```

## Crate Responsibilities

| Crate | Role |
|-------|------|
| `jereko-core` | Domain types, session models, shared errors |
| `jereko-config` | Config loading, precedence merge, `opencode.json` / `tui.json` types |
| `jereko-server` | Axum HTTP server, v1/v2 adapter layer |
| `jereko-cli` | CLI entry point (`serve`, `run`, `version`) |
| `jereko-providers` | Provider trait, registry (designed for 75+ providers) |
| `conformance` | Owned fixture-driven compatibility tests |

## Terminology: Adapters vs Seams

Jereko uses "adapter" in three distinct senses. Each maps to a **seam** in [codebase-design](../.agents/skills/codebase-design/SKILL.md) vocabulary (two adapters = real seam):

| Concept | Location | Role |
|---------|----------|------|
| **Wire adapter** | `jereko-server/src/adapters/v1/`, `v2/` | Translates v1 or v2 HTTP wire format ↔ normalized types |
| **Provider adapter** | `jereko-providers` (`Provider` trait) | Implements LLM backend behavior (Anthropic, OpenAI, `StubProvider`, etc.) |
| **Sidecar adapter** | Rust ↔ Bun IPC | Transport for BunPluginHost — production (spawn Bun) or test (in-memory) |
| **Plugin host** | `PluginOrchestrator` | `PluginHost` trait — Bun, native dylib, or WASM implementations |

**Seam** = where a module's interface lives. **Adapter** = concrete implementation at that seam.

Do not conflate wire adapters (HTTP version translation) with provider adapters (LLM backends) or sidecar adapters (process IPC). Each seam has its own interface and test strategy — see [conformance.md](./conformance.md).

## Module Seams

Map of crates to codebase-design vocabulary:

| Crate / module | Seam | Interface | Adapters |
|----------------|------|-----------|----------|
| `jereko-server/adapters/` | HTTP wire normalization | Normalized request/response types | v1 wire adapter, v2 wire adapter |
| `jereko-server/router` | HTTP routing | Axum routes on normalized types | (single implementation; tested in-process) |
| `jereko-providers` | LLM provider | `Provider` trait | Per-provider HTTP adapters, `StubProvider` |
| `jereko-config` | Configuration | Merge loader API | File/env/CLI sources (Phase 1) |
| `jereko-core` | Domain | Session, message types | (pure domain; no external adapters) |
| Plugin orchestrator (Phase 2) | Plugin hook dispatch | `PluginOrchestrator`, `PluginHost` trait | BunPluginHost, (Phase 2.5) NativePluginHost, (Phase 4) WasmPluginHost |
| Sidecar IPC (Phase 2) | Sidecar transport | `SidecarPort` trait | Bun process adapter (feeds BunPluginHost), in-memory test adapter |

**Depth goal:** handlers and domain logic stay deep (small interface, lots of behavior hidden). Wire and provider adapters stay thin (translate format, delegate).

## HTTP Adapter Layer

Both v1 and v2 HTTP APIs are supported through a **pluggable adapter layer** that normalizes requests and responses as early as possible:

```text
Client (v1) ──► adapters/v1 ──► adapters/normalized ──► handlers
Client (v2) ──► adapters/v2 ──► adapters/normalized ──► handlers
```

**Design goals:**

- Handlers operate exclusively on `adapters/normalized` types.
- Version-specific serde shapes stay inside `adapters/v1` and `adapters/v2`.
- Deprecating v1 later means removing the v1 adapter module, not rewriting core logic.

## Binary Naming

- **Primary binary**: `jereko`
- **Optional aliases**: `opencode`, `opencode2` (symlinks or install-time aliases)

Aliases are not separate implementations — they point to the same `jereko` binary. See the root README for alias setup instructions.

## Plugin Orchestrator & Dual Hosts

Plugins are loaded and dispatched through a **PluginOrchestrator** in Rust that coordinates multiple **PluginHost** implementations. Bun and native hosts run **together** in an ordered hook chain — see [ADR 002](./adr/002-dual-plugin-runtime.md).

```text
┌─────────────────────────────────────────────────────────────────┐
│                     PluginOrchestrator (Rust)                   │
│  hook registry · load order · priority · capability merge       │
└──────────┬──────────────────────┬──────────────────────┬────────┘
           │                      │                      │
           ▼                      ▼                      ▼
┌──────────────────┐  ┌──────────────────────┐  ┌──────────────────┐
│ Internal hooks   │  │  NativePluginHost    │  │  BunPluginHost   │
│ (built-in)       │  │  dylib / C ABI       │  │  SidecarPort IPC │
│                  │  │  Phase 2.5+          │  │  default path    │
└──────────────────┘  └──────────────────────┘  └────────┬─────────┘
                                                         │
                                                         ▼
                                            ┌──────────────────────┐
                                            │  Bun sidecar process │
                                            └──────────────────────┘

Phase 4 (optional): WasmPluginHost — sandboxed untrusted plugins
```

### Host Types

| Host | Config form | Phase | Role |
|------|-------------|-------|------|
| **BunPluginHost** | `"@acme/server-plugin"` (unqualified string, default) | 2 | Full OpenCode/Bun fidelity via sidecar IPC |
| **NativePluginHost** | `{ "native": "./path/to/plugin.so" }` | 2.5 | In-process dylib; server hooks (tools, providers, transforms) |
| **WasmPluginHost** | `{ "wasm": "./path/to/plugin.wasm" }` | 4 | Sandboxed untrusted plugins |

**Load order:** internal → native → bun. The orchestrator builds a single ordered hook chain across all active hosts with failure isolation per plugin.

**TUI plugins:** Bun-only until Phase 5, when a `TuiPluginHost` trait and optional native bridge are introduced.

### SidecarPort → BunPluginHost

`SidecarPort` remains the Rust-side transport seam, but it now feeds **BunPluginHost specifically** — not the sole plugin abstraction. New plugin code should depend on `PluginHost` / `PluginOrchestrator`, not on `SidecarPort` directly.

Sidecar IPC is a **remote-but-owned** dependency (DEEPENING category 3):

```rust
// Conceptual — not yet implemented
pub trait SidecarPort: Send + Sync {
    async fn send(&self, message: SidecarMessage) -> Result<SidecarResponse, SidecarError>;
    async fn receive(&self) -> Result<SidecarMessage, SidecarError>;
}
```

| Adapter | Role |
|---------|------|
| **Production** | Spawn Bun process, JSON-line stdio transport (used by BunPluginHost) |
| **Test** | In-memory message queue; no subprocess |

See [sidecar/README.md](../sidecar/README.md) for the IPC contract.

### Plugin Sidecar Strategy (Bun default)

The default TUI and plugin path uses a **Bun sidecar** (`sidecar/`) via BunPluginHost:

- Rust spawns the sidecar as a child process.
- JSON-line IPC over stdio.
- Plugins run with full Bun/TypeScript fidelity.
- Server plugins can register additional HTTP routes via IPC (Phase 2).

This avoids embedding a JavaScript runtime inside Rust while preserving plugin compatibility. Native and WASM hosts complement Bun for performance and security use cases — see ADR 002.

## Provider Registry

`jereko-providers` uses a trait-based registry designed from day one for **75+ providers**:

- `Provider` trait: `list_models`, `complete`, `health_check`
- `ProviderRegistry`: O(1) lookup by provider id, ordered listing
- Built-in providers ship in-crate; plugin-provided providers register via sidecar (Phase 2)
- Shared HTTP/auth utilities will grow as submodules per provider family

## Config Precedence

Matching OpenCode semantics (lowest → highest):

1. Built-in defaults
2. Global config (`~/.config/opencode/`)
3. Project config (`.opencode/`)
4. Environment variables
5. CLI flags

Phase 0 implements merge stubs; full JSONC parsing and env/CLI overrides come in Phase 1.

## Rust Standards

Engineering conventions for all Rust crates. Full details: [development.md](./development.md).

| Rule | Detail |
|------|--------|
| Library errors | `thiserror` in `jereko-core`, `jereko-config`, `jereko-server`, `jereko-providers`, `conformance` |
| CLI errors | `anyhow` in `jereko-cli` only |
| Panics | No `unwrap()`/`expect()` outside tests |
| Linting | `cargo clippy --all-targets --all-features --locked -- -D warnings` |
| Documentation | `#![warn(missing_docs)]` on public library crates |
| Testing | At pre-agreed seams; see [conformance.md](./conformance.md) |

## Future Distribution and Integration (Out of Scope)

These paths are noted for future consideration, not Phase 0–2:

- **Pinokio / Gepeto launchers** — optional 1-click install for end users who prefer a launcher over `cargo install`.
- **Cursor SDK** — `jereko serve` could expose an agent-consumable HTTP API for SDK-based automations.
- **Native Rust TUI** — alternative to Bun sidecar for zero-Node dependency (see below).

## Future: Native TUI (Documented Only)

A native Rust TUI (e.g. ratatui-based) is a **future optional path**, not implemented in Phase 0. It would:

- Replace the Bun sidecar for users who prefer zero Node/Bun dependency.
- Require reimplementing plugin APIs or providing a compatibility shim.
- Remain secondary to the Bun sidecar default for plugin fidelity.

## Conformance Testing Strategy

See [conformance.md](./conformance.md).

- **No fork-and-merge** — upstream OpenCode source is never imported.
- **Owned fixtures** — request/response pairs under `conformance/fixtures/`.
- **Spec-derived** — fixtures authored from public API behavior.
- OpenCode is referenced only as the **compatibility target**, not as a dependency.

## Architecture Decision Records

Architecture decisions are recorded in [docs/adr/](./adr/):

- [ADR 001: Phase 0 Architecture Decisions](./adr/001-architecture-decisions.md)
- [ADR 002: Dual Plugin Runtime Architecture](./adr/002-dual-plugin-runtime.md) — extends Decision 3 (Bun sidecar) with orchestrator and multi-host strategy

## Remaining Work

Production adapters for Bun spawn, SQLite sessions, native/WASM hosts, first providers, and core tools are in place. Incremental gaps (streaming, full MCP/LSP/PTY, WASI hooks, 75+ providers) are tracked in [roadmap-remaining.md](./roadmap-remaining.md).

## Upstream Reference

OpenCode (the upstream project) is mentioned here solely as the behavioral compatibility reference. Jereko does not depend on, submodule, or vendor OpenCode code. Compatibility is validated through owned conformance tests.
