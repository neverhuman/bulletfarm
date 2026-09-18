#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
prepare_ci_directory "$REPO_ROOT" .ci-artifacts/coverage \
  || { refuse CI_ARTIFACT_ROOT_INVALID .ci-artifacts/coverage; exit 1; }
rm -f .ci-artifacts/coverage/cobertura.xml
raw_report="$(mktemp)"
normalized_report="$(mktemp)"
rm -f -- "$normalized_report"
cleanup() { rm -f -- "$raw_report" "$normalized_report"; }
trap cleanup EXIT
bash ops/ci/test-partitions.sh
cargo llvm-cov clean --workspace
cargo llvm-cov nextest --locked --workspace --no-report --remap-path-prefix
cargo llvm-cov report --locked --remap-path-prefix --cobertura --output-path "$raw_report"
bash ops/ci/coverage-sanitize.sh normalize "$raw_report" "$normalized_report"
bash ops/ci/coverage-sanitize.sh check "$normalized_report"
mv "$normalized_report" .ci-artifacts/coverage/cobertura.xml
bash ops/ci/assert-coverage.sh
log "scheduled coverage report passed"
