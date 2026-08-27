# CLI ↔ OpenCode Parity Roadmap

**Status:** Active — command-surface and behavioral CLI parity (foundation HTTP/plugin parity closed)  
**Date:** 2026-08-27  
**Related:** [roadmap-parity.md](./roadmap-parity.md) (closed R0–P3e foundation) · [roadmap-releases.md](./roadmap-releases.md) (packaging / distribution — **owned there**, sequenced here) · [conformance.md](./conformance.md) · [architecture.md](./architecture.md) · [ADR 001](./adr/001-architecture-decisions.md)

Goal: make `jerekode` (and aliases `opencode` / `opencode2`) a **practical drop-in CLI** for OpenCode users where we choose to compete — proven by owned black-box fixtures, not by copying upstream source.

---

## Why this roadmap exists

Documented parity slices **R0–P3e** closed the **runtime contract**: HTTP v1/v2 wire, config merge, providers/streaming, tools/policy, Bun/native/WASM plugins, MCP/LSP/PTY depth, CLI **smoke** only (`version` + `serve` health / session create).

That left the **user-facing CLI** under-specified. Today the binary is **not** one-to-one with OpenCode:

| Dimension | Shipped reality | OpenCode (public docs) |
|-----------|-----------------|------------------------|
| Subcommands | `serve`, `run`, `version` | Dozens (`agent`, `auth`, `mcp`, `models`, `session`, `web`, …) |
| Default argv | Subcommand **required** | Bare `opencode` → TUI |
| `run` | Sidecar bootstrap then **shutdown** (stub) | Non-interactive prompt / agent loop (+ `--attach`, session flags, …) |
| `serve` flags | `--host`, `--port`, … | `--hostname`, `--port`, `--cors`, `--mdns`, … |
| Version UX | `version` subcommand; clap `--version` disabled | Global `-v` / `--version` |
| Conformance | Layer 6 smoke only | N/A — we must own CLI fixtures |

**Parity contract today (implicit):** HTTP + config + plugins + tools at pre-agreed seams.  
**Under-specified:** argv surface, flag aliases, exit codes, stdout/stderr shapes, default-no-args TUI, auth/models/session management CLIs, web/acp/github product commands.

---

## Goals

1. **Define** an explicit CLI parity contract (in-scope commands, alias policy, output stability).
2. **Close high-value gaps** so common OpenCode workflows (`serve`, interactive TUI / bare invoke, `run "prompt"`, `models`, `auth`, session list/delete) work on jerekode with documented deltas.
3. **Extend conformance** with black-box CLI seams and independent fixtures (no upstream source).
4. **Sequence** remaining [roadmap-releases.md](./roadmap-releases.md) packaging work so installers/distribution land when the CLI story is coherent (or clearly labeled as packaging-only).

## Non-goals

- Vendoring or forking OpenCode (ADR 001).
- Byte-identical help text or identical internal architecture.
- Replacing Bun TUI as the default interactive path (ADR 001/002); optional `native-tui` stays secondary.
- One-shot parity with every OpenCode command (GitHub agent, `pr`, `uninstall`/`upgrade` installers, experimental env flags) unless prioritized below.
- Weakening Bun IPC / native dylib CI hard-gates.
- Implementing CLI features in the same PR as this planning doc.

---

## What “parity contract” means (widened)

| Layer | Contract | Status after R0–P3e | CLI roadmap focus |
|-------|----------|---------------------|-------------------|
| **HTTP wire** | v1/v2 fixtures at router + black-box serve | Strong | Keep; CLI drives server via same APIs |
| **Config** | JSONC merge precedence; `opencode.json` / `tui.json` | Strong | Flag/env naming alignment |
| **Plugins** | Bun + native + WASM hosts; hook fixtures | Strong | `plugin` install CLI later |
| **Tools / policy / extensions** | `/tools/execute`, MCP/LSP/PTY | Strong (depth grows) | Expose via `mcp` / `run` UX |
| **CLI argv + UX** | Subcommands, flags, exit codes, stdout | **Smoke only** | **This roadmap** |
| **Packaging** | Archives → installers → package managers | Active in releases roadmap | Cross-linked phases below |

**Compatibility stance (proposed):**

