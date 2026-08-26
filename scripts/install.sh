#!/usr/bin/env bash
# Install jereko and optional opencode / opencode2 aliases.
set -euo pipefail

PREFIX="${JEREKO_PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"
mkdir -p "$BIN_DIR"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Building jereko (release)..."
cargo build -p jereko-cli --release

install -m 755 "$ROOT/target/release/jereko" "$BIN_DIR/jereko"
ln -sfn "$BIN_DIR/jereko" "$BIN_DIR/opencode"
ln -sfn "$BIN_DIR/jereko" "$BIN_DIR/opencode2"

echo "Installed:"
echo "  $BIN_DIR/jereko"
echo "  $BIN_DIR/opencode -> jereko"
echo "  $BIN_DIR/opencode2 -> jereko"
echo
echo "Ensure $BIN_DIR is on your PATH."
echo "Sidecar requires Bun (>=1.1): https://bun.sh"
