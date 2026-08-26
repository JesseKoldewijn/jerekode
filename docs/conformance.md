# Conformance Testing Strategy

Jereko achieves OpenCode API compatibility through **conformance-test-driven development** without importing upstream source code.

## Why Not Fork-and-Merge?

Fork-and-merge creates ongoing maintenance burden: merge conflicts, license entanglement, and architectural coupling. Instead, Jereko:

1. Defines owned API fixtures derived from public specifications.
2. Implements adapters that pass those fixtures.
3. Iterates until behavioral parity is demonstrated by tests.

## Test Seams

Tests live at **pre-agreed seams** — public interfaces where behavior is observed without reaching inside implementations.

| Seam | Crate / module | Test type | Example |
|------|----------------|-----------|---------|
| Normalized ↔ v1/v2 wire | `jereko-server/adapters/` | Unit (in-crate) | normalize/denormalize round-trip |
| HTTP router | `jereko-server/router` | In-process integration | fixture request → router → response |
| HTTP server (black-box) | `jereko serve` | Black-box integration | fixture → running server → JSON |
| `Provider` trait | `jereko-providers` | Unit + stub / wiremock | `StubProvider`; HTTP adapter mocks |
| Streaming | `Provider::complete_stream` | Unit + router SSE | chunk parsers; `/messages/stream` |
| Config merge | `jereko-config` | Unit | loader precedence with owned JSONC |
| Sidecar IPC | `SidecarPort` | Integration | JSON-line contract; Bun CI hard-gate |
| PluginOrchestrator | hook dispatch | Integration | ordered chain across hosts |
| `tool.execute.before` | server tools + plugins | Integration | bash command mutation before execute |
| NativePluginHost | dylib / C ABI | Integration | test cdylib; CI hard-gate |
| `@jerekode/rtk` | packages/rtk | Unit + conformance | shared rules; OpenCode2 + native |
| WasmPluginHost | WASM hook ABI | Unit | `jereko_hook` fixture module |
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
| **Black-box `jereko serve` tests** | Slower | Full HTTP stack conformance gate |

## Test Layers

### Layer 0: Workspace Smoke (`conformance` crate)

```bash
cargo test -p jereko-conformance
```

### Layer 1: Unit Tests (in-crate)

- `jereko-config`: merge precedence, type serialization
- `jereko-core`: session round-trip
- `jereko-providers`: registry + stubs + HTTP adapters
- `jereko-server`: adapters, tools, persistence, extensions, policy

### Layer 2: In-Process Router Tests

1. Load request JSON from `conformance/fixtures/v1/` or `fixtures/v2/`.
2. Send through the in-process router (`tower::ServiceExt`).
3. Compare response against owned expected JSON / shape fixtures.

### Layer 3: Black-Box HTTP Conformance

Spawn `jereko serve` (or bind helper) and exercise fixtures over the network stack.

### Layer 4: Sidecar / Native CI Gates

CI jobs **must hard-fail** when Bun IPC or native dylib loading regresses. Soft-skips are forbidden.

## Fixture Rules

- Fixtures live under `conformance/fixtures/`.
- Expected values are an **independent source of truth** — not copied from implementation output.
- Prefer shape fixtures when dynamic fields (ids, timestamps) appear.
- No upstream OpenCode source trees or generated dumps as fixtures.

## Related

- [architecture.md](./architecture.md)
- [development.md](./development.md)
- [roadmap-parity.md](./roadmap-parity.md) (closed parity checklist)
- [roadmap-releases.md](./roadmap-releases.md) (active packaging plan)
