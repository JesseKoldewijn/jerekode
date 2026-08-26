# True OpenCode / opencode2 Parity Roadmap

**Status:** Documented slices complete  
**Date:** 2026-08-26  
**Supplements:** [roadmap-remaining.md](./roadmap-remaining.md) (foundation archive), [conformance.md](./conformance.md), [CONTEXT.md](../CONTEXT.md)

Goal: behavioral parity with OpenCode / opencode2 proven by owned fixtures and hard CI gates — not soft-skips, not scaffold-only seams.

---

## Progress board

| ID | Slice | Status | PR |
|----|-------|--------|-----|
| R0 | Release pipeline green on main (Actions PR perms + sync resilience) | Done | #5–#13; v0.1.9+ |
| P0a | CI hard-gates: Bun IPC + native dylib (no soft-skip) | Done | #8 |
| P0b | Wire tools into HTTP `/v1|/v2/tools/execute` | Done | #12 |
| P1a | Provider streaming seam (`complete_stream` / SSE) | Done | #17 |
| P1b | Bun sidecar loads/runs real plugins + hook fixtures | Done | #15 |
| P2a | MCP depth beyond list_tools | Done | #21 |
| P2b | LSP depth beyond initialize stub | Done | #21 |
| P2c | portable-pty OS I/O | Done | #21 |
| P2d | WASM WASI hook ABI | Done | #24 |
| P3a | HTTP v1/v2 surface expansion via fixtures | Done | #28 |
| P3b | More providers (incremental registry growth) | Done | #26 |
| P3c | Sandbox policy engine | Done | #26 |
| P3d | Native TUI interactive MVP (optional) | Done | #26 |
| P3e | Criterion nightly workflow | Done | #24 |
| DOC | Refactor all repo documentation for accuracy | Done | this PR |

---

## Gap matrix (current → ongoing growth)

| Area | Current | Ongoing growth |
|------|---------|----------------|
| Bun IPC | Real spawn; CI hard-gates | Broader OpenCode-compatible hook surface |
| Native plugins | libloading + CI hard-gates | More server hook coverage |
| Sidecar plugins | Dynamic import + `invoke_hook` | Growing plugin ecosystem |
| Tools | Wired via `/tools/execute` + policy | Agent-loop depth + fixtures |
| Providers | OpenAI / Anthropic / Ollama / Groq / OpenRouter + streaming | Growing matrix toward 75+ |
| MCP / LSP / PTY | call_tool + hover + portable-pty I/O | Broader protocol matrix |
| WASM | `jereko_hook` export + host fallback | Richer WASI surface |
| HTTP | v1/v2 sessions list/get/delete + messages + stream + tools | Broader fixture coverage |
| Release | Proven green publish on main merge | Keep green |
| Docs | Aligned with shipped capability | Keep updated with each slice |

---

## Execution rules

1. **PR-only** to `main`; auto-merge when `rust` + `bun-sidecar` are green (except risky/large PRs).
2. **Vertical slices:** fixture/test first → minimal impl → green CI.
3. **No upstream OpenCode source** — owned fixtures only.
4. **Do not weaken CI** — convert soft-skips to hard gates, never the reverse.
5. Update this board as slices land; keep [roadmap-remaining.md](./roadmap-remaining.md) as historical foundation status.

---

## Out of scope (unchanged)

- Forking / vendoring OpenCode (ADR 001)
- Embedding a JS runtime in Rust (ADR 001/002)
- Replacing Bun TUI as the default
