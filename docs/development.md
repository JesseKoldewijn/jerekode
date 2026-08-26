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
cargo test --workspace --locked
cargo build -p jereko-cli --release
```

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

## TDD at seams

Follow [.agents/skills/tdd/SKILL.md](../.agents/skills/tdd/SKILL.md):

1. Red — failing test at a documented seam / fixture.
2. Green — minimal implementation.
3. Refactor — during PR review until a dedicated code-review skill is installed.

`cargo insta` is optional for adapter normalize/denormalize round-trips.

## Workspace layout tips

- Handlers in `jereko-server` operate only on **normalized** types.
- Provider HTTP adapters live in `jereko-providers`; registry stubs are for tests.
- Extension hosts (MCP/LSP/PTY/WASM) hang off `AppState` / `ExtensionHosts`.
- Sandbox policy for tools: `jereko-server` `ToolPolicy`.

## Releases

See [releases.md](./releases.md) for current auto-release and `/build`. Local packaging: `scripts/package-release.sh`. Upcoming packaging / version reset: [roadmap-releases.md](./roadmap-releases.md).
