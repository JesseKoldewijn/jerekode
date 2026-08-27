# `@jerekode/rtk`

First-party **RTK** adapter for Jerekode: one package directory, two artifacts.

| Artifact | Path | Host |
|----------|------|------|
| OpenCode2 / Bun plugin | `src/opencode2.ts` (package `@jerekode/rtk`) | Bun sidecar / upstream OpenCode2 |
| Native cdylib | `native/` (`jerekode-rtk-plugin`) | `NativePluginHost` |

Both share [`rules/commands.json`](rules/commands.json). Rewrite prefers `rtk rewrite` on `PATH` when available; CI uses the table path (no `rtk` binary required).

## Prerequisites

Install the [rtk](https://github.com/rtk-ai/rtk) CLI separately for live compression. Without it, commands are still rewritten to `rtk …` via the table so the agent invokes RTK when present.

## Enable in Jerekode (`opencode.json`)

```jsonc
{
  "plugins": [
    // Bun / OpenCode2-compatible entry (workspace path)
    "file:./packages/rtk",
    // Native (build first: cargo build -p jerekode-rtk-plugin)
    { "native": "./target/debug/libjerekode_rtk_plugin.so" }
  ]
}
```

Windows native library name: `jerekode_rtk_plugin.dll`. macOS: `libjerekode_rtk_plugin.dylib`.

## Upstream OpenCode2

Point `plugins` at this package (path or workspace) the same way as other OpenCode plugins. The default export exposes `tool.execute.before`.

## Scope

Bash/shell tool rewrite only (same limitation as upstream OpenCode RTK plugin). Native `read` / `grep` tools are not rewritten.

## Tests

Unit (table path) plus **true e2e** (real Bun process sidecar + native dylib) in conformance — required for in-house plugins ([docs/conformance.md](../../docs/conformance.md) Layer 5).

```bash
bun test ./packages/rtk
cargo build -p jerekode-rtk-plugin --locked
cargo test -p jerekode-conformance --locked -- rtk_
```
