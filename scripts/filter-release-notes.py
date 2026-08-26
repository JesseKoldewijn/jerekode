#!/usr/bin/env python3
"""Post-filter GitHub-generated release notes for jereko publishes."""
from __future__ import annotations

import re
import sys


def filter_notes(text: str) -> str:
    # Drop New Contributors (bogus on short/noisy history).
    text = re.split(r"(?im)^##\s+New Contributors\s*$", text, maxsplit=1)[0]

    kept: list[str] = []
    for line in text.splitlines():
        lower = line.lower()
        if "[skip release]" in lower:
            continue
        if "github-actions[bot]" in lower:
            continue
        kept.append(line)

    # Collapse excess blank lines after filtering.
    out: list[str] = []
    blank = False
    for line in kept:
        if not line.strip():
            if blank:
                continue
            blank = True
            out.append("")
        else:
            blank = False
            out.append(line)
    return "\n".join(out).strip()


def main() -> None:
    filtered = filter_notes(sys.stdin.read())
    if filtered:
        sys.stdout.write(filtered + "\n")


if __name__ == "__main__":
    main()
