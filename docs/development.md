# Development

Engineering standards for the Jereko Rust workspace and Bun sidecar.

## Toolchain

- **Edition:** 2024
- **Channel:** stable (CI uses `dtolnay/rust-toolchain@stable`)
- **MSRV:** 1.85+
- **License:** MIT

Prefer `cargo +stable …` locally if your default rustc is a nightly mismatch with CI.

## Commands

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo build -p jereko-test-native-plugin --locked
cargo build -p jereko-rtk-plugin --locked
cargo test --workspace --locked
cargo build -p jereko-cli --release
```

CLI runtime smoke (included in workspace tests): `jereko version` + `jereko serve` v1/v2 session create — see `crates/jereko-cli/tests/cli_smoke.rs`.

First-party plugins (e.g. `@jerekode/rtk`) need **true e2e** against Bun process + native dylib — see [conformance.md](./conformance.md) Layer 5.

Sidecar (matches CI `bun-sidecar` job):

```bash
cd sidecar && bun install && bun run check && bun test
```

Optional native TUI:

```bash
cargo test -p jereko-cli --features native-tui
```

Criterion benches (not PR-gated; nightly workflow):

```bash
cargo bench -p jereko-core --bench hot_paths
```

## Standards

- Zero clippy warnings (`-D warnings` in CI).
- `cargo fmt` required; CI fails on format drift.
- Prefer small PRs that deepen one seam (fixture → impl → green CI).
- Never soft-skip Bun IPC or native plugin CI gates.

## CI

PR and `main` pushes run `.github/workflows/ci.yml` jobs `rust` and `bun-sidecar` (required on `main`). No path filters — see [CONTRIBUTING.md](../CONTRIBUTING.md#ci-on-pull-requests).

PRs also run `.github/workflows/coverage.yml`: Rust **diff coverage** vs `origin/main` (default gate **80%** of changed lines), sticky PR comment with uncovered regions, and informational Bun coverage. See [conformance.md](./conformance.md#coverage-gate-ci).

## TDD at seams

Follow [.agents/skills/tdd/SKILL.md](../.agents/skills/tdd/SKILL.md):

1. Red — failing test at a documented seam / fixture.
2. Green — minimal implementation.
3. Refactor — during PR review until a dedicated code-review skill is installed.

`cargo insta` is optional for adapter normalize/denormalize round-trips.

## Workspace layout tips

- **Monorepo:** Bun workspaces (`packages/*`, `sidecar`) + Cargo workspace (`crates/*`, `packages/rtk/native`, `conformance`).
- Handlers in `jereko-server` operate only on **normalized** types.
- Provider HTTP adapters live in `jereko-providers`; registry stubs are for tests.
- Extension hosts (MCP/LSP/PTY/WASM) hang off `AppState` / `ExtensionHosts`.
- Sandbox policy for tools: `jereko-server` `ToolPolicy`.
- First-party RTK adapter: `packages/rtk` (see [ADR 004](./adr/004-rtk-dual-adapter.md)).

```bash
# Root JS install (workspaces)
bun install
bun test ./packages/rtk
cd sidecar && bun run check && bun test

cargo build -p jereko-rtk-plugin --locked
cargo test -p jereko-rtk-plugin --locked
```

## Releases

See [releases.md](./releases.md) for current auto-release and `/build`. Local packaging: `scripts/package-release.sh`. Upcoming packaging / version reset: [roadmap-releases.md](./roadmap-releases.md).
