# Jerekode

Jerekode is a **Rust port of OpenCode** — an OpenCode-compatible AI coding agent runtime. Implementation is a **Rust core** plus a **Bun plugin sidecar**, with dual plugin hosts (Bun + native; WASM optional) and owned conformance fixtures.

It is not a line-for-line fork and does not vendor upstream OpenCode source; compatibility is conformance-driven (owned fixtures at public seams).

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
│   ├── jerekode-core/          # Domain types, session models
│   ├── jerekode-config/        # Config loading and merge precedence
│   ├── jerekode-server/        # HTTP server + v1/v2 adapters + tools/extensions
│   ├── jerekode-cli/           # CLI binary (jerekode)
│   ├── jerekode-providers/     # Provider registry + streaming adapters
│   ├── jerekode-plugins/       # PluginOrchestrator + Bun/native/WASM hosts
│   ├── jerekode-plugin-sdk/    # Native plugin SDK / C ABI
│   └── jerekode-test-native-plugin/  # Test cdylib for NativePluginHost CI
├── sidecar/                  # Bun plugin host (TUI + server plugins)
├── conformance/              # Owned fixture-driven compatibility tests
└── docs/                     # Architecture, conformance, roadmaps, ADRs
```

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2024, stable; MSRV 1.85+)
- [Bun](https://bun.sh/) (for sidecar / `jerekode run`)

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

The primary CLI binary is **`jerekode`**:

```bash
cargo run -p jerekode-cli -- version
cargo run -p jerekode-cli -- serve
cargo run -p jerekode-cli -- run
```

Optional native TUI helpers are behind the `native-tui` Cargo feature (ratatui MVP in `jerekode-plugins`; Bun `jerekode run` remains the default interactive path):

```bash
cargo build -p jerekode-cli --features native-tui
```

## Binary Aliases

The primary binary is `jerekode`. Optional **`opencode`** and **`opencode2`** aliases point to the same binary.

### Symlinks (Unix/macOS)

```bash
cargo build --release
ln -s target/release/jerekode ~/.local/bin/opencode
ln -s target/release/jerekode ~/.local/bin/opencode2
```

### Cargo bin aliases

Add to `~/.cargo/config.toml`:

```toml
[alias]
opencode = "run -p jerekode-cli --"
opencode2 = "run -p jerekode-cli --"
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
