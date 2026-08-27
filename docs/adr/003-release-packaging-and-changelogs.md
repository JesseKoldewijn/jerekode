# ADR 003: Release Packaging, Changelogs, and Distribution Variants

**Status:** Accepted (P0 notes fix + version wipe executed 2026-08-26; P2 installers shipped 2026-08-26)  
**Date:** 2026-08-26  
**Context:** Auto-release on `main` was producing unusable GitHub Release notes; assets are archives only; users want installers, a clean version reset, and optional Bun-free builds. Companion plan: [roadmap-releases.md](../roadmap-releases.md). Extends ADR 001 (binary `jerekode`) and ADR 002 (plugin hosts).

## Decision (summary)

1. **Stop trusting unfiltered GitHub auto notes** for published Releases. Prefer curated / filtered notes; keep auto-release cadence until a deliberate semver cutover.
2. **Destructive version reset** to **0.0.1** with purge of pre-reset GitHub Releases and `v0.1.*` tags (**executed** after explicit maintainer approval).
3. **Ship installers** over time (Windows NSIS primary; macOS `.pkg`; Linux `.deb` + `.rpm` + AppImage; Homebrew/winget/AUR/Nix later; signing last).
4. **Plan two distribution variants:** **full** (Bun sidecar path) and **native-only** (no Bun required). Implement behind Cargo features; do **not** invent a second product binary name for the default CLI.

Detailed phasing and open questions live in [roadmap-releases.md](../roadmap-releases.md).

## Current state (investigation)

### Workflow and packaging

| Piece | Today |
|-------|--------|
| Workflow | [`.github/workflows/release.yml`](../../.github/workflows/release.yml) |
| Version on `main` push | Next sequential patch from `Cargo.toml` (`0.0.1` → `0.0.2` …) via `scripts/set-version.sh`; sync PR or optional `RELEASE_PUSH_TOKEN` |
| Sync to protected `main` | Branch `release/sync-*`, label `release-sync`, squash auto-merge with `[skip release]` |
| Pre-publish gate | Release workflow `check` job (fmt, clippy, tests, Bun) — PromptComposer-style |
| Publish | `softprops/action-gh-release@v2` with **`generate_release_notes: false`**; body = static blurb + filtered notes via API `previous_tag_name` + `scripts/filter-release-notes.py` |
| Assets | Archives via `scripts/package-release.sh`; installers via `scripts/package-installers.sh` (NSIS, deb, rpm, AppImage, macOS pkg, Arch `.pkg.tar.zst`) |
| Platforms built | linux-x64, macos-x64, macos-arm64, windows-x64 (no free GHA linux/windows arm64) |
| Changelog config | [`.github/release.yml`](../../.github/release.yml) excludes `release-sync` + bot authors |

### PromptComposer comparison

