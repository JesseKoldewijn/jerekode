# Conformance Tests

Jereko validates OpenCode compatibility through **owned conformance fixtures and specs** — not by vendoring or forking upstream source code.

## Principles

1. **No upstream code in repo** — We do not submodule, vendor, or clone OpenCode into this repository.
2. **Fixture-driven** — Request/response pairs, config samples, and API schemas live under `conformance/fixtures/`.
3. **Spec-derived** — Fixtures are authored from public API documentation and behavioral observations, maintained by Jereko contributors.
4. **Automated** — `cargo test -p jereko-conformance` runs the harness (config, HTTP black-box, tools, e2e).

## Directory Layout

```
conformance/
├── Cargo.toml
├── README.md
├── fixtures/
│   ├── config/
│   ├── plugins/
│   ├── tools/
│   ├── v1/
│   └── v2/
└── src/
    ├── lib.rs
    ├── config_tests.rs
    ├── http_blackbox_tests.rs
    ├── e2e_tests.rs
    └── workspace_tests.rs
```

## Running Tests

```bash
# All workspace tests including conformance
cargo test

# Conformance crate only
cargo test -p jereko-conformance
```

## Coverage (current)

- HTTP adapter / black-box tests against `jereko serve` with owned v1/v2 fixtures
- Config merge tests with owned `opencode.json` / `tui.json` samples
- Tools and workspace integration coverage as implemented under `src/`

Seam policy and fixture rules: [docs/conformance.md](../docs/conformance.md).

## Compatibility Reference

OpenCode is mentioned only as the **compatibility reference target** — the external behavior we aim to match. It is not a dependency of this repository.
