# True OpenCode / opencode2 Parity Roadmap

**Status:** Active execution plan  
**Date:** 2026-08-26  
**Supplements:** [roadmap-remaining.md](./roadmap-remaining.md) (foundation P0–P3), [conformance.md](./conformance.md), [CONTEXT.md](../CONTEXT.md)

Goal: behavioral parity with OpenCode / opencode2 proven by owned fixtures and hard CI gates — not soft-skips, not scaffold-only seams.

---

## Progress board

| ID | Slice | Status | PR |
|----|-------|--------|-----|
| R0 | Release pipeline green on main (Actions PR perms + sync resilience) | In progress | #5 synced; #6 harden; #7 sync 0.1.6 |
| P0a | CI hard-gates: Bun IPC + native dylib (no soft-skip) | Done | #8 |
| P0b | Wire tools into HTTP `/v1|/v2/tools/execute` | In progress | this PR |
| P1a | Provider streaming seam (`complete_stream` / SSE) | Pending | — |
| P1b | Bun sidecar loads/runs real plugins + hook fixtures | Pending | — |
| P2a | MCP depth beyond list_tools | Pending | — |
| P2b | LSP depth beyond initialize stub | Pending | — |
| P2c | portable-pty OS I/O | Pending | — |
| P2d | WASM WASI hook ABI | Pending | — |
| P3a | HTTP v1/v2 surface expansion via fixtures | Pending | — |
| P3b | More providers (incremental registry growth) | Pending | — |
| P3c | Sandbox policy engine | Pending | — |
| P3d | Native TUI interactive MVP (optional) | Pending | — |
| P3e | Criterion nightly workflow | Pending | — |
| DOC | Refactor all repo documentation for accuracy | Final step | — |

---

## Gap matrix (current → target)

| Area | Current | Target for parity |
|------|---------|-------------------|
| Bun IPC | Real spawn; Rust test soft-skips without Bun; CI rust job has no Bun | Hard-fail integration test in CI |
| Native plugins | libloading works; test soft-skips if dylib missing | Prebuild test cdylib in CI; hard-fail |
| Sidecar plugins | Echo/log only — does not load plugin modules | Load config plugins; dispatch hooks; fixture parity |
| Tools | `read`/`write`/`edit`/`bash`/`grep` library only | Wired into session/message agent loop + fixtures |
| Providers | OpenAI / Anthropic / Ollama request-response | Streaming + growing matrix toward 75+ |
| MCP / LSP / PTY | Status seams + stubs | Real protocol depth (call_tool, JSON-RPC methods, OS PTY) |
| WASM | Load/validate | WASI hook invoke |
| HTTP | Minimal v1/v2 session/message/providers | Full owned fixture surface |
| Release | Sync PR blocked until Actions perms | Proven green publish on every main merge |
| Docs | Some “stub” labels outdated | Final pass aligns all docs with reality |

---

## Execution rules

1. **PR-only** to `main`; auto-merge when `rust` + `bun-sidecar` are green (except risky/large PRs).
2. **Vertical slices:** fixture/test first → minimal impl → green CI.
3. **No upstream OpenCode source** — owned fixtures only.
4. **Do not weaken CI** — convert soft-skips to hard gates, never the reverse.
5. Update this board as slices land; keep [roadmap-remaining.md](./roadmap-remaining.md) for historical foundation status.

---

## Out of scope (unchanged)

- Forking / vendoring OpenCode (ADR 001)
- Embedding a JS runtime in Rust (ADR 001/002)
- Replacing Bun TUI as the default
