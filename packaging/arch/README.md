# Arch Linux / AUR

## GitHub Releases (P2)

Each linux-x64 release includes `jerekode-{version}-release-linux-x64.pkg.tar.zst` built in CI via `makepkg` (Arch Docker).

Install:

```bash
pacman -U jerekode-0.0.3-release-linux-x64.pkg.tar.zst
```

## AUR (P3)

The in-repo [PKGBUILD](PKGBUILD) installs the same layout as CI. To publish or install from AUR:

1. Copy `PKGBUILD` (+ optional `.SRCINFO`) to an AUR package repo (e.g. `jerekode`).
2. Set `pkgver` / `sha256sums` to match the release tarball URL from GitHub Releases.
3. Submit to AUR or install locally:

```bash
yay -S jerekode
# or
git clone https://aur.archlinux.org/jerekode.git && cd jerekode && makepkg -si
```

Maintainer: publish under your AUR account when ready; CI does not push to AUR automatically.