[PromptComposer](https://github.com/JesseKoldewijn/PromptComposer) shares the **0.1.\<run_number\> auto-bump on main** idea, but:

- Uses **Tauri** bundles (NSIS / deb / AppImage), not raw archives.
- Publishes a **hand-written `releaseBody`** (install/upgrade instructions) — **does not** rely on GitHub `generate_release_notes`.
- Signs Tauri updater payloads; optional Apple/Windows code signing is still a separate concern.

Jerekode is a **Rust CLI**, not a Tauri GUI — installer tooling differs (`cargo-dist` / `cargo-packager` / WiX / nfpm), but the **notes lesson** transfers: do not ship unfiltered auto notes.

### Why changelogs and "New Contributors" are wrong

Observed on `v0.1.16` (representative):

- **What's Changed** lists nearly **every PR since the empty `main`**, including `chore: release v0.1.N [skip release]` sync PRs and bot authorship.
- **New Contributors** re-credits `@JesseKoldewijn` (PR #1) and `@github-actions[bot]` (sync PR) as first-time contributors.
- **Full Changelog** links to `/commits/v0.1.16` rather than a `previous...tag` compare — consistent with GitHub treating the notes as if there were **no usable previous tag**.

Root causes (combined):

1. **`generate_release_notes: true`** without reliable `previous_tag_name` / without `.github/release.yml` exclusions.
2. **Short / noisy history** — every main merge fires a release; sync PRs and bot commits pollute the PR list GitHub uses for notes.
3. **"New contributors"** is GitHub's first-contribution-in-repo heuristic over the generated range — when the range is "whole history", everyone looks new every time; bots are not excluded by default.
4. Empty initial `main` + rapid `0.1.N` tags amplify the noise; editing old Releases will not fix the generator.

### Bun / sidecar coupling (dual-build feasibility)

| Fact | Detail |
|------|--------|
| Binary name | `jerekode` (`jerekode-cli` `[[bin]]`) |
| Cargo features today | `native-tui` only (optional ratatui stub via `jerekode-plugins/native-tui`). **No** `bun-sidecar` / `native-only` feature. |
| Bun coupling | Always-on in code paths that use it: `BunProcessSidecarPort::spawn` runs `Command::new("bun")`; `jerekode run` always builds orchestrator with `NativePluginHost` + `BunPluginHost` + `WasmPluginHost`. |
| Release archives | **Do not** bundle Bun or `sidecar/` sources — `SIDECAR.txt` tells users to install Bun and run from repo (system Bun). |
| Without Bun | OpenCode-fidelity TUI (SolidJS / sidecar), unqualified npm/TS plugins, and `tui.render` via sidecar are unavailable. Native dylib plugins + WASM (when enabled) + HTTP `serve` remain in scope for a native-only variant. `native-tui` is a stub, not a Bun replacement yet. |

**Feasibility:** Yes. Gate `BunPluginHost` / `BunProcessSidecarPort` (and CLI wiring in `jerekode run`) behind a Cargo feature such as `bun-sidecar` (default **on**). Native-only builds compile without depending on a Bun runtime at start; unqualified Bun plugin configs must fail with a clear error.

## Changelog approach (recommendation)

| Option | Fit for jerekode | Verdict |
|--------|----------------|---------|
| Raw GitHub `generate_release_notes` (current) | Fast, but broken here | **Stop for published body** (or heavily filter) |
| Filtered GitHub notes (`.github/release.yml` + explicit previous tag) | Low effort; excludes labels/authors | **Good P0 stopgap** |
| Keep a Changelog + conventional commits (manual or scripted) | High quality; needs commit discipline | **Target after reset** |
| release-please | Semver + changelog PRs; fights run_number cadence | **Optional later** if leaving run_number |
| changesets | JS-ecosystem oriented | **Skip** |
| PromptComposer-style static body only | Clear install text; weak change list | Use as **prefix**, not sole notes |

**Recommended path:**

1. **P0:** Disable or replace unfiltered `generate_release_notes`. Publish: short install/artifact blurb + **filtered** notes (exclude `release-sync`, titles containing `[skip release]`, authors `github-actions[bot]` / Dependabot) via `.github/release.yml` and/or `gh api repos/.../releases/generate-notes` with `previous_tag_name` + post-filter. Optionally omit the "New Contributors" section entirely until history is clean.
2. **After version reset:** Prefer **Conventional Commits** + a generated Keep a Changelog section (or release-please) once cadence/semver policy is chosen.
3. **Cleaning existing bad Releases:** Prefer **delete via the planned history purge** (below) over hand-editing dozens of bodies. If purge is delayed, a one-shot script can `gh release edit` to strip auto sections — temporary only.

### Versioning after reset

| Scheme | Pros | Cons |
|--------|------|------|
| Stay `0.0.<run_number>` | Matches current automation | Implies patch spam; weak semver signal |
| `0.0.1` once, then conventional-commit bumps (`0.0.2`, `0.1.0`, …) | Honest pre-1.0; good changelogs | Needs release-please or equivalent; not "every main merge" |
| Jump to `0.1.0` + conventional commits | Familiar "first real minor" | Collides with wiped `0.1.*` tag namespace if tags are not fully purged first |

**Recommendation:** Reset workspace to **0.0.1**. For the **first** post-wipe release only, publish `v0.0.1` with curated notes. Then either:

- **A (keep auto-release):** `0.0.<run_number>` with **filtered** notes (document that `0.0.1` was the cutover seed; subsequent run numbers may skip), or
- **B (preferred long-term):** leave run_number; use **release-please / conventional commits** so versions move only when user-facing changes land (`0.0.2`, …, then `0.1.0` when API surface warrants).

Do **not** resume `0.1.<run_number>` after wipe — it reuses the deleted tag namespace and confuses mirrors/caches.

## Version reset and release history purge

**Status:** Executed 2026-08-26 (maintainer-approved). Pre-reset `v0.1.*` Releases/tags purged; workspace seed `0.0.1`; auto-release scheme `0.0.<run_number>` + filtered notes.

### Goals

1. Set `[workspace.package] version` to **0.0.1** (`Cargo.toml` + `Cargo.lock` via `scripts/set-version.sh`).
2. Delete **all** GitHub Releases and tags created before the reset (current set includes at least `v0.1.8`–`v0.1.16`; delete any other `v0.1.*` / pre-reset tags found at execution time).
3. Update docs that teach `0.1.<run_number>` (`docs/releases.md`, examples, any README badges).
4. Land changelog filtering **in the same cutover window** so the first `v0.0.1` (or first post-wipe auto release) is clean.

### Safe procedure (checklist — execute later)

```text
# 0. Preconditions
# - Explicit user/maintainer approval for destructive wipe
# - No open release/sync-* PRs you care about (close or merge)
# - Pause or temporarily skip auto-release on main (or ensure [skip release]
#   on the version-reset PR) so a half-wiped tag set is not recreated mid-flight
# - Confirm no known external mirrors/pins of v0.1.* (open question)

# 1. Delete GitHub Releases (assets go with them)
gh release list --limit 100
gh release delete 'v0.1.N' --yes --cleanup-tag   # per tag; or loop

# 2. Delete remote tags if --cleanup-tag was not used
git ls-remote --tags origin 'v0.1.*'
git push origin --delete 'v0.1.N'   # per tag

# 3. Delete local tags
git tag -d 'v0.1.N'

# 4. Version bump on a PR (PR-only policy)
./scripts/set-version.sh 0.0.1
# commit: chore: reset version to 0.0.1 [skip release]
# merge via PR

# 5. Docs: replace 0.1.run_number examples; point at roadmap
# 6. Close leftover release/sync-0.1.* branches/PRs
# 7. Optional: bump Actions cache keys / shared-key prefixes so stale
#    release-* caches are not confused with new version line
# 8. First good release under new notes policy (manual dispatch or next merge)
```

### Risks

| Risk | Mitigation |
|------|------------|
| Anyone who pinned `v0.1.*` URLs/assets breaks | Announce in README/release notes; accept for pre-1.0 |
| Sync PRs / `release/sync-*` branches race the wipe | Close sync PRs; pause Release workflow or use `[skip release]` |
| Branch protection / required checks | Version bump still via PR; do not force-push `main` |
| Actions caches keyed by old versions | Harmless usually; rotate `shared-key` if builds look wrong |
| Tag recreation by a concurrent Release run | Disable/skip release until wipe + notes fix merge |
| Mirrors / forks | Open question — confirm before wipe |

### Phasing relative to installers (recommendation)

**Do the wipe + changelog fix as P0 together, before investing in the installer matrix on bad history.**

Rationale: installer assets attached to `v0.1.*` would be deleted; run_number and notes policy change at cutover; dual-variant artifact naming should start on the clean line. Develop installer **scripts** on feature branches anytime; **publish** installers only after P0 cutover (or as P2 immediately after).

Alternative (reject as default): build full installer pipeline first, then wipe — wastes CI attaching installers to tags you intend to delete.

## Installer formats (recommendation)

Unsigned first is acceptable for a pre-1.0 CLI; document "unsigned / Gatekeeper / SmartScreen warnings expected."

| Platform | Candidates | Pros / cons | Recommendation |
|----------|------------|-------------|----------------|
| **Windows x64** | NSIS `.exe`, WiX MSI, MSIX, scoop/winget, zip+exe | NSIS: simple PATH/install UX (PromptComposer precedent). MSI: enterprise-friendly, more CI. MSIX: store-oriented, heavier. Zip: keep as fallback. | **Primary: NSIS** (or cargo-packager NSIS). Keep zip portable. **winget** manifest in P3. Windows ARM when runner exists. |
| **macOS x64 + arm64** | `.pkg`, `.dmg`, Homebrew, raw tarball | `.pkg` fits CLI to `/usr/local` or `/opt`. `.dmg` is more GUI-app flavored. Homebrew is how most CLI users install. Notarization needs Apple Developer secrets. | **Primary unsigned `.pkg`** (or tarball until pkg scripts ready) + **Homebrew tap in P3**. Universal binary optional later. **Notarization = P4.** |
| **Linux x64 + arm64** | `.deb`, `.rpm`, Arch PKGBUILD/AUR, Nix flake, AppImage, flatpak | deb/rpm cover Debian/Fedora families. AppImage is distro-agnostic single file. AUR/Nix are packager-driven. Flatpak is awkward for CLI PATH tools. | **deb + rpm + AppImage** as release assets. **In-repo Nix flake** early (low cost). **AUR** in P3. Cross/qemu or `ubuntu-24.04-arm` when available for arm64. |

**Signing:** Apple notarization and Windows Authenticode need paid certs/secrets — **P4**. Ship unsigned installers with clear docs until then.

Tooling options to evaluate in implementation PRs: [cargo-dist](https://opensource.axo.dev/cargo-dist/), [cargo-packager](https://github.com/crabnebula-dev/cargo-packager), nfpm/fpm, WiX.

## Dual distribution: full vs native-only

### Product intent

| Variant | Audience | Bun | Plugins / TUI |
|---------|----------|-----|----------------|
| **Full** (`bun-sidecar` on) | OpenCode / JS/TS plugin users | System Bun required (P0–P2); optionally bundle Bun later | BunPluginHost default; sidecar TUI plugins |
| **Native-only** (`bun-sidecar` off) | Server/CLI users who only need Rust/native (and WASM) plugins | Not shipped; not required | NativePluginHost (+ WasmPluginHost); `jerekode run` must not spawn Bun; clear errors if config requests Bun/TS plugins |

### Implementation approach (recommendation)

1. **Cargo feature `bun-sidecar` (default = enabled)** on `jerekode-plugins` / `jerekode-cli`, wrapping `bun_host`, `BunProcessSidecarPort`, and CLI registration. Mirror optional compile-out of Bun-only paths.
2. **Do not** create a separate workspace binary crate unless packaging forces it. Prefer **one crate, two release artifacts**:
   - `jerekode-{ver}-release-{os}-{arch}` — full (default)
   - `jerekode-{ver}-native-release-{os}-{arch}` — build with Bun sidecar feature off  
   Installers: `jerekode-setup.exe` vs `jerekode-native-setup.exe` (names TBD).
3. Keep **`native-tui` orthogonal** — it does not mean "native-only distribution." Native-only may later enable `native-tui` by default once the stub is real.
4. **Runtime UX:** native-only builds should error clearly if config selects Bun/unqualified npm plugins or sidecar entry points ("this build was compiled without Bun sidecar support; download the full build or use native/wasm plugins").
5. **CI matrix:** `os × arch × variant` doubles artifact count. **Phase:** ship **full only** through P1–P2; add **native-only** for linux-x64 + windows-x64 first (highest demand / cheapest), then expand.

### Feature flag vs separate bins

| Approach | Pros | Cons |
|----------|------|------|
| Feature flag + two artifacts (recommended) | One codebase; ADR 002 hosts stay clear; smaller native binary | Matrix cost; must test both |
| Separate bins (`jerekode` + `jerekode-native`) | Very explicit download names | Duplicate clap surface; drift risk |
| Always one fat binary | Simplest CI | Cannot remove Bun *requirement* from UX; users still need Bun on PATH for `run` |

**Size/startup:** Native-only mainly removes the need to spawn Bun and ship/sidecar docs expectations; Rust code size savings are modest unless Bun host modules are `cfg`'d out. Biggest win is **operational** (no Bun install), not megabytes.

### Bun inside the full package?

| Policy | Pros | Cons |
|--------|------|------|
| **Require system Bun** (current) | Small artifacts; Bun upgrades independent | Friction; version skew |
| Bundle Bun binary in full installer | One-click fidelity | Huge artifacts; licensing/update burden; multi-arch Bun |

**Recommendation:** Keep **system Bun** for full builds through P2; document minimum Bun version. Revisit bundling only if install friction dominates (P3+).

## Consequences

- Release notes become trustworthy after P0; history wipe is irreversible for `v0.1.*` URLs.
- Installer and dual-variant work multiplies CI time and secrets surface — sequence after notes/version cutover.
- ADR 002 remains valid: native-only is a **distribution profile**, not a removal of BunPluginHost from the architecture.

## Open questions

See [roadmap-releases.md](../roadmap-releases.md#open-questions) — signing certs, Homebrew tap name, default download variant, confirm destructive wipe, mirrors, post-wipe version scheme, whether to bundle Bun.
