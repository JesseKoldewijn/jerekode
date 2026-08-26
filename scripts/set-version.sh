#!/usr/bin/env bash
# Read or bump workspace package version for CI releases (PromptComposer-style).
# Usage:
#   ./scripts/set-version.sh --print     # print workspace.package.version
#   ./scripts/set-version.sh 0.0.42     # set version (LF-normalized write)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_TOML="${ROOT}/Cargo.toml"

read_version() {
  python3 - "${CARGO_TOML}" <<'PY'
import re
import sys

path = sys.argv[1]
# newline=None: universal newlines so CRLF files still parse.
text = open(path, encoding="utf-8", newline=None).read()
# Normalize to LF in-memory for section matching.
text = text.replace("\r\n", "\n").replace("\r", "\n")
match = re.search(
    r"(?ms)^\[workspace\.package\]\n(.*?)(?=^\[|\Z)",
    text,
)
if not match:
    raise SystemExit("could not find [workspace.package] section")
m = re.search(r'(?m)^version\s*=\s*"([^"]*)"', match.group(1))
if not m:
    raise SystemExit("version field missing in [workspace.package]")
print(m.group(1))
PY
}

if [[ "${1:-}" == "--print" ]]; then
  read_version
  exit 0
fi

VERSION="${1:-}"
if [[ -z "${VERSION}" || ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "usage: $0 <--print|semver>" >&2
  exit 1
fi

if ! grep -qE '^\[workspace\.package\]' "${CARGO_TOML}"; then
  echo "missing [workspace.package] in Cargo.toml" >&2
  exit 1
fi

python3 - "${CARGO_TOML}" "${VERSION}" <<'PY'
import re
import sys

path, version = sys.argv[1], sys.argv[2]
raw = open(path, "rb").read()
# Preserve intentional content; normalize line endings to LF on write.
text = raw.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")
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
open(path, "wb").write(updated.encode("utf-8"))
print(f"updated Cargo.toml workspace.package.version → {version}")
PY

# Refresh lockfile metadata for workspace members if present.
if [[ -f "${ROOT}/Cargo.lock" ]]; then
  python3 - "${ROOT}/Cargo.lock" "${VERSION}" <<'PY'
import re
import sys

path, version = sys.argv[1], sys.argv[2]
raw = open(path, "rb").read()
text = raw.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")
pattern = re.compile(
    r'(?ms)(\[\[package\]\]\nname = "jereko(?:-[^"]+)?"\n)version = "[^"]+"'
)
updated, n = pattern.subn(rf'\1version = "{version}"', text)
if n:
    open(path, "wb").write(updated.encode("utf-8"))
    print(f"updated {n} jereko* entries in Cargo.lock → {version}")
else:
    print("no jereko* package versions updated in Cargo.lock (ok if unresolved)")
PY
fi
