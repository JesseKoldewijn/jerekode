# Contributing to Jereko

Thanks for contributing. Please read [CONTEXT.md](CONTEXT.md), [AGENTS.md](AGENTS.md), and [docs/development.md](docs/development.md) before opening a PR.

## Pull requests only (no direct pushes to `main`)

**All changes land on `main` only via pull request merge.**

- **Never** push commits directly to `main` — not developers, maintainers, or AI agents.
- Open a feature/fix/chore branch, push it, and merge through a GitHub pull request.
- Prefer squash or merge commits as configured on the repository; do not bypass review when required.

### Exception: trusted CI release automation

The **Release** workflow (`.github/workflows/release.yml`) bumps `0.0.<run_number>` after a successful merge (PromptComposer-style; cutover seed `0.0.1`). On protected `main`, it opens a `release-sync` PR (`release/sync-0.0.<n>`), enables **auto-merge (squash)**, and publishes from the bumped commit in the same run. Commit/PR title include `[skip release]` so the sync merge does not start another release. Optional secret `RELEASE_PUSH_TOKEN` (admin PAT) restores a direct push to `main`. Further packaging (installers, dual-build) is tracked in [docs/roadmap-releases.md](docs/roadmap-releases.md) / [ADR 003](docs/adr/003-release-packaging-and-changelogs.md).

- Prefer PRs even for automation when practical.
- Humans and agents must never push to `main` directly; branch protection should block them.
- Skip an automatic release by including `[skip release]` in the merge commit message.

### Recommended GitHub branch protection (`main`)

Enable under **Settings → Branches → Branch protection rules** (or Rulesets):

| Setting | Recommended |
|---------|-------------|
| Require a pull request before merging | **On** |
| Require status checks to pass | **On** — `rust`, `bun-sidecar` (CI workflow jobs) |
| Require conversation resolution | Optional but encouraged |
| Do not allow force pushes | **On** |
| Do not allow deletions | **On** |
| Allow auto-merge | **On** — used by release sync PRs |
| Allow specified actors to bypass | Org plans only — personal repos: sync PR + auto-merge or `RELEASE_PUSH_TOKEN` |

### CI on pull requests

The **CI** workflow (`.github/workflows/ci.yml`) runs on every `pull_request` to `main`/`master` (and on pushes to those branches). There are **no** `paths` / `paths-ignore` filters — docs-only PRs still run `rust` and `bun-sidecar` so required checks are never left pending.

| Trigger | Purpose |
|---------|---------|
| `pull_request` | Attaches `rust` / `bun-sidecar` to the PR (satisfies branch protection) |
| `push` to `main`/`master` | Post-merge verification |
| `workflow_dispatch` | Optional manual run on a branch; does **not** replace `pull_request` for merge gates |

If a PR shows no check runs (for example during a [GitHub Actions](https://www.githubstatus.com/) outage): wait until Actions is healthy, then push an empty commit (`git commit --allow-empty -m "ci: retrigger pull_request checks"`) or use **Re-run all jobs** / re-request checks on the PR. Prefer that over `--admin` merge when possible.

### Required Actions permissions (release sync PRs)

Under **Settings → Actions → General → Workflow permissions**:

| Setting | Required |
|---------|----------|
| Workflow permissions | **Read and write** |
| Allow GitHub Actions to create and approve pull requests | **On** |

Without these, the Release job can push `release/sync-*` but cannot open the sync PR (`createPullRequest` GraphQL error). Optional secret `RELEASE_PUSH_TOKEN` (admin PAT) bypasses that path by pushing the bump to `main` directly.

If the repository plan does not allow required status checks, still require PRs and disallow force pushes/deletions.

## Local checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cd sidecar && bun install && bun run check && bun test
```

## License

Contributions are licensed under the [MIT License](LICENSE).
