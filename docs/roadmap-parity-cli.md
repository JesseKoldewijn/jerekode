# CLI ↔ OpenCode Parity Roadmap

**Status:** Active — full OpenCode CLI drop-in plan (foundation HTTP/plugin parity closed)  
**Date:** 2026-08-27 (decisions locked same day)  
**Related:** [roadmap-parity.md](./roadmap-parity.md) (closed R0–P3e foundation) · [roadmap-releases.md](./roadmap-releases.md) (packaging / distribution — **owned there**, sequenced here) · [conformance.md](./conformance.md) · [architecture.md](./architecture.md) · [ADR 001](./adr/001-architecture-decisions.md)

Goal: make `jerekode` (and aliases `opencode` / `opencode2`) a **true drop-in OpenCode CLI** — full command-surface and behavioral mirror for 1.0 scope — proven by owned black-box fixtures, not by copying upstream source.

Reference inventory: [opencode.ai/docs/cli](https://opencode.ai/docs/cli/) · [opencode.ai/docs/server](https://opencode.ai/docs/server/) (public docs, 2026-08).

---

## Why this roadmap exists

Documented parity slices **R0–P3e** closed the **runtime contract**: HTTP v1/v2 wire, config merge, providers/streaming, tools/policy, Bun/native/WASM plugins, MCP/LSP/PTY depth, CLI **smoke** only (`version` + `serve` health / session create).

That left the **user-facing CLI** under-specified. Today the binary is **not** one-to-one with OpenCode:

| Dimension | Shipped reality | OpenCode (public docs) |
|-----------|-----------------|------------------------|
| Subcommands | `serve`, `run`, `version` | Dozens (`agent`, `auth`, `mcp`, `models`, `session`, …) |
| Default argv | Subcommand **required** | Bare `opencode` → TUI |
| `run` | Sidecar bootstrap then **shutdown** (stub) | Non-interactive prompt / agent loop (+ `--attach`, session flags, …) |
| `serve` flags | `--host`, `--port`, … | `--hostname`, `--port`, `--cors`, `--mdns`, basic auth env |
| Version UX | `version` subcommand; clap `--version` disabled | Global `-v` / `--version` |
| Conformance | Layer 6 smoke only | N/A — we must own CLI fixtures |

**CLI contract (locked):** full OpenCode CLI mirror for 1.0 scope; **post-v1** for `web` / `acp` / `github` / `pr`; see [Decided](#decided--locked-compatibility-contract). Remaining work is inventoried below — including **real `run` parity** (not a permanent stub).

---

## Goals

1. **Plan and close** the entire remaining OpenCode CLI surface needed for drop-in use (1.0 scope).
2. Deliver bare TUI, real `run`, `serve`/`attach`, auth/models/session/mcp/agent/plugin/export/import/db/stats, and flag/env fidelity.
3. **Extend conformance** with black-box CLI seams — proposed seams shown for approval before the first CLI fixture PR.
4. **Sequence** [roadmap-releases.md](./roadmap-releases.md) so default downloads are **full (Bun)** when drop-in messaging ships.

## Non-goals

- Vendoring or forking OpenCode (ADR 001).
- Byte-identical help text or identical internal architecture.
- Replacing Bun TUI as the default interactive path (ADR 001/002); optional `native-tui` stays secondary.
- **`web` / `acp` / `github` / `pr` for 1.0** — **post-v1** (Decided #7).
- `upgrade` / `uninstall` competing with package managers (prefer releases roadmap; may land later as thin wrappers).
- Weakening Bun IPC / native dylib CI hard-gates.
- Listing unimplemented commands in `--help` (omit until ready).

---

## Decided — locked compatibility contract

Locked for JesseKoldewijn/jerekode (2026-08-27). Phase work must follow these; do not reopen without maintainer agreement.

| # | Topic | Decision | Phase implication |
|---|--------|----------|-------------------|
| 1 | **Compatibility bar** | **Full OpenCode CLI mirror** — true drop-in, not a documented subset. | Plan and ship the [remaining-work inventory](#remaining-work-inventory--full-drop-in-10); raise priority of auth, models, session, attach, `run`, flag/env fidelity. |
| 2 | **Bare invoke** | Bare `jerekode` (or `opencode` / `opencode2`) **starts the TUI**. | **CLI-P0**. |
| 3 | **`run` + remaining work** | Plan the **entirety** of remaining work to match OpenCode behaviour; **real `run` parity** is in that plan (not a permanent stub). Interim honest failure OK only until one-shot/agent path lands — never silent bootstrap-and-exit as the advertised behavior. | **CLI-P0** starts `run` (one-shot); **CLI-P1–P2** complete agent loop + flags; see inventory. |
| 4 | **Model flags** | **Mirror OpenCode exactly** (`--model` / `-m` as `provider/model`, plus related flags per public docs). | Align clap in every command that takes a model; jerekode-only flags must not break OpenCode argv. |
| 5 | **Auth store** | **Import** from OpenCode credentials; **store** under a **jerekode-specific** path (do **not** overwrite OpenCode’s store). | **CLI-P2** `auth`; document paths in `docs/cli.md`. |
| 6 | **Help stubs** | **Omit until ready** — do not list unimplemented commands in `--help`. | Help = shipped commands only. |
| 7 | **`web` / `acp` / `github` / `pr`** | **Post-v1** — out of 1.0 CLI parity. | **CLI-P4**. |
| 8 | **Serve basic auth** | **Required** for drop-in `attach` / remote `run --attach` parity (`OPENCODE_SERVER_PASSWORD` / username + `--password` / `--username` on clients). | Schedule with `serve` hardening + `attach` (**CLI-P1–P2**); do not ship remote attach without it. |
| 9 | **Default download** | **Default = full (Bun included)** for OpenCode fidelity; **native-only = advanced / future**. | Lock in [roadmap-releases.md](./roadmap-releases.md); marketing and install tables lead with full. |
| 10 | **Conformance seams** | Proposed seams **shown for approval** before the first CLI fixture PR. | No full fixture table invented here yet; smoke may expand until approved. |

---

## What “parity contract” means (widened)

| Layer | Contract | Status after R0–P3e | CLI roadmap focus |
|-------|----------|---------------------|-------------------|
| **HTTP wire** | v1/v2 fixtures at router + black-box serve | Strong | Keep; CLI drives server via same APIs |
| **Config** | JSONC merge precedence; `opencode.json` / `tui.json` | Strong | Flag/env naming alignment |
| **Plugins** | Bun + native + WASM hosts; hook fixtures | Strong | `plugin` install CLI |
| **Tools / policy / extensions** | `/tools/execute`, MCP/LSP/PTY | Strong (depth grows) | Expose via `mcp` / `run` / agent UX |
| **CLI argv + UX** | Subcommands, flags, exit codes, stdout | **Smoke only** | **This roadmap** — full mirror |
| **Packaging** | Archives → installers → package managers | Active in releases | Default **full** download (Decided #9) |

**Compatibility stance (locked):**

- Primary name: `jerekode`; aliases `opencode` / `opencode2` are the same binary (ADR 001); bare → TUI.
- OpenCode-compatible flags and `--model provider/model` are the bar.
- Auth: import OpenCode store; write only jerekode-specific store.
- Default user-facing artifact: **full** (Bun); native-only labeled advanced/future.
- Behavioral parity via **owned fixtures**, not OpenCode git SHAs.

---

## Inventory: jerekode CLI today

Source: `crates/jerekode-cli/` (`main.rs`, `commands/*`, `tests/cli_smoke.rs`).

| Command | Flags (today) | Behavior |
|---------|---------------|----------|
| `serve` | `--host`, `-p/--port`, `--provider`, `--model`, `--project` | Load config; bind HTTP (default `127.0.0.1:4096`); Axum v1/v2 |
| `run` | `--provider`, `--model`, `--project` | Spawn Bun sidecar, bootstrap, **Shutdown** — stub, not OpenCode `run` |
| `version` | (none) | Prints version + “Phase 0 scaffold” |
| *(no subcommand)* | — | Clap error — **must become TUI** |
| `--help` / `-h` | — | Lists only what exists (keep that policy) |
| `--version` / `-v` | — | **Disabled** |

Binary aliases: [distribution.md](./distribution.md). Tests: Layer 6 smoke only.

---

## Remaining-work inventory — full drop-in (1.0)

Comprehensive backlog to match OpenCode CLI behaviour/functionality for **1.0**. Post-v1 items are listed but **not** required for 1.0. Status: **Todo** unless noted. Phases are sequencing hints, not separate products.

### A. Invocation & globals

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| Bare invoke → TUI | `opencode` / `opencode [project]` starts TUI | Subcommand required | **CLI-P0** |
| `tui` / bare flags | `--continue`/`-c`, `--session`/`-s`, `--fork`, `--prompt`, `--model`/`-m`, `--agent`, `--auto`, `--port`, `--hostname`, `--mdns`, `--mdns-domain`, `--cors` | Missing | **CLI-P0–P1** |
| Global `-v` / `--version` | Print version | Disabled; `version` subcommand wording stale | **CLI-P0** |
| Global `--help` | Honest help (omit unfinished cmds) | OK policy; expand as cmds land | **CLI-P0+** |
| Global `--print-logs`, `--log-level` | stderr logs / DEBUG\|INFO\|WARN\|ERROR | Partial (`RUST_LOG`) | **CLI-P1** |
| Global `--pure` | Run without external plugins | Missing | **CLI-P1–P2** |
| `--dir` / project root | Working directory for TUI/`run`/`attach` | `--project` only | **CLI-P0–P1** (compat alias) |

### B. `run` — real non-interactive agent path

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| Positional `run [message..]` | One-shot prompt, print reply, exit | Bootstrap stub | **CLI-P0** (minimal one-shot) → **CLI-P2** (full) |
| Remove silent stub | Never advertise bootstrap-and-exit as `run` | Current behaviour | **CLI-P0** (fail loud *or* one-shot — prefer one-shot ASAP) |
| `--model`/`-m`, `--agent`, `--variant` | `provider/model` + agent + reasoning variant | Split `--provider`/`--model` | **CLI-P0–P1** |
| Session continuity | `--continue`, `--session`, `--fork`, `--title`, `--share` | Missing | **CLI-P1–P2** |
| `--format default\|json` | Formatted vs raw JSON events | Missing | **CLI-P1** |
| `--file`/`-f`, `--thinking`, `--auto`, `--command` | Attach files, show thinking, auto-approve, slash command | Missing | **CLI-P2** |
| `--attach` + basic auth flags | Attach to running `serve`; `--password`/`--username` | Missing | **CLI-P2** (needs #8) |
| `--port` for local server | Local ephemeral server port | Missing | **CLI-P1–P2** |
| Agent loop + tools | Tools, permissions, streaming stdout | Sidecar only stub | **CLI-P2** |

### C. `serve` & `attach`

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| `--hostname` alias | Bind hostname | `--host` only | **CLI-P0** |
| `--cors` (repeatable) | Extra CORS origins | Missing | **CLI-P1** |
| `--mdns`, `--mdns-domain` | Discovery | Missing | **CLI-P2** |
| Basic auth | `OPENCODE_SERVER_PASSWORD` (+ username env) | Missing | **CLI-P1–P2** (**locked #8**) |
| `attach [url]` | TUI against remote serve | Missing | **CLI-P2** |
| `attach` flags | `--dir`, session continue/fork, `--password`/`-p`, `--username`/`-u` | Missing | **CLI-P2** |

### D. Discovery & session management CLIs

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| `models [provider]` | List `provider/model`; `--refresh`, `--verbose` | Missing | **CLI-P1** |
| `session list` | Table/json; `--max-count`/`-n`, `--format` | HTTP only | **CLI-P1** |
| `session delete <id>` | Delete session | HTTP only | **CLI-P1** |
| `export` / `import` | Session JSON / share URL; `--sanitize` | Missing | **CLI-P2** |
| `stats` | Token/cost; `--days`, `--tools`, `--models`, `--project` | Missing | **CLI-P3** |

### E. Auth

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| `auth login` | Interactive / `--provider` / `--method` | Missing | **CLI-P2** |
| `auth list` / `ls` | List authenticated providers | Missing | **CLI-P2** |
| `auth logout` | Clear provider from store | Missing | **CLI-P2** |
| Import OpenCode `auth.json` | Read `~/.local/share/opencode/auth.json` (and platform equivalents) | Missing | **CLI-P2** |
| jerekode-specific store | Write credentials only under jerekode data dir | Missing | **CLI-P2** |

### F. MCP, plugins, agents, db

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| `mcp add` / `list`/`ls` | Config + connection status | HTTP extensions only | **CLI-P2** |
| `mcp auth` / `logout` / `debug` | OAuth MCP flows | Missing | **CLI-P2–P3** |
| `plugin` / `plug <module>` | Install into config; `--global`, `--force` | Orchestrator loads; no CLI | **CLI-P2** |
| `agent create` / `list` | Agent files + permissions; non-interactive flags | Missing | **CLI-P2–P3** |
| `db` / `db path` | DB query tools / print path | Optional `sessionDb` | **CLI-P2** |
| `debug` | Troubleshooting subcommands | Missing | **CLI-P3** |

### G. Environment & config fidelity

| Work item | OpenCode behaviour | Gap today | Phase |
|-----------|-------------------|-----------|-------|
| Core `OPENCODE_*` | Config path, config dir, inline config, permissions, client id, server password/username, models URL, etc. | Partial merge | **CLI-P1+** (document + implement toward mirror) |
| Feature toggles | Disable plugins/LSP download/autocompact/Claude-code/mouse/… | Partial | **CLI-P2–P3** |
| Experimental env | Umbrella + specific experimental flags | Mostly missing | **CLI-P4** / as needed (not all required for 1.0 marketing if documented deltas) |

### H. Packaging (owned by releases roadmap)

| Work item | Decision / gap | Phase |
|-----------|----------------|-------|
| Default download = **full (Bun included)** | **Locked #9** | Rel / CLI-P3 messaging |
| Native-only artifacts | Advanced / future; clear labeling; error if Bun plugin required | Rel-P1+ |
| arm64, brew/winget/Nix, signing | See [roadmap-releases.md](./roadmap-releases.md) | Rel-P1–P4 |

### I. Post-v1 (explicitly out of 1.0 CLI parity)

| Work item | Notes |
|-----------|-------|
| `web` | Serve + browser UI |
| `acp` | Agent Client Protocol stdio |
| `github` / `pr` | GH Actions agent / PR checkout |
| `upgrade` / `uninstall` | Prefer package managers; optional later |

---

## Closed parity checklist vs CLI lag

From [roadmap-parity.md](./roadmap-parity.md) — **runtime slices are Done**; CLI still lags. Do **not** reopen that board; track work with `CLI-P0` … here and the inventory above.

---

## Phased plan

### CLI-P0 — Entry surface (bare TUI, honesty, `run` start)

**Outcome:** Drop-in invoke works; help is honest; `run` begins real parity (one-shot), not a silent stub.

- [ ] Bare invoke → Bun TUI (Decided #2).
- [ ] Enable `-v` / `--version`; clean `version` output.
- [ ] `serve --hostname` alias for `--host`.
- [ ] Help lists **only** implemented commands (Decided #6).
- [ ] Start **real `run`**: positional message + OpenCode `--model` form; remove silent bootstrap-and-exit (Decided #3). Interim loud “not implemented” only if one-shot cannot land in the same slice — never as the end state.
- [ ] Smoke: version flag; hostname; bare invoke / help honesty. Fixtures wait for seam approval (#10).

### CLI-P1 — Compat flags & thin management CLIs

**Outcome:** Automation/discovery CLIs and serve hardening toward attach.

- [ ] `serve --cors`; logging flags ↔ `--log-level` / `RUST_LOG`.
- [ ] Begin **basic auth** on `serve` (Decided #8) — complete before remote attach.
- [ ] `models` (`--refresh`, `--verbose`); `session list` / `delete`.
- [ ] `run` essentials: `--format`, session continue flags, `--dir`.
- [ ] Global `--pure` / document core `OPENCODE_*` in `docs/cli.md` or [distribution.md](./distribution.md).
- [ ] After seam approval: `conformance/fixtures/cli/` argv fixtures.

### CLI-P2 — Behavioral parity (agent loop, attach, auth, ecosystem CLIs)

**Outcome:** Day-to-day OpenCode workflows work as a drop-in.

- [ ] Full `run` agent loop (tools, streaming, `--auto`, `--file`, `--thinking`, `--attach` + auth flags).
- [ ] `attach` against `jerekode serve` (requires basic auth #8).
- [ ] `auth login|list|logout` — import OpenCode credentials; jerekode-specific store (#5).
- [ ] `mcp` add/list/auth; `plugin`/`plug`; `export`/`import`; `db path`; `agent` create/list.
- [ ] `serve --mdns` / `--mdns-domain` as needed for mirror.

### CLI-P3 — Packaging integration + remaining 1.0 depth

**Ownership of packaging checkboxes:** [roadmap-releases.md](./roadmap-releases.md).

- [ ] Lead install/docs with **full (Bun)** default (Decided #9); native-only advanced/future.
- [ ] `stats`, `debug`; remaining env fidelity for advertised drop-in.
- [ ] Prefer CLI-P0+ honesty before “OpenCode-compatible CLI” marketing.

| Releases ID | When relative to CLI |
|-------------|----------------------|
| Rel-P1 arm64 / native-only feature | Parallel; native-only clearly secondary |
| Rel-P2 installers | Align command names / download tables |
| Rel-P3 package managers | After CLI-P0; formulae describe real commands |
| Rel-P4 signing | Independent |

### CLI-P4 — Post-v1

- `web`, `acp`, `github`, `pr` (Decided #7).
- Optional `upgrade`/`uninstall`; experimental env umbrella; Pinokio / Gepeto / Cursor SDK productization.

---

## Releases roadmap — fold-in

| ID | Status | Notes for CLI |
|----|--------|---------------|
| Rel-P0 | **Done** | Historical |
| Rel-P1 | Partial — arm64 & native-only **open** | Native-only = advanced/future (#9) |
| Rel-P2 | Largely **done** | Default artifact messaging = **full** |
| Rel-P3–P4 | Open | After CLI-P0; signing independent |
| Dual-build `bun-sidecar` | Planned | Required before shipping native-only claims |

---

## Conformance / test strategy (CLI)

**Process (Decided #10):** Propose CLI seams for approval **before** the first CLI fixture PR. Until then expand `crates/jerekode-cli/tests/cli_smoke.rs` only.

**Rules:** Independent expected values; no upstream OpenCode source as fixtures; shape fixtures when IDs vary; vertical slices; do not weaken Bun/native hard-gates.

---

## Documentation updates (when implementing)

| Doc | Change |
|-----|--------|
| [CONTEXT.md](../CONTEXT.md) | Active plan → this file + releases |
| [README.md](../README.md) | Commands vs OpenCode; full download default |
| [conformance.md](./conformance.md) | Approved CLI seams only |
| [roadmap-parity.md](./roadmap-parity.md) | Keep closed; link here |
| New `docs/cli.md` | User command reference + auth import/store paths |

---

## Open questions (narrow)

Most compatibility questions are **locked** above. Remaining:

1. **Exact jerekode auth data path** (platform dirs) — pick when implementing `auth` (must not write OpenCode’s `auth.json`).
2. **Conformance seam table** — propose for approval at first CLI fixture PR (#10).
3. **Experimental `OPENCODE_EXPERIMENTAL_*` matrix** — which flags are required for “drop-in” vs documented deltas for 1.0.
4. **`upgrade` / `uninstall`** — stay non-goal for 1.0, or thin wrappers after package-manager story?
5. Releases-only: signing certs, Homebrew tap name, whether to **bundle** Bun inside full installers later (default remains system Bun until revisited).

---

## Execution rules

1. **PR-only** to `main`; Conventional Commits (`feat(cli):`, `test(cli):`, `docs:`).
2. **No upstream OpenCode source** in-repo — public docs + owned fixtures only.
3. Prefer vertical slices that deepen existing HTTP/plugin seams.
4. Packaging stays on [roadmap-releases.md](./roadmap-releases.md); keep both docs aligned on **default = full**.
5. Treat [roadmap-parity.md](./roadmap-parity.md) as closed foundation.
6. Honor [Decided](#decided--locked-compatibility-contract); do not downgrade to subset parity or permanent `run` stub.

---

## References

- Public OpenCode CLI: https://opencode.ai/docs/cli/
- Public OpenCode server: https://opencode.ai/docs/server/
- [releases.md](./releases.md) — auto-release ops  
- [distribution.md](./distribution.md) — aliases and runtime deps  
- [ADR 002](./adr/002-dual-plugin-runtime.md) — Bun / native / WASM  
- [ADR 003](./adr/003-release-packaging-and-changelogs.md) — packaging / dual-build  
