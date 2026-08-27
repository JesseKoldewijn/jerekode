# CLI fixtures

Owned black-box CLI fixtures for Layer 6 smoke tests (`crates/jerekode-cli/tests/cli_smoke.rs`).

Fixtures are an **independent source of truth** — expected stdout shapes, argv, and exit codes — not copied from implementation output at test time.

| File | Seam |
|------|------|
| `version_stdout_shape.json` | `jerekode version` |
| `help_stdout_shape.json` | `jerekode --help` (shipped commands only) |
| `run_one_shot_stub.json` | `jerekode run` one-shot (stub provider) |
| `run_model_slash_form.json` | `run -m provider/model` |
| `run_missing_message.json` | `run` without message → non-zero exit |
| `models_stdout_contains.json` | `jerekode models` line shape |
| `serve_health_body.txt` | `/health` body when auth is off |
| `session_list_json_shape.json` | `session list --format json` |

Approved seams: `docs/conformance.md` Layer 6 / CLI seams table.

Add new fixtures here as CLI subcommands land; wire them from `cli_smoke.rs` via `tests/common/mod.rs`.
