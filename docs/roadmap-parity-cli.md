# CLI ↔ OpenCode Parity Roadmap

**Status:** Active — command-surface and behavioral CLI parity (foundation HTTP/plugin parity closed)  
**Date:** 2026-08-27 (decisions locked same day)  
**Related:** [roadmap-parity.md](./roadmap-parity.md) (closed R0–P3e foundation) · [roadmap-releases.md](./roadmap-releases.md) (packaging / distribution — **owned there**, sequenced here) · [conformance.md](./conformance.md) · [architecture.md](./architecture.md) · [ADR 001](./adr/001-architecture-decisions.md)

Goal: make `jerekode` (and aliases `opencode` / `opencode2`) a **true drop-in OpenCode CLI** — full command-surface mirror where we compete before 1.0 — proven by owned black-box fixtures, not by copying upstream source.

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
**CLI contract (locked):** full OpenCode CLI mirror for in-scope commands; post-v1 for `web` / `acp` / `github` (and related); see [Decided](#decided--locked-compatibility-contract).

---

## Goals

1. **Define** an explicit CLI parity contract (full OpenCode mirror for pre-v1 scope; alias policy; output stability).
2. **Close high-value gaps** so common OpenCode workflows (`serve`, interactive TUI / bare invoke, `run "prompt"`, `models`, `auth`, session list/delete, attach) work on jerekode as a drop-in.
3. **Extend conformance** with black-box CLI seams and independent fixtures (no upstream source) — seams shown for approval before the first CLI fixture PR.
4. **Sequence** remaining [roadmap-releases.md](./roadmap-releases.md) packaging work so installers/distribution land when the CLI story is coherent (or clearly labeled as packaging-only).

## Non-goals

- Vendoring or forking OpenCode (ADR 001).
- Byte-identical help text or identical internal architecture.
- Replacing Bun TUI as the default interactive path (ADR 001/002); optional `native-tui` stays secondary.
- **`web` / `acp` / `github` / `pr` for 1.0 CLI parity** — explicitly **post-v1** (see Decided).
- `upgrade` / `uninstall` installers competing with package managers (prefer releases roadmap).
- Weakening Bun IPC / native dylib CI hard-gates.
- Implementing CLI features in the same PR as this planning doc.
- Listing unimplemented commands in `--help` (omit until ready).

---

## Decided — locked compatibility contract

Locked for JesseKoldewijn/jerekode (2026-08-27). Phase work must follow these; do not reopen without maintainer agreement.

| # | Topic | Decision | Phase implication |
|---|--------|----------|-------------------|
| 1 | **Compatibility bar** | **Full OpenCode CLI mirror** — true drop-in replacement, not a documented subset. | Raises priority of auth, models, session, attach, and flag/env fidelity alongside serve/TUI/`run`. |
| 2 | **Bare invoke** | Same as OpenCode: bare `jerekode` (or alias `opencode` / `opencode2`) **starts the TUI**. | **CLI-P0** must deliver bare → Bun TUI (not “subcommand required”). |
| 4 | **Model flags** | **Mirror OpenCode exactly** (e.g. `--model provider/model` form and related flags as public docs describe). | Align clap parsing in CLI-P0/P1; keep jerekode-only flags only if they do not break OpenCode argv. |
| 5 | **Auth store** | Mirror OpenCode enough to **import** from OpenCode credentials; **store** under a **jerekode-specific** path (do **not** overwrite OpenCode’s store). | **CLI-P2** `auth` implements import + own store; document paths in `docs/cli.md` when shipping. |
| 6 | **Help stubs** | **Omit until ready** — do not list unimplemented commands in `--help`. | CLI-P0 help matrix = only shipped commands; no “not yet” stubs in help. |
| 7 | **`web` / `acp` / `github`** | **Post-v1** — out of scope for 1.0 CLI parity. | Track under CLI-P4 / product backlog; do not block 1.0 on these. |
| 10 | **Conformance seams** | Proposed seams will be **shown for approval** before the first CLI fixture PR. | No inventing a full fixture table yet; smoke tests may expand until seams are approved. |

**Still open** (do not treat as decided): **#3** `run` stub policy, **#8** serve basic auth timing, **#9** default full vs native-only download messaging — see [Clarifications pending](#clarifications-pending).

---

## What “parity contract” means (widened)

| Layer | Contract | Status after R0–P3e | CLI roadmap focus |
|-------|----------|---------------------|-------------------|
| **HTTP wire** | v1/v2 fixtures at router + black-box serve | Strong | Keep; CLI drives server via same APIs |
| **Config** | JSONC merge precedence; `opencode.json` / `tui.json` | Strong | Flag/env naming alignment |
| **Plugins** | Bun + native + WASM hosts; hook fixtures | Strong | `plugin` install CLI later |
| **Tools / policy / extensions** | `/tools/execute`, MCP/LSP/PTY | Strong (depth grows) | Expose via `mcp` / `run` UX |
| **CLI argv + UX** | Subcommands, flags, exit codes, stdout | **Smoke only** | **This roadmap** — full mirror (Decided #1) |
| **Packaging** | Archives → installers → package managers | Active in releases roadmap | Cross-linked phases below |

**Compatibility stance (locked):**

- Primary name: `jerekode`.
- Aliases `opencode` / `opencode2` remain the **same binary** (ADR 001); bare invoke → TUI.
- **OpenCode-compatible** flag names and model forms are the bar (Decided #4); jerekode-only flags allowed when they do not break drop-in argv.
- Behavioral parity is proven by **owned fixtures**, not by matching OpenCode git SHAs.
- Auth: **import** OpenCode credentials; **write** only to a jerekode-specific store (Decided #5).

---

## Inventory: jerekode CLI today

Source: `crates/jerekode-cli/` (`main.rs`, `commands/*`, `tests/cli_smoke.rs`).

| Command | Flags (today) | Behavior |
|---------|---------------|----------|
| `serve` | `--host`, `-p/--port`, `--provider`, `--model`, `--project` | Load config; bind HTTP (default host `127.0.0.1`, port **4096**); Axum v1/v2 |
| `run` | `--provider`, `--model`, `--project` | Spawn Bun sidecar, load plugins, dispatch `tui.render` bootstrap, **send Shutdown** — not a lasting TUI or prompt runner |
| `version` | (none) | Prints `jerekode {version} (Phase 0 scaffold)` + alias note |
| *(no subcommand)* | — | Clap error (subcommand required) — **must become TUI** (Decided #2) |
| `--help` / `-h` | — | Clap help (smoke-tested) — list only implemented commands (Decided #6) |
| `--version` / `-v` | — | **Disabled** (`disable_version_flag = true`) |

Binary aliases (`opencode`, `opencode2`) are install-time only — see [distribution.md](./distribution.md).

**Tests:** Layer 6 smoke — `version` contains package version; `serve` `/health` + v1/v2 session create; `--help` mentions serve/version. No argv matrix fixtures yet.

---

## Inventory: OpenCode CLI (public docs)

Reference: [opencode.ai/docs/cli](https://opencode.ai/docs/cli/) and [server docs](https://opencode.ai/docs/server/) (2026-08). Rows marked **TBD** need confirmation against a pinned OpenCode release notes / live `--help` (no upstream tree in-repo).

### Command / behavior matrix

| OpenCode command | OpenCode role (summary) | jerekode today | Gap | Priority |
|------------------|-------------------------|----------------|-----|----------|
| *(bare / default)* | Start TUI; optional `[project]` | Missing (requires subcommand) | Default → interactive path | **CLI-P0** (locked) |
| `tui` / bare flags | `--continue`, `--session`, `--model`, `--port`, `--hostname`, `--cors`, … | N/A | Flag set + session continue | **CLI-P0–P1** |
| `serve` | Headless HTTP API | Partial | Flag names (`--hostname` vs `--host`); missing `--cors`, `--mdns`; basic auth **pending** | **CLI-P0–P2** |
| `run [message..]` | Non-interactive prompt / agent | Stub (bootstrap+exit) | Real prompt path **or** honest failure — **pending** | **CLI-P0** (policy open) |
| `attach [url]` | TUI against remote `serve`/`web` | Missing | Client attach | **CLI-P1–P2** (raised by full mirror) |
| `version` / `-v` | Print version | Partial | Global `-v`; drop “Phase 0 scaffold” wording | **CLI-P0** |
| `models [provider]` | List `provider/model` | Missing | CLI over registry HTTP | **CLI-P1** (raised) |
| `auth` (login/list/logout) | Provider credentials | Missing | Import OpenCode store; write jerekode-specific path | **CLI-P2** (raised) |
| `session list/delete` | Session management | Missing (HTTP only) | Thin CLI over session store / HTTP | **CLI-P1** (raised) |
| `mcp` (add/list/auth/…) | MCP config UX | Missing (HTTP extensions only) | Config + status CLI | **CLI-P2** |
| `agent` (create/list) | Agent files / permissions | Missing | Product detail within full-mirror bar | **CLI-P2–P3** |
| `export` / `import` | Session JSON / share URL | Missing | Persistence format | **CLI-P2** |
| `stats` | Token/cost stats | Missing | Needs metering | **CLI-P3** / TBD |
| `web` | Serve + browser UI | Missing | **Post-v1** | Out of 1.0 scope |
| `acp` | Agent Client Protocol stdio | Missing | **Post-v1** | Out of 1.0 scope |
| `plugin` / `plug` | Install plugin into config | Missing | Orchestrator already loads | **CLI-P2** |
| `github` / `pr` | GH Actions / PR checkout | Missing | **Post-v1** | Out of 1.0 scope |
| `db` / `db path` | DB tools | Missing | Optional with `sessionDb` | **CLI-P2** |
| `debug` | Troubleshooting | Missing | Low priority | **CLI-P3** |
| `upgrade` / `uninstall` | Self-update / remove | Missing | Prefer package managers | Releases roadmap |
| Global `--pure`, `--log-level`, … | Runtime toggles | Partial (tracing `RUST_LOG`) | Document mapping | **CLI-P1** |
| Env `OPENCODE_*` | Large matrix | Partial (config/env merge) | Full-mirror bar → expand + document | **CLI-P1+** |

### Flag spotlight: `serve`

| Flag / concern | OpenCode | jerekode | Notes |
|----------------|----------|----------|-------|
| Port | `--port` (default 4096) | `--port` / `-p` (default 4096) | Aligned default |
| Bind host | `--hostname` | `--host` | **Compat alias** cheap win |
| CORS | `--cors` (repeatable) | Missing | Needed for browser clients |
| mDNS | `--mdns`, `--mdns-domain` | Missing | Optional / later within mirror |
| Basic auth | `OPENCODE_SERVER_PASSWORD` (+ username) | Missing | **Not locked** — see Clarifications #8 |
| Provider/model override | (via config / other cmds) | `--provider`, `--model` | Align with OpenCode model form (Decided #4) |
| Project root | TBD | `--project` | Keep; map to OpenCode `--dir` if applicable |

### Flag spotlight: `run`

| Flag / concern | OpenCode | jerekode | Notes |
|----------------|----------|----------|-------|
| Prompt args | `run [message..]` | None | **Core gap**; stub policy **open** (Clarifications #3) |
| `--attach` | Attach to running server | Missing | Depends on durable `serve` |
| `--continue` / `--session` / `--fork` | Session continuity | Missing | Needs session UX |
| `--model` / `-m` | `provider/model` | `--model` (+ separate `--provider`) | **Mirror OpenCode** (Decided #4) |
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

**Outcome:** Users can invoke jerekode like OpenCode for primary modes without false advertising. Full-mirror bar means bare TUI is mandatory; help stays honest (omit unfinished commands).

- [ ] **Bare invoke** → Bun TUI (locked Decided #2). Optional explicit `tui` subcommand if OpenCode exposes it; bare argv is the drop-in path.
- [ ] **`run` honesty:** resolve Clarifications #3 — either minimal one-shot prompt **or** fail loudly with “not implemented”; **remove** silent bootstrap-and-exit as the only behavior.
- [ ] **`serve` flag aliases:** add `--hostname` as alias of `--host`; document dual support.
- [ ] **Version UX:** enable `-v` / `--version`; clean `version` subcommand output (drop “Phase 0 scaffold”).
- [ ] **Help matrix:** `--help` lists **only implemented** commands (Decided #6 — omit stubs).
- [ ] **Model flag form:** begin aligning with OpenCode (Decided #4) where CLI-P0 touches `run`/`serve`/`tui`.
- [ ] Conformance: extend Layer 6 smoke — version flag; serve `--hostname`; help does not advertise unfinished cmds. **Full CLI fixture table waits for seam approval** (Decided #10).

### CLI-P1 — Flag / compat aliases & thin management CLIs

**Outcome:** Common automation and discovery commands work against existing HTTP/config seams (raised by full-mirror bar).

- [ ] `serve`: `--cors` (and document security defaults); map logging flags ↔ `RUST_LOG` / `--log-level` if adopted.
- [ ] `models [provider]` — list from provider registry (table + optional `--format json`).
- [ ] `session list` / `session delete` — CLI over session store or local `serve` HTTP.
- [ ] `run` essentials (once #3 chosen): positional message, OpenCode `--model` form, `--format default|json`, exit codes.
- [ ] Document supported `OPENCODE_*` / jerekode env matrix toward full-mirror bar in [distribution.md](./distribution.md) or a new `docs/cli.md`.
- [ ] Conformance: after maintainer approves proposed seams, add argv fixtures under `conformance/fixtures/cli/`.

### CLI-P2 — Behavioral parity (agent loop & attach)

**Outcome:** Headless and attached workflows match OpenCode’s mental model for day-to-day use.

- [ ] Durable `run` agent loop (tools, streaming stdout, permissions/`--auto` policy as OpenCode docs require).
- [ ] `run --attach` / `attach` against `jerekode serve`.
- [ ] `auth login|list|logout` — **import** from OpenCode credentials; **store** in jerekode-specific location only (Decided #5).
- [ ] `mcp list|add` thin UX over config + extension health.
- [ ] `plugin` / `plug` install into config (Bun string + native/wasm forms per ADR 002).
- [ ] `export` / `import` session JSON (owned schema fixtures).
- [ ] `db path` when `sessionDb` configured.
- [ ] Basic auth for `serve` — **if** Clarifications #8 chooses required-for-parity; otherwise defer.

### CLI-P3 — Packaging / distribution (from releases roadmap)

**Ownership:** Implementation and checkboxes remain in [roadmap-releases.md](./roadmap-releases.md). This phase is the **integration order** relative to CLI work.

| Releases ID | Item | When relative to CLI |
|-------------|------|----------------------|
| Rel-P1 (remaining) | linux/windows **arm64**; optional **native-only** artifacts + `bun-sidecar` Cargo feature | Parallel with CLI-P1; native-only binary must error clearly if Bun plugins required |
| Rel-P2 | Installers (largely shipped) | Keep docs aligned with CLI command names |
| Rel-P3 | Homebrew / winget / Nix; AUR publish; expand native-only; Bun bundling revisit | Prefer after CLI-P0 so formulae don’t document a stub `run` |
| Rel-P4 | Apple notarization / Authenticode | After installers stable; unsigned OK until then |

**Do not** block CLI-P0 on signing or brew taps. **Do** avoid publishing “drop-in OpenCode replacement” marketing until CLI-P0 honesty items land. Default download full vs native-only messaging is **open** (Clarifications #9) — also tracked as releases Q7.

### CLI-P4 — Deferred / post-v1 / product

- **`web`, `acp`, `github`, `pr`** — **post-v1** (Decided #7); not part of 1.0 CLI parity.
- `stats`, `upgrade`, `uninstall`, experimental env umbrella (as capacity allows after 1.0).
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

Open packaging questions (signing certs, brew tap name, **default full vs native-only** — Clarifications #9 / releases Q7, Bun bundling) stay in the releases doc; CLI clarifications are below.

---

## Conformance / test strategy (CLI)

Aligned with [conformance.md](./conformance.md).

**Process (locked Decided #10):** Proposed CLI seams will be **shown for approval before the first CLI fixture PR**. Do **not** invent a full fixture table in this roadmap yet. Until then:

1. Expand `crates/jerekode-cli/tests/cli_smoke.rs` for thin checks only.
2. When ready to land durable contracts, propose a seam table (location / style / fixture layout) in a reviewable doc or PR description for maintainer approval.
3. After approval, add seams to [conformance.md](./conformance.md) and fixtures under `conformance/fixtures/cli/`.

**Rules (unchanged):**

1. Independent expected values — never “capture current stdout as truth” without review.
2. No upstream OpenCode clones, dumps, or copied source as fixtures.
3. Prefer shape fixtures when IDs/timestamps vary.
4. Vertical slices: one failing CLI fixture → minimal clap/command impl → green CI.
5. Do not weaken Bun/native hard-gates.

---

## Documentation updates (when implementing)

| Doc | Change |
|-----|--------|
| [CONTEXT.md](../CONTEXT.md) | Point “active forward plan” at this file **and** releases |
| [README.md](../README.md) | CLI section: real commands vs OpenCode deltas |
| [conformance.md](./conformance.md) | Add **approved** CLI seams to the table (after Decided #10 process) |
| [roadmap-parity.md](./roadmap-parity.md) | Keep closed; link here as active CLI track |
| New `docs/cli.md` (optional) | User-facing command reference + compatibility notes (incl. auth import/store paths) |

---

## Clarifications pending

Answer these when ready; until then keep work flexible around them.

### 3 — `run` stub

OpenCode `run` actually sends a prompt / runs the agent once then exits. Today jerekode’s `run` only boots the Bun sidecar and shuts down (bootstrap stub).

**Choice:**

- **A.** In CLI-P0, make `run` fail loudly with “not implemented” until the agent loop exists (honest drop-in: wrong behavior is worse than clear failure).
- **B.** In CLI-P0, implement a minimal one-shot prompt path so `jerekode run "..."` starts working sooner.

### 8 — Serve basic auth

OpenCode can protect `serve` with basic auth so remote `attach` isn’t open to the world.

**Choice:**

- **A.** Required for drop-in attach parity → implement when `serve`/`attach` land (CLI-P2 or earlier if serve is used remotely).
- **B.** Defer until someone needs remote attach; local serve stays open.

### 9 — Native-only vs full download messaging

Release roadmap discusses **full** builds (Bun sidecar for OpenCode plugin fidelity) vs **native-only** builds. Once we advertise “drop-in OpenCode replacement”, the default download should probably be **full**. Also tracked as [roadmap-releases.md](./roadmap-releases.md) open question 7.

**Choice:**

- **A.** Default download = full (OpenCode fidelity); native-only clearly labeled advanced/server.
- **B.** Something else (user preference).

---

## Execution rules

1. **PR-only** to `main`; Conventional Commits (`feat(cli):`, `test(cli):`, `docs:`).
2. **No upstream OpenCode source** in-repo — public docs + owned fixtures only.
3. Prefer vertical slices that deepen existing HTTP/plugin seams rather than parallel reimplementations.
4. Packaging/signing work stays on [roadmap-releases.md](./roadmap-releases.md); update both docs when sequencing changes.
5. Treat [roadmap-parity.md](./roadmap-parity.md) as historical/closed foundation unless a maintainer reopens an ID.
6. Honor [Decided](#decided--locked-compatibility-contract); do not silently downgrade to “documented subset” parity.

---

## References

- Public OpenCode CLI: https://opencode.ai/docs/cli/
- Public OpenCode server: https://opencode.ai/docs/server/
- [releases.md](./releases.md) — current auto-release ops  
- [distribution.md](./distribution.md) — aliases and runtime deps  
- [ADR 002](./adr/002-dual-plugin-runtime.md) — Bun / native / WASM  
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — packaging / dual-build  
