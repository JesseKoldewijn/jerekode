# ADR 002: Dual Plugin Runtime Architecture

**Status:** Accepted  
**Date:** 2026-08-26  
**Context:** Phase 2 plugin strategy — extend ADR 001 Decision 3 (Bun sidecar) with a unified orchestrator and multiple host implementations

## Decision

Jereko plugins are loaded and dispatched through a **PluginOrchestrator** in Rust that coordinates multiple **PluginHost** implementations. **BunPluginHost** (default for unqualified plugin strings) and **NativePluginHost** (explicit dylib config) run **together** in an ordered hook chain. **WasmPluginHost** is an optional third host for sandboxed plugins (initial load/`jereko_hook` support shipped; deeper sandbox policy remains incremental).

## Background

ADR 001 established the Bun sidecar as the default plugin/TUI path and introduced `SidecarPort` as the Rust-side transport seam. That decision remains correct for Bun fidelity, but a single sidecar-only model cannot serve:

- High-performance or low-latency server hooks (tools, providers, transforms) in-process
- Explicit opt-in for native plugins without forcing all plugins through Bun
- Future untrusted plugin sandboxes (WASM)

This ADR **extends** ADR 001 Decision 3: `SidecarPort` becomes the transport layer for **BunPluginHost** specifically, not the sole plugin abstraction.

## Architecture

```text
┌─────────────────────────────────────────────────────────────────┐
│                     PluginOrchestrator (Rust)                   │
│  unified hook registry · load order · priority · capability     │
│  merge · ordered dispatch · failure isolation                   │
└──────────┬──────────────────────┬──────────────────────┬────────┘
           │                      │                      │
           ▼                      ▼                      ▼
┌──────────────────┐  ┌──────────────────────┐  ┌──────────────────┐
│ Internal hooks   │  │  NativePluginHost    │  │  BunPluginHost   │
│ (built-in)       │  │  dylib via C ABI     │  │  SidecarPort IPC │
│                  │  │  jereko_plugin.h     │  │  full OpenCode   │
│                  │  │  Phase 2.5+          │  │  fidelity        │
└──────────────────┘  └──────────────────────┘  └──────────────────┘
                                                           │
                                                           ▼
                                              ┌──────────────────────┐
                                              │  Bun sidecar process │
                                              │  (sidecar/)          │
                                              └──────────────────────┘

Also:
┌──────────────────┐
│  WasmPluginHost  │
│  sandboxed       │
└──────────────────┘
```

### 1. PluginOrchestrator (Rust)

Central coordinator living in the **`jereko-plugins`** crate:

- **Unified hook registry** — all hook types (server tools, providers, transforms, HTTP routes, etc.) registered in one place regardless of host
- **Load order** — deterministic ordering across internal, native, and Bun plugins (see below)
- **Priority** — per-plugin priority within a host; orchestrator merges and sorts
- **Capability merge** — combines contributions from all hosts (e.g. multiple tools from different plugins)
- **Ordered hook dispatch** — runs hook chain in load order; each host's plugins participate
- **Failure isolation** — a failing plugin in one host does not prevent other hosts' plugins from running; errors are logged and surfaced per-plugin

### 2. PluginHost trait

Generalizes the `SidecarPort` concept. Each host implements `PluginHost`:

```rust
// Implemented in `jereko-plugins` (shape simplified for the ADR).
pub trait PluginHost: Send + Sync {
    fn host_id(&self) -> HostId;
    async fn load(&self, spec: &PluginSpec) -> Result<LoadedPlugin, PluginError>;
    async fn invoke_hook(&self, plugin: &LoadedPlugin, hook: HookCall) -> Result<HookResult, PluginError>;
    async fn unload(&self, plugin: &LoadedPlugin) -> Result<(), PluginError>;
}
```

| Host | Implementation | Transport |
|------|----------------|-----------|
| `BunPluginHost` | Default for unqualified strings | `SidecarPort` → Bun process |
| `NativePluginHost` | Explicit `{ "native": "..." }` config | In-process dylib via stable C ABI |
| `WasmPluginHost` | Explicit `{ "wasm": "..." }` config | WASM runtime (`jereko_hook` ABI) |

### 3. BunPluginHost — default path

- **Default for unqualified plugin strings** — `"@acme/server-plugin"` resolves to Bun
- Full **OpenCode fidelity** via sidecar IPC (TypeScript/Bun plugins run unchanged)
- Uses **`SidecarPort`** as its transport adapter (production: spawn Bun; test: in-memory)
- Isolated in a **separate sidecar process** — Bun crash does not take down the Rust server

### 4. NativePluginHost — in-process dylib

- Loads plugins as **dynamic libraries** (`.so` / `.dylib` / `.dll`) via a **stable C ABI** defined in `jereko_plugin.h`
- **Server hooks first** (Phase 2.5): tools, providers, transforms
- Requires **explicit config** — never loaded implicitly from an unqualified string
- Rust SDK crate: **`jereko-plugin-sdk`** (see [development.md](../development.md))

### 5. WasmPluginHost — sandboxed plugins

- Optional third host for **sandboxed** or user-supplied plugins
- WASM runtime with `jereko_hook` export (host fallback when absent); deeper isolation policy is incremental
- Explicit `{ "wasm": "./path/to/plugin.wasm" }` config only

