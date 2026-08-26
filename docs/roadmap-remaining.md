# Remaining Work Roadmap

**Status:** Living plan (post Phase 0–5 scaffolding)  
**Date:** 2026-08-26  
**Scope:** Document remaining implementation work only — this file is not a license to change ADR decisions without a new ADR.

Phases 0–5 scaffolding landed in commits `5c1d46e` and `6c7d386`. Seams, traits, stubs, fixtures, and docs exist; most production adapters are still stubs. This roadmap turns that scaffolding into shippable behavior.

Related: [architecture.md](./architecture.md), [conformance.md](./conformance.md), [development.md](./development.md), [perf-baseline.md](./perf-baseline.md), [releases.md](./releases.md), [distribution.md](./distribution.md), [ADR 001](./adr/001-architecture-decisions.md), [ADR 002](./adr/002-dual-plugin-runtime.md), [CONTEXT.md](../CONTEXT.md).

---

## Executive Summary

### Done (scaffolding)

| Area | Reality today |
|------|----------------|
| Workspace / crates | `jereko-core`, `jereko-config`, `jereko-server`, `jereko-cli`, `jereko-providers`, `jereko-plugins`, `conformance` |
| Config | JSONC load + merge precedence (`jereko-config`) |
| HTTP | Axum router, v1/v2 wire adapters, normalized handlers, in-process + black-box tests |
| Sessions | In-memory `SessionStore` (`jereko-server/src/session_store.rs`) |
| Providers | `Provider` trait + `ProviderRegistry` + `StubProvider` only |
| Plugins | `PluginOrchestrator`, `PluginHost`, `BunPluginHost`, stub `NativePluginHost` / `WasmPluginHost` |
| Sidecar IPC seam | `SidecarPort` + `InMemorySidecarPort` (real); `BunProcessSidecarPort` (stub transport) |
| Sidecar process | `sidecar/src/index.ts` reads JSON-lines; not yet spawned by Rust |
| Native ABI | `jereko-plugin-sdk/include/jereko_plugin.h` + SDK crate stub (crate **not** in workspace `members`) |
| Extensions | MCP / LSP / PTY status stubs at `/extensions/*` |
| Persistence | `SqliteSessionStore` stub (`enabled: false`) |
| CI | Rust fmt / clippy / test; Bun job commented out |
| Perf / native TUI | Hooks documented; `native-tui` feature is a no-op |
| Conformance fixtures | HTTP v1/v2, config samples, plugin hook sample under `conformance/fixtures/plugins/` |

### Remaining (high value)

Make stubs real in priority order: **Bun process spawn + CI → SQLite → native dylib fidelity → real providers/tools → MCP/LSP/PTY → WASM / native TUI / Criterion / distribution**.

No new ADR is required for this plan; work deepens ADR 001/002 seams.

---

## Prioritization Rationale

| Priority | Theme | Why now |
|----------|-------|---------|
| **P0** | Make stubs real (foundation) | `jereko run` already wires `BunProcessSidecarPort` but IPC is a no-op; sessions do not survive process restart; CI does not guard the Bun contract |
| **P1** | Dual plugin fidelity | Orchestrator + C ABI header exist; without `libloading` + test dylibs, NativePluginHost cannot share host-agnostic fixtures with Bun |
| **P2** | Provider & tool depth | Product usefulness: real LLM backends, agent tools, and extension protocols beyond status stubs |
| **P3** | Polish & optional paths | WASM sandbox, native TUI, Criterion, install packaging — valuable but not blocking the Bun-default path |

---

## P0 — Make Stubs Real (Foundation)

### P0.1 — Real `BunProcessSidecarPort` (spawn + JSON-line stdio)

