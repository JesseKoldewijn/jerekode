# Packaging & Distribution Notes

## Install script

```bash
./scripts/install.sh
# or
JEREKO_PREFIX=$HOME/.local ./scripts/install.sh
```

Installs `jereko` plus aliases `opencode` and `opencode2` into `$JEREKO_PREFIX/bin` (default `~/.local/bin`).

## Manual aliases

```bash
cargo build -p jereko-cli --release
ln -s "$(pwd)/target/release/jereko" ~/.local/bin/opencode
ln -s "$(pwd)/target/release/jereko" ~/.local/bin/opencode2
```

## Windows

Copy or hardlink `jereko.exe` as `opencode.exe` / `opencode2.exe` onto a directory in `%PATH%`, or use PowerShell:

```powershell
cargo build -p jereko-cli --release
Copy-Item target\release\jereko.exe $env:USERPROFILE\.local\bin\jereko.exe
Copy-Item target\release\jereko.exe $env:USERPROFILE\.local\bin\opencode.exe
Copy-Item target\release\jereko.exe $env:USERPROFILE\.local\bin\opencode2.exe
```

## Runtime deps

| Component | Requirement |
|-----------|-------------|
| Core CLI / server | Rust-built binary only |
| `jereko run` sidecar | Bun >= 1.1 on PATH |
| SQLite sessions | Optional `sessionDb` in `opencode.json` |
| Native plugins | Platform dylib built against `jereko-plugin-sdk` |

## Feature flags

```bash
cargo build -p jereko-cli --features native-tui
cargo bench -p jereko-plugins
```

## CI releases

See [releases.md](./releases.md) for tagged GitHub Releases and PR `/build` workflow artifacts.
