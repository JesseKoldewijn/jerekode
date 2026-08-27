#!/usr/bin/env bash
# Smoke-test a packaged release archive by extracting and running `jerekode version`.
#
# Usage:
#   smoke-release-artifact.sh <archive-path>
#
# Supports `.tar.gz` (linux/macos) and `.zip` (windows) from package-release.sh.

set -euo pipefail

ARCHIVE="${1:?archive path required}"
if [[ ! -f "$ARCHIVE" ]]; then
  echo "error: archive not found: $ARCHIVE" >&2
  exit 1
fi

WORKDIR="$(mktemp -d)"
trap 'rm -rf "$WORKDIR"' EXIT

if [[ "$ARCHIVE" == *.zip ]]; then
  if command -v unzip >/dev/null 2>&1; then
    unzip -q "$ARCHIVE" -d "$WORKDIR"
  else
    powershell.exe -NoProfile -Command \
      "Expand-Archive -Path '$(cygpath -w "$ARCHIVE" 2>/dev/null || echo "$ARCHIVE")' -DestinationPath '$(cygpath -w "$WORKDIR" 2>/dev/null || echo "$WORKDIR")' -Force"
  fi
else
  tar -xzf "$ARCHIVE" -C "$WORKDIR"
fi

BIN=""
while IFS= read -r -d '' candidate; do
  BIN="$candidate"
  break
done < <(find "$WORKDIR" -type f \( -name jerekode -o -name jerekode.exe \) -print0)

if [[ -z "$BIN" ]]; then
  echo "error: jerekode binary not found inside $ARCHIVE" >&2
  find "$WORKDIR" -type f >&2 || true
  exit 1
fi

chmod +x "$BIN" 2>/dev/null || true
echo "Smoke: $BIN version"
"$BIN" version