| | |
|--|--|
| **Goal** | Production `SidecarPort` adapter spawns Bun and exchanges JSON-line messages over stdio. |
| **Scope** | `crates/jereko-plugins/src/sidecar.rs` (`BunProcessSidecarPort`); wire from `jereko-cli` `run` (`crates/jereko-cli/src/commands/run.rs`); align message field naming with `sidecar/src/index.ts` (Rust `#[serde(tag = "type", rename_all = "snake_case")]` ↔ TS types — keep one contract; update `sidecar/README.md` if docs still show dotted names). |
| **Acceptance** | `jereko run` starts Bun child on configured entry (default `sidecar/src/index.ts`); `Init` → sidecar `ready`; `Shutdown` exits cleanly; failed spawn / invalid JSON maps to `PluginError::Sidecar`; in-memory adapter tests remain green. |
| **Dependencies** | Bun on PATH; existing `SidecarOutbound` / `SidecarInbound` enums; sidecar entry script. |
| **Effort** | M (2–4 days) |
| **Risks** | Stdio buffering / partial lines; Windows vs Unix process semantics; hanging `recv` without timeouts; contract drift between README and serde tags. |
| **TDD seams** | `SidecarPort` (conformance.md); extend `InMemorySidecarPort` contract tests; add integration test that spawns real Bun when available (gate with `#[ignore]` or CI Bun job); diagnose via Layer 4 IPC loop. |

### P0.2 — Bun sidecar CI job

| | |
|--|--|
| **Goal** | CI validates sidecar install, TypeScript checks/tests, and (once P0.1 lands) IPC against a real or scripted Bun process. |
| **Scope** | `.github/workflows/ci.yml` — uncomment/extend the Bun job; `sidecar/package.json` scripts (`bun test`, typecheck); optional contract fixture shared with Rust. |
| **Acceptance** | PR CI fails if sidecar package is broken; documents required Bun version; does not block Rust job on Bun-only flakes without retry/clear failure messages. |
| **Dependencies** | Prefer after P0.1 for end-to-end IPC; can ship package lint/test first. |
| **Effort** | S (0.5–1 day) |
| **Risks** | Bun setup action version skew; flaky spawn under Actions runners. |
| **TDD seams** | Sidecar IPC seam; future “Bun sidecar contract validation” noted in conformance.md CI section. |

### P0.3 — SQLite session persistence

| | |
|--|--|
| **Goal** | Replace or augment in-memory `SessionStore` with durable SQLite-backed storage. |
| **Scope** | Deepen `crates/jereko-server/src/persistence.rs` (`SqliteSessionStore`); introduce a store trait or shared API used by `handlers.rs` / `state.rs`; keep in-memory adapter for tests; schema for session id, messages, status, timestamps. |
| **Acceptance** | Create session → restart process → get session returns same data; unit tests with temp DB file; handlers unchanged at the normalized-type boundary. |
| **Dependencies** | Choose crate (`rusqlite` / `sqlx` / `sqlite`); path from config (open question). |
| **Effort** | M (2–4 days) |
| **Risks** | Lock poisoning / sync vs async store; migration story; accidental coupling of handlers to SQLite types. |
| **TDD seams** | Prefer a new confirmed seam: `SessionStore` trait with in-memory + SQLite adapters (confirm with user before adding outside conformance.md table); Layer 4 session flow fixtures. |

---

## P1 — Dual Plugin Fidelity

### P1.1 — `NativePluginHost` + `libloading` + test dylibs

| | |
|--|--|
| **Goal** | Load real `.so` / `.dylib` / `.dll` plugins via `jereko_plugin.h` and invoke hooks in-process. |
| **Scope** | `crates/jereko-plugins/src/native_host.rs`; add `libloading` dependency; build test plugin(s) under e.g. `crates/jereko-plugins/tests/fixtures/` or `examples/native-plugin/`; resolve symbols `jereko_plugin_info` / `jereko_plugin_invoke`. |
| **Acceptance** | Load test dylib → `invoke_hook` returns fixture-matching JSON; empty/missing path errors; unload releases library; orchestrator cross-host test uses real native output (not `"stub": true`). |
| **Dependencies** | Stable ABI in `jereko-plugin-sdk/include/jereko_plugin.h`; platform linker for cdylib tests. |
| **Effort** | M–L (3–6 days) |
| **Risks** | ABI/string lifetime rules; panic across FFI; Windows DLL search paths; CI needing compile of cdylib. |
| **TDD seams** | NativePluginHost (conformance.md); host-agnostic fixtures in `conformance/fixtures/plugins/`. |

### P1.2 — `jereko-plugin-sdk` crate (real)

