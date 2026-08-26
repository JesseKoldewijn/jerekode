#!/usr/bin/env bash
# Bump workspace package version for CI releases (PromptComposer-style).
# Usage: ./scripts/set-version.sh 0.1.42
set -euo pipefail

VERSION="${1:-}"
if [[ -z "${VERSION}" || ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: $0 <semver>" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${ROOT}/Cargo.toml"

if ! grep -q '^\[workspace\.package\]' "${CARGO_TOML}"; then
  echo "missing [workspace.package] in Cargo.toml" >&2
  exit 1
fi

python3 - "${CARGO_TOML}" "${VERSION}" <<'PY'
import re
import sys

path, version = sys.argv[1], sys.argv[2]
text = open(path, encoding="utf-8").read()
pattern = re.compile(
    r"(?ms)(^\[workspace\.package\]\n)(.*?)(?=^\[|\Z)"
)
match = pattern.search(text)
if not match:
    raise SystemExit("could not find [workspace.package] section")

section = match.group(2)
new_section, n = re.subn(
    r'(?m)^version\s*=\s*"[^"]*"',
    f'version = "{version}"',
    section,
    count=1,
)
if n != 1:
    raise SystemExit("version field not updated in [workspace.package]")

updated = text[: match.start(2)] + new_section + text[match.end(2) :]
open(path, "w", encoding="utf-8").write(updated)
print(f"updated Cargo.toml workspace.package.version → {version}")
PY

# Refresh lockfile metadata for workspace members if present.
if [[ -f "${ROOT}/Cargo.lock" ]]; then
  # Best-effort: cargo will rewrite lock entries on the next build.
  # Update known package name stanzas that mirror workspace.version.
  python3 - "${ROOT}/Cargo.lock" "${VERSION}" <<'PY'
import re
import sys

path, version = sys.argv[1], sys.argv[2]
text = open(path, encoding="utf-8").read()
# Only bump packages that are clearly our workspace crates (name starts with jereko).
pattern = re.compile(
    r'(?ms)(\[\[package\]\]\nname = "jereko(?:-[^"]+)?"\n)version = "[^"]+"'
)
updated, n = pattern.subn(rf'\1version = "{version}"', text)
if n:
    open(path, "w", encoding="utf-8").write(updated)
    print(f"updated {n} jereko* entries in Cargo.lock → {version}")
else:
    print("no jereko* package versions updated in Cargo.lock (ok if unresolved)")
PY
fi
