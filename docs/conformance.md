# Conformance Testing Strategy

Jereko achieves OpenCode API compatibility through **conformance-test-driven development** without importing upstream source code.

## Why Not Fork-and-Merge?

Fork-and-merge creates ongoing maintenance burden: merge conflicts, license entanglement, and architectural coupling. Instead, Jereko:

1. Defines owned API fixtures derived from public specifications.
2. Implements adapters that pass those fixtures.
3. Iterates until behavioral parity is demonstrated by tests.

## Test Seams

Tests live at **pre-agreed seams** — public interfaces where behavior is observed without reaching inside implementations. This table is the default seam registry for the project.

| Seam | Crate / module | Test type | Phase | Example |
|------|----------------|-----------|-------|---------|
| Normalized ↔ v1/v2 wire | `jereko-server/adapters/` | Unit (in-crate) | 1 | normalize/denormalize round-trip |
| HTTP router | `jereko-server/router` | In-process integration | 1 | fixture request → router → compare normalized response |
| HTTP server (black-box) | `jereko serve` | Black-box integration | 1 | fixture request → running server → compare response JSON |
| `Provider` trait | `jereko-providers` | Unit + stub | 1 | `StubProvider` at trait boundary |
| Config merge | `jereko-config` | Unit | 1 | loader precedence with owned JSON samples |
| Sidecar IPC | `SidecarPort` (BunPluginHost transport) | Integration | 2 | JSON-line contract via in-memory test adapter |
| PluginOrchestrator hook dispatch | `PluginOrchestrator` | Integration | 2+ | Host-agnostic hook fixtures; ordered chain across hosts |
| NativePluginHost | `NativePluginHost` / C ABI | Integration | 2.5+ | Load test dylib; same fixtures as Bun where applicable |
| End-to-end session flow | workspace | Integration | 2+ | config → session → message → stub response |
| Workspace smoke | `conformance` crate | Smoke | 0 | all crates link; basic invariants hold |

### TDD Policy

Follow the [tdd](../.agents/skills/tdd/SKILL.md) skill:

- **Default seams** are documented in the table above. Use them without per-test confirmation.
- **New seams** outside this table require explicit confirmation with the user before writing tests.
- Work in **vertical slices**: one seam, one failing test, minimal implementation, repeat.
- Refactoring is not part of the red-green loop — it happens during PR review until a `code-review` skill is added.

### HTTP Testing: In-Process vs Black-Box

Both styles are used; they serve different roles:

| Style | Speed | Role |
|-------|-------|------|
| **In-process router tests** | Fast (milliseconds) | Adapter unit tests, router wiring, normalized type assertions |
| **Black-box `jereko serve` tests** | Slower (spawns server) | Layer 3 conformance — full HTTP stack, headers, serialization, bind/listen path |

In-process tests are the primary TDD feedback loop. Black-box tests are the conformance gate that catches integration issues invisible to in-process tests.

## Test Layers

Layers map to the seam table and phase rollout:

### Layer 0: Workspace Smoke (`conformance` crate)

Validates all crates link and basic invariants hold:

```bash
cargo test -p jereko-conformance
```

### Layer 1: Unit Tests (in-crate)

Each crate carries focused unit tests at its seams:

- `jereko-config`: merge precedence, type serialization
- `jereko-core`: session round-trip
- `jereko-providers`: registry registration/resolution; `StubProvider` behavior
- `jereko-server`: wire adapter normalize/denormalize functions

### Layer 2: In-Process Router Tests (Phase 1)

Fast integration tests against the Axum router without spawning a server:

1. Load request JSON from `conformance/fixtures/v1/` or `fixtures/v2/`.
2. Send through the in-process router (via `tower::ServiceExt` or equivalent).
3. Compare normalized or wire response against owned expected JSON.

### Layer 3: Black-Box HTTP Conformance (Phase 1)

Full-stack conformance against a running server:

1. Start `jereko serve` on a test port.
2. Load request JSON from fixtures.
3. Send HTTP request (curl, `reqwest`, or test harness).
4. Compare response against owned expected JSON (flexible field matching where needed).

