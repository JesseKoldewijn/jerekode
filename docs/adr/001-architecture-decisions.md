# ADR 001: Phase 0 Architecture Decisions

**Status:** Accepted  
**Date:** 2026-08-26  
**Context:** Phase 0 scaffolding and Phase 0.5 agent/engineering clarifications

## Decision

Jereko is built as a **Rust core + Bun sidecar** runtime targeting OpenCode API compatibility through owned conformance tests — without importing upstream source code.

## Decisions Recorded

### 1. Primary binary: `jerekode`

- The CLI binary is named **`jerekode`** (crate remains `jereko-cli`).
- Optional aliases **`opencode`** and **`opencode2`** point to the same binary (symlinks or install-time aliases).
- Aliases are not separate implementations.

**Rationale:** Distinct product identity while preserving drop-in compatibility for users of OpenCode-style tooling.

### 2. v1/v2 wire adapter layer

- Both v1 and v2 HTTP APIs are supported via **wire adapters** that normalize to shared types as early as possible.
- Handlers operate exclusively on `adapters/normalized` types.
- Deprecating v1 means removing the v1 wire adapter module, not rewriting core logic.

**Rationale:** Two wire formats justify a real seam (codebase-design: two adapters = real seam). Early normalization minimizes version-specific logic in handlers.

### 3. Bun sidecar as default plugin/TUI path

- TUI and plugins run in a **Bun sidecar** (`sidecar/`), spawned by Rust as a child process.
- Communication uses JSON-line IPC over stdio (Phase 1+).
- A native Rust TUI remains a documented future option, secondary to Bun for plugin fidelity.

**Extended by [ADR 002](./002-dual-plugin-runtime.md):** Bun is the default host for unqualified plugin strings via **BunPluginHost**, coordinated by a **PluginOrchestrator** alongside optional **NativePluginHost** (Phase 2.5) and **WasmPluginHost** (Phase 4). `SidecarPort` feeds BunPluginHost specifically.

**Rationale:** Avoid embedding a JavaScript runtime in Rust while preserving full Bun/TypeScript plugin compatibility.

### 4. Full provider registry from day one

- `jereko-providers` uses a trait-based registry designed for **75+ providers**.
- Built-in providers ship in-crate; plugin-provided providers register via sidecar (Phase 2).
- Tests use `StubProvider` at the trait boundary; real provider tests mock at the HTTP boundary only.

**Rationale:** Registry shape is harder to retrofit than to design upfront; stub-at-trait keeps unit tests fast and boundary-correct.

### 5. Conformance-only upstream reference

- OpenCode is referenced as a **behavioral compatibility target** only.
- No fork, submodule, vendor, or clone of upstream source in this repository.
- Compatibility is validated through **owned fixtures** under `conformance/fixtures/`.

**Rationale:** Avoid merge conflicts, license entanglement, and architectural coupling while still targeting behavioral parity.

### 6. SidecarPort seam (Phase 2 prep)

- Rust will expose a **`SidecarPort`** trait for sidecar IPC.
- Production adapter: spawn Bun process, JSON-line stdio.
- Test adapter: in-memory message queue.

**Rationale:** DEEPENING category 3 (remote-but-owned service). Design the seam before Phase 2 to avoid ad-hoc IPC calls.

### 7. Engineering and workflow defaults (Phase 0.5)

| Topic | Decision |
|-------|----------|
| `Cargo.lock` | Tracked — jereko is an application binary |
| Agent context | `CONTEXT.md` + `AGENTS.md` at repo root |
| ADRs | `docs/adr/` created now; new decisions get numbered ADRs |
| Skills | `.agents/skills/` committed with `skills-lock.json` |
| HTTP tests | Both in-process router tests (fast) and black-box `jerekode serve` tests (Layer 3 conformance) |
| Snapshot testing | JSON fixtures primary; `cargo insta` optional for adapter round-trips later |
| Refactor stage | PR review until `code-review` skill is added |
| Distribution | Cargo/install for now; Pinokio/Gepeto launchers and Cursor SDK integration noted as future paths |

## Consequences

- Phase 1 work focuses on config, HTTP adapter round-trips, and fixture-driven tests — not upstream integration.
- Phase 2 work must implement `PluginOrchestrator` and `PluginHost` trait (see ADR 002), with `SidecarPort` scoped to BunPluginHost — not ad-hoc sidecar calls scattered across crates.
- CI runs `cargo test`, `cargo clippy --locked`, and `cargo fmt --check` (see `.github/workflows/ci.yml`).
- Agents and contributors use shared vocabulary from `CONTEXT.md` and codebase-design skill.

## Related Documents

- [docs/architecture.md](../architecture.md)
- [docs/conformance.md](../conformance.md)
- [docs/development.md](../development.md)
- [CONTEXT.md](../../CONTEXT.md)
- [ADR 002: Dual Plugin Runtime Architecture](./002-dual-plugin-runtime.md) — extends Decision 3 and Decision 6
