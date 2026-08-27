#!/usr/bin/env bash
# Generate Rust + Bun/TS coverage and enforce diff gates (default 80% changed lines).
# Used by .github/workflows/coverage.yml and for local preview.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT_DIR="${COVERAGE_OUT_DIR:-target/coverage}"
mkdir -p "$OUT_DIR"
OUT_DIR="$(cd "$OUT_DIR" && pwd)"

COMPARE_BRANCH="${COMPARE_BRANCH:-origin/main}"
DIFF_FAIL_UNDER="${DIFF_COVERAGE_FAIL_UNDER:-80}"

strip_ansi_file() {
  # Remove CSI/OSC escapes so sticky PR comments stay UTF-8 plain text.
  python3 - "$1" <<'PY'
import re
import sys
from pathlib import Path

path = Path(sys.argv[1])
if not path.is_file():
    raise SystemExit(0)
ansi = re.compile(
    r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~]|\][^\x07\x1B]*(?:\x07|\x1B\\))"
)
text = path.read_text(encoding="utf-8", errors="replace")
path.write_text(ansi.sub("", text), encoding="utf-8", newline="\n")
PY
}

run_diff_cover() {
  local label="$1"
  local lcov_path="$2"
  local txt_out="$3"
  local md_out="$4"
  local json_out="$5"

  echo "==> diff-cover ${label} vs ${COMPARE_BRANCH} (fail-under=${DIFF_FAIL_UNDER}%)"
  set +e
  # Disable pygments/terminal colors so tee'd text and PR comments stay clean.
  env NO_COLOR=1 TERM=dumb PYGMENTIZE_STYLE=none "${DIFF_COVER}" "${lcov_path}" \
    --diff-file="$OUT_DIR/pr.diff" \
    --fail-under="${DIFF_FAIL_UNDER}" \
    --ignore-staged \
    --ignore-unstaged \
    --show-uncovered \
    --format "markdown:${md_out},json:${json_out}" \
    2>&1 | tee "${txt_out}"
  local status=${PIPESTATUS[0]}
  set -e

  strip_ansi_file "${txt_out}"
  strip_ansi_file "${md_out}"

  [[ -s "${json_out}" ]] || echo '{}' > "${json_out}"
  [[ -s "${md_out}" ]] || echo "_diff-cover markdown unavailable_" > "${md_out}"

  return "${status}"
}

echo "==> cargo llvm-cov (workspace)"
# Ensure the native test dylib exists where host tests look (incl. llvm-cov target dir).
cargo build -p jerekode-test-native-plugin --locked
cargo llvm-cov --workspace --locked \
  --lcov --output-path "$OUT_DIR/lcov.info"

echo "==> Rust summary"
cargo llvm-cov report --summary-only | tee "$OUT_DIR/summary.txt"
cargo llvm-cov report --json --output-path "$OUT_DIR/coverage.json" || true

echo "==> Ensure compare branch is available (${COMPARE_BRANCH})"
if ! git rev-parse --verify "${COMPARE_BRANCH}" >/dev/null 2>&1; then
  git fetch --no-tags origin "${COMPARE_BRANCH#origin/}:${COMPARE_BRANCH}" 2>/dev/null \
    || git fetch --no-tags origin "${COMPARE_BRANCH#origin/}"
fi

echo "==> Write clean PR diff (ignore working-tree / CRLF noise)"
# diff-cover's default includes staged/unstaged working-tree changes, which on
# some runners appears as huge CRLF-only "diffs" and false gate failures.
git diff "${COMPARE_BRANCH}...HEAD" > "$OUT_DIR/pr.diff"

echo "==> Install/use diff-cover (venv)"
VENV_DIR="${OUT_DIR}/.venv-diff-cover"
if [[ ! -x "${VENV_DIR}/bin/diff-cover" ]]; then
  python3 -m venv "${VENV_DIR}"
  "${VENV_DIR}/bin/pip" install -q 'diff-cover>=9.2'
fi
DIFF_COVER="${VENV_DIR}/bin/diff-cover"

