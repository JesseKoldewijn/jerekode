# Agent Instructions

Brief orientation for AI agents working in this repository.

## Start Here

1. Read [CONTEXT.md](CONTEXT.md) — crate map, vocabulary, current capability snapshot.
2. Check [docs/adr/](docs/adr/) for architectural decisions in the area you are changing.
3. Follow installed skills in [.agents/skills/](.agents/skills/) for the task at hand.
4. Parity board is closed ([docs/roadmap-parity.md](docs/roadmap-parity.md)); CLI ↔ OpenCode work uses [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md) (**Decided** + **remaining-work inventory** — scope CLI PRs from that); packaging / releases uses [docs/roadmap-releases.md](docs/roadmap-releases.md) and [ADR 003](docs/adr/003-release-packaging-and-changelogs.md) (default download = full/Bun). First-party RTK dual adapter: [ADR 004](docs/adr/004-rtk-dual-adapter.md) / `packages/rtk`.

## Git: pull requests only

- **Never** push commits directly to `main` (agents, developers, maintainers).
- All changes land on `main` **only via pull request merge**.
- Exception: the trusted **Release** CI workflow may open a version-bump sync PR (or push with `RELEASE_PUSH_TOKEN`) after a merge — see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/releases.md](docs/releases.md).
- Include `[skip release]` in a merge commit message / release sync title to skip auto-release.

## Commit messages

Use **Conventional Commits** prefixes on every commit you create:

- `feat:` — user-visible capability
- `fix:` — bug fix
- `docs:` — documentation only
- `chore:` — maintenance, deps, tooling (no behavior change)
- `refactor:` — internal restructuring without behavior change
- `test:` — tests only
- `ci:` — CI / workflow changes

Format: `prefix:` or `prefix(scope):` + short imperative summary (e.g. `feat(cli): add version subcommand`). This aligns with the long-term semver/changelog direction in [roadmap P0c option B](docs/roadmap-releases.md#p0c--post-wipe-version-policy-pick-one); the Release workflow still bumps `0.0.x` sequentially and does not parse prefixes yet.

## Key Docs

| Document | Purpose |
|----------|---------|
| [CONTEXT.md](CONTEXT.md) | Project map and vocabulary |
| [docs/architecture.md](docs/architecture.md) | System design, seams, adapters |
| [docs/conformance.md](docs/conformance.md) | Test seams, fixture rules, TDD policy |
| [docs/development.md](docs/development.md) | Rust standards, build commands |
| [docs/roadmap-parity.md](docs/roadmap-parity.md) | Closed parity checklist (R0–P3e) |
| [docs/roadmap-parity-cli.md](docs/roadmap-parity-cli.md) | Active CLI ↔ OpenCode full drop-in plan + remaining-work inventory |
| [docs/roadmap-remaining.md](docs/roadmap-remaining.md) | Foundation archive (historical) |
| [docs/releases.md](docs/releases.md) | Auto-release and `/build` (current ops) |
| [docs/roadmap-releases.md](docs/roadmap-releases.md) | Active packaging / changelog / version-reset plan |

## Conformance Rules

- Test at **pre-agreed seams** documented in [docs/conformance.md](docs/conformance.md).
- Fixtures must be an **independent source of truth** — no tautological expected values.
- Confirm with the user before adding seams outside the conformance table.
- No upstream OpenCode code in this repository.
- **Do not weaken CI** hard-gates (Bun IPC, native dylib).

## Skills

| Skill | Path |
|-------|------|
| codebase-design | `.agents/skills/codebase-design/SKILL.md` |
| tdd | `.agents/skills/tdd/SKILL.md` |
| diagnosing-bugs | `.agents/skills/diagnosing-bugs/SKILL.md` |
| rust-best-practices | `.agents/skills/rust-best-practices/SKILL.md` |

The TDD skill references a `code-review` skill for the refactor stage; until that skill is added, refactoring happens during PR review (see [docs/development.md](docs/development.md)).