| | |
|--|--|
| **Goal** | Safe Rust bindings over the C ABI for native plugin authors. |
| **Scope** | Add `crates/jereko-plugin-sdk` to workspace `members` in root `Cargo.toml`; expand beyond `ABI_VERSION` stub; macros/helpers for exporting `jereko_plugin_info` / `jereko_plugin_invoke`. |
| **Acceptance** | Example plugin depends only on SDK; builds as cdylib; loads under P1.1 host. |
| **Dependencies** | P1.1 ABI freeze for v1; header stays source of truth. |
| **Effort** | M (2–3 days) |
| **Risks** | Premature ABI churn; SDK not in workspace today (easy to forget in CI). |
| **TDD seams** | Same as NativePluginHost; SDK unit tests for JSON encode/decode helpers. |

### P1.3 — Plugin hook conformance fixtures (host-agnostic)

| | |
|--|--|
| **Goal** | Same fixture input → same expected hook output for Bun (in-memory port) and native (test dylib). |
| **Scope** | Expand `conformance/fixtures/plugins/`; drive `PluginOrchestrator::dispatch_hook` tests; document fixture authorship rules already in conformance.md. |
| **Acceptance** | At least one shared hook (e.g. `before_transform`) passes on both hosts; failure isolation covered (one host error does not drop the other). |
| **Dependencies** | P0.1 helpful for Bun realism; P1.1 required for native realism. |
| **Effort** | S–M (1–3 days) |
| **Risks** | Tautological fixtures; Bun IPC hook surface still thinner than native C ABI. |
| **TDD seams** | PluginOrchestrator hook dispatch; NativePluginHost; SidecarPort / BunPluginHost. |

**Known scaffolding bug to fix when touching orchestrator:** `PluginEntry::Wasm` currently maps to `LoadTier::Bun` in `orchestrator.rs` — correct to a WASM tier when implementing WasmPluginHost (P3.1).

---

## P2 — Provider & Tool Depth

### P2.1 — Real providers (OpenAI, Anthropic, Ollama first)

| | |
|--|--|
| **Goal** | Thin HTTP adapters implementing `Provider` for the first production backends; grow toward 75+. |
| **Scope** | `crates/jereko-providers/` — modules per family (e.g. `openai/`, `anthropic/`, `ollama/`); shared auth/HTTP helpers; register in `ProviderRegistry` from config; keep `StubProvider` for tests. |
| **Acceptance** | `list_models` / `complete` / `health_check` against `wiremock` / `httptest` fixtures; env-based API keys; config provider id resolves to real adapter. |
| **Dependencies** | Existing `Provider` trait; config provider fields. |
| **Effort** | L for first three (1–2 weeks); ongoing for 75+. |
| **Risks** | Streaming/API shape drift; secret handling in tests; over-mocking internals (forbidden — mock HTTP only). |
| **TDD seams** | `Provider` trait; HTTP boundary per conformance.md provider testing policy. |

### P2.2 — Core tools in session engine

| | |
|--|--|
| **Goal** | Agent-usable tools: read, write, edit, bash, grep (and related) in the session / server path. |
| **Scope** | New module under `jereko-server` or `jereko-core` (open question); tool registry callable from message handling; permission/sandbox policy stub at minimum. |
| **Acceptance** | Session message can trigger tool calls with deterministic fixture results; tools testable without network; clear seam between tool execution and provider completion. |
| **Dependencies** | Session model; provider `complete` loop (tool-call round trips may need protocol extension). |
| **Effort** | L (1–2 weeks) |
| **Risks** | Scope creep into full agent runtime; unsafe bash defaults; missing OpenCode tool wire parity. |
| **TDD seams** | Prefer confirmed tool-execution seam + Layer 4 session flow; confirm new seams with user before adding. |

### P2.3 — MCP / LSP / PTY beyond stubs

| | |
|--|--|
| **Goal** | Replace `/extensions/{mcp,lsp,pty}` stubs with real client/server integrations. |
| **Scope** | `crates/jereko-server/src/extensions/mod.rs` and related; protocol clients; config for endpoints/commands. |
| **Acceptance** | Status endpoints report live connectivity; at least one happy-path integration test per extension (can be gated); stubs removed or feature-gated. |
| **Dependencies** | Config shape; possibly tools (P2.2) for PTY/bash overlap. |
| **Effort** | L (multi-week, can ship incrementally MCP → LSP → PTY). |
| **Risks** | Heavy dependencies; OS-specific PTY; MCP schema churn. |
| **TDD seams** | Extension HTTP routes (in-process router); black-box when wire format matters; confirm deeper protocol seams with user. |

