# Releases and PR builds

How jereko auto-releases on `main`, how to cut a manual/tag release, download binaries, and trigger on-demand PR builds via `/build`.

Related workflows:

| Workflow | File | Creates GitHub Release? |
|----------|------|-------------------------|
| CI (fmt/clippy/test + Bun sidecar) | [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) | No |
| Release | [`.github/workflows/release.yml`](../.github/workflows/release.yml) | **Yes** |
| PR Build (`/build`) | [`.github/workflows/pr-build.yml`](../.github/workflows/pr-build.yml) | **No** |

Packaging helper: [`scripts/package-release.sh`](../scripts/package-release.sh).  
Version bump helper: [`scripts/set-version.sh`](../scripts/set-version.sh).  
Local install aliases: [`scripts/install.sh`](../scripts/install.sh) / [distribution.md](./distribution.md).

## Auto-release on merge to `main` (PromptComposer-style)

Every push/merge to `main` triggers the **Release** workflow (unless the commit message contains `[skip release]`):

1. **Bump** `[workspace.package] version` to next `0.0.<N+1>` from Cargo.toml via `scripts/set-version.sh` (cutover seed on `main` is `0.0.1`; each main auto-release increments the patch after the wipe seed).
2. **Commit** `chore: release v0.0.<n> [skip release]`, then land it on `main`:
   - **Default (protected `main`):** push branch `release/sync-0.0.<n>`, open a PR labeled `release-sync`, enable **auto-merge (squash)**. It merges when required checks (`rust`, `bun-sidecar`) pass. This run’s build/publish uses the bumped commit immediately and does **not** wait for that merge.
   - **Optional:** secret `RELEASE_PUSH_TOKEN` (admin PAT) pushes the bump straight to `main` (PromptComposer-style) and skips the sync PR.
3. **Build** multi-platform `jereko` binaries (release profile) from the bumped commit; **check** job runs fmt/clippy/tests first (PromptComposer-style).
4. **Package** portable archives + OS installers (NSIS, deb/rpm/AppImage, macOS pkg, Arch pkg).
5. **Publish** a GitHub Release tagged `v0.0.<n>` with all assets attached.
6. **Notes:** static install/artifact body plus GitHub-generated notes **since the previous `v*` tag only**, filtered by [`.github/release.yml`](../.github/release.yml) and [`scripts/filter-release-notes.py`](../scripts/filter-release-notes.py) (drops `release-sync` / bots / `[skip release]` lines and the **New Contributors** section). `generate_release_notes` on softprops is **off**.

### Sync PR auto-merge & loop prevention

Repo settings required:

- **Allow auto-merge** (Settings → General)
- **Actions → Workflow permissions:** Read and write + **Allow GitHub Actions to create and approve pull requests**

Without the Actions PR checkbox, `gh pr create` fails with `GitHub Actions is not permitted to create or approve pull requests`. The workflow still pushes `release/sync-*` and continues build/publish from the bumped SHA; open/merge the sync PR manually (or fix the setting and re-run).

The workflow runs `gh pr merge --auto --squash` on the sync PR.

| Guard | Role |
|-------|------|
| Commit message + PR title include `[skip release]` | Squash into `main` does not start another Release run |
| Label `release-sync` | Marks automation PRs in the UI |
| Branch `release/sync-*` | Stable head name for retries / `gh pr view` |

