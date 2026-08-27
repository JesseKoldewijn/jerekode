# Jerekode

Jerekode is a **Rust port of OpenCode** — an OpenCode-compatible AI coding agent runtime. Implementation is a **Rust core** plus a **Bun plugin sidecar**, with dual plugin hosts (Bun + native; WASM optional) and owned conformance fixtures.

It is not a line-for-line fork and does not vendor upstream OpenCode source; compatibility is conformance-driven (owned fixtures at public seams).

## Architecture

See [docs/architecture.md](docs/architecture.md) for system design, adapter layer strategy, and provider registry design.

Conformance: [docs/conformance.md](docs/conformance.md).  
Orientation: [CONTEXT.md](CONTEXT.md), [AGENTS.md](AGENTS.md), [docs/development.md](docs/development.md), [docs/adr/](docs/adr/).  
Closed parity checklist: [docs/roadmap-parity.md](docs/roadmap-parity.md).  
Active CLI ↔ OpenCode command/behavior plan: [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md).  
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

- [Rust](https://rustup.rs/) (edition 2024, stable; MSRV 1.85+) — for building from source
- [Bun](https://bun.sh/) (for sidecar / `jerekode run`) — not bundled in release archives

## Install

Download binaries and installers from [GitHub Releases](https://github.com/JesseKoldewijn/jerekode/releases). Pre-1.0 packages are **unsigned**. The Bun sidecar is **not** included — install Bun separately if you use `jerekode run` (see [sidecar/README.md](sidecar/README.md)).

Supported release targets today: **Windows x64**, **Linux x64**, **macOS x64**, **macOS arm64** (no linux-arm64 / windows-arm64 builds yet).

### Windows (x64)

- **NSIS installer:** run [`jerekode-x64-setup.exe`](https://github.com/JesseKoldewijn/jerekode/releases/latest) (stable alias), or `jerekode-{version}-release-windows-x64-setup.exe`. SmartScreen may warn — unsigned.
- **Portable:** unzip `jerekode-{version}-release-windows-x64.zip` and put `jerekode.exe` on your `PATH`.

### Linux (x64)

```bash
# Arch
sudo pacman -U jerekode-*-release-linux-x64.pkg.tar.zst

# Debian / Ubuntu
sudo dpkg -i jerekode-*-release-linux-x64.deb

# Fedora / RHEL
sudo rpm -i jerekode-*-release-linux-x64.rpm

# AppImage
chmod +x jerekode-*-linux-x64.AppImage
./jerekode-*-linux-x64.AppImage version

# Portable archive
tar -xzf jerekode-*-release-linux-x64.tar.gz
```

### macOS (x64 / arm64)

```bash
# Unsigned .pkg (pick arch)
sudo installer -pkg jerekode-*-macos-arm64.pkg -target /
# sudo installer -pkg jerekode-*-macos-x64.pkg -target /

# Portable archive
tar -xzf jerekode-*-release-macos-arm64.tar.gz
# tar -xzf jerekode-*-release-macos-x64.tar.gz
```

Gatekeeper may block unsigned packages — allow the app in System Settings, or open via Finder (right-click → Open).

### Package managers (templates / future)

AUR, Homebrew, and winget templates live under [`packaging/`](packaging/) — see [packaging/README.md](packaging/README.md) and [packaging/arch/README.md](packaging/arch/README.md). Not published as official taps yet.

### From source

Build with Cargo (below), then optionally install aliases with [`scripts/install.sh`](scripts/install.sh). More detail: [docs/distribution.md](docs/distribution.md), [docs/releases.md](docs/releases.md).

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

- Platform installers and download names: [Install](#install) above; full ops: [docs/releases.md](docs/releases.md)
- Local install / alias helpers: [docs/distribution.md](docs/distribution.md)
- Upcoming packaging (signing, changelog policy, full vs native-only): [docs/roadmap-releases.md](docs/roadmap-releases.md)
- CLI ↔ OpenCode command parity (argv gaps, phases): [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md)

## License

[MIT](LICENSE) © 2026 Jesse Koldewijn
