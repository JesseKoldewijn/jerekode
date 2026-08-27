# Jerekode Architecture

Jerekode is a **Rust port of OpenCode**: an OpenCode-compatible AI coding agent runtime built as a **Rust core + Bun sidecar**. Compatibility is conformance-driven — this repository does not vendor upstream OpenCode source.

## System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                        jerekode CLI                           │
│  (primary binary: `jerekode`; aliases: opencode, opencode2)   │
└──────────────┬──────────────────────────────┬───────────────┘
               │                              │
               ▼                              ▼
┌──────────────────────────┐    ┌────────────────────────────┐
│     jerekode-server        │    │   Bun Sidecar (sidecar/)   │
│  HTTP + adapter layer    │◄──►│  TUI + server plugins      │
│  v1 / v2 normalization   │    │  JSON-line IPC             │
└──────────────┬───────────┘    └────────────────────────────┘
               │
               ▼
┌──────────────────────────┐    ┌────────────────────────────┐
│     jerekode-core          │    │   jerekode-providers         │
│  sessions, messages      │    │  provider registry         │
└──────────────────────────┘    └────────────────────────────┘
               │
               ▼
┌──────────────────────────┐
│     jerekode-config        │
│  JSONC config + merge    │
└──────────────────────────┘
```

## Crate Responsibilities

| Crate | Role |
|-------|------|
| `jerekode-core` | Domain types, session models, shared errors |
| `jerekode-config` | Config loading, precedence merge, `opencode.json` / `tui.json` types |
| `jerekode-server` | Axum HTTP server, v1/v2 adapters, tools, extensions, policy |
| `jerekode-cli` | CLI entry point (`serve`, `run`, `version`) |
| `jerekode-providers` | Provider trait, registry, HTTP adapters |
| `jerekode-plugins` | PluginOrchestrator, Bun/native/WASM hosts, SidecarPort |
| `jerekode-plugin-sdk` | Native plugin C ABI / Rust helpers |
| `jerekode-rtk-plugin` | Native RTK adapter (`packages/rtk/native`) |

**JS packages (Bun workspaces):** `sidecar/` (plugin host), `packages/rtk/` (`@jerekode/rtk` OpenCode2 entry + shared rules). See [ADR 004](./adr/004-rtk-dual-adapter.md).

| `jerekode-providers` | Provider trait, registry, streaming HTTP adapters |
| `jerekode-plugins` | PluginOrchestrator, Bun/native/WASM hosts, SidecarPort |
| `jerekode-plugin-sdk` | Native plugin C ABI / Rust helpers |
| `conformance` | Owned fixture-driven compatibility tests |

## Terminology: Adapters vs Seams

Jerekode uses "adapter" in three distinct senses. Each maps to a **seam** in [codebase-design](../.agents/skills/codebase-design/SKILL.md) vocabulary:

| Concept | Location | Role |
|---------|----------|------|
| **Wire adapter** | `jerekode-server/src/adapters/v1/`, `v2/` | Translates v1 or v2 HTTP wire format ↔ normalized types |
| **Provider adapter** | `jerekode-providers` (`Provider` trait) | Implements LLM backend behavior (Anthropic, OpenAI, Groq, `StubProvider`, etc.) |
| **Sidecar adapter** | Rust ↔ Bun IPC | Transport for BunPluginHost — production (spawn Bun) or test (in-memory) |
| **Plugin host** | `PluginOrchestrator` | `PluginHost` trait — Bun, native dylib, or WASM implementations |

**Seam** = where a module's interface lives. **Adapter** = concrete implementation at that seam.

## Module Seams

| Crate / module | Seam | Interface | Adapters |
|----------------|------|-----------|----------|
| `jerekode-server/adapters/` | HTTP wire normalization | Normalized request/response types | v1 wire adapter, v2 wire adapter |
| `jerekode-server/router` | HTTP routing | Axum routes on normalized types | single implementation; in-process tests |
| `jerekode-providers` | LLM provider | `Provider` trait (`complete`, `complete_stream`, …) | HTTP adapters + `StubProvider` |
| `jerekode-config` | Configuration | Merge loader API | File/env/CLI sources |
| `jerekode-core` | Domain | Session, message types | pure domain |
| Plugin orchestrator | Plugin hook dispatch | `PluginOrchestrator`, `PluginHost` | BunPluginHost, NativePluginHost, WasmPluginHost |
| Sidecar IPC | Sidecar transport | `SidecarPort` | Bun process adapter, in-memory test adapter |
| Session store | Persistence | `SessionStorePort` | in-memory, SQLite |
| Tools | Agent tools | `ToolExecutor` + `ToolPolicy` | read/write/edit/grep/bash |

**Depth goal:** handlers and domain logic stay deep. Wire and provider adapters stay thin.

## HTTP Adapter Layer

Both v1 and v2 HTTP APIs are supported through a **pluggable adapter layer** that normalizes requests and responses as early as possible:

```text
Client (v1) ──► adapters/v1 ──► adapters/normalized ──► handlers
Client (v2) ──► adapters/v2 ──► adapters/normalized ──► handlers
```

Shipped surface includes sessions (create/get/list/delete), messages (list/send/SSE stream), providers, and tools. Extensions expose MCP/LSP/PTY helpers under `/extensions/*`.

**Design goals:**

- Handlers operate exclusively on `adapters/normalized` types.
- Version-specific serde shapes stay inside `adapters/v1` and `adapters/v2`.
- Deprecating v1 later means removing the v1 adapter module, not rewriting core logic.

## Binary Naming

- **Primary binary**: `jerekode`
- **Optional aliases**: `opencode`, `opencode2` (symlinks or install-time aliases)

Aliases are not separate implementations — they point to the same `jerekode` binary.

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
└──────────────────┘  └──────────────────────┘  └────────┬─────────┘
                                                         │
                                                         ▼
                                            ┌──────────────────────┐
                                            │  Bun sidecar process │
                                            └──────────────────────┘

Also: WasmPluginHost — sandboxed modules with `jerekode_hook` export
```

### Host Types

| Host | Config form | Role |
|------|-------------|------|
| **BunPluginHost** | `"@acme/server-plugin"` (unqualified string, default) | OpenCode/Bun fidelity via sidecar IPC; dynamic import + `invoke_hook` |
| **NativePluginHost** | `{ "native": "./path/to/plugin.so" }` | In-process dylib; server hooks |
| **WasmPluginHost** | `{ "wasm": "./path/to/plugin.wasm" }` | Sandboxed plugins; `jerekode_hook` ABI (host fallback if export missing) |

**Load order:** internal → native → bun. The orchestrator builds a single ordered hook chain across all active hosts with failure isolation per plugin.

**TUI:** Bun `jerekode run` is the default. Optional `native-tui` feature provides an interactive ratatui MVP — not a Bun replacement.

### SidecarPort → BunPluginHost

`SidecarPort` is the Rust-side transport seam feeding **BunPluginHost**. New plugin code should depend on `PluginHost` / `PluginOrchestrator`, not on `SidecarPort` directly.

| Adapter | Role |
|---------|------|
| **Production** | Spawn Bun process, JSON-line stdio transport |
| **Test** | In-memory message queue; no subprocess |

See [sidecar/README.md](../sidecar/README.md) for the IPC contract.

## Provider Registry

`jerekode-providers` uses a trait-based registry designed for **75+ providers**:

- `Provider` trait: `list_models`, `complete`, `complete_stream`, `health_check`
- Shipped adapters: OpenAI, Anthropic, Ollama, Groq, OpenRouter (+ stubs for tests)
- `ProviderRegistry`: O(1) lookup by provider id, ordered listing
- SSE / NDJSON stream parsers for HTTP SSE endpoints on the server

## Config Precedence

Matching OpenCode semantics (lowest → highest):

1. Built-in defaults
2. Global config (`~/.config/opencode/`)
3. Project config (`.opencode/`)
4. Environment variables
5. CLI flags

JSONC parsing and merge are implemented; optional `sessionDb` selects SQLite persistence.

## Rust Standards

Full details: [development.md](./development.md).

| Rule | Detail |
|------|--------|
| Library errors | `thiserror` in library crates |
| CLI errors | `anyhow` in `jerekode-cli` only |
| Panics | No `unwrap()`/`expect()` outside tests |
| Linting | `cargo clippy --all-targets --all-features --locked -- -D warnings` |
| Documentation | `#![warn(missing_docs)]` on public library crates |
| Testing | At pre-agreed seams; see [conformance.md](./conformance.md) |

## Future Distribution and Integration

Release packaging (installers, changelog quality, version reset, full vs native-only builds): [roadmap-releases.md](./roadmap-releases.md) and [ADR 003](./adr/003-release-packaging-and-changelogs.md).

Also documented for later productization (not blocking parity):

- **Pinokio / Gepeto launchers** — optional 1-click install
- **Cursor SDK** — `jerekode serve` as an agent-consumable HTTP API
- Broader native TUI / plugin surface beyond the MVP

## Conformance Testing Strategy

See [conformance.md](./conformance.md).

- **No fork-and-merge** — upstream OpenCode source is never imported.
- **Owned fixtures** — request/response pairs under `conformance/fixtures/`.
- **Spec-derived** — fixtures authored from public API behavior.

## Architecture Decision Records

- [ADR 001: Architecture Decisions](./adr/001-architecture-decisions.md)
- [ADR 002: Dual Plugin Runtime Architecture](./adr/002-dual-plugin-runtime.md)
- [ADR 003: Release Packaging, Changelogs, and Distribution Variants](./adr/003-release-packaging-and-changelogs.md)

## Ongoing work

Documented foundation and parity slices are complete — see [roadmap-parity.md](./roadmap-parity.md). Incremental growth (more providers, richer protocols) continues without new ADRs unless decisions change.

**Active forward plan:** release packaging, changelogs, version reset, installers — [roadmap-releases.md](./roadmap-releases.md) / [ADR 003](./adr/003-release-packaging-and-changelogs.md).

Historical foundation notes: [roadmap-remaining.md](./roadmap-remaining.md).

## Upstream Reference

OpenCode is the behavioral compatibility reference only. Jerekode does not depend on, submodule, or vendor OpenCode code.
