# Release quality roadmap

Phased plan for trustworthy changelogs, a clean version line, installers, and optional Bun-free builds. Decision record: [ADR 003](./adr/003-release-packaging-and-changelogs.md). Operational how-to today: [releases.md](./releases.md).

**Status:** Active forward plan (parity board closed).
**This document is planning.** Destructive wipe and dual-build Cargo features are **planned, not implemented**. Do not check wipe / dual-build items until they land. A few P0/P1 checklist rows below are already true in repo (static release body; archive naming) and are marked done.

## Goals

1. Stop publishing faulty GitHub Release notes / bogus "New Contributors."
2. Reset versioning to **0.0.1** and purge pre-reset Releases/tags (approved wipe only).
3. Move from tarball/zip-only assets toward real installers per OS.
4. Offer **full** (Bun sidecar) and **native-only** distribution variants without forking the product architecture ([ADR 002](./adr/002-dual-plugin-runtime.md)).

## Recommended sequence (summary)

```text
P0  Changelog fix + version reset wipe (same cutover window)
P1  Multi-arch binaries + clear asset naming (+ optional native-only linux/windows x64)
P2  Real installers (NSIS, macOS pkg, deb/rpm/AppImage) — full variant first
P3  Homebrew / winget / AUR / Nix; expand native-only matrix; consider Bun bundling
P4  Signing / notarization / Authenticode
```

Do **not** publish installer matrices onto `v0.1.*` history you plan to delete. Scripts may land earlier; **publish** after P0.

---

## P0 — Changelog quality + version reset wipe

### P0a — Stop bad notes (code/workflow)

- [ ] Turn off unfiltered `generate_release_notes: true` **or** drive notes via API with explicit `previous_tag_name`.
- [ ] Add `.github/release.yml` exclusions: label `release-sync`, authors `github-actions[bot]` / bots, and ideally titles matching `[skip release]`.
- [ ] Post-filter or omit **New Contributors** until stable.
- [x] Keep a short static body (artifacts, platforms, Bun sidecar note) like PromptComposer’s install blurb. *(present in `release.yml`; still prepended to unfiltered auto notes)*
- [ ] Temporarily pause auto-release or require `[skip release]` during the wipe PR window.

### P0b — Version reset and history purge (destructive — confirm first)

See [ADR 003](./adr/003-release-packaging-and-changelogs.md#version-reset-and-release-history-purge) for the full checklist. Summary:

- [ ] Maintainer confirmation (and mirrors check).
- [ ] `gh release delete` all pre-reset releases (with tag cleanup).
- [ ] Delete any remaining remote/local `v0.1.*` tags.
- [ ] PR: `scripts/set-version.sh 0.0.1` + doc example updates + `[skip release]`.
- [ ] Close stale `release/sync-0.1.*` PRs/branches.
- [ ] First clean release under new notes policy (`v0.0.1` or first post-cutover auto tag).

### P0c — Post-wipe version policy (pick one)

| Option | When to choose |
|--------|----------------|
| **A.** `0.0.<run_number>` + filtered notes | Keep PromptComposer-style every-merge releases |
| **B.** Conventional commits / release-please (recommended long-term) | Quality changelogs and intentional semver |

**Avoid** returning to `0.1.<run_number>` after wiping `v0.1.*`.

---

## P1 — Multi-arch binaries and naming

- [x] Keep stem `jereko-{version}-release-{os}-{arch}`; document arch tags (`x64`, `arm64`). *(shipped in `package-release.sh` / `release.yml`; documented in [releases.md](./releases.md))*
- [ ] Linux/Windows arm64 when free GHA runners (or self-hosted/qemu) exist.
- [ ] Optional: first **native-only** artifacts for **linux-x64** and **windows-x64** only (`…-native-release-…`) to prove the Cargo feature without doubling full matrix cost.
- [ ] CI: `os × arch × variant` documented; fail-fast false; cache keys include variant.

---

## P2 — Real installers (full variant first)

| OS | Ship |
|----|------|
| Windows | NSIS setup exe (+ keep zip) |
| macOS | `.pkg` unsigned (+ keep tarball) |
| Linux | `.deb`, `.rpm`, AppImage |

- [ ] Evaluate cargo-dist vs cargo-packager vs nfpm for a Rust CLI.
- [ ] Installer matrix initially **full** builds only; native-only installers follow once names/UX are stable.
- [ ] Update [releases.md](./releases.md) / [distribution.md](./distribution.md) download tables.

---

## P3 — Package-manager distribution

- [ ] Homebrew tap (name TBD).
- [ ] winget manifest (and optional scoop).
- [ ] AUR `PKGBUILD` (community or official).
- [ ] In-repo **Nix flake** (cheap; good for NixOS users).
- [ ] Expand native-only to macos + arm64 as demand warrants.
- [ ] Revisit **bundling Bun** inside full installers (default remains system Bun).

---

## P4 — Signing and notarization

- [ ] Apple Developer ID + notarization secrets for macOS.
- [ ] Windows Authenticode (or cloud signing).
- [ ] Document trust model; keep unsigned fallback notes until complete.

---

## Dual-build phasing (full vs native-only)

| Phase | Dual-build work |
|-------|-----------------|
| P0 | Document only; no matrix change required for wipe (**feature not implemented yet**) |
| P1 | Introduce `bun-sidecar` feature; optional native-only artifacts on 1–2 platforms (**planned**) |
| P2 | Installers for **full**; native-only remains archive or slim installer |
| P3+ | Native-only in brew/winget/Nix as separate formulae/packages if useful |

**Default download recommendation:** **full** (OpenCode fidelity), with native-only clearly labeled for advanced/server users — confirm in open questions.

---

## Distro guidance (Linux)

| Distro | Path |
|--------|------|
| Debian/Ubuntu | `.deb` from Releases; optional apt repo much later |
| Fedora/RHEL | `.rpm` from Releases |
| Arch | AUR PKGBUILD (P3) |
| NixOS | In-repo flake (P1–P3); optional nixpkgs submit later |
| Generic | AppImage + tarball |

---

## Open questions

### Version reset / history

1. **Confirm destructive wipe** of all pre-reset GitHub Releases and `v0.1.*` tags?
2. Any **mirrors**, forks, or downstream pins of `v0.1.*` assets?
3. After wipe: keep **0.0.\<run_number\>** or move to **conventional-commit semver**?

### Packaging / signing

4. Possess (or willing to buy) **Apple** and/or **Windows** code-signing certificates?
5. Preferred **Homebrew tap** name (e.g. `JesseKoldewijn/homebrew-jereko`)?
6. Accept **unsigned** installers until P4?

### Dual builds

7. **Default download:** full (with Bun) or native-only?
8. Should `jereko` on native-only **error clearly** when config enables a Bun/TS plugin? (**Recommend: yes.**)
9. **Full package:** keep requiring **system Bun**, or bundle Bun later?
10. Artifact naming: `jereko-native-…` vs `…-native-release-…` vs separate package display names only?

---

## References

- [releases.md](./releases.md) — current auto-release and `/build` behavior
- [distribution.md](./distribution.md) — local install aliases
- [ADR 002](./adr/002-dual-plugin-runtime.md) — Bun / native / WASM hosts
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — packaging, changelogs, wipe, dual-build
- PromptComposer release workflow (Tauri NSIS/deb/AppImage + custom body, no GitHub auto notes)

