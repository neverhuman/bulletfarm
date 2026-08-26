#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "lint lane: Rust, workflow, shell, and CI meta-controls"
for tool in actionlint shellcheck rg; do require_tool "$tool" || exit 1; done
require_exact_output "1.7.8" actionlint -version
shellcheck_version="$(shellcheck --version | awk '$1 == "version:" { print $2 }')"
[[ "$shellcheck_version" == "0.10.0" ]] \
  || { refuse TOOL_VERSION_MISMATCH "expected ShellCheck 0.10.0, found $shellcheck_version"; exit 1; }
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings -A clippy::disallowed_methods
cargo clippy --locked -p bullet-family --lib --bins --no-deps -- \
  -D warnings -F clippy::disallowed_methods
cargo clippy --locked -p bullet-wire --lib --bins --no-deps -- \
  -D warnings -D clippy::disallowed_methods
if cargo tree --locked --offline -e features -i serde_json \
  | grep -Eq 'serde_json feature "(raw_value|arbitrary_precision)"'; then
  refuse SERDE_JSON_UNBOUNDED_FEATURE_ENABLED "serde_json raw_value/arbitrary_precision bypasses the bounded document decoder"
  exit 1
fi
bash ops/ci/disallowed-methods-test.sh
mapfile -t workflow_files < <(
  find .github/workflows -maxdepth 1 -type f \( -name '*.yml' -o -name '*.yaml' \) | LC_ALL=C sort
)
[[ "${#workflow_files[@]}" -gt 0 ]] || { refuse WORKFLOW_INVENTORY_EMPTY "no workflows"; exit 1; }
actionlint "${workflow_files[@]}"
mapfile -t shell_files < <(
  {
    rg --files -g '*.sh'
    rg -l --hidden -g '!target/**' -g '!.git/**' '^#!.*(bash|/sh)' .
  } | sed 's#^\./##' | LC_ALL=C sort -u
)
[[ "${#shell_files[@]}" -gt 0 ]] || { refuse SHELL_INVENTORY_EMPTY "no shell files"; exit 1; }
shellcheck -x -P ops/ci "${shell_files[@]}"
bash ops/ci/test-partitions.sh
bash ops/ci/junit-test.sh
bash ops/ci/aggregate-test.sh
bash ops/ci/observation-test.sh
bash ops/ci/coverage-sanitize-test.sh
bash ops/ci/family-report-check-test.sh
bash ops/ci/family-observation-test.sh
bash ops/ci/check-links-test.sh
bash ops/ci/checkout-subject-test.sh
bash ops/ci/doctor-test.sh
bash ops/ci/workflow-policy.sh
bash ops/ci/dev-supervision-test.sh
log "lint lane passed"
