# Remaining Work Roadmap

**Status:** Living plan — P0–P3 foundation implemented 2026-08-26  
**Date:** 2026-08-26  
**Scope:** Document remaining implementation work only — this file is not a license to change ADR decisions without a new ADR.

Related: [architecture.md](./architecture.md), [conformance.md](./conformance.md), [development.md](./development.md), [perf-baseline.md](./perf-baseline.md), [distribution.md](./distribution.md), [ADR 001](./adr/001-architecture-decisions.md), [ADR 002](./adr/002-dual-plugin-runtime.md), [CONTEXT.md](../CONTEXT.md).

---

## Executive Summary

### Done

| Area | Reality today |
|------|----------------|
| Workspace / crates | Core crates + `jereko-plugin-sdk` + `jereko-test-native-plugin` in workspace |
| Config | JSONC load + merge; optional `sessionDb` |
| HTTP | Axum router, v1/v2 adapters, in-process + black-box tests |
| Sessions | `SessionStorePort` trait; in-memory + **SQLite** (`SqliteSessionStore`) |
| Providers | `Provider` trait + **OpenAI / Anthropic / Ollama** HTTP adapters + `StubProvider`; wiremock tests |
| Plugins | Orchestrator; **BunProcessSidecarPort** (real spawn); **NativePluginHost** (libloading); **WasmPluginHost** (wasmtime validate/load) |
| Sidecar | JSON-line stdio; Bun CI job; contract tests |
| Native ABI | `jereko_plugin.h` + SDK export macro + test cdylib |
| Extensions | MCP list_tools / LSP initialize / PTY session registry seams |
| Tools | `read` / `write` / `edit` / `bash` / `grep` in `jereko-server` |
| CI | Rust fmt / clippy / test + Bun sidecar job |
| Perf / native TUI | Criterion `hot_paths` bench; `native-tui` ratatui stub |
| Distribution | `scripts/install.sh` + [distribution.md](./distribution.md) |

### Remaining gaps (incremental)

| Item | Notes |
|------|-------|
| Full WASM hook ABI | Module load/validate works; WASI hook invoke still structured stub |
| Real portable-pty | PTY manager registers sessions; no OS PTY bytes yet |
| Full MCP/LSP protocol | Happy-path seams only (list tools / initialize) |
| 75+ providers | First three shipped; registry ready to grow |
| Streaming completions | Request/response only |
| Bash/PTY sandbox policy | Project-root jail for tools; bash allow flag; no full policy engine |
| Native TUI MVP | Feature-flagged stub frame, not interactive Bun replacement |
| Criterion in CI | Local/nightly only (not PR-gated) |

No new ADR is required for deepening these seams.

---

## Prioritization — Completion Status

| Priority | Theme | Status |
|----------|-------|--------|
| **P0** | Bun spawn + CI + SQLite | **Done** |
| **P1** | Dual plugin fidelity (native + fixtures + LoadTier) | **Done** |
| **P2** | Providers, tools, MCP/LSP/PTY seams | **Done** (protocol depth remains incremental) |
| **P3** | WASM load, native-tui stub, Criterion, install | **Done** (see gaps above) |

---

## P0 — Make Stubs Real (Foundation) — DONE

### P0.1 — Real `BunProcessSidecarPort` — DONE
- Spawns `bun run <entry>`, JSON-line stdio, graceful shutdown, integration test when Bun is on PATH.

### P0.2 — Bun sidecar CI job — DONE
- `.github/workflows/ci.yml` `bun-sidecar` job: install, `bun run check`, `bun test` (Bun 1.2.5).

### P0.3 — SQLite session persistence — DONE
- `SessionStorePort` + `SqliteSessionStore` (rusqlite); `sessionDb` config; persist/reload tests.

---

## P1 — Dual Plugin Fidelity — DONE

### P1.1 / P1.2 — NativePluginHost + SDK + test dylib — DONE
### P1.3 — Host-agnostic fixtures — DONE
### LoadTier Wasm mapping — FIXED (`LoadTier::Wasm`)

---

## P2 — Provider & Tool Depth — DONE (incremental depth remains)

### P2.1 — OpenAI / Anthropic / Ollama — DONE (wiremock)
### P2.2 — Core tools — DONE (`read`/`write`/`edit`/`bash`/`grep`)
### P2.3 — MCP / LSP / PTY — DONE at seam level (not full protocols)

---

## P3 — Polish & Optional — DONE (see gaps)

### P3.1 — WasmPluginHost — DONE (load/validate tiny module; hook ABI TODO)
### P3.2 — Native TUI — DONE (ratatui stub behind `native-tui`)
### P3.3 — Criterion — DONE (`cargo bench -p jereko-plugins`)
### P3.4 — Distribution — DONE (`scripts/install.sh`, docs)

---

## Out of Scope / Deferred

| Item | Notes |
|------|-------|
| Forking / vendoring OpenCode | Forbidden (ADR 001 Decision 5) |
| Embedding a JS runtime in Rust | Sidecar remains default (ADR 001/002) |
| Full 75+ providers in one PR | First three shipped |
| Pinokio / Gepeto / Cursor SDK productization | Future paths |
| Replacing Bun TUI as default | Native TUI stays optional |
| Full WASI plugin hook ABI | Documented gap under P3.1 |
| OS-level portable-pty | Documented gap under P2.3 |

---

## Open Questions (resolved / still open)

1. **SQLite API shape** — Resolved: `SessionStorePort` trait + in-memory / SQLite adapters.
2. **Session DB location** — Resolved for v1: optional `sessionDb` config path (no implicit default file).
3. **Bun version pin** — Resolved: CI pins `1.2.5`; engines `>=1.1`.
4. **IPC contract** — Resolved: snake_case tags/fields canonical.
5. **`jereko-plugin-sdk` workspace** — Resolved: member + test plugin.
6. **Tool execution home** — Resolved for now: `jereko-server::tools`.
7. **Provider streaming** — Still open (request/response first).
8. **Sandbox policy** — Partial: project-root path jail + bash allow flag.
9. **75+ provider inventory** — Still open.
10. **Criterion in CI** — Still open (local/nightly preferred).
