# Conformance Testing Strategy

As a Rust port of OpenCode, Jerekode achieves API compatibility through **conformance-test-driven development** without importing upstream source code.

## Why Not Fork-and-Merge?

Fork-and-merge creates ongoing maintenance burden: merge conflicts, license entanglement, and architectural coupling. Instead, Jerekode:

1. Defines owned API fixtures derived from public specifications.
2. Implements adapters that pass those fixtures.
3. Iterates until behavioral parity is demonstrated by tests.

## Test Seams

Tests live at **pre-agreed seams** — public interfaces where behavior is observed without reaching inside implementations.

| Seam | Crate / module | Test type | Example |
|------|----------------|-----------|---------|
| Normalized ↔ v1/v2 wire | `jerekode-server/adapters/` | Unit (in-crate) | normalize/denormalize round-trip |
| HTTP router | `jerekode-server/router` | In-process integration | fixture request → router → response |
| HTTP server (black-box) | `jerekode serve` | Black-box integration | fixture → running server → JSON |
| `Provider` trait | `jerekode-providers` | Unit + stub / wiremock | `StubProvider`; HTTP adapter mocks |
| Streaming | `Provider::complete_stream` | Unit + router SSE | chunk parsers; `/messages/stream` |
| Config merge | `jerekode-config` | Unit | loader precedence with owned JSONC |
| Sidecar IPC | `SidecarPort` | Integration | JSON-line contract; Bun CI hard-gate |
| PluginOrchestrator | hook dispatch | Integration | ordered chain across hosts |
| `tool.execute.before` | server tools + plugins | Integration | bash command mutation before execute |
| NativePluginHost | dylib / C ABI | Integration | test cdylib; CI hard-gate |
| First-party plugins | `packages/*` + native | **True e2e** | real Bun process + real native dylib; shared fixtures; CI hard-gate |
| `@jerekode/rtk` | packages/rtk | Unit + true e2e | shared rules; OpenCode2 path + native cdylib |
| CLI runtime | `jerekode` binary | Smoke e2e | `version` + `serve` health / v1+v2 session create |
| WasmPluginHost | WASM hook ABI | Unit | `jerekode_hook` fixture module |
| Tools / policy | `ToolExecutor` | Unit + router | `/tools/execute` fixtures |
| Extensions | MCP / LSP / PTY | Unit + router | call_tool, hover, pty I/O |
| Session store | `SessionStorePort` | Unit | in-memory + SQLite delete/list |
| End-to-end session flow | workspace | Integration | config → session → message → response |
| Workspace smoke | `conformance` crate | Smoke | crates link; basic invariants |

### TDD Policy

Follow the [tdd](../.agents/skills/tdd/SKILL.md) skill:

- **Default seams** are documented in the table above.
- **New seams** outside this table require confirmation with the user before writing tests.
- Work in **vertical slices**: one seam, one failing test, minimal implementation, repeat.
- Refactoring happens during PR review until a `code-review` skill is added.

### HTTP Testing: In-Process vs Black-Box

| Style | Speed | Role |
|-------|-------|------|
| **In-process router tests** | Fast | Adapter unit tests, router wiring |
| **Black-box `jerekode serve` tests** | Slower | Full HTTP stack conformance gate |

## Test Layers

### Layer 0: Workspace Smoke (`conformance` crate)

```bash
cargo test -p jerekode-conformance
```

### Layer 1: Unit Tests (in-crate)

- `jerekode-config`: merge precedence, type serialization
- `jerekode-core`: session round-trip
- `jerekode-providers`: registry + stubs + HTTP adapters
- `jerekode-server`: adapters, tools, persistence, extensions, policy

### Layer 2: In-Process Router Tests

1. Load request JSON from `conformance/fixtures/v1/` or `fixtures/v2/`.
2. Send through the in-process router (`tower::ServiceExt`).
3. Compare response against owned expected JSON / shape fixtures.

### Layer 3: Black-Box HTTP Conformance

Spawn `jerekode serve` (or bind helper) and exercise fixtures over the network stack.

### Layer 4: Sidecar / Native CI Gates

CI jobs **must hard-fail** when Bun IPC or native dylib loading regresses. Soft-skips are forbidden in CI (`CI` env set).

### Layer 5: First-party (in-house) plugin true e2e

Plugins that ship in this repo (`packages/*` and their native crates) **must** prove behavior with **real hosts**, not in-memory fakes:

| Requirement | Detail |
|-------------|--------|
| Bun | `BunProcessSidecarPort` + sidecar entry; load the package by path (or workspace name); invoke the production hook |
| Native | Load the built cdylib via `NativePluginHost`; same fixture payload/expected as Bun |
| Fixtures | Host-agnostic JSON under `conformance/fixtures/`; expected values are independent of implementation |
| Dual agree | At least one test asserts Bun process and native produce the same command mutation |
| CI | Hard-fail if bun/dylib missing when `CI` is set; table-path only (no optional binaries on `PATH`) |

`InMemorySidecarPort` remains for orchestrator wiring tests. It **must not** fake product-specific rewrite logic.

### Layer 6: CLI runtime smoke

Spawn the real `jerekode` binary (`CARGO_BIN_EXE_jerekode`):

- `jerekode version` — exits 0; prints package version
- `jerekode serve` — `/health` ok; create session on **v1** and **v2**

These live in `crates/jerekode-cli/tests/cli_smoke.rs` and run under `cargo test --workspace`.

**Forward work:** widen to argv/help/flag-compat and `run` behavior fixtures — proposed seams and `conformance/fixtures/cli/` layout in [roadmap-parity-cli.md](./roadmap-parity-cli.md). New seams outside this table still need maintainer confirmation before landing.

## Fixture Rules

- Fixtures live under `conformance/fixtures/`.
- Expected values are an **independent source of truth** — not copied from implementation output.
- Prefer shape fixtures when dynamic fields (ids, timestamps) appear.
- No upstream OpenCode source trees or generated dumps as fixtures.


## Coverage gate (CI)

Pull requests to `main` run `.github/workflows/coverage.yml`, which:

1. Collects Rust workspace coverage via `cargo llvm-cov` (LCOV).
2. Collects Bun/TypeScript coverage for `packages/rtk` and `sidecar` via `bun test --coverage` (LCOV).
3. Computes **diff coverage** of changed lines vs `origin/main` with `diff-cover` for **both** Rust and Bun/TS.
4. **Fails** when changed-line coverage is below **80%** in either stack (override locally with `DIFF_COVERAGE_FAIL_UNDER`).
5. Upserts a **sticky PR comment** (`<!-- jerekode-coverage-sticky -->`) with Rust and Bun diff summaries, uncovered changed regions, and per-package coverage tables.

Whole-repo floor is intentionally not enforced — the gate is on **new/changed lines** in each stack. Reports also appear in the Actions job summary and as the `coverage-reports` artifact.

Local preview:

```bash
cargo install cargo-llvm-cov
cargo build -p jerekode-test-native-plugin --locked
cargo build -p jerekode-rtk-plugin --locked
./scripts/coverage.sh
```

## Related

- [architecture.md](./architecture.md)
- [development.md](./development.md)
- [roadmap-parity.md](./roadmap-parity.md) (closed runtime parity checklist)
- [roadmap-parity-cli.md](./roadmap-parity-cli.md) (active CLI ↔ OpenCode command/behavior plan)
- [roadmap-releases.md](./roadmap-releases.md) (active packaging plan)
