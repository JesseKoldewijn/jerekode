# Agent Instructions

Brief orientation for AI agents working in this repository.

## Start Here

1. Read [CONTEXT.md](CONTEXT.md) — crate map, vocabulary, current capability snapshot.
2. Check [docs/adr/](docs/adr/) for architectural decisions in the area you are changing.
3. Follow installed skills in [.agents/skills/](.agents/skills/) for the task at hand.
4. Check [docs/roadmap-parity.md](docs/roadmap-parity.md) before starting large parity work.

## Git: pull requests only

- **Never** push commits directly to `main` (agents, developers, maintainers).
- All changes land on `main` **only via pull request merge**.
- Exception: the trusted **Release** CI workflow may open a version-bump sync PR (or push with `RELEASE_PUSH_TOKEN`) after a merge — see [CONTRIBUTING.md](CONTRIBUTING.md) and [docs/releases.md](docs/releases.md).
- Include `[skip release]` in a merge commit message / release sync title to skip auto-release.

## Key Docs

| Document | Purpose |
|----------|---------|
| [CONTEXT.md](CONTEXT.md) | Project map and vocabulary |
| [docs/architecture.md](docs/architecture.md) | System design, seams, adapters |
| [docs/conformance.md](docs/conformance.md) | Test seams, fixture rules, TDD policy |
| [docs/development.md](docs/development.md) | Rust standards, build commands |
| [docs/roadmap-parity.md](docs/roadmap-parity.md) | Parity progress board |
| [docs/releases.md](docs/releases.md) | Auto-release and `/build` |
| [docs/roadmap-releases.md](docs/roadmap-releases.md) | Packaging / changelog / version-reset plan |

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
