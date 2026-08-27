# ADR 004: First-party RTK dual adapter package

**Status:** Accepted  
**Date:** 2026-08-26  
**Context:** Local OpenCode2 + native plugins for [rtk-ai/rtk](https://github.com/rtk-ai/rtk); monorepo `packages/` layout

## Decision

Ship **one product directory** [`packages/rtk/`](../../packages/rtk/) (`@jerekode/rtk`) with **two artifacts**:

1. **OpenCode2 / Bun** TypeScript plugin (`tool.execute.before`) — usable by upstream OpenCode2 and Jerekode’s Bun sidecar  
2. **Native** cdylib (`jerekode-rtk-plugin`) — `NativePluginHost` via `jerekode-plugin-sdk`

Shared rewrite rules live in `packages/rtk/rules/commands.json`. Prefer `rtk rewrite` when the CLI is on `PATH`; CI uses the table path (no `rtk` binary required).

Jerekode’s HTTP `/tools/execute` path dispatches `tool.execute.before` through `PluginOrchestrator` before bash runs ([ADR 002](./002-dual-plugin-runtime.md)).

## Monorepo

- **Bun workspaces** at repo root: `packages/*`, `sidecar`
- **Cargo workspace** includes `packages/rtk/native`
- Sidecar path stays `sidecar/` for stability

## Consequences

- First-party example of dual-host plugins with host-agnostic conformance fixtures under `conformance/fixtures/plugins/rtk/`
- **True e2e required:** Bun via `BunProcessSidecarPort` + path import of `packages/rtk`, and native via built `jerekode-rtk-plugin` dylib, sharing the same fixtures (see [conformance.md](../conformance.md) Layer 5). In-memory stubs must not stand in for product rewrite proof.
- Does not bundle the `rtk` binary into releases
- Bash/shell rewrite only (same scope as upstream OpenCode RTK plugin)
