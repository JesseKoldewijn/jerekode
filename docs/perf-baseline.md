# Performance Baseline Hooks (Phase 5)

Jereko defines performance measurement seams without shipping a full benchmark suite yet.

## Hooks

| Hook | Location | Metric |
|------|----------|--------|
| HTTP router latency | `jereko-server/tests/router_tests.rs` | In-process request round-trip |
| Black-box serve | `conformance/src/http_blackbox_tests.rs` | Full stack bind + HTTP |
| Sidecar IPC | `jereko-plugins/src/sidecar.rs` | In-memory port send/recv |
| Plugin dispatch | `jereko-plugins/src/orchestrator.rs` | Hook chain invocation count |

## Future Work

- Load tests against `jereko serve` with concurrent sessions
- Sidecar spawn + IPC throughput with real Bun process (optional nightly)
- Threshold gates in CI (optional)

## Running benches

```bash
cargo bench -p jereko-plugins
```

Benches cover JSON hook round-trip, in-memory SidecarPort send/recv, and orchestrator dispatch.

## Native TUI Feature Flag

Optional native TUI rendering is gated behind the `native-tui` feature:

```toml
# crates/jereko-cli/Cargo.toml
[features]
native-tui = ["jereko-plugins/native-tui"]
```

Build with:

```bash
cargo build -p jereko-cli --features native-tui
```

When enabled, `jereko_plugins::render_stub_frame` draws a minimal ratatui frame (test backend). Bun remains the default interactive path.

## Criterion nightly

Scheduled workflow: .github/workflows/bench-nightly.yml runs cargo bench -p jereko-plugins nightly and uploads Criterion HTML artifacts. Not PR-gated.
