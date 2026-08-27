# Release packaging (P2+)

Installers ship alongside portable archives on GitHub Releases. Unsigned pre-1.0; see [docs/roadmap-releases.md](../docs/roadmap-releases.md) P4 for signing and [docs/releases.md](../docs/releases.md) for the trust model.

## Locked P2 matrix (full variant)

| Platform | Formats on Releases |
|----------|---------------------|
| Windows x64 | `.zip` (portable) + NSIS `*-setup.exe` + stable alias `jerekode-x64-setup.exe` |
| Linux x64 | `.tar.gz` + `.deb` + `.rpm` + AppImage + Arch `.pkg.tar.zst` |
| macOS x64 / arm64 | `.tar.gz` + unsigned `.pkg` |

**Not built yet (no free GHA runners):** linux-arm64, windows-arm64 — matrix stubs are commented in [`.github/workflows/release.yml`](../.github/workflows/release.yml); enable when runners exist. Do not fake runners.

**Native-only (advanced / future):** archives named `jerekode-{version}-native-release-{os}-{arch}` built with `cargo build -p jerekode-cli --release --no-default-features` (`bun-sidecar` off). Default download remains **full**. Optional matrix rows are commented in `release.yml`.

Tooling: [nfpm](https://nfpm.goreleaser.com/) (deb/rpm), NSIS (Windows), `pkgbuild` (macOS), AppImageKit, Arch `makepkg` (Docker).

## Publish checklist (package managers) — do not automate without maintainer accounts

None of these publish from CI today. Complete the in-repo templates, then follow the checklist manually.

### AUR

See [packaging/arch/README.md](arch/README.md). In-repo `PKGBUILD` mirrors the CI-built package.

- [ ] Copy `PKGBUILD` (+ `.SRCINFO`) to your AUR package repo
- [ ] Set `pkgver` / `sha256sums` to the GitHub Release tarball
- [ ] `makepkg --printsrcinfo > .SRCINFO` and push to AUR

### Homebrew

Template: [packaging/homebrew/jerekode.rb.template](homebrew/jerekode.rb.template)

- [ ] Create tap `JesseKoldewijn/homebrew-jerekode` (or chosen name)
- [ ] Copy template → `Formula/jerekode.rb`
- [ ] Fill `version` + macOS (and optional Linux) sha256 from Release assets
- [ ] `brew audit --strict --online jerekode` locally
- [ ] Push tap; users: `brew tap JesseKoldewijn/jerekode && brew install jerekode`

### winget

Template: [packaging/winget/jerekode.yaml.template](winget/jerekode.yaml.template) (multi-file layout documented in comments)

- [ ] Split template into version / installer / locale manifests under `manifests/j/JesseKoldewijn/jerekode/{{VERSION}}/`
- [ ] Fill version + `InstallerSha256` for `jerekode-x64-setup.exe`
- [ ] Open PR to [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs)
- [ ] Respond to winget-bot validation

### Nix

In-repo flake: [`flake.nix`](../flake.nix)

```bash
nix build                # full CLI (bun-sidecar default)
nix build .#jerekode-native
nix run . -- version
```

- [ ] Optional later: submit to nixpkgs once the flake is stable
- [ ] Bun is **not** packaged by the flake; install Bun separately for sidecar plugins

## Signing (P4 — stubs only)

Apple notarization and Windows Authenticode activate when secrets exist (`APPLE_*`, `WINDOWS_CERT*`). Workflow stubs are commented in `release.yml` so unsigned releases keep working. Trust model: [docs/releases.md](../docs/releases.md#trust-model--code-signing-pre-10).

## Local smoke test

```bash
cargo build --release -p jerekode-cli --locked
VERSION=0.0.0-local ./scripts/package-installers.sh linux x64 target/release/jerekode dist

# Native-only binary (no Bun spawn paths):
cargo build --release -p jerekode-cli --no-default-features --locked
```
