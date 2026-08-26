# Jereko — Agent Context

Start here for a navigable map of the repository. Human-oriented docs live in [docs/](docs/); this file is optimized for agents and contributors who need orientation fast.

## Current state

Jereko is a **working** OpenCode-compatible AI coding agent runtime: Rust core, Bun plugin sidecar, dual plugin hosts (ADR 002), owned conformance fixtures, and a green release pipeline on `main`.

Documented parity slices **R0–P3e** are complete — see [docs/roadmap-parity.md](docs/roadmap-parity.md). Further growth (more providers, richer MCP/LSP/WASM surfaces) is incremental, not foundation scaffolding. **Active forward plan:** release packaging / changelogs / version reset — [docs/roadmap-releases.md](docs/roadmap-releases.md).

## Crate Map

| Crate | Path | Responsibility |
|-------|------|----------------|
| `jereko-core` | `crates/jereko-core/` | Domain types, session models, shared errors |
| `jereko-config` | `crates/jereko-config/` | Config loading, merge precedence, `opencode.json` / `tui.json` types |
| `jereko-server` | `crates/jereko-server/` | Axum HTTP server, v1/v2 wire adapters, tools, extensions, policy |
| `jereko-cli` | `crates/jereko-cli/` | CLI binary (`jereko`; aliases `opencode`, `opencode2`) |
| `jereko-providers` | `crates/jereko-providers/` | `Provider` trait, registry, streaming, HTTP adapters |
| `jereko-plugins` | `crates/jereko-plugins/` | PluginOrchestrator, Bun/native/WASM hosts, SidecarPort |
| `jereko-plugin-sdk` | `crates/jereko-plugin-sdk/` | Native plugin C ABI / Rust SDK |
| `jereko-conformance` | `conformance/` | Owned fixture-driven compatibility tests |

Supporting directories:

| Path | Role |
|------|------|
| `sidecar/` | Bun plugin host (TUI + server plugins); JSON-line IPC with Rust core |
| `docs/` | Architecture, conformance, development, ADRs, roadmaps |
| `.agents/skills/` | Installed agent skills (codebase-design, tdd, diagnosing-bugs, rust-best-practices) |

## Contribution rule

All code changes reach `main` **only via pull request**. Never push directly to `main`. Release CI may open a version-bump sync PR after merge — see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/releases.md](docs/releases.md).

## Domain Vocabulary

Use these terms consistently (see [docs/architecture.md](docs/architecture.md) for full disambiguation):

| Term | Meaning |
|------|---------|
| **Session** | A conversation context in `jereko-core` — messages, status, identity |
| **Provider** | An LLM backend registered in `jereko-providers`; implements the `Provider` trait |
| **Sidecar** | The Bun process (`sidecar/`) that hosts TUI and plugin code; communicates with Rust via JSON-line IPC; transport for **BunPluginHost** |
| **PluginOrchestrator** | Rust coordinator for plugin hook registry, load order, priority, and dispatch across all hosts |
| **PluginHost** | Trait generalizing plugin loading — implementations: BunPluginHost, NativePluginHost, WasmPluginHost |
| **BunPluginHost** | Default plugin host for unqualified config strings; OpenCode-compatible hooks via SidecarPort IPC |
| **NativePluginHost** | In-process dylib host via stable C ABI (`jereko_plugin.h`); explicit config only |
| **WasmPluginHost** | Sandboxed WASM host; `jereko_hook` export (host fallback when absent) |
| **Normalized types** | Version-agnostic request/response shapes in `jereko-server/src/adapters/normalized/`; handlers operate only on these |
| **Wire adapter** | Translates v1 or v2 HTTP wire format ↔ normalized types (`jereko-server/src/adapters/v1/`, `v2/`) |
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
| WASM | Module load + `jereko_hook` ABI |
| MCP / LSP / PTY | call_tool, initialize/hover, portable-pty I/O |
| TUI | Bun `jereko run` default; optional `native-tui` interactive MVP |
| Release | Auto-release on `main` merge (`0.1.<run_number>` today); archives only — see [roadmap-releases.md](docs/roadmap-releases.md) |

Historical phase notes and foundation archive: [docs/roadmap-remaining.md](docs/roadmap-remaining.md).  
Active parity board: [docs/roadmap-parity.md](docs/roadmap-parity.md).

## Key docs

| Document | Purpose |
|----------|---------|
| [docs/architecture.md](docs/architecture.md) | System design, seams, adapters |
| [docs/conformance.md](docs/conformance.md) | Test seams, fixture rules, TDD policy |
| [docs/development.md](docs/development.md) | Rust standards, build commands |
| [docs/releases.md](docs/releases.md) | Auto-release, `/build`, artifacts |
| [docs/roadmap-parity.md](docs/roadmap-parity.md) | True OpenCode parity progress board |
| [docs/roadmap-remaining.md](docs/roadmap-remaining.md) | Historical foundation P0–P3 archive |
| [docs/roadmap-releases.md](docs/roadmap-releases.md) | Release packaging, changelogs, version reset plan |

## Architecture Decisions

Recorded in [docs/adr/](docs/adr/). Start with [001-architecture-decisions.md](docs/adr/001-architecture-decisions.md). Plugin runtime strategy: [002-dual-plugin-runtime.md](docs/adr/002-dual-plugin-runtime.md). Release packaging: [003-release-packaging-and-changelogs.md](docs/adr/003-release-packaging-and-changelogs.md).

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
