#!/usr/bin/env bash
# Package the jereko binary into a named archive.
#
# Usage:
#   package-release.sh <archive_stem> <binary-path> <out-dir> [profile]
#
# Produces:
#   {archive_stem}.tar.gz  (non-windows stems, or when stem has no .zip hint)
#   {archive_stem}.zip     (when archive_stem contains "-windows-")
#
# Convention for archive_stem:
#   Release:  jereko-{version}-release-{os}-{arch}
#   PR build: jereko-pr{N}-{profile}-{os}-{arch}
#
# Archive contents: binary, README snippet, sidecar notes.

set -euo pipefail

STEM="${1:?archive stem required}"
BINARY="${2:?binary path required}"
OUT_DIR="${3:?out-dir required}"
PROFILE="${4:-release}"

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found: $BINARY" >&2
  exit 1
fi

STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

STAGE_ROOT="${STAGING}/${STEM}"
mkdir -p "$STAGE_ROOT"
mkdir -p "$OUT_DIR"

BIN_BASENAME="$(basename "$BINARY")"
cp "$BINARY" "${STAGE_ROOT}/${BIN_BASENAME}"
if [[ "$STEM" != *-windows-* ]]; then
  chmod +x "${STAGE_ROOT}/${BIN_BASENAME}"
fi

cat >"${STAGE_ROOT}/README.txt" <<EOF
jereko (${PROFILE} profile)
Archive: ${STEM}

Binary: ${BIN_BASENAME}

Quick start:
  ./${BIN_BASENAME} version
  ./${BIN_BASENAME} serve
  ./${BIN_BASENAME} run

Docs: https://github.com/jerekode/jerekode
Releases: docs/releases.md in the repository.
EOF

cat >"${STAGE_ROOT}/SIDECAR.txt" <<EOF
Bun sidecar (optional plugin host)
=================================

The Rust binary is self-contained for core serve/CLI flows.
TUI plugins and Bun-hosted plugins need the sidecar from the
repository (sidecar/) and Bun >= 1.1 (CI pins 1.2.5):

  cd sidecar && bun install && bun run start

Release archives intentionally ship the Rust binary only.
Package the sidecar from source when you need plugin fidelity.
See sidecar/README.md for the JSON-line IPC contract.
EOF

OUT_DIR="$(cd "$OUT_DIR" && pwd)"
(
  cd "$STAGING"
  if [[ "$STEM" == *-windows-* ]]; then
    ARCHIVE="${STEM}.zip"
    if command -v zip >/dev/null 2>&1; then
      zip -9 -r "${OUT_DIR}/${ARCHIVE}" "$STEM"
    else
      # Compress in-place then move. Git Bash `pwd` paths like /d/a/... are not
      # valid PowerShell filesystem paths (become \d\a\...).
      powershell.exe -NoProfile -Command \
        "Compress-Archive -Path '${STEM}' -DestinationPath '${ARCHIVE}' -Force"
      mv "${ARCHIVE}" "${OUT_DIR}/${ARCHIVE}"
    fi
  else
    ARCHIVE="${STEM}.tar.gz"
    tar -czf "${OUT_DIR}/${ARCHIVE}" "$STEM"
  fi
  echo "Wrote ${OUT_DIR}/${ARCHIVE}"
)