BUN_DIFF_STATUS=0
if command -v bun >/dev/null 2>&1; then
  echo "==> bun test --coverage (packages/rtk + sidecar)"
  mkdir -p "$OUT_DIR/bun/rtk" "$OUT_DIR/bun/sidecar"
  (
    cd packages/rtk
    bun test --coverage \
      --coverage-reporter=text \
      --coverage-reporter=lcov \
      --coverage-dir="$OUT_DIR/bun/rtk" \
      2>&1 | tee "$OUT_DIR/bun-rtk-coverage.txt"
  )
  (
    cd sidecar
    bun test --coverage \
      --coverage-reporter=text \
      --coverage-reporter=lcov \
      --coverage-dir="$OUT_DIR/bun/sidecar" \
      2>&1 | tee "$OUT_DIR/bun-sidecar-coverage.txt"
  )

  if [[ -f "$OUT_DIR/bun/rtk/lcov.info" && -f "$OUT_DIR/bun/sidecar/lcov.info" ]]; then
    cat "$OUT_DIR/bun/rtk/lcov.info" "$OUT_DIR/bun/sidecar/lcov.info" > "$OUT_DIR/bun-lcov.info"
  elif [[ -f "$OUT_DIR/bun/rtk/lcov.info" ]]; then
    cp "$OUT_DIR/bun/rtk/lcov.info" "$OUT_DIR/bun-lcov.info"
  elif [[ -f "$OUT_DIR/bun/sidecar/lcov.info" ]]; then
    cp "$OUT_DIR/bun/sidecar/lcov.info" "$OUT_DIR/bun-lcov.info"
  else
    echo "Bun coverage: missing lcov output"
    exit 1
  fi

  run_diff_cover "(Bun/TS)" "$OUT_DIR/bun-lcov.info" \
    "$OUT_DIR/bun-diff-cover.txt" \
    "$OUT_DIR/bun-diff-cover.md" \
    "$OUT_DIR/bun-diff-cover.json" || BUN_DIFF_STATUS=$?
else
  if [[ "${CI:-}" == "true" ]]; then
    echo "bun is required in CI for the TypeScript coverage gate"
    exit 1
  fi
  echo "bun not on PATH; skipping TS coverage gate (local only)"
  : > "$OUT_DIR/bun-rtk-coverage.txt"
  : > "$OUT_DIR/bun-sidecar-coverage.txt"
  : > "$OUT_DIR/bun-lcov.info"
  echo "No Bun/TS diff (bun not installed locally)" > "$OUT_DIR/bun-diff-cover.txt"
  echo "_skipped locally_" > "$OUT_DIR/bun-diff-cover.md"
  echo '{}' > "$OUT_DIR/bun-diff-cover.json"
fi

RUST_DIFF_STATUS=0
run_diff_cover "(Rust)" "$OUT_DIR/lcov.info" \
  "$OUT_DIR/diff-cover.txt" \
  "$OUT_DIR/diff-cover.md" \
  "$OUT_DIR/diff-cover.json" || RUST_DIFF_STATUS=$?

python3 "$ROOT/scripts/coverage-comment.py" \
  --out "$OUT_DIR/comment.md" \
  --summary "$OUT_DIR/summary.txt" \
  --diff-md "$OUT_DIR/diff-cover.md" \
  --diff-json "$OUT_DIR/diff-cover.json" \
  --diff-txt "$OUT_DIR/diff-cover.txt" \
  --bun-diff-md "$OUT_DIR/bun-diff-cover.md" \
  --bun-diff-json "$OUT_DIR/bun-diff-cover.json" \
  --bun-diff-txt "$OUT_DIR/bun-diff-cover.txt" \
  --compare-branch "${COMPARE_BRANCH}" \
  --fail-under "${DIFF_FAIL_UNDER}" \
  --sha "${GITHUB_SHA:-$(git rev-parse HEAD)}" \
  --bun-rtk "$OUT_DIR/bun-rtk-coverage.txt" \
  --bun-sidecar "$OUT_DIR/bun-sidecar-coverage.txt"

if [[ "${RUST_DIFF_STATUS}" -ne 0 || "${BUN_DIFF_STATUS}" -ne 0 ]]; then
  if [[ "${RUST_DIFF_STATUS}" -ne 0 ]]; then
    echo "Rust diff coverage gate failed (threshold ${DIFF_FAIL_UNDER}% of changed lines)."
  fi
  if [[ "${BUN_DIFF_STATUS}" -ne 0 ]]; then
    echo "Bun/TS diff coverage gate failed (threshold ${DIFF_FAIL_UNDER}% of changed lines)."
  fi
  exit 1
fi

exit 0
