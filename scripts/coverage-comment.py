#!/usr/bin/env python3
"""Render a sticky PR coverage comment from cargo-llvm-cov + diff-cover outputs."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

MARKER = "<!-- jerekode-coverage-sticky -->"

# CSI / OSC-style ANSI sequences (colors from diff-cover --show-uncovered / pygments).
_ANSI_RE = re.compile(
    r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~]|\][^\x07\x1B]*(?:\x07|\x1B\\))"
)


def strip_ansi(text: str) -> str:
    return _ANSI_RE.sub("", text)


def read_text(path: Path | None) -> str:
    if path is None or not path.is_file():
        return ""
    return strip_ansi(path.read_text(encoding="utf-8", errors="replace"))


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
    if "_skipped locally_" in diff_txt:
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


def gate_section(title: str, diff_txt: str, diff_md: str, diff_json_raw: str) -> str:
    ok = gate_passed(diff_txt)
    diff_pct = extract_diff_pct(diff_txt)
    status_emoji = "✅" if ok else "❌"
    status_text = "passed" if ok else "failed"
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

    return f"""#### {title} ({status_emoji} {status_text})

Diff coverage: **{diff_pct}%**

```
{diff_txt.strip() or "(missing diff-cover output)"}
```

<details>
<summary>diff-cover markdown</summary>

{diff_md.strip() or "_none_"}

</details>

**Under-covered parts of this PR**

{gaps_section}
"""


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", required=True)
    ap.add_argument("--summary", type=Path)
    ap.add_argument("--diff-md", type=Path)
    ap.add_argument("--diff-json", type=Path)
    ap.add_argument("--diff-txt", type=Path)
    ap.add_argument("--bun-diff-md", type=Path)
    ap.add_argument("--bun-diff-json", type=Path)
    ap.add_argument("--bun-diff-txt", type=Path)
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
    bun_diff_md = read_text(args.bun_diff_md)
    bun_diff_txt = read_text(args.bun_diff_txt)
    bun_diff_json_raw = read_text(args.bun_diff_json)
    bun_rtk = read_text(args.bun_rtk)
    bun_sidecar = read_text(args.bun_sidecar)

    rust_ok = gate_passed(diff_txt)
    bun_ok = gate_passed(bun_diff_txt)
    overall_ok = rust_ok and bun_ok
    status_emoji = "✅" if overall_ok else "❌"
    status_text = "passed" if overall_ok else "failed"
    sha = (args.sha or "")[:12] or "unknown"

    rust_section = gate_section("Rust diff coverage", diff_txt, diff_md, diff_json_raw)
    bun_section = gate_section(
        "Bun / TypeScript diff coverage", bun_diff_txt, bun_diff_md, bun_diff_json_raw
    )

    body = f"""{MARKER}
### {status_emoji} Coverage report ({status_text})

Gate: ≥ **{args.fail_under}%** diff coverage vs `{args.compare_branch}` on changed lines (Rust **and** Bun/TS).

Commit: `{sha}`

{rust_section}

#### Rust (workspace) summary

```
{summary.strip() or "(missing summary)"}
```

{bun_section}

<details>
<summary>Bun package coverage tables</summary>

**packages/rtk**

{bun_snippet(bun_rtk)}

**sidecar**

{bun_snippet(bun_sidecar)}

</details>

---
_Sticky comment updated on each push. Rust and Bun/TS **diff** coverage both block merge when below the gate._
"""

    Path(args.out).write_text(strip_ansi(body), encoding="utf-8", newline="\n")
    print(f"wrote {args.out}")


if __name__ == "__main__":
    main()