This matches [PromptComposer](https://github.com/JesseKoldewijn/PromptComposer): release on every successful `main` merge with a sequential `0.0` patch from Cargo.toml — not changeset/release-please/semantic-release. PromptComposer pushes bumps directly because its `main` is unprotected; this repo keeps PR-only protection and uses sync PR + auto-merge (or `RELEASE_PUSH_TOKEN`). Opening a new sync PR closes superseded open `release-sync` PRs. Opening a new sync PR closes superseded open `release-sync` PRs.

Humans and agents must still land code on `main` **only via pull request** (see [CONTRIBUTING.md](../CONTRIBUTING.md)). The release job’s version-bump write is the documented CI exception.

## Artifact layout

| Kind | Archive / installer name |
|------|---------------------------|
| Tagged / auto release (portable) | `jereko-{version}-release-{os}-{arch}.tar.gz` / `.zip` |
| Windows installer | `jereko-{version}-release-windows-x64-setup.exe`, stable alias `jereko-x64-setup.exe` |
| Linux installers | `jereko-{version}-release-linux-x64.deb`, `.rpm`, `.AppImage`, Arch `.pkg.tar.zst` |
| macOS installer | `jereko-{version}-release-macos-{x64\|arm64}.pkg` |
| PR build | `jereko-pr{N}-{profile}-{os}-{arch}` (workflow artifact + matching archive) |

### Install examples (unsigned pre-1.0)

| Platform | Command |
|----------|---------|
| Arch | `pacman -U jereko-0.0.3-release-linux-x64.pkg.tar.zst` |
| Debian/Ubuntu | `sudo dpkg -i jereko-0.0.3-release-linux-x64.deb` |
| Fedora/RHEL | `sudo rpm -i jereko-0.0.3-release-linux-x64.rpm` |
| Generic Linux | `chmod +x jereko-*-linux-x64.AppImage && ./jereko-*-linux-x64.AppImage version` |
| Windows | Run `jereko-x64-setup.exe` (SmartScreen may warn — unsigned) |
| macOS | `sudo installer -pkg jereko-*-macos-arm64.pkg -target /` |

Packaging scripts: [`scripts/package-release.sh`](../scripts/package-release.sh), [`scripts/package-installers.sh`](../scripts/package-installers.sh). Matrix locked in [`packaging/README.md`](../packaging/README.md).

AUR: [`packaging/arch/README.md`](../packaging/arch/README.md). Homebrew/winget templates under `packaging/`.

Examples:

```text
jereko-0.0.42-release-linux-x64.tar.gz
jereko-pr42-release-linux-x64.tar.gz
jereko-pr42-debug-macos-arm64.tar.gz
jereko-pr42-debug-windows-x64.zip
```

Contents of each archive:

- `jereko` / `jereko.exe`
- `README.txt`
- `SIDECAR.txt` (Bun sidecar is **not** bundled — see `sidecar/README.md`)

Version sources:

| Trigger | Version / naming |
|---------|------------------|
| Push/merge to `main` | `0.0.<N+1>` (sequential patch) (auto bump + tag) |
| Tag `v1.2.3` | `1.2.3` (warn if Cargo.toml differs) |
| Release `workflow_dispatch` | Input, or `workspace.package.version` |
| PR `/build` | Label `0.0.0-pr.{N}.{shortsha}`; artifacts use `jereko-pr{N}-{profile}-…` |

## Manual / tag releases

Tag and push still work (kept alongside auto-release):

```bash
git tag v0.2.0
git push origin v0.2.0
```

Or: **Actions → Release → Run workflow** (optional version override). Publishes tag `v{version}`.

Release builds always use the **release** Cargo profile.

### Platforms (GitHub-hosted free runners)

| OS | Arch | Runner | Status |
|----|------|--------|--------|
| Linux | x64 | `ubuntu-22.04` | Built |
| macOS | x64 | `macos-15-intel` | Built |
| macOS | arm64 | `macos-14` | Built |
| Windows | x64 | `windows-latest` | Built |
| Linux | arm64 | — | **Skipped** — no free GHA linux-arm64 runner |
| Windows | arm64 | — | **Skipped** — no free GHA windows-arm64 runner |

Caching: [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache) (PromptComposer-style).

## PR `/build` (workflow artifacts only)

Comment on a pull request (or use workflow_dispatch):

| Command | Profile | Cargo |
|---------|---------|-------|
| `/build` | **release** (default) | `cargo build --release` |
| `/build release` | release | `cargo build --release` |
| `/build debug` | debug | `cargo build` |
| `/build profile=debug` | debug | `cargo build` |
| `/build --profile debug` | debug | `cargo build` |

Canonical convention: **`/build`** or **`/build <profile>`** where `<profile>` is `release` or `debug`. The `profile=` / `--profile` forms are also accepted.

### Sticky comments

1. **Started** — sticky comment (marker `<!-- jereko-pr-build-sticky:{profile} -->`) saying the build started, naming the **profile**, and linking the in-progress run:

   `{server}/{owner}/{repo}/actions/runs/{run_id}`

2. **Finished** — same sticky comment updated with status, profile, artifact names/platforms, and a link to:

   `{…}/actions/runs/{run_id}#artifacts`

Separate stickies per profile so `/build debug` and `/build release` do not overwrite each other.

**PR builds never create a GitHub Release.**

### Permissions

- `contents: read`
- `pull-requests: write`
- `actions: write`

### Manual dispatch

**Actions → PR Build → Run workflow**

| Input | Description |
|-------|-------------|
| `pr_number` | PR to build |
| `profile` | `release` (default) or `debug` |

## Local packaging smoke test

```bash
cargo build --release -p jereko-cli --locked
./scripts/package-release.sh jereko-0.0.0-local-release-linux-x64 target/release/jereko /tmp/jereko-dist release
ls /tmp/jereko-dist
```

## Notes policy / upcoming packaging

Release bodies use a **static artifact blurb** plus **filtered** notes since the previous tag (see workflow + `.github/release.yml`). **P2 installers** (NSIS, deb/rpm/AppImage, macOS pkg, Arch pkg) ship on GitHub Releases via [`scripts/package-installers.sh`](../scripts/package-installers.sh). Signing (P4) and native-only variants remain on the roadmap:

- [roadmap-releases.md](./roadmap-releases.md) — phased plan (P0–P4)
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — packaging / changelog decisions
- [packaging/README.md](../packaging/README.md) — locked installer matrix