- Primary name: `jerekode`.
- Aliases `opencode` / `opencode2` remain the **same binary** (ADR 001).
- Prefer **OpenCode-compatible flag names** where cheap (`--hostname` alias for `--host`); keep jerekode-only flags documented.
- Behavioral parity is proven by **owned fixtures**, not by matching OpenCode git SHAs.

---

## Inventory: jerekode CLI today

Source: `crates/jerekode-cli/` (`main.rs`, `commands/*`, `tests/cli_smoke.rs`).

| Command | Flags (today) | Behavior |
|---------|---------------|----------|
| `serve` | `--host`, `-p/--port`, `--provider`, `--model`, `--project` | Load config; bind HTTP (default host `127.0.0.1`, port **4096**); Axum v1/v2 |
| `run` | `--provider`, `--model`, `--project` | Spawn Bun sidecar, load plugins, dispatch `tui.render` bootstrap, **send Shutdown** — not a lasting TUI or prompt runner |
| `version` | (none) | Prints `jerekode {version} (Phase 0 scaffold)` + alias note |
| *(no subcommand)* | — | Clap error (subcommand required) |
| `--help` / `-h` | — | Clap help (smoke-tested) |
| `--version` / `-v` | — | **Disabled** (`disable_version_flag = true`) |

Binary aliases (`opencode`, `opencode2`) are install-time only — see [distribution.md](./distribution.md).

**Tests:** Layer 6 smoke — `version` contains package version; `serve` `/health` + v1/v2 session create; `--help` mentions serve/version. No argv matrix fixtures yet.

---

## Inventory: OpenCode CLI (public docs)

