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
ARCH_STAGING=""
trap 'cleanup_staging' EXIT

cleanup_staging() {
  if [[ -n "$ARCH_STAGING" && -d "$ARCH_STAGING" ]]; then
    reclaim_arch_staging_ownership "$ARCH_STAGING"
    rm -rf "$ARCH_STAGING"
  fi
  rm -rf "$STAGING"
}

reclaim_arch_staging_ownership() {
  local dir="$1"
  if command -v docker >/dev/null 2>&1; then
    docker run --rm -v "${dir}:/build" archlinux:latest \
      chown -R "$(id -u):$(id -g)" /build 2>/dev/null || true
  fi
}

to_native_path() {
  local p="$1"
  if command -v cygpath >/dev/null 2>&1; then
    cygpath -w "$p"
  elif command -v wslpath >/dev/null 2>&1; then
    wslpath -w "$p"
  else
    echo "$p"
  fi
}

write_readme_files() {
  cat >"${STAGING}/README.txt" <<EOF
jerekode ${VERSION} (release)
OpenCode-compatible AI coding agent runtime (Rust).

  jerekode version
  jerekode serve
  jerekode run

Docs: https://github.com/JesseKoldewijn/jerekode
EOF
  cat >"${STAGING}/SIDECAR.txt" <<EOF
Bun sidecar is not bundled. Install Bun >= 1.1 for plugin fidelity.
See sidecar/README.md in the repository.
EOF
}

copy_binary() {
  local dest_name="jerekode"
  if [[ "$TARGET_OS" == "windows" ]]; then
    dest_name="jerekode.exe"
  fi
  cp "$BINARY" "${STAGING}/${dest_name}"
  if [[ "$TARGET_OS" != "windows" ]]; then
    chmod +x "${STAGING}/${dest_name}"
  fi
}

package_linux() {
  write_readme_files
  copy_binary
  cp "${STAGING}/jerekode" "${STAGING}/jerekode.bin"

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
  envsubst '${NFPM_VERSION} ${NFPM_ARCH}' < packaging/nfpm/jerekode.yaml >"$NFPM_CFG"
  mkdir -p "${STAGING}/staging"
  cp "${STAGING}/jerekode" "${STAGING}/staging/jerekode"
  cp "${STAGING}/README.txt" "${STAGING}/staging/"
  cp "${STAGING}/SIDECAR.txt" "${STAGING}/staging/"
  (cd "$STAGING" && "$NFPM_BIN" pkg --config nfpm.yaml --packager deb --target "${OUT_DIR}/jerekode-${VERSION}-release-linux-${ARCH}.deb")
  (cd "$STAGING" && "$NFPM_BIN" pkg --config nfpm.yaml --packager rpm --target "${OUT_DIR}/jerekode-${VERSION}-release-linux-${ARCH}.rpm")

  # AppImage
  APPIMAGETOOL="${APPIMAGETOOL:-${HOME}/.cache/appimagetool/appimagetool-x86_64.AppImage}"
  if [[ ! -f "$APPIMAGETOOL" ]]; then
    mkdir -p "$(dirname "$APPIMAGETOOL")"
    curl -sSfL -o "$APPIMAGETOOL" \
      "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage"
    chmod +x "$APPIMAGETOOL"
  fi
  APPDIR="${STAGING}/jerekode.AppDir"
  rm -rf "$APPDIR"
  mkdir -p "${APPDIR}/usr/bin" "${APPDIR}/usr/share/doc/jerekode"
  cp "${STAGING}/jerekode" "${APPDIR}/usr/bin/jerekode"
  cp "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "${APPDIR}/usr/share/doc/jerekode/"
  cat >"${APPDIR}/jerekode.desktop" <<EOF
[Desktop Entry]
Name=jerekode
Comment=OpenCode-compatible AI coding agent runtime
Exec=jerekode
Icon=jerekode
Terminal=true
Type=Application
Categories=Development;
EOF
  # Minimal icon silences appimagetool missing-icon warnings
  printf '\x89PNG\r\n\x1a\n' > "${APPDIR}/jerekode.png" 2>/dev/null || touch "${APPDIR}/jerekode.png"
  cat >"${APPDIR}/AppRun" <<'EOF'
#!/bin/sh
HERE="$(dirname "$(readlink -f "$0")")"
export PATH="${HERE}/usr/bin:${PATH}"
exec "${HERE}/usr/bin/jerekode" "$@"
EOF
  chmod +x "${APPDIR}/AppRun" "${APPDIR}/usr/bin/jerekode"
  # appimagetool requires fuse on some hosts; --appimage-extract-and-run works without fuse in CI
  ARCH=x86_64 "$APPIMAGETOOL" --appimage-extract-and-run "$APPDIR" \
    "${OUT_DIR}/jerekode-${VERSION}-release-linux-${ARCH}.AppImage"

  # Arch .pkg.tar.zst via Docker (makepkg on Arch Linux).
  # Use a separate temp dir: makepkg leaves root-owned files that break STAGING trap cleanup.
  if command -v docker >/dev/null 2>&1; then
    ARCH_STAGING="$(mktemp -d)"
    ARCH_PKG_DIR="${ARCH_STAGING}/archpkg"
    mkdir -p "$ARCH_PKG_DIR"
    cp packaging/arch/PKGBUILD "$ARCH_PKG_DIR/"
    cp "${STAGING}/jerekode" "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "$ARCH_PKG_DIR/"
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
    PKG="$(find "$ARCH_PKG_DIR" -maxdepth 1 -name 'jerekode-*.pkg.tar.zst' | head -n1 || true)"
    if [[ -n "$PKG" ]]; then
      cp "$PKG" "${OUT_DIR}/jerekode-${VERSION}-release-linux-${ARCH}.pkg.tar.zst"
    else
      echo "warning: Arch package build skipped or failed (docker/makepkg)" >&2
    fi
    reclaim_arch_staging_ownership "$ARCH_STAGING"
    rm -rf "$ARCH_STAGING"
    ARCH_STAGING=""
  else
    echo "warning: docker not available; skipping Arch .pkg.tar.zst" >&2
  fi
}

