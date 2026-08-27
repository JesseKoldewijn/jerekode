# Packaging & Distribution Notes

## Install script

```bash
./scripts/install.sh
# or
JEREKODE_PREFIX=$HOME/.local ./scripts/install.sh
```

Installs `jerekode` plus aliases `opencode` and `opencode2` into `$JEREKODE_PREFIX/bin` (default `~/.local/bin`).

## Manual aliases

```bash
cargo build -p jerekode-cli --release
ln -s "$(pwd)/target/release/jerekode" ~/.local/bin/opencode
ln -s "$(pwd)/target/release/jerekode" ~/.local/bin/opencode2
```

## Windows

Copy or hardlink `jerekode.exe` as `opencode.exe` / `opencode2.exe` onto a directory in `%PATH%`, or use PowerShell:

```powershell
cargo build -p jerekode-cli --release
Copy-Item target\release\jerekode.exe $env:USERPROFILE\.local\bin\jerekode.exe
Copy-Item target\release\jerekode.exe $env:USERPROFILE\.local\bin\opencode.exe
Copy-Item target\release\jerekode.exe $env:USERPROFILE\.local\bin\opencode2.exe
```

## Runtime deps

| Component | Requirement |
|-----------|-------------|
| Core CLI / server | Rust-built binary only |
| `jerekode run` sidecar | Bun >= 1.1 on PATH |
| SQLite sessions | Optional `sessionDb` in `opencode.json` |
| Native plugins | Platform dylib built against `jerekode-plugin-sdk` |

## Feature flags

```bash
cargo build -p jerekode-cli --features native-tui
cargo bench -p jerekode-plugins
```

## CI releases

See [releases.md](./releases.md) for tagged GitHub Releases and PR `/build` workflow artifacts.

## Upcoming packaging

Installer formats, version reset, changelog quality, and full vs native-only builds: [roadmap-releases.md](./roadmap-releases.md) / [ADR 003](./adr/003-release-packaging-and-changelogs.md).