Reference: [opencode.ai/docs/cli](https://opencode.ai/docs/cli/) and [server docs](https://opencode.ai/docs/server/) (2026-08). Rows marked **TBD** need maintainer confirmation against a pinned OpenCode release notes / live `--help` (no upstream tree in-repo).

### Command / behavior matrix

| OpenCode command | OpenCode role (summary) | jerekode today | Gap | Priority hint |
|------------------|-------------------------|----------------|-----|---------------|
| *(bare / default)* | Start TUI; optional `[project]` | Missing (requires subcommand) | Default → interactive path | **P0** |
| `tui` / bare flags | `--continue`, `--session`, `--model`, `--port`, `--hostname`, `--cors`, … | N/A | Flag set + session continue | **P0–P1** |
| `serve` | Headless HTTP API | Partial | Flag names (`--hostname` vs `--host`); missing `--cors`, `--mdns`, basic auth env | **P0–P1** |
| `run [message..]` | Non-interactive prompt / agent | Stub (bootstrap+exit) | Real prompt path, stdout format, `--attach`, session flags | **P0–P2** |
| `attach [url]` | TUI against remote `serve`/`web` | Missing | Client attach | **P1–P2** |
| `version` / `-v` | Print version | Partial | Global `-v`; drop “Phase 0 scaffold” wording | **P0** |
| `models [provider]` | List `provider/model` | Missing | CLI over registry HTTP | **P1** |
| `auth` (login/list/logout) | Provider credentials | Missing | Auth store UX (**TBD** path vs OpenCode `auth.json`) | **P1–P2** |
| `session list/delete` | Session management | Missing (HTTP only) | Thin CLI over session store / HTTP | **P1** |
| `mcp` (add/list/auth/…) | MCP config UX | Missing (HTTP extensions only) | Config + status CLI | **P2** |
| `agent` (create/list) | Agent files / permissions | Missing | Product decision | **P2–P3** / TBD |
| `export` / `import` | Session JSON / share URL | Missing | Persistence format | **P2** |
| `stats` | Token/cost stats | Missing | Needs metering | **P3** / TBD |
| `web` | Serve + browser UI | Missing | Product decision | **P3** / out of scope? |
| `acp` | Agent Client Protocol stdio | Missing | Product decision | TBD |
| `plugin` / `plug` | Install plugin into config | Missing | Orchestrator already loads | **P2** |
| `github` / `pr` | GH Actions / PR checkout | Missing | Likely **out of scope** initially | Non-goal / later |
| `db` / `db path` | DB tools | Missing | Optional with `sessionDb` | **P2** |
| `debug` | Troubleshooting | Missing | Low priority | **P3** |
| `upgrade` / `uninstall` | Self-update / remove | Missing | Conflicts with package managers | Prefer releases roadmap |
| Global `--pure`, `--log-level`, … | Runtime toggles | Partial (tracing `RUST_LOG`) | Document mapping | **P1** |
| Env `OPENCODE_*` | Large matrix | Partial (config/env merge) | Document supported subset | **P1** |

### Flag spotlight: `serve`

| Flag / concern | OpenCode | jerekode | Notes |
|----------------|----------|----------|-------|
| Port | `--port` (default 4096) | `--port` / `-p` (default 4096) | Aligned default |
| Bind host | `--hostname` | `--host` | **Compat alias** cheap win |
| CORS | `--cors` (repeatable) | Missing | Needed for browser clients |
| mDNS | `--mdns`, `--mdns-domain` | Missing | Optional / later |
| Basic auth | `OPENCODE_SERVER_PASSWORD` (+ username) | Missing | Security-sensitive; decide before shipping |
| Provider/model override | (via config / other cmds) | `--provider`, `--model` | jerekode-extra; keep |
| Project root | TBD | `--project` | Keep; map to OpenCode `--dir` if applicable |

### Flag spotlight: `run`

| Flag / concern | OpenCode | jerekode | Notes |
|----------------|----------|----------|-------|
| Prompt args | `run [message..]` | None | **Core gap** |
| `--attach` | Attach to running server | Missing | Depends on durable `serve` |
| `--continue` / `--session` / `--fork` | Session continuity | Missing | Needs session UX |
| `--model` / `-m` | `provider/model` | `--model` (+ separate `--provider`) | Align form **TBD** |
| `--format json` | Event stream JSON | Missing | Automation seam |
| `--file` / `--thinking` / `--auto` | Rich run UX | Missing | Phase after prompt works |
| Sidecar lifecycle | Stays for TUI / attach | Bootstrap then exit | Must become real interactive or headless runner |

---

## Closed parity checklist vs CLI lag

From [roadmap-parity.md](./roadmap-parity.md) — **runtime slices are Done**; CLI still lags:

| Slice | Runtime claim | CLI lag |
|-------|---------------|---------|
| R0–P0a | Release + CI hard-gates | N/A |
| P0b / P3a | Tools + HTTP surface | No `session`/`models` CLI wrappers |
| P1a–P1b | Streaming + Bun plugins | `run` does not expose streaming chat UX |
| P2a–P2d | MCP/LSP/PTY/WASM | No `mcp` / `plugin` CLIs |
| P3b–P3d | Providers, sandbox, native-tui | Bare TUI default missing; `native-tui` feature not CLI-documented as OpenCode path |
| P3e / DOC | Criterion + docs | Docs correctly call CLI “smoke” only |

**Verdict:** Treat R0–P3e as **closed foundation**. Do **not** reopen that board; track CLI work with new IDs here (`CLI-P0` …).

---

## Phased plan

### CLI-P0 — Command surface & honesty (compatibility entry)

**Outcome:** Users can invoke jerekode like OpenCode for the three primary modes without false advertising.

- [ ] **Bare invoke** → default interactive path (Bun TUI), matching OpenCode’s no-args TUI (or explicit `tui` subcommand + bare alias).
- [ ] **`run` honesty:** either implement minimal non-interactive `run [message..]` (create session → send message → print reply → exit) **or** fail loudly with “not implemented” until ready — remove silent bootstrap-and-exit as the only behavior.
- [ ] **`serve` flag aliases:** add `--hostname` as alias of `--host`; document jerekode `--host` as preferred or dual-supported.
- [ ] **Version UX:** enable `-v` / `--version`; clean `version` subcommand output (drop “Phase 0 scaffold”).
- [ ] **Help matrix:** `--help` lists intended OpenCode-facing commands (even if some are stubs that exit non-zero with a stable message — **decision:** stubs vs omit; see open questions).
- [ ] Conformance: extend Layer 6 — help text fixtures; version flag; serve `--hostname` smoke.

### CLI-P1 — Flag / compat aliases & thin management CLIs

**Outcome:** Common automation and discovery commands work against the existing HTTP/config seams.

- [ ] `serve`: `--cors` (and document security defaults); map logging flags ↔ `RUST_LOG` / `--log-level` if adopted.
- [ ] `models [provider]` — list from provider registry (table + optional `--format json`).
- [ ] `session list` / `session delete` — CLI over session store or local `serve` HTTP.
- [ ] `run` essentials: positional message, `--model` / provider form, `--format default|json`, exit codes.
- [ ] Document supported `OPENCODE_*` / jerekode env subset in [distribution.md](./distribution.md) or a new `docs/cli.md`.
- [ ] Conformance: argv fixtures under `conformance/fixtures/cli/` (see test strategy).

### CLI-P2 — Behavioral parity (agent loop & attach)

**Outcome:** Headless and attached workflows match OpenCode’s mental model for day-to-day use.

- [ ] Durable `run` agent loop (tools, streaming stdout, permissions/`--auto` policy **TBD**).
- [ ] `run --attach` / `attach` against `jerekode serve`.
- [ ] `auth login|list|logout` — credential store design (**TBD** path; do not copy upstream files).
- [ ] `mcp list|add` thin UX over config + extension health.
- [ ] `plugin` / `plug` install into config (Bun string + native/wasm forms per ADR 002).
- [ ] `export` / `import` session JSON (owned schema fixtures).
- [ ] `db path` when `sessionDb` configured.
- [ ] Optional: basic auth for `serve` (`OPENCODE_SERVER_PASSWORD` compatibility).

### CLI-P3 — Packaging / distribution (from releases roadmap)

**Ownership:** Implementation and checkboxes remain in [roadmap-releases.md](./roadmap-releases.md). This phase is the **integration order** relative to CLI work.

| Releases ID | Item | When relative to CLI |
|-------------|------|----------------------|
| Rel-P1 (remaining) | linux/windows **arm64**; optional **native-only** artifacts + `bun-sidecar` Cargo feature | Parallel with CLI-P1; native-only binary must error clearly if Bun plugins required |
| Rel-P2 | Installers (largely shipped) | Keep docs aligned with CLI command names |
| Rel-P3 | Homebrew / winget / Nix; AUR publish; expand native-only; Bun bundling revisit | Prefer after CLI-P0 so formulae don’t document a stub `run` |
| Rel-P4 | Apple notarization / Authenticode | After installers stable; unsigned OK until then |

**Do not** block CLI-P0 on signing or brew taps. **Do** avoid publishing “OpenCode-compatible CLI” marketing until CLI-P0 honesty items land.

### CLI-P4 — Deferred / product decisions

- `web`, `acp`, `github`, `pr`, `stats`, `upgrade`, `uninstall`, experimental env umbrella.
- Full OpenCode env parity.
- Pinokio / Gepeto / Cursor SDK productization ([architecture.md](./architecture.md) future notes).

---

## Releases roadmap — remaining work (explicit fold-in)

From [roadmap-releases.md](./roadmap-releases.md) as of this doc:

| ID | Status | Owner doc | Notes for CLI plan |
|----|--------|-----------|--------------------|
| Rel-P0 (notes + wipe + policy A) | **Done** | releases | Historical |
| Rel-P1 multi-arch / naming | Partial — naming done; arm64 & native-only **open** | releases | CLI-P3 linkage |
| Rel-P2 installers | Largely **done** | releases | Keep README/CLI help consistent |
| Rel-P3 package managers | Mostly **open** (AUR in-repo) | releases | After CLI-P0 |
| Rel-P4 signing | **Open** | releases | Independent |
| Dual-build `bun-sidecar` feature | **Planned, not implemented** | releases + ADR 003 | Required before native-only claims |

Open packaging questions (signing certs, brew tap name, default full vs native-only, Bun bundling) stay in the releases doc; CLI open questions are listed below.

---

## Conformance / test strategy (CLI)

Aligned with [conformance.md](./conformance.md). **New seams require maintainer confirmation** before landing outside the table — proposed additions:

| Proposed seam | Location | Style | Fixtures |
|---------------|----------|-------|----------|
| CLI argv / help / version | `jerekode-cli` binary | Black-box (`CARGO_BIN_EXE_jerekode`) | `conformance/fixtures/cli/help_*.txt` or structured JSON expectations |
| CLI `serve` flag contract | binary + HTTP | Black-box | Bind via `--hostname`/`--port`; `/health` |
| CLI `run` stdout contract | binary | Black-box | Golden **shape** fixtures for `--format json` events (ids dynamic) |
| CLI `models` / `session` | binary | Black-box or in-process later | Owned lists / exit codes |
| Alias argv | `opencode`/`opencode2` if present on PATH in install tests | Optional smoke | Same as `jerekode` |

**Rules (unchanged):**

1. Independent expected values — never “capture current stdout as truth” without review.
2. No upstream OpenCode clones, dumps, or copied source as fixtures.
3. Prefer shape fixtures when IDs/timestamps vary.
4. Vertical slices: one failing CLI fixture → minimal clap/command impl → green CI.
5. Do not weaken Bun/native hard-gates.
6. Until seams are approved, keep expanding `crates/jerekode-cli/tests/cli_smoke.rs` for thin checks; migrate durable contracts into `conformance/fixtures/cli/` once confirmed.

**Suggested fixture layout:**

```text
conformance/fixtures/cli/
  help_toplevel.json          # required subcommand names / exit code
  version_flag.json           # -v / version stdout shape
  serve_hostname_alias.json   # --hostname binds like --host
  run_prompt_shape.json       # TBD after CLI-P0 decision
```

---

## Documentation updates (when implementing)

| Doc | Change |
|-----|--------|
| [CONTEXT.md](../CONTEXT.md) | Point “active forward plan” at this file **and** releases |
| [README.md](../README.md) | CLI section: real commands vs OpenCode deltas |
| [conformance.md](./conformance.md) | Add approved CLI seams to the table |
| [roadmap-parity.md](./roadmap-parity.md) | Keep closed; link here as active CLI track |
| New `docs/cli.md` (optional) | User-facing command reference + compatibility notes |

---

## Open questions (need user / maintainer decisions)

1. **Compatibility bar:** Full OpenCode CLI mirror, or **documented subset** (serve + TUI + run + models + auth + session)? *(Recommendation: subset with explicit non-goals.)*
2. **Bare `jerekode`:** Must start Bun TUI like OpenCode, or is `jerekode run` / `jerekode tui` enough if documented?
3. **`run` stub:** Ship failing-not-implemented until agent loop exists, or prioritize minimal one-shot prompt in CLI-P0?
4. **Model flag form:** Adopt OpenCode `--model provider/model` only, keep split `--provider`/`--model`, or support both?
5. **Auth store location:** Compatible path with OpenCode’s credentials file, jerekode-specific path, or env-only for v1?
6. **Help stubs:** List unimplemented OpenCode commands in `--help` with “not yet”, or omit until implemented?
7. **`web` / `acp` / `github`:** Explicitly out of scope for 1.0 CLI parity?
8. **Basic auth on `serve`:** Required for OpenCode attach compatibility — implement in CLI-P2 or earlier?
9. **Native-only default download** (releases Q7) vs “full OpenCode fidelity” messaging once CLI aliases are advertised.
10. **Approve new conformance seams** in the table above before first CLI fixture PR?

---

## Execution rules

1. **PR-only** to `main`; Conventional Commits (`feat(cli):`, `test(cli):`, `docs:`).
2. **No upstream OpenCode source** in-repo — public docs + owned fixtures only.
3. Prefer vertical slices that deepen existing HTTP/plugin seams rather than parallel reimplementations.
4. Packaging/signing work stays on [roadmap-releases.md](./roadmap-releases.md); update both docs when sequencing changes.
5. Treat [roadmap-parity.md](./roadmap-parity.md) as historical/closed foundation unless a maintainer reopens an ID.

---

## References

- Public OpenCode CLI: https://opencode.ai/docs/cli/
- Public OpenCode server: https://opencode.ai/docs/server/
- [releases.md](./releases.md) — current auto-release ops  
- [distribution.md](./distribution.md) — aliases and runtime deps  
- [ADR 002](./adr/002-dual-plugin-runtime.md) — Bun / native / WASM  
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — packaging / dual-build  
