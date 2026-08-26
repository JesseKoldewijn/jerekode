# Jereko — Agent Context

Start here for a navigable map of the repository. Human-oriented docs live in [docs/](docs/); this file is optimized for agents and contributors who need orientation fast.

## Crate Map

| Crate | Path | Responsibility |
|-------|------|----------------|
| `jereko-core` | `crates/jereko-core/` | Domain types, session models, shared errors |
| `jereko-config` | `crates/jereko-config/` | Config loading, merge precedence, `opencode.json` / `tui.json` types |
| `jereko-server` | `crates/jereko-server/` | Axum HTTP server, v1/v2 wire adapters, normalized handler types |
| `jereko-cli` | `crates/jereko-cli/` | CLI binary (`jereko`; aliases `opencode`, `opencode2`) |
| `jereko-providers` | `crates/jereko-providers/` | `Provider` trait, registry (designed for 75+ providers) |
| `jereko-conformance` | `conformance/` | Owned fixture-driven compatibility tests |

Supporting directories:

| Path | Role |
|------|------|
| `sidecar/` | Bun plugin host (TUI + server plugins); JSON-line IPC with Rust core |
| `docs/` | Architecture, conformance, development, ADRs |
| `.agents/skills/` | Installed agent skills (codebase-design, tdd, diagnosing-bugs, rust-best-practices) |

## Contribution rule

All code changes reach `main` **only via pull request**. Never push directly to `main`. Release CI may push a version bump after merge — see [CONTRIBUTING.md](CONTRIBUTING.md).

## Domain Vocabulary

Use these terms consistently (see [docs/architecture.md](docs/architecture.md) for full disambiguation):

| Term | Meaning |
|------|---------|
| **Session** | A conversation context in `jereko-core` — messages, status, identity |
| **Provider** | An LLM backend registered in `jereko-providers`; implements the `Provider` trait |
| **Sidecar** | The Bun process (`sidecar/`) that hosts TUI and plugin code; communicates with Rust via JSON-line IPC; transport for **BunPluginHost** |
| **PluginOrchestrator** | Rust coordinator for plugin hook registry, load order, priority, and dispatch across all hosts |
| **PluginHost** | Trait generalizing plugin loading — implementations: BunPluginHost, NativePluginHost, WasmPluginHost |
| **BunPluginHost** | Default plugin host for unqualified config strings; full OpenCode fidelity via SidecarPort IPC |
| **NativePluginHost** | In-process dylib host via stable C ABI (`jereko_plugin.h`); explicit config only; server hooks (Phase 2.5+) |
| **WasmPluginHost** | Sandboxed WASM host for untrusted plugins; explicit config only (Phase 4) |
| **Normalized types** | Version-agnostic request/response shapes in `jereko-server/src/adapters/normalized/`; handlers operate only on these |
| **Wire adapter** | Translates v1 or v2 HTTP wire format ↔ normalized types (`jereko-server/src/adapters/v1/`, `v2/`) |
| **Provider adapter** | A concrete `Provider` implementation (e.g. Anthropic, OpenAI, `StubProvider`) |
| **Sidecar adapter** | Rust-side transport for BunPluginHost IPC — production (spawn Bun) or test (in-memory); see `SidecarPort` in architecture docs |

**Seam** and **interface** follow [codebase-design](.agents/skills/codebase-design/SKILL.md) vocabulary: a seam is where a module's interface lives; an adapter satisfies that interface.

## Phase Roadmap

| Phase | Focus | Status |
|-------|-------|--------|
| **0** | Workspace scaffolding, stubs, architecture foundations | Complete |
| **0.5** | Agent context, ADRs, engineering standards, CI stub | Complete |
| **1** | Config JSONC, HTTP adapter round-trips, fixture-driven conformance | Complete |
| **2** | PluginOrchestrator + BunPluginHost, SidecarPort IPC, TUI via `jereko run` | Complete (real Bun spawn) |
| **2.5** | NativePluginHost — in-process dylib, server hooks | Complete (libloading + SDK + test cdylib) |
| **3** | Bun TUI plugins; full provider implementations | Partial (OpenAI/Anthropic/Ollama + tools; TUI bootstrap via IPC) |
| **4** | WasmPluginHost, MCP/LSP/PTY, SQLite persistence | Partial (SQLite real; WASM load; MCP/LSP/PTY seams) |
| **5** | Native TUI plugins; perf baseline; `native-tui` feature | Partial (ratatui stub + Criterion benches) |

**What's next:** Incremental depth only — see remaining gaps in [docs/roadmap-remaining.md](docs/roadmap-remaining.md) (streaming providers, full MCP/LSP/PTY protocols, WASI hook ABI, portable-pty, 75+ providers).

Detailed design: [docs/architecture.md](docs/architecture.md).  
Testing approach: [docs/conformance.md](docs/conformance.md).  
Engineering standards: [docs/development.md](docs/development.md).  
Releases & `/build`: [docs/releases.md](docs/releases.md).  
Remaining work: [docs/roadmap-remaining.md](docs/roadmap-remaining.md).

## Architecture Decisions

Recorded in [docs/adr/](docs/adr/). Start with [001-architecture-decisions.md](docs/adr/001-architecture-decisions.md) for Phase 0 decisions. Plugin runtime strategy: [002-dual-plugin-runtime.md](docs/adr/002-dual-plugin-runtime.md).

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