This layer maps to diagnosing-bugs **loop #2** (curl/HTTP script against a running dev server).

### Layer 4: Integration Tests (Phase 2+)

End-to-end flows at the session seam:

- Config load → session create → message → provider stub response
- Sidecar IPC via `SidecarPort` test adapter

Maps to diagnosing-bugs **loop #1** (failing integration test at the seam that reaches the bug).

## Diagnosing-Bugs Feedback Loops

Map from [diagnosing-bugs](../.agents/skills/diagnosing-bugs/SKILL.md) loop types to conformance layers:

| Loop type | Conformance layer | Use case |
|-----------|-------------------|----------|
| #1 Failing test at seam | Layer 4 (e2e) | Session flow bugs, sidecar IPC regressions |
| #2 Curl / HTTP script | Layer 3 (black-box) | HTTP wire format mismatches, header issues |
| #9 Differential | Any layer with fixtures | Compare fixture expected vs actual response |
| Unit test at adapter seam | Layer 1–2 | Normalize/denormalize bugs, config merge errors |

When debugging compatibility regressions, prefer a differential loop: run the same fixture input, diff expected JSON (independent source of truth) against actual output.

## Fixture Authorship Rules

All fixtures under `conformance/fixtures/` are **authored and maintained by Jereko**:

1. **Independent source of truth** — expected values come from public API docs, OpenAPI specs, or observed behavior. Not from re-running the implementation under test.
2. **No tautological assertions** — do not assert `expect(normalize(x)).toEqual(normalize(x))` or derive expected JSON by hand the same way the code does. If the test passes by construction, it cannot catch bugs.
3. **Host-agnostic for plugins** — plugin hook fixtures under `conformance/fixtures/plugins/` (Phase 2+) define expected behavior independent of whether Bun, native, or WASM loaded the plugin. Same input → same expected output regardless of host.
4. **Not copied wholesale** from upstream repositories.
5. **Versioned alongside adapter changes** — when wire format changes, update fixtures in the same PR.

Example fixture layout (Phase 1):

```
conformance/fixtures/
├── v1/
│   ├── create_session_request.json
│   └── create_session_response.json
├── v2/
│   ├── create_session_request.json
│   └── create_session_response.json
└── config/
    ├── opencode_minimal.json
    └── tui_minimal.json
```

### Snapshot Testing

- **JSON fixtures** are the primary source of truth for HTTP conformance.
- **`cargo insta`** is optional for adapter normalize/denormalize round-trips in Phase 1+; snapshots supplement, not replace, independently authored fixtures.

## Provider Testing Policy

Per [tdd/mocking.md](../.agents/skills/tdd/mocking.md): mock at system boundaries only.

| Context | Mock location | Approach |
|---------|---------------|----------|
| Unit / registry tests | `Provider` trait boundary | `StubProvider` — no HTTP mocking needed |
| Conformance / integration | Provider registry | Register stub providers |
| Real provider implementations | HTTP boundary | `wiremock`, `httptest`, or equivalent — mock external LLM HTTP APIs, not internal provider methods |

Do not mock internal methods of provider implementations. Test each provider adapter against recorded HTTP interactions or a local mock server at the wire boundary.

## CI Integration

CI runs on every push and pull request (see [`.github/workflows/ci.yml`](../.github/workflows/ci.yml)):

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
```

`Cargo.lock` is tracked — jereko is an application binary and CI uses `--locked` for reproducible builds.

Future (Phase 2+): Bun sidecar contract validation in CI.

## Compatibility Reference

**OpenCode** is the external compatibility target — the behavior Jereko aims to match. It is:

- Mentioned in architecture docs only.
- Not a git submodule, vendored dependency, or cloned directory in this repo.
- Not required to run conformance tests (fixtures are self-contained).

## Deprecation Path for v1

Because adapters normalize early, v1 deprecation becomes:

1. Mark v1 routes as deprecated in docs/responses.
2. Monitor v1 fixture test usage.
3. Remove `adapters/v1/` when v1 traffic reaches zero.
4. Core handlers and normalized types remain unchanged.