---

## P3 — Polish & Optional Paths

### P3.1 — `WasmPluginHost` sandbox

| | |
|--|--|
| **Goal** | Sandboxed untrusted plugins via explicit `{ "wasm": "..." }` config (ADR 002). |
| **Scope** | `crates/jereko-plugins/src/wasm_host.rs`; WASM runtime choice; same hook surface as native where feasible; fix Wasm load tier in orchestrator. |
| **Acceptance** | Load wasm fixture → invoke hook → fixture match; untrusted plugin cannot escape sandbox (document threat model). |
| **Dependencies** | P1 hook fixtures; runtime crate selection. |
| **Effort** | L |
| **Risks** | Runtime size; WASI capability model; ABI dual-maintenance with native. |
| **TDD seams** | PluginOrchestrator + host-agnostic plugin fixtures. |

### P3.2 — Native TUI + `TuiPluginHost`

| | |
|--|--|
| **Goal** | Optional native TUI path behind `native-tui` (today a no-op in `jereko-cli` / `jereko-plugins`). |
| **Scope** | Per [perf-baseline.md](./perf-baseline.md) and ADR 002 Phase 5; ratatui (or similar) + `TuiPluginHost` bridge; Bun remains default. |
| **Acceptance** | `--features native-tui` builds a usable TUI stub→MVP; Bun path unchanged when feature off. |
| **Dependencies** | Stable session + provider loops helpful; not blocked on WASM. |
| **Effort** | L |
| **Risks** | Plugin API reimplementation vs shim; split UX vs Bun TUI. |
| **TDD seams** | New TUI host seam (confirm with user); keep sidecar IPC tests independent. |

### P3.3 — Criterion benchmarks

| | |
|--|--|
| **Goal** | Turn [perf-baseline.md](./perf-baseline.md) hooks into measurable Criterion benches. |
| **Scope** | Adapter normalize/denormalize; orchestrator dispatch; Sidecar IPC throughput (in-memory + optional real Bun); document baselines. |
| **Acceptance** | `cargo bench` (or documented subset) runs locally; CI optional/nightly; numbers recorded in perf-baseline or adjacent doc. |
| **Dependencies** | Stable hot paths (P0.1 for real IPC bench). |
| **Effort** | M |
| **Risks** | Noisy CI; premature optimization focus. |
| **TDD seams** | Perf hooks listed in perf-baseline.md (not correctness seams). |

### P3.4 — Distribution

| | |
|--|--|
| **Goal** | Install scripts, binary aliases packaging, optional Pinokio/Gepeto launchers (architecture “future distribution”). |
| **Scope** | README alias story → scripted install; release artifacts; launcher notes remain out of core runtime. |
| **Acceptance** | Documented one-command install for common platforms; aliases `opencode` / `opencode2` installable. |
| **Dependencies** | Stable CLI surface (`serve`, `run`, `version`). |
| **Effort** | S–M |
| **Risks** | Platform path differences; over-promising OpenCode drop-in before provider/tool parity. |
| **TDD seams** | Smoke / CLI version only — packaging is mostly manual/E2E. |

**Partial progress:** GitHub Release + PR `/build` workflows and packaging docs landed — see [releases.md](./releases.md) and [distribution.md](./distribution.md). Remaining: polish install UX / optional launchers.

---

## Suggested Sequencing

```text
P0.1 BunProcessSidecarPort ──┬──► P0.2 Bun CI
                             │
                             ├──► P1.3 shared hook fixtures (Bun side)
                             │
P0.3 SQLite persistence ─────┘     (parallel with P0.1/P0.2)

P1.1 NativePluginHost + libloading ──► P1.2 jereko-plugin-sdk
         │
         └──► P1.3 host-agnostic fixtures (native side)

P2.1 Providers (OpenAI / Anthropic / Ollama) ──► expand registry
P2.2 Core tools ──► depends on session + provider loop clarity
P2.3 MCP / LSP / PTY ──► incremental; PTY may share policy with tools

P3.1 WasmPluginHost ──► after P1 fixtures
P3.2 Native TUI ──► optional; after usable session/provider
P3.3 Criterion ──► after P0.1 (+ preferably P1 dispatch)
P3.4 Distribution ──► anytime after CLI stable; marketing-ready after P2.1+
```

