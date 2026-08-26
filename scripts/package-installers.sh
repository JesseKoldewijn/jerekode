#!/usr/bin/env bash
# Build OS-specific installers alongside portable archives (P2).
#
# Usage:
#   package-installers.sh <target_os> <arch> <binary-path> <out-dir> [version]
#
# target_os: linux | macos | windows
# version defaults to NFPM_VERSION / PKG_VERSION env or reads Cargo.toml via set-version.sh

set -euo pipefail

TARGET_OS="${1:?target_os required (linux|macos|windows)}"
ARCH="${2:?arch required (x64|arm64)}"
BINARY="${3:?binary path required}"
OUT_DIR="${4:?out-dir required}"
VERSION="${5:-${PKG_VERSION:-}}"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ -z "$VERSION" ]]; then
  chmod +x scripts/set-version.sh
  VERSION="$(./scripts/set-version.sh --print)"
fi

if [[ ! -f "$BINARY" ]]; then
  echo "error: binary not found: $BINARY" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

write_readme_files() {
  cat >"${STAGING}/README.txt" <<EOF
jereko ${VERSION} (release)
OpenCode-compatible AI coding agent runtime (Rust).

  jereko version
  jereko serve
  jereko run

Docs: https://github.com/JesseKoldewijn/jerekode
EOF
  cat >"${STAGING}/SIDECAR.txt" <<EOF
Bun sidecar is not bundled. Install Bun >= 1.1 for plugin fidelity.
See sidecar/README.md in the repository.
EOF
}

copy_binary() {
  local dest_name="jereko"
  if [[ "$TARGET_OS" == "windows" ]]; then
    dest_name="jereko.exe"
  fi
  cp "$BINARY" "${STAGING}/${dest_name}"
  if [[ "$TARGET_OS" != "windows" ]]; then
    chmod +x "${STAGING}/${dest_name}"
  fi
}

package_linux() {
  write_readme_files
  copy_binary
  cp "${STAGING}/jereko" "${STAGING}/jereko.bin"

  NFPM_ARCH="amd64"
  [[ "$ARCH" == "arm64" ]] && NFPM_ARCH="arm64"
  export NFPM_VERSION="$VERSION" NFPM_ARCH

  NFPM_BIN="${NFPM_BIN:-}"
  if [[ -z "$NFPM_BIN" ]]; then
    NFPM_VERSION_BIN="${NFPM_TOOL_VERSION:-2.47.0}"
    NFPM_CACHE="${HOME}/.cache/nfpm"
    mkdir -p "$NFPM_CACHE"
    NFPM_BIN="${NFPM_CACHE}/nfpm"
    if [[ ! -x "$NFPM_BIN" ]]; then
      curl -sSfL \
        "https://github.com/goreleaser/nfpm/releases/download/v${NFPM_VERSION_BIN}/nfpm_${NFPM_VERSION_BIN}_Linux_x86_64.tar.gz" \
        | tar xz -C "$NFPM_CACHE" nfpm
      chmod +x "$NFPM_BIN"
    fi
  fi

  NFPM_CFG="${STAGING}/nfpm.yaml"
  envsubst '${NFPM_VERSION} ${NFPM_ARCH}' < packaging/nfpm/jereko.yaml >"$NFPM_CFG"
  mkdir -p "${STAGING}/staging"
  cp "${STAGING}/jereko" "${STAGING}/staging/jereko"
  cp "${STAGING}/README.txt" "${STAGING}/staging/"
  cp "${STAGING}/SIDECAR.txt" "${STAGING}/staging/"
  (cd "$STAGING" && "$NFPM_BIN" pkg --config nfpm.yaml --packager deb --target "${OUT_DIR}/jereko-${VERSION}-release-linux-${ARCH}.deb")
  (cd "$STAGING" && "$NFPM_BIN" pkg --config nfpm.yaml --packager rpm --target "${OUT_DIR}/jereko-${VERSION}-release-linux-${ARCH}.rpm")

  # AppImage
  APPIMAGETOOL="${APPIMAGETOOL:-${HOME}/.cache/appimagetool/appimagetool-x86_64.AppImage}"
  if [[ ! -f "$APPIMAGETOOL" ]]; then
    mkdir -p "$(dirname "$APPIMAGETOOL")"
    curl -sSfL -o "$APPIMAGETOOL" \
      "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
  fi
  APPDIR="${STAGING}/jereko.AppDir"
  rm -rf "$APPDIR"
  mkdir -p "${APPDIR}/usr/bin" "${APPDIR}/usr/share/doc/jereko"
  cp "${STAGING}/jereko" "${APPDIR}/usr/bin/jereko"
  cp "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "${APPDIR}/usr/share/doc/jereko/"
  cat >"${APPDIR}/jereko.desktop" <<EOF
[Desktop Entry]
Name=jereko
Comment=OpenCode-compatible AI coding agent runtime
Exec=jereko
Icon=jereko
Terminal=true
Type=Application
Categories=Development;
EOF
  # Minimal icon silences appimagetool missing-icon warnings
  printf '\x89PNG\r\n\x1a\n' > "${APPDIR}/jereko.png" 2>/dev/null || touch "${APPDIR}/jereko.png"
  cat >"${APPDIR}/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/jereko" "$@"
EOF
  chmod +x "${APPDIR}/AppRun" "${APPDIR}/usr/bin/jereko"
  # appimagetool requires fuse on some hosts; --appimage-extract-and-run works without fuse in CI
  ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" \
    "${OUT_DIR}/jereko-${VERSION}-release-linux-${ARCH}.AppImage"

  # Arch .pkg.tar.zst via Docker (makepkg on Arch Linux)
  if command -v docker >/dev/null 2>&1; then
    ARCH_PKG_DIR="${STAGING}/archpkg"
    rm -rf "$ARCH_PKG_DIR"
    mkdir -p "$ARCH_PKG_DIR"
    cp packaging/arch/PKGBUILD "$ARCH_PKG_DIR/"
    cp "${STAGING}/jereko" "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "$ARCH_PKG_DIR/"
    sed -i "s/^pkgver=.*/pkgver=${VERSION}/" "$ARCH_PKG_DIR/PKGBUILD"
    docker run --rm \
      -v "${ARCH_PKG_DIR}:/build" \
      -w /build \
      archlinux:latest \
      bash -euxo pipefail -c '
        pacman -Sy --noconfirm base-devel
        useradd -m builder || true
        chown -R builder:builder /build
        su builder -c "cd /build && makepkg --noconfirm --skippgpcheck"
      ' || true
    PKG="$(find "$ARCH_PKG_DIR" -maxdepth 1 -name 'jereko-*.pkg.tar.zst' | head -n1 || true)"
    if [[ -n "$PKG" ]]; then
      cp "$PKG" "${OUT_DIR}/jereko-${VERSION}-release-linux-${ARCH}.pkg.tar.zst"
    else
      echo "warning: Arch package build skipped or failed (docker/makepkg)" >&2
    fi
  else
    echo "warning: docker not available; skipping Arch .pkg.tar.zst" >&2
  fi
}

