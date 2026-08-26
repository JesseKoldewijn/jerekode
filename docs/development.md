# Development Guide

Rust engineering standards and build commands for Jereko contributors and agents.

## Prerequisites

- [Rust](https://rustup.rs/) (2021 edition, stable)
- [Bun](https://bun.sh/) (for sidecar, Phase 2+)

## Build Commands

```bash
# Build all crates
cargo build

# Build release binary
cargo build --release

# Run all workspace tests
cargo test --workspace

# Lint (must pass with zero warnings)
cargo clippy --all-targets --all-features --locked -- -D warnings

# Format check (CI uses --check; use cargo fmt locally to fix)
cargo fmt --check

# Conformance crate only
cargo test -p jereko-conformance
```

The primary CLI binary is **`jereko`**:

```bash
cargo run -p jereko-cli -- version
cargo run -p jereko-cli -- serve
```

## Rust Standards

These standards align with the [rust-best-practices](../.agents/skills/rust-best-practices/SKILL.md) skill and apply from Phase 1 onward.

### Error Handling

| Context | Crate type | Approach |
|---------|------------|----------|
| Library crates | `jereko-core`, `jereko-config`, `jereko-server`, `jereko-providers`, `conformance` | `thiserror` for typed errors |
| Binary | `jereko-cli` | `anyhow` for top-level error propagation |

- Return `Result<T, E>` for fallible operations; avoid `panic!` in production code.
- **No `unwrap()` or `expect()` outside tests.**

### Linting and Documentation

- Run `cargo clippy --all-targets --all-features --locked -- -D warnings` before pushing.
- Public library crates should use `#![warn(missing_docs)]` (moving to `deny` as APIs stabilize).
- Use `#[expect(clippy::lint)]` over `#[allow(...)]` with a justification comment.

### Testing Conventions

- Name tests descriptively: `normalize_should_map_v1_session_to_normalized()`.
- Prefer one assertion per test when practical.
- Use doc tests (`///`) for public API examples.
- Test at **seams** documented in [conformance.md](./conformance.md) — not internals.

### Snapshot Testing

- **JSON fixtures** under `conformance/fixtures/` are the primary source of truth for HTTP conformance.
- **`cargo insta`** is optional for adapter normalize/denormalize round-trips in Phase 1+; not required for Phase 0.

## TDD and Refactoring

Follow the [tdd](../.agents/skills/tdd/SKILL.md) skill:

- Red → green → stop. Refactoring is **not** part of the implementation loop.
- The TDD skill references a `code-review` skill for the refactor stage. That skill is not yet installed. Until it is, **refactoring happens during PR review** — keep red-green cycles minimal and leave structural cleanup for review.

Default test seams are documented in [conformance.md](./conformance.md). Confirm with the user before adding seams outside that table.

## Debugging

Follow the [diagnosing-bugs](../.agents/skills/diagnosing-bugs/SKILL.md) skill:

1. Build a tight, red-capable feedback loop before hypothesizing.
2. Map loop types to conformance layers (see conformance doc).
3. Turn minimized repros into regression tests at the correct seam.

## Module Design

Follow the [codebase-design](../.agents/skills/codebase-design/SKILL.md) skill for seam placement, depth, and adapter vocabulary. See [architecture.md](./architecture.md) for Jereko-specific seam mapping.

## Plugin Host Development (Phase 2+)

Plugin hosts implement the `PluginHost` trait and register hooks with the `PluginOrchestrator`. See [ADR 002](./adr/002-dual-plugin-runtime.md) for the dual-runtime design.

| Host | Development path |
|------|------------------|
| **Bun** | TypeScript/Bun plugins in `sidecar/`; no Rust SDK needed |
| **Native** | Rust SDK crate planned as **`jereko-plugin-sdk`**; stable C ABI in `jereko_plugin.h` (Phase 2.5) |
| **WASM** | Same hook surface via WASM imports/exports (Phase 4) |

Native plugin authors will depend on `jereko-plugin-sdk` for safe Rust bindings over the C ABI. Bun plugin authors continue using the existing sidecar plugin API.

## Agent Context

- Start with [CONTEXT.md](../CONTEXT.md) for crate map and vocabulary.
- Check [docs/adr/](./adr/) before changing architectural decisions.
- See [AGENTS.md](../AGENTS.md) for agent-specific orientation.
- Remaining implementation priorities: [roadmap-remaining.md](./roadmap-remaining.md).

## CI

GitHub Actions runs test, clippy, and fmt check on push/PR — see [`.github/workflows/ci.yml`](../.github/workflows/ci.yml).

`Cargo.lock` is tracked (application binary). CI uses `--locked` to ensure reproducible builds.
