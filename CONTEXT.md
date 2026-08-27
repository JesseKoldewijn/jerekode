# Jerekode — Agent Context

Start here for a navigable map of the repository. Human-oriented docs live in [docs/](docs/); this file is optimized for agents and contributors who need orientation fast.

## Current state

Jerekode is a **working Rust port of OpenCode** — an OpenCode-compatible AI coding agent runtime (Rust core, Bun plugin sidecar, dual plugin hosts per ADR 002). Compatibility is conformance-driven with owned fixtures (no upstream OpenCode source in-repo). Release pipeline on `main` is green.

Documented parity slices **R0–P3e** are complete — see [docs/roadmap-parity.md](docs/roadmap-parity.md). Further growth (more providers, richer MCP/LSP/WASM surfaces) is incremental, not foundation scaffolding. **Active forward plans:** CLI ↔ OpenCode full drop-in (locked decisions + remaining-work inventory) — [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md); release packaging / changelogs / installers (default download = **full**/Bun) — [docs/roadmap-releases.md](docs/roadmap-releases.md).

## Crate Map

| Crate | Path | Responsibility |
|-------|------|----------------|
| `jerekode-core` | `crates/jerekode-core/` | Domain types, session models, shared errors |
| `jerekode-config` | `crates/jerekode-config/` | Config loading, merge precedence, `opencode.json` / `tui.json` types |
| `jerekode-server` | `crates/jerekode-server/` | Axum HTTP server, v1/v2 wire adapters, tools, extensions, policy |
| `jerekode-cli` | `crates/jerekode-cli/` | CLI binary (`jerekode`; aliases `opencode`, `opencode2`) |
| `jerekode-providers` | `crates/jerekode-providers/` | `Provider` trait, registry, streaming, HTTP adapters |
| `jerekode-plugins` | `crates/jerekode-plugins/` | PluginOrchestrator, Bun/native/WASM hosts, SidecarPort |
| `jerekode-plugin-sdk` | `crates/jerekode-plugin-sdk/` | Native plugin C ABI / Rust SDK |
| `jerekode-conformance` | `conformance/` | Owned fixture-driven compatibility tests |
| `jerekode-rtk-plugin` | `packages/rtk/native/` | Native RTK adapter cdylib |

Supporting directories:

| Path | Role |
|------|------|
| `packages/` | Bun workspace packages (first-party plugins such as `@jerekode/rtk`) |
| `packages/rtk/` | RTK OpenCode2 + native dual adapter ([ADR 004](docs/adr/004-rtk-dual-adapter.md)) |
| `sidecar/` | Bun plugin host (TUI + server plugins); JSON-line IPC with Rust core |
| `docs/` | Architecture, conformance, development, ADRs, roadmaps |
| `.agents/skills/` | Installed agent skills (codebase-design, tdd, diagnosing-bugs, rust-best-practices) |

## Contribution rule

All code changes reach `main` **only via pull request**. Never push directly to `main`. Release CI may open a version-bump sync PR after merge — see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/releases.md](docs/releases.md).

## Domain Vocabulary

Use these terms consistently (see [docs/architecture.md](docs/architecture.md) for full disambiguation):

| Term | Meaning |
|------|---------|
| **Session** | A conversation context in `jerekode-core` — messages, status, identity |
| **Provider** | An LLM backend registered in `jerekode-providers`; implements the `Provider` trait |
| **Sidecar** | The Bun process (`sidecar/`) that hosts TUI and plugin code; communicates with Rust via JSON-line IPC; transport for **BunPluginHost** |
| **PluginOrchestrator** | Rust coordinator for plugin hook registry, load order, priority, and dispatch across all hosts |
| **PluginHost** | Trait generalizing plugin loading — implementations: BunPluginHost, NativePluginHost, WasmPluginHost |
| **BunPluginHost** | Default plugin host for unqualified config strings; OpenCode-compatible hooks via SidecarPort IPC |
| **NativePluginHost** | In-process dylib host via stable C ABI (`jerekode_plugin.h`); explicit config only |
| **WasmPluginHost** | Sandboxed WASM host; `jerekode_hook` export (host fallback when absent) |
| **Normalized types** | Version-agnostic request/response shapes in `jerekode-server/src/adapters/normalized/`; handlers operate only on these |
| **Wire adapter** | Translates v1 or v2 HTTP wire format ↔ normalized types (`jerekode-server/src/adapters/v1/`, `v2/`) |
| **Provider adapter** | A concrete `Provider` implementation (e.g. Anthropic, OpenAI, Groq, `StubProvider`) |
| **Sidecar adapter** | Rust-side transport for BunPluginHost IPC — production (spawn Bun) or test (in-memory); see `SidecarPort` |

