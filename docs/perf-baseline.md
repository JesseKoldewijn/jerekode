# Performance Baseline Hooks

Jerekode defines performance measurement seams and ships Criterion benches for plugin hot paths (`cargo bench -p jerekode-plugins`).

## Hooks

| Hook | Location | Metric |
|------|----------|--------|
| HTTP router latency | `jerekode-server/tests/router_tests.rs` | In-process request round-trip |
| Black-box serve | `conformance/src/http_blackbox_tests.rs` | Full stack bind + HTTP |
| Sidecar IPC | `jerekode-plugins/src/sidecar.rs` | In-memory port send/recv |
| Plugin dispatch | `jerekode-plugins/src/orchestrator.rs` | Hook chain invocation count |

## Future Work

- Load tests against `jerekode serve` with concurrent sessions
- Sidecar spawn + IPC throughput with real Bun process (optional nightly)
- Threshold gates in CI (optional)

## Running benches

```bash
cargo bench -p jerekode-plugins
```

Benches cover JSON hook round-trip, in-memory SidecarPort send/recv, and orchestrator dispatch.

## Native TUI Feature Flag

Optional native TUI rendering is gated behind the `native-tui` feature:

```toml
# crates/jerekode-cli/Cargo.toml
[features]
native-tui = ["jerekode-plugins/native-tui"]
```

Build with:

```bash
cargo build -p jerekode-cli --features native-tui
```

When enabled, `jerekode_plugins::render_stub_frame` / `run_interactive` provide a minimal ratatui path. Bun `jerekode run` remains the default interactive path. There is no `jerekode run --native` CLI flag today.


## Criterion nightly

Scheduled workflow: `.github/workflows/bench-nightly.yml` runs `cargo bench -p jerekode-plugins` nightly and uploads Criterion HTML artifacts. Not PR-gated.