package_macos() {
  write_readme_files
  copy_binary
  PKG_ROOT="${STAGING}/pkgroot"
  rm -rf "$PKG_ROOT"
  mkdir -p "${PKG_ROOT}/usr/local/bin" "${PKG_ROOT}/usr/local/share/doc/jerekode"
  cp "${STAGING}/jerekode" "${PKG_ROOT}/usr/local/bin/jerekode"
  cp "${STAGING}/README.txt" "${STAGING}/SIDECAR.txt" "${PKG_ROOT}/usr/local/share/doc/jerekode/"
  chmod +x "${PKG_ROOT}/usr/local/bin/jerekode"
  pkgbuild --root "$PKG_ROOT" \
    --identifier "com.jerekode.jerekode" \
    --version "$VERSION" \
    --install-location "/" \
    "${OUT_DIR}/jerekode-${VERSION}-release-macos-${ARCH}.pkg"
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
  NSIS_OUT="${OUT_DIR}/jerekode-${VERSION}-release-windows-${ARCH}-setup.exe"
  NSIS_BINARY="$(to_native_path "${STAGING}/jerekode.exe")"
  NSIS_OUTFILE="$(to_native_path "$NSIS_OUT")"
  if [[ ! -f "${STAGING}/jerekode.exe" ]]; then
    echo "error: staged binary missing: ${STAGING}/jerekode.exe" >&2
    exit 1
  fi
  # Git Bash converts /V2 to a path under Program Files/Git; exclude NSIS /D defines too.
  # NSIS needs Windows-native paths (cygpath -w); Git Bash /tmp/... is invisible to makensis.
  MSYS2_ARG_CONV_EXCL='*' "$NSIS" /V2 \
    /DVERSION="$VERSION" \
    /DBINARY="$NSIS_BINARY" \
    /DOUTFILE="$NSIS_OUTFILE" \
    packaging/nsis/jerekode.nsi
  cp "$NSIS_OUT" "${OUT_DIR}/jerekode-x64-setup.exe"
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
