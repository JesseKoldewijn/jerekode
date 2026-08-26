#!/usr/bin/env bash
# Generate Rust coverage (lcov + summary) and informational Bun coverage.
# Used by .github/workflows/coverage.yml and for local preview.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT_DIR="${COVERAGE_OUT_DIR:-target/coverage}"
mkdir -p "$OUT_DIR"

COMPARE_BRANCH="${COMPARE_BRANCH:-origin/main}"
DIFF_FAIL_UNDER="${DIFF_COVERAGE_FAIL_UNDER:-80}"

echo "==> cargo llvm-cov (workspace)"
cargo llvm-cov --workspace --locked \
  --lcov --output-path "$OUT_DIR/lcov.info"

echo "==> Rust summary"
cargo llvm-cov report --summary-only | tee "$OUT_DIR/summary.txt"
cargo llvm-cov report --json --output-path "$OUT_DIR/coverage.json" || true

# Bun coverage is informational (not part of the merge gate).
if command -v bun >/dev/null 2>&1; then
  echo "==> bun test --coverage (packages/rtk + sidecar)"
  (
    cd packages/rtk
    bun test --coverage 2>&1 | tee "$OUT_DIR/bun-rtk-coverage.txt" || true
  )
  (
    cd sidecar
    bun test --coverage 2>&1 | tee "$OUT_DIR/bun-sidecar-coverage.txt" || true
  )
else
  echo "bun not on PATH; skipping TS coverage"
  : > "$OUT_DIR/bun-rtk-coverage.txt"
  : > "$OUT_DIR/bun-sidecar-coverage.txt"
fi

echo "==> Ensure compare branch is available (${COMPARE_BRANCH})"
if ! git rev-parse --verify "${COMPARE_BRANCH}" >/dev/null 2>&1; then
  git fetch --no-tags origin "${COMPARE_BRANCH#origin/}:${COMPARE_BRANCH}" 2>/dev/null \
    || git fetch --no-tags origin "${COMPARE_BRANCH#origin/}"
fi

echo "==> Install/use diff-cover (venv)"
VENV_DIR="${OUT_DIR}/.venv-diff-cover"
if [[ ! -x "${VENV_DIR}/bin/diff-cover" ]]; then
  python3 -m venv "${VENV_DIR}"
  "${VENV_DIR}/bin/pip" install -q 'diff-cover>=9.2'
fi
DIFF_COVER="${VENV_DIR}/bin/diff-cover"

echo "==> diff-cover vs ${COMPARE_BRANCH} (fail-under=${DIFF_FAIL_UNDER}%)"
set +e
"${DIFF_COVER}" "$OUT_DIR/lcov.info" \
  --compare-branch="${COMPARE_BRANCH}" \
  --fail-under="${DIFF_FAIL_UNDER}" \
  --show-uncovered \
  --format "markdown:${OUT_DIR}/diff-cover.md,json:${OUT_DIR}/diff-cover.json" \
  2>&1 | tee "$OUT_DIR/diff-cover.txt"
DIFF_STATUS=${PIPESTATUS[0]}
set -e

[[ -s "$OUT_DIR/diff-cover.json" ]] || echo '{}' > "$OUT_DIR/diff-cover.json"
[[ -s "$OUT_DIR/diff-cover.md" ]] || echo "_diff-cover markdown unavailable_" > "$OUT_DIR/diff-cover.md"

python3 "$ROOT/scripts/coverage-comment.py" \
  --out "$OUT_DIR/comment.md" \
  --summary "$OUT_DIR/summary.txt" \
  --diff-md "$OUT_DIR/diff-cover.md" \
  --diff-json "$OUT_DIR/diff-cover.json" \
  --diff-txt "$OUT_DIR/diff-cover.txt" \
  --compare-branch "${COMPARE_BRANCH}" \
  --fail-under "${DIFF_FAIL_UNDER}" \
  --sha "${GITHUB_SHA:-$(git rev-parse HEAD)}" \
  --bun-rtk "$OUT_DIR/bun-rtk-coverage.txt" \
  --bun-sidecar "$OUT_DIR/bun-sidecar-coverage.txt"

exit "$DIFF_STATUS"