**Parallelism:** P0.1 and P0.3 are independent. P1.1 can start once ABI is treated as frozen for v1. P2.1 can proceed in parallel with P1. Avoid large P3 until P0 foundation and at least one real provider exist.

**Dependency graph (compact):**

```text
                    ┌──────── P0.2 CI
P0.1 spawn ─────────┤
                    └──────── P3.3 IPC benches
                              P1.3 (Bun)
P0.3 SQLite ──────── handlers durability

P1.1 libloading ──── P1.2 SDK ──── P1.3 (native) ──── P3.1 WASM
P2.1 providers ───── P2.2 tools ── P2.3 extensions
P2.* + P0.* ──────── P3.2 native TUI, P3.4 dist
```

---

## Out of Scope / Deferred

| Item | Notes |
|------|-------|
| Forking / vendoring OpenCode | Forbidden (ADR 001 Decision 5) |
| Embedding a JS runtime in Rust | Sidecar remains default (ADR 001/002) |
| Full 75+ providers in one PR | Registry designed for it; ship first three, then expand |
| Pinokio / Gepeto / Cursor SDK productization | Documented future paths only until explicitly prioritized |
| Replacing Bun TUI as default | Native TUI stays optional (`native-tui`) |
| New architectural ADRs for this roadmap | Not required — deepen existing seams |
| Implementing this roadmap in the same change as this doc | Documentation-only |

---

## Open Questions

1. **SQLite API shape** — Trait-ify `SessionStore` (in-memory + SQLite adapters) vs grow `SqliteSessionStore` behind a feature flag? Preferred path for test seams?
2. **Session DB location** — Default path (`~/.local/share/jereko/`, project `.jereko/`, or config key)? Migration policy for schema v1?
3. **Bun version pin** — Exact Bun version for CI and local docs?
4. **IPC contract freeze** — Confirm serde snake_case tags (`session_start`, `tui_render`, …) as canonical vs README dotted names (`session.start`); any Unix-socket transport in P0 or later?
5. **`jereko-plugin-sdk` workspace membership** — Add immediately with P1, or earlier as empty member for visibility?
6. **Tool execution home** — `jereko-core` vs `jereko-server` vs new `jereko-tools` crate?
7. **Provider streaming** — Does v1 MVP require SSE/streaming `complete`, or request/response only first?
8. **Sandbox policy for bash/PTY** — Default deny, allowlist, or project-root jail for P2?
9. **75+ provider source of truth** — Maintain an owned inventory doc, or grow ad hoc from config demand?
10. **Criterion in CI** — Nightly only, or threshold gates on PRs?

---

## Quick Reference — Stub → Real Map

| Stub / partial | Path | Priority |
|----------------|------|----------|
| `BunProcessSidecarPort` | `crates/jereko-plugins/src/sidecar.rs` | P0.1 |
| Bun CI job | `.github/workflows/ci.yml` | P0.2 |
| `SqliteSessionStore` | `crates/jereko-server/src/persistence.rs` | P0.3 |
| In-memory only sessions | `crates/jereko-server/src/session_store.rs` | P0.3 |
| `NativePluginHost` | `crates/jereko-plugins/src/native_host.rs` | P1.1 |
| `jereko-plugin-sdk` | `crates/jereko-plugin-sdk/` (not in workspace) | P1.2 |
| Plugin fixtures | `conformance/fixtures/plugins/` | P1.3 |
| `StubProvider` only | `crates/jereko-providers/` | P2.1 |
| Core tools | (not present) | P2.2 |
| MCP/LSP/PTY | `crates/jereko-server/src/extensions/mod.rs` | P2.3 |
| `WasmPluginHost` | `crates/jereko-plugins/src/wasm_host.rs` | P3.1 |
| `native-tui` feature | `jereko-cli` / `jereko-plugins` Cargo.toml | P3.2 |
| Criterion | [perf-baseline.md](./perf-baseline.md) | P3.3 |
