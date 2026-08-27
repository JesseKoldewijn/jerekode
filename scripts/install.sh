#!/usr/bin/env bash
# Install jerekode and optional opencode / opencode2 aliases.
set -euo pipefail

PREFIX="${JEREKODE_PREFIX:-${JEREKO_PREFIX:-$HOME/.local}}"
BIN_DIR="$PREFIX/bin"
mkdir -p "$BIN_DIR"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Building jerekode (release)..."
cargo build -p jerekode-cli --release

install -m 755 "$ROOT/target/release/jerekode" "$BIN_DIR/jerekode"
ln -sfn "$BIN_DIR/jerekode" "$BIN_DIR/opencode"
ln -sfn "$BIN_DIR/jerekode" "$BIN_DIR/opencode2"

echo "Installed:"
echo "  $BIN_DIR/jerekode"
echo "  $BIN_DIR/opencode -> jerekode"
echo "  $BIN_DIR/opencode2 -> jerekode"
echo
echo "Ensure $BIN_DIR is on your PATH."
echo "Sidecar requires Bun (>=1.1): https://bun.sh"
