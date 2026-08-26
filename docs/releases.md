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

1. **Bump** `[workspace.package] version` to `0.1.<github.run_number>` via `scripts/set-version.sh`.
2. **Commit** `chore: release v0.1.<n> [skip release]`, then land it on `main`:
   - **Default (protected `main`):** push branch `release/sync-0.1.<n>`, open a PR labeled `release-sync`, enable **auto-merge (squash)**. It merges when required checks (`rust`, `bun-sidecar`) pass. This run’s build/publish uses the bumped commit immediately and does **not** wait for that merge.
   - **Optional:** secret `RELEASE_PUSH_TOKEN` (admin PAT) pushes the bump straight to `main` (PromptComposer-style) and skips the sync PR.
3. **Build** multi-platform `jereko` binaries (release profile) from the bumped commit.
4. **Publish** a GitHub Release tagged `v0.1.<n>` with archives attached.

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

This matches [PromptComposer](https://github.com/JesseKoldewijn/PromptComposer): release on every successful `main` merge with a monotonic patch from the workflow run number — not changeset/release-please/semantic-release. PromptComposer pushes bumps directly because its `main` is unprotected; this repo keeps PR-only protection and uses sync PR + auto-merge (or `RELEASE_PUSH_TOKEN`).

Humans and agents must still land code on `main` **only via pull request** (see [CONTRIBUTING.md](../CONTRIBUTING.md)). The release job’s version-bump write is the documented CI exception.

## Artifact layout

| Kind | Archive / artifact name |
|------|-------------------------|
| Tagged / auto release | `jereko-{version}-release-{os}-{arch}.tar.gz` / `.zip` |
| PR build | `jereko-pr{N}-{profile}-{os}-{arch}` (workflow artifact + matching archive) |

Examples:

```text
jereko-0.1.42-release-linux-x64.tar.gz
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
| Push/merge to `main` | `0.1.<run_number>` (auto bump + tag) |
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

## Known issues / upcoming

Current Releases use `softprops/action-gh-release` with `generate_release_notes: true`, which has been producing **unusable changelogs** and incorrect **New Contributors** sections. Assets today are **tarballs/zip only** (not installers). A planned **version reset to 0.0.1** (with purge of pre-reset `v0.1.*` tags/Releases), installer formats, and full vs native-only builds are documented in:

- [roadmap-releases.md](./roadmap-releases.md) — phased plan
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — decisions and wipe procedure (**plan only** until explicitly approved)

Do not treat current `0.1.<n>` Release bodies as the long-term notes format.

