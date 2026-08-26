# Jereko

Jereko is an AI coding agent runtime with OpenCode API compatibility: a **Rust core** plus a **Bun plugin sidecar**, dual plugin hosts (Bun + native; WASM optional), and owned conformance fixtures.

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design, adapter layer strategy, and provider registry design.

Conformance: [docs/conformance.md](docs/conformance.md).  
Orientation: [CONTEXT.md](CONTEXT.md), [AGENTS.md](AGENTS.md), [docs/development.md](docs/development.md), [docs/adr/](docs/adr/).  
Closed parity checklist: [docs/roadmap-parity.md](docs/roadmap-parity.md).  
Active packaging / release plan: [docs/roadmap-releases.md](docs/roadmap-releases.md).

## Project Structure

```
jerekode/
├── crates/
│   ├── jereko-core/          # Domain types, session models
│   ├── jereko-config/        # Config loading and merge precedence
│   ├── jereko-server/        # HTTP server + v1/v2 adapters + tools/extensions
│   ├── jereko-cli/           # CLI binary (jereko)
│   ├── jereko-providers/     # Provider registry + streaming adapters
│   ├── jereko-plugins/       # PluginOrchestrator + Bun/native/WASM hosts
│   ├── jereko-plugin-sdk/    # Native plugin SDK / C ABI
│   └── jereko-test-native-plugin/  # Test cdylib for NativePluginHost CI
├── sidecar/                  # Bun plugin host (TUI + server plugins)
├── conformance/              # Owned fixture-driven compatibility tests
└── docs/                     # Architecture, conformance, roadmaps, ADRs
```

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable; MSRV 1.85+)
- [Bun](https://bun.sh/) (for sidecar / `jereko run`)

## Contributing

All changes land on `main` **only via pull request**. Never push directly to `main`. See [CONTRIBUTING.md](CONTRIBUTING.md).

## Build

```bash
# Build all crates
cargo build

# Build release binary
cargo build --release

# Run tests (matches CI)
cargo test --workspace --locked

# Lint (zero warnings required)
cargo clippy --all-targets --all-features --locked -- -D warnings

# Format check (matches CI)
cargo fmt --all -- --check
```

The primary CLI binary is **`jereko`**:

```bash
cargo run -p jereko-cli -- version
cargo run -p jereko-cli -- serve
cargo run -p jereko-cli -- run
```

Optional native TUI helpers are behind the `native-tui` Cargo feature (ratatui MVP in `jereko-plugins`; Bun `jereko run` remains the default interactive path):

```bash
cargo build -p jereko-cli --features native-tui
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

Install helpers: [scripts/install.sh](scripts/install.sh), [docs/distribution.md](docs/distribution.md).

## HTTP surface (v1 / v2)

Both adapter versions are served from one process. Representative routes:

| Concern | v1 | v2 |
|---------|----|----|
| Sessions | `POST/GET/DELETE /v1/session[/{id}]` | `POST/GET/DELETE /v2/sessions[/{id}]` |
| Messages | `GET/POST /v1/session/{id}/message` | `GET/POST /v2/sessions/{id}/messages` |
| Stream | `POST .../message/stream` | `POST .../messages/stream` |
| Providers | `GET /v1/providers` | `GET /v2/providers` |
| Tools | `POST /v1/tools/execute` | `POST /v2/tools/execute` |

Extensions: `/extensions/mcp/*`, `/extensions/lsp/*`, PTY helpers.

## Sidecar

The Bun sidecar hosts TUI and JS/TS plugins over JSON-line IPC. CI hard-gates Bun IPC and native dylib loading — soft-skips are not allowed.

## Releases & PR builds

- Auto-release on merge to `main`, tags, and `/build` PR artifacts: [docs/releases.md](docs/releases.md)
- Install / alias helpers: [docs/distribution.md](docs/distribution.md)
- Upcoming packaging (installers, changelog fix, version reset, full vs native-only): [docs/roadmap-releases.md](docs/roadmap-releases.md)

## License

[MIT](LICENSE) © 2026 Jesse Koldewijn