**Seam** and **interface** follow [codebase-design](.agents/skills/codebase-design/SKILL.md) vocabulary: a seam is where a module's interface lives; an adapter satisfies that interface.

## Capability snapshot

| Area | Status |
|------|--------|
| Config | JSONC load + merge; optional `sessionDb` (SQLite) |
| HTTP | v1/v2 sessions (create/get/list/delete), messages (+ SSE stream), providers, tools |
| Providers | OpenAI, Anthropic, Ollama, Groq, OpenRouter + stubs; `complete` / `complete_stream` |
| Tools | read/write/edit/grep/bash via `/tools/execute` + sandbox policy |
| Bun plugins | Real sidecar spawn, dynamic import, `invoke_hook`; CI hard-gates |
| Native plugins | libloading + test cdylib; CI hard-gates |
| RTK adapter | `@jerekode/rtk` OpenCode2 + `jerekode-rtk-plugin` native; true Bun+native e2e ([ADR 004](docs/adr/004-rtk-dual-adapter.md)) |
| CLI smoke | `jerekode version` + `jerekode serve` v1/v2 session create (binary e2e); full OpenCode CLI drop-in plan in [roadmap-parity-cli.md](docs/roadmap-parity-cli.md) |
| WASM | Module load + `jerekode_hook` ABI |
| MCP / LSP / PTY | call_tool, initialize/hover, portable-pty I/O |
| TUI | Bun `jerekode run` default; optional `native-tui` interactive MVP |
| Release | Auto-release on `main` merge (`0.0.<run_number>` after wipe; seed `0.0.1`); archives only — see [roadmap-releases.md](docs/roadmap-releases.md) |

Historical phase notes and foundation archive: [docs/roadmap-remaining.md](docs/roadmap-remaining.md).  
Closed parity checklist: [docs/roadmap-parity.md](docs/roadmap-parity.md).  
Active CLI parity plan (full drop-in + remaining-work inventory): [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md).  
Active packaging plan: [docs/roadmap-releases.md](docs/roadmap-releases.md).

## Key docs

| Document | Purpose |
|----------|---------|
| [docs/architecture.md](docs/architecture.md) | System design, seams, adapters |
| [docs/conformance.md](docs/conformance.md) | Test seams, fixture rules, TDD policy |
| [docs/development.md](docs/development.md) | Rust standards, build commands |
| [docs/releases.md](docs/releases.md) | Auto-release, `/build`, artifacts |
| [docs/roadmap-parity.md](docs/roadmap-parity.md) | Closed parity checklist (R0–P3e) |
| [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md) | Active CLI ↔ OpenCode command/behavior parity |
| [docs/roadmap-remaining.md](docs/roadmap-remaining.md) | Historical foundation P0–P3 archive |
| [docs/roadmap-releases.md](docs/roadmap-releases.md) | Active packaging / changelog / version-reset plan |

## Architecture Decisions

Recorded in [docs/adr/](docs/adr/). Start with [001-architecture-decisions.md](docs/adr/001-architecture-decisions.md). Plugin runtime: [002-dual-plugin-runtime.md](docs/adr/002-dual-plugin-runtime.md). Release packaging: [003-release-packaging-and-changelogs.md](docs/adr/003-release-packaging-and-changelogs.md). RTK dual adapter: [004-rtk-dual-adapter.md](docs/adr/004-rtk-dual-adapter.md).

## Agent Skills

Installed skills in `.agents/skills/` (also tracked via `skills-lock.json`):

| Skill | When to use |
|-------|-------------|
| [codebase-design](.agents/skills/codebase-design/SKILL.md) | Module design, seam placement, deepening |
| [tdd](.agents/skills/tdd/SKILL.md) | Red-green-refactor, test-at-seams policy |
| [diagnosing-bugs](.agents/skills/diagnosing-bugs/SKILL.md) | Build feedback loops before hypothesizing |
| [rust-best-practices](.agents/skills/rust-best-practices/SKILL.md) | Idiomatic Rust, clippy, error handling |

Read this file first, then the skill relevant to your task. Check ADRs before changing architectural decisions.

## Constraints

- **No upstream code** — OpenCode is a compatibility reference only; no fork, submodule, or vendored source.
- **Conformance-driven** — behavioral parity is proven by owned fixtures, not by importing upstream.
- **Do not weaken CI** — convert soft-skips to hard gates, never the reverse.
