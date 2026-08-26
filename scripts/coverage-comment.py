#!/usr/bin/env python3
"""Render a sticky PR coverage comment from cargo-llvm-cov + diff-cover outputs."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

MARKER = "<!-- jereko-coverage-sticky -->"


def read_text(path: Path | None) -> str:
    if path is None or not path.is_file():
        return ""
    return path.read_text(encoding="utf-8", errors="replace")


def extract_diff_pct(diff_txt: str) -> str:
    m = re.search(r"Coverage:\s*([\d.]+)%", diff_txt)
    if m:
        return m.group(1)
    m = re.search(r"([\d.]+)%\s+coverage", diff_txt, re.I)
    return m.group(1) if m else "?"


def gate_passed(diff_txt: str) -> bool:
    if re.search(r"\bFailure\b|below required|failed", diff_txt, re.I):
        return False
    if re.search(r"\bPassed\b|meets? the required", diff_txt, re.I):
        return True
    if "No lines with coverage information" in diff_txt or not diff_txt.strip():
        return True
    return True


def uncovered_rows(diff_json_raw: str, diff_md: str) -> list[str]:
    rows: list[str] = []
    if diff_json_raw.strip():
        try:
            data = json.loads(diff_json_raw)
        except json.JSONDecodeError:
            data = {}
        src = data.get("src_stats") or data.get("report") or data
        if isinstance(src, dict):
            for path, stats in src.items():
                if not isinstance(stats, dict):
                    continue
                uncovered = (
                    stats.get("violation_lines")
                    or stats.get("uncovered_lines")
                    or []
                )
                pct = stats.get("percent_covered")
                if not uncovered and (pct is None or float(pct) >= 100):
                    continue
                if isinstance(uncovered, list) and uncovered:
                    preview = ", ".join(str(x) for x in uncovered[:12])
                    extra = (
                        ""
                        if len(uncovered) <= 12
                        else f", … (+{len(uncovered) - 12})"
                    )
                    pct_s = f"{pct}" if pct is not None else "—"
                    rows.append(f"| `{path}` | {preview}{extra} | {pct_s}% |")
                else:
                    pct_s = f"{pct}" if pct is not None else "—"
                    rows.append(f"| `{path}` | — | {pct_s}% |")

    if not rows and diff_md:
        for line in diff_md.splitlines():
            if (
                line.startswith("|")
                and "`" in line
                and "File" not in line
                and "---" not in line
            ):
                rows.append(line)
    return rows


def bun_snippet(text: str, limit: int = 40) -> str:
    if not text.strip():
        return "_not collected_"
    lines = [ln for ln in text.splitlines() if ln.strip()]
    chunk = lines[-limit:] if len(lines) > limit else lines
    return "```\n" + "\n".join(chunk) + "\n```"


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", type=Path)
    ap.add_argument("--diff-md", type=Path)
    ap.add_argument("--diff-json", type=Path)
    ap.add_argument("--diff-txt", type=Path)
    ap.add_argument("--compare-branch", default="origin/main")
    ap.add_argument("--fail-under", default="80")
    ap.add_argument("--sha", default="")
    ap.add_argument("--bun-rtk", type=Path)
    ap.add_argument("--bun-sidecar", type=Path)
    args = ap.parse_args()

    summary = read_text(args.summary)
    diff_md = read_text(args.diff_md)
    diff_txt = read_text(args.diff_txt)
    diff_json_raw = read_text(args.diff_json)
    bun_rtk = read_text(args.bun_rtk)
    bun_sidecar = read_text(args.bun_sidecar)

    ok = gate_passed(diff_txt)
    diff_pct = extract_diff_pct(diff_txt)
    status_emoji = "✅" if ok else "❌"
    status_text = "passed" if ok else "failed"
    sha = (args.sha or "")[:12] or "unknown"

    gap_rows = uncovered_rows(diff_json_raw, diff_md)
    if gap_rows:
        gaps_section = "\n".join(
            [
                "| File | Uncovered changed lines | File % |",
                "|------|-------------------------|--------|",
                *gap_rows[:40],
            ]
        )
    else:
        gaps_section = (
            "_No uncovered changed lines reported "
            "(or the diff has no executable lines)._"
        )

    body = f"""{MARKER}
### {status_emoji} Coverage report ({status_text})

Diff coverage vs `{args.compare_branch}`: **{diff_pct}%** (gate: ≥ **{args.fail_under}%** of changed lines).

Commit: `{sha}`

#### Rust (workspace) summary

```
{summary.strip() or "(missing summary)"}
```

#### Diff coverage (changed lines)

```
{diff_txt.strip() or "(missing diff-cover output)"}
```

<details>
<summary>diff-cover markdown</summary>

{diff_md.strip() or "_none_"}

</details>

#### Under-covered parts of this PR

{gaps_section}

#### Bun / TypeScript (informational)

**packages/rtk**

{bun_snippet(bun_rtk)}

**sidecar**

{bun_snippet(bun_sidecar)}

---
_Sticky comment updated on each push. Rust **diff** coverage is the merge gate; Bun coverage is informational._
"""

    Path(args.out).write_text(body, encoding="utf-8")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
