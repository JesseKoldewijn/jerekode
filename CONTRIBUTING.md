# Contributing to Jereko

Thanks for contributing. Please read [CONTEXT.md](CONTEXT.md), [AGENTS.md](AGENTS.md), and [docs/development.md](docs/development.md) before opening a PR.

## Pull requests only (no direct pushes to `main`)

**All changes land on `main` only via pull request merge.**

- **Never** push commits directly to `main` — not developers, maintainers, or AI agents.
- Open a feature/fix/chore branch, push it, and merge through a GitHub pull request.
- Prefer squash or merge commits as configured on the repository; do not bypass review when required.

### Exception: trusted CI release automation

The **Release** workflow (`.github/workflows/release.yml`) bumps `0.1.<run_number>` after a successful merge (PromptComposer-style). On personal repos, “require a pull request” usually blocks `GITHUB_TOKEN` from pushing to `main`; the workflow then publishes via **tag** and opens a **sync PR** (commit message includes `[skip release]`). Optional repo secret `RELEASE_PUSH_TOKEN` (admin PAT) restores direct main pushes.

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
| Allow specified actors to bypass | Org plans only — personal repos: use sync PR or `RELEASE_PUSH_TOKEN` |

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
