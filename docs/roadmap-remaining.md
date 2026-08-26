# Foundation Roadmap Archive

**Status:** Archive — foundation P0–P3 and parity R0–P3e are complete  
**Date:** 2026-08-26  
**Active board:** [roadmap-parity.md](./roadmap-parity.md)

This file preserves the historical foundation plan. Do not treat the “remaining gaps” sections below as current blockers — they were closed by subsequent parity PRs (#15–#28). Prefer the parity board for present-day status.

Related: [architecture.md](./architecture.md), [conformance.md](./conformance.md), [CONTEXT.md](../CONTEXT.md), [ADR 001](./adr/001-architecture-decisions.md), [ADR 002](./adr/002-dual-plugin-runtime.md).

---

## Executive Summary (as of archive close-out)

### Shipped capability

| Area | Reality |
|------|---------|
| Workspace / crates | Core crates + plugin SDK + test native plugin |
| Config | JSONC load + merge; optional `sessionDb` |
| HTTP | Axum v1/v2: sessions list/get/delete, messages, SSE stream, providers, tools |
| Sessions | `SessionStorePort`; in-memory + SQLite |
| Providers | OpenAI / Anthropic / Ollama / Groq / OpenRouter + stubs; streaming |
| Plugins | Orchestrator; Bun spawn + real plugin load; NativePluginHost; Wasm `jereko_hook` |
| Sidecar | JSON-line stdio; Bun CI hard-gate |
| Extensions | MCP call_tool; LSP initialize/hover; portable-pty I/O |
| Tools | read/write/edit/bash/grep + sandbox policy |
| TUI | Bun default; optional interactive `native-tui` |
| Perf | Criterion benches + nightly workflow (not PR-gated) |
| Release | Auto-release on `main` merge |

### Ongoing growth (not foundation blockers)

| Item | Notes |
|------|-------|
| 75+ providers | Registry ready; grow incrementally |
| Broader MCP/LSP methods | Happy-path depth shipped; expand with fixtures |
| Richer WASI surface | Hook ABI shipped; deepen as needed |
| Agent-loop tooling depth | `/tools/execute` + policy shipped |

No new ADR is required for deepening these seams.

---

## Historical prioritization (complete)

| Priority | Theme | Status |
|----------|-------|--------|
| **P0** | Bun spawn + CI + SQLite | **Done** |
| **P1** | Dual plugin fidelity | **Done** |
| **P2** | Providers, tools, MCP/LSP/PTY | **Done** |
| **P3** | WASM hooks, native-tui, Criterion, install | **Done** |
| **Parity R0–P3e** | See [roadmap-parity.md](./roadmap-parity.md) | **Done** |

---

## Out of Scope (unchanged)

| Item | Notes |
|------|-------|
| Forking / vendoring OpenCode | Forbidden (ADR 001) |
| Embedding a JS runtime in Rust | Sidecar remains default (ADR 001/002) |
| Replacing Bun TUI as default | Native TUI stays optional |
| Pinokio / Gepeto / Cursor SDK productization | Future paths |

---

## Open Questions (resolved)

1. **SQLite API shape** — `SessionStorePort` + in-memory / SQLite adapters.
2. **Session DB location** — optional `sessionDb` config path.
3. **Bun version pin** — CI pins `1.2.5`; engines `>=1.1`.
4. **IPC contract** — snake_case tags/fields canonical.
5. **`jereko-plugin-sdk` workspace** — member + test plugin.
6. **Tool execution home** — `jereko-server::tools`.
7. **Provider streaming** — shipped (`complete_stream` + SSE HTTP).
8. **Sandbox policy** — `ToolPolicy` (deny `.git/`, bash allow, timeouts).
