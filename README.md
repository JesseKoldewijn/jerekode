# Jereko

Jereko is an AI coding agent runtime with OpenCode API compatibility, built as a Rust core with a Bun plugin sidecar.

**Phase 0** provides workspace scaffolding — stubs and architecture foundations for subsequent implementation phases.

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design, adapter layer strategy, and provider registry design.

Conformance testing approach: [docs/conformance.md](docs/conformance.md).

Agent and contributor orientation: [CONTEXT.md](CONTEXT.md), [AGENTS.md](AGENTS.md), [docs/development.md](docs/development.md), [docs/adr/](docs/adr/).

## Project Structure

```
jerekode/
├── crates/
│   ├── jereko-core/       # Domain types, session models
│   ├── jereko-config/     # Config loading and merge precedence
│   ├── jereko-server/     # HTTP server + v1/v2 adapter layer
│   ├── jereko-cli/        # CLI binary (jereko)
│   └── jereko-providers/  # Provider registry (75+ designed)
├── sidecar/               # Bun plugin host (TUI + server plugins)
├── conformance/           # Owned fixture-driven compatibility tests
└── docs/                  # Architecture and conformance documentation
```

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable; MSRV 1.85+)
- [Bun](https://bun.sh/) (for sidecar, Phase 2+)

## Contributing

All changes land on `main` **only via pull request**. Never push directly to `main`. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Build

```bash
# Build all crates
cargo build

# Build release binary
cargo build --release

# Run tests
cargo test

# Lint (zero warnings required)
cargo clippy --all-targets --all-features --locked -- -D warnings

# Format check
cargo fmt --check
```

The primary CLI binary is **`jereko`**:

```bash
cargo run -p jereko-cli -- version
cargo run -p jereko-cli -- serve
cargo run -p jereko-cli -- run
```

## Binary Aliases

The primary binary is `jereko`. Optional **`opencode`** and **`opencode2`** aliases point to the same binary.

### Symlinks (Unix/macOS)

```bash
cargo build --release
ln -s target/release/jereko ~/.local/bin/opencode
ln -s target/release/jereko ~/.local/bin/opencode2
```

### Cargo bin aliases

Add to `~/.cargo/config.toml`:

```toml
[alias]
opencode = "run -p jereko-cli --"
opencode2 = "run -p jereko-cli --"
```

Then: `cargo opencode version`

### Windows

```powershell
# After cargo build --release
New-Item -ItemType HardLink -Path "$env:USERPROFILE\.cargo\bin\opencode.exe" -Target "target\release\jereko.exe"
```

Or use `mklink` for symlinks (requires Developer Mode or admin).

## Sidecar (stub)

```bash
cd sidecar
bun install
bun run start
```

See [sidecar/README.md](sidecar/README.md) for the IPC contract.

## Releases & PR builds

- Cut a tagged release or run the Release workflow: [docs/releases.md](docs/releases.md)
- On a PR, comment `/build` (or `/build debug`) to upload multi-platform workflow artifacts — **no** GitHub Release
- Install / alias helpers: [docs/distribution.md](docs/distribution.md)

## License

[MIT](LICENSE) © 2026 Jesse Koldewijn