### 6. Both hosts active together

Bun and native hosts run **simultaneously**, not as either/or:

- Orchestrator builds a single **ordered hook chain** spanning all loaded plugins across all active hosts
- A native tool plugin and a Bun provider plugin can coexist; dispatch follows load order
- **Failure isolation** — native plugin panic/error is contained; Bun sidecar failure is process-isolated

## Config Shape

Plugin entries in `opencode.json` (and related config) resolve to a host by form:

| Config entry | Host | Notes |
|--------------|------|-------|
| `"@acme/server-plugin"` | **Bun** (default) | Unqualified string → BunPluginHost |
| `{ "native": "./path/to/plugin.so" }` | **Native** | Explicit dylib path |
| `{ "wasm": "./path/to/plugin.wasm" }` | **WASM** | Explicit path; `jereko_hook` ABI |

Priority and ordering modifiers (when supported) apply within and across hosts; the orchestrator merges the final dispatch list.

## Hook Dispatch Rules

1. **Registration** — each host registers its plugins' hooks with the orchestrator at load time
2. **Sort key** — `(load_tier, priority, registration_order)` where load tiers are fixed (see Load Order)
3. **Invocation** — for a given hook point (e.g. `before_transform`), orchestrator calls each registered handler in sort order
4. **Short-circuit** — only where hook semantics require it (documented per hook type); default is run-all with result aggregation
5. **Errors** — per-plugin errors are collected; orchestrator decides abort-vs-continue per hook type
6. **Cross-host** — no special casing; native and Bun plugins are peers in the chain once registered

## Load Order

Fixed tier ordering (lowest runs first unless hook semantics invert):

```text
1. Internal (built-in hooks registered by jereko itself)
2. Native  (NativePluginHost dylibs, explicit config)
3. Bun     (BunPluginHost sidecar plugins, default strings)
```

Within each tier, plugins sort by configured **priority** (higher first) then **registration order**.

Rationale: built-in behavior is always available; native plugins are opt-in and trusted; Bun plugins are the compatibility default but run last in the chain so native overrides can take precedence when configured.

## TUI Plugins

- **Phase 2–4:** TUI plugins are **Bun-only** (sidecar hosts all TUI plugin code)
- **Phase 5:** Introduce **`TuiPluginHost`** trait and optional bridge for native TUI plugins
- Native server hooks (Phase 2.5) do not imply native TUI support until Phase 5

## Security Model

| Host | Trust model | Loading |
|------|-------------|---------|
| **Bun** | Isolated sidecar process | Default for unqualified strings; process boundary contains failures |
| **Native** | Trusted, in-process | **Requires explicit config**; never auto-loaded |
| **WASM** | Sandboxed | Explicit config only |

Native plugins run in-process with full process privileges — hence explicit opt-in. WASM is the path for untrusted third-party code.

## Phasing

| Phase | Scope |
|-------|-------|
| **2** | `PluginOrchestrator` + **BunPluginHost only**; `SidecarPort` feeds BunPluginHost; server plugin routes via IPC |
| **2.5** | **NativePluginHost** — server hooks (tools, providers, transforms); `jereko_plugin.h` C ABI; `jereko-plugin-sdk` crate |
| **3** | Bun TUI plugins via sidecar; `jereko run` |
| **4** | **WasmPluginHost** for untrusted plugins |
| **5** | Native TUI plugins; `TuiPluginHost` trait + optional bridge |

Phase 2 ships the orchestrator abstraction even with only one host — this avoids retrofitting the dispatch layer when native arrives in 2.5.

## Conformance Testing

Plugin hook behavior is validated with **host-agnostic fixtures**:

- Same fixture inputs and expected outputs regardless of which host loaded the plugin
- Test adapters: in-memory `SidecarPort` for Bun; temp dylib or mock host for native
- Fixtures live under `conformance/fixtures/plugins/` (Phase 2+)
- **Not tautological** — expected behavior authored independently; see [conformance.md](../conformance.md)

Example seam tests:

| Seam | Phase | Approach |
|------|-------|----------|
| PluginOrchestrator hook dispatch | 2+ | Fixture hook call → orchestrator → compare aggregated result |
| NativePluginHost | 2.5+ | Load test dylib → invoke hook → compare fixture output |
| BunPluginHost | 2+ | In-memory SidecarPort adapter → same fixtures as native where applicable |

## Consequences

- Phase 2 must implement `PluginOrchestrator` and `PluginHost` trait, not ad-hoc sidecar calls
- `SidecarPort` is scoped to `BunPluginHost`; new code should not depend on `SidecarPort` directly except inside that host
- Phase 2.5 adds `jereko_plugin.h` and SDK crate without changing orchestrator dispatch semantics
- Config parser must distinguish unqualified strings (Bun) from `{ "native": ... }` / `{ "wasm": ... }` objects
- TUI remains Bun-only until Phase 5; document this clearly to avoid confusion

## Related Documents

- [ADR 001: Phase 0 Architecture Decisions](./001-architecture-decisions.md) — Decision 3 (Bun sidecar), Decision 6 (SidecarPort)
- [docs/architecture.md](../architecture.md) — Plugin Orchestrator & Dual Hosts section
- [docs/conformance.md](../conformance.md) — plugin hook seam registry
- [CONTEXT.md](../../CONTEXT.md) — domain vocabulary
