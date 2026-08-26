# Conformance Tests

Jereko validates OpenCode compatibility through **owned conformance fixtures and specs** — not by vendoring or forking upstream source code.

## Principles

1. **No upstream code in repo** — We do not submodule, vendor, or clone OpenCode into this repository.
2. **Fixture-driven** — Request/response pairs, config samples, and API schemas live under `conformance/fixtures/`.
3. **Spec-derived** — Fixtures are authored from public API documentation and behavioral observations, maintained by Jereko contributors.
4. **Automated** — `cargo test -p jereko-conformance` runs the harness; HTTP adapter tests will hit a local server with fixture payloads.

## Directory Layout

```
conformance/
├── Cargo.toml          # conformance crate (workspace member)
├── README.md           # this file
├── fixtures/           # owned test fixtures (JSON, snapshots)
│   └── .gitkeep
└── src/
    ├── lib.rs
    └── workspace_tests.rs
```

## Running Tests

```bash
# All workspace tests including conformance
cargo test

# Conformance crate only
cargo test -p jereko-conformance
```

## Phase 1 Additions

- HTTP adapter round-trip tests (v1 and v2) against fixture payloads
- Config merge tests with owned `opencode.json` / `tui.json` samples
- Provider registry contract tests
- Optional: black-box tests against a running `jereko serve` instance

## Compatibility Reference

OpenCode is mentioned only as the **compatibility reference target** — the external behavior we aim to match. It is not a dependency of this repository.

See also: [docs/conformance.md](../docs/conformance.md)
