# Releases and PR builds

How to cut a jereko release, download binaries, and trigger on-demand PR builds via `/build`.

Related workflows:

| Workflow | File | Creates GitHub Release? |
|----------|------|-------------------------|
| CI (fmt/clippy/test + Bun sidecar) | [`.github/workflows/ci.yml`](../.github/workflows/ci.yml) | No |
| Release | [`.github/workflows/release.yml`](../.github/workflows/release.yml) | **Yes** |
| PR Build (`/build`) | [`.github/workflows/pr-build.yml`](../.github/workflows/pr-build.yml) | **No** |

Packaging helper: [`scripts/package-release.sh`](../scripts/package-release.sh).  
Local install aliases: [`scripts/install.sh`](../scripts/install.sh) / [distribution.md](./distribution.md).

## Artifact layout

| Kind | Archive / artifact name |
|------|-------------------------|
| Tagged release | `jereko-{version}-release-{os}-{arch}.tar.gz` / `.zip` |
| PR build | `jereko-pr{N}-{profile}-{os}-{arch}` (workflow artifact + matching archive) |

Examples:

```text
jereko-0.1.0-release-linux-x64.tar.gz
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
| Tag `v1.2.3` | `1.2.3` (warn if Cargo.toml differs) |
| Release `workflow_dispatch` | Input, or `workspace.package.version` |
| PR `/build` | Label `0.0.0-pr.{N}.{shortsha}`; artifacts use `jereko-pr{N}-{profile}-…` |

Keep `Cargo.toml` `[workspace.package] version` aligned with the tag you push.

## Cutting a release

1. Bump `[workspace.package] version` in the root `Cargo.toml` (commit on `main`).
2. Tag and push:

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

3. The **Release** workflow builds the matrix (`cargo build --release`), uploads workflow artifacts, and publishes a GitHub Release with archives attached.

Or: **Actions → Release → Run workflow** (optional version override). Publishes tag `v{version}`.

Release builds always use the **release** Cargo profile.

### Platforms (GitHub-hosted free runners)

| OS | Arch | Runner | Status |
|----|------|--------|--------|
| Linux | x64 | `ubuntu-22.04` | Built |
| macOS | x64 | `macos-13` | Built |
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