package_macos() {
  write_readme_files
  copy_binary
  PKG_ROOT="${STAGING}/pkgroot"
  rm -rf "$PKG_ROOT"
  mkdir -p "${PKG_ROOT}/usr/local/bin" "${PKG_ROOT}/usr/local/share/doc/jereko"
  cp "${STAGING}/jereko" "${PKG_ROOT}/usr/local/bin/jereko"
  cp "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "${PKG_ROOT}/usr/local/share/doc/jereko/"
  chmod +x "${PKG_ROOT}/usr/local/bin/jereko"
  pkgbuild --root "$PKG_ROOT" \
    --identifier "com.jerekode.jereko" \
    --version "$VERSION" \
    --install-location "/" \
    "${OUT_DIR}/jereko-${VERSION}-release-macos-${ARCH}.pkg"
}

package_windows() {
  write_readme_files
  copy_binary
  NSIS="${NSIS:-}"
  if [[ -z "$NSIS" ]]; then
    for candidate in \
      "/c/Program Files (x86)/NSIS/makensis.exe" \
      "/c/Program Files/NSIS/makensis.exe" \
      makensis; do
      if command -v "$candidate" >/dev/null 2>&1 || [[ -f "$candidate" ]]; then
        NSIS="$candidate"
        break
      fi
    done
  fi
  if [[ -z "$NSIS" ]]; then
    echo "warning: makensis not found; skipping NSIS installer" >&2
    return 0
  fi
  NSIS_OUT="${OUT_DIR}/jereko-${VERSION}-release-windows-${ARCH}-setup.exe"
  # Git Bash converts /V2 to a path under Program Files/Git; exclude NSIS /D defines too.
  MSYS2_ARG_CONV_EXCL='*' "$NSIS" /V2 \
    /DVERSION="$VERSION" \
    /DBINARY="${STAGING}/jereko.exe" \
    /DOUTFILE="$NSIS_OUT" \
    packaging/nsis/jereko.nsi
  cp "$NSIS_OUT" "${OUT_DIR}/jereko-x64-setup.exe"
}

case "$TARGET_OS" in
  linux) package_linux ;;
  macos) package_macos ;;
  windows) package_windows ;;
  *)
    echo "error: unknown target_os: $TARGET_OS" >&2
    exit 1
    ;;
esac

echo "Installers written to ${OUT_DIR}"
