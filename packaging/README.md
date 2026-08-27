# Release packaging (P2+)

Installers ship alongside portable archives on GitHub Releases. Unsigned pre-1.0; see [docs/roadmap-releases.md](../docs/roadmap-releases.md) P4 for signing.

## Locked P2 matrix (full variant)

| Platform | Formats on Releases |
|----------|---------------------|
| Windows x64 | `.zip` (portable) + NSIS `*-setup.exe` + stable alias `jerekode-x64-setup.exe` |
| Linux x64 | `.tar.gz` + `.deb` + `.rpm` + AppImage + Arch `.pkg.tar.zst` |
| macOS x64 / arm64 | `.tar.gz` + unsigned `.pkg` |

Tooling: [nfpm](https://nfpm.goreleaser.com/) (deb/rpm), NSIS (Windows), `pkgbuild` (macOS), AppImageKit, Arch `makepkg` (Docker).

## AUR (P3)

See [packaging/arch/README.md](arch/README.md). In-repo `PKGBUILD` mirrors the CI-built package; publish to AUR separately.

## Homebrew / winget (P3 — templates)

- [packaging/homebrew/jerekode.rb.template](homebrew/jerekode.rb.template)
- [packaging/winget/jerekode.yaml.template](winget/jerekode.yaml.template)

Update version + SHA256 after each release; submit via tap / winget-pkgs PR.

## Signing (P4 — not yet)

Apple notarization and Windows Authenticode are tracked in [docs/roadmap-releases.md](../docs/roadmap-releases.md) P4. Installers ship **unsigned** until certs/secrets are configured.

## Local smoke test

```bash
cargo build --release -p jerekode-cli --locked
VERSION=0.0.0-local ./scripts/package-installers.sh linux x64 target/release/jerekode dist
```
