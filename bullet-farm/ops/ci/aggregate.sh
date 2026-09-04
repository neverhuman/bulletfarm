#!/usr/bin/env bash
# Converge six hosted predecessor results against lane-isolated, hashed,
# clean-subject observations downloaded from this exact workflow attempt.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
lanes=(source-scan fast lint contract security docs)
[[ "$#" -eq 10 ]] \
  || { refuse CI_JOB_MISSING "expected root, commit, run ID, attempt, and six results"; exit 1; }
artifact_root="$1"
expected_commit="$2"
run_id="$3"
run_attempt="$4"
shift 4
[[ -d "$artifact_root" && ! -L "$artifact_root" ]] \
  || { refuse CI_ARTIFACT_ROOT_MISSING "$artifact_root"; exit 1; }
[[ "$expected_commit" =~ ^[0-9a-f]{40}$ ]] \
  || { refuse CI_COMMIT_INVALID "$expected_commit"; exit 1; }
[[ "$run_id" =~ ^[1-9][0-9]*$ && "$run_attempt" =~ ^[1-9][0-9]*$ ]] \
  || { refuse CI_RUN_ID_INVALID "$run_id/$run_attempt"; exit 1; }

expected_origins="$({
  for lane in "${lanes[@]}"; do
    printf 'hub-%s-%s-%s\n' "$lane" "$run_id" "$run_attempt"
  done
} | LC_ALL=C sort)"
actual_origins="$(find "$artifact_root" -mindepth 1 -maxdepth 1 -printf '%f\n' | LC_ALL=C sort)"
[[ "$actual_origins" == "$expected_origins" ]] \
  || { refuse CI_ARTIFACT_ORIGIN_INVENTORY_INVALID "$artifact_root"; exit 1; }

for index in "${!lanes[@]}"; do
  lane="${lanes[$index]}"
  result="$1"
  shift
  [[ "$result" == success ]] \
    || { refuse CI_JOB_NOT_SUCCESSFUL "$lane=${result:-missing}"; exit 1; }
  origin="$artifact_root/hub-$lane-$run_id-$run_attempt"
  [[ -d "$origin" && ! -L "$origin" ]] \
    || { refuse CI_ARTIFACT_ORIGIN_INVALID "$origin"; exit 1; }
  [[ "$(find "$origin" -mindepth 1 -maxdepth 1 -printf '%f\n')" == .ci-artifacts \
    && -d "$origin/.ci-artifacts" && ! -L "$origin/.ci-artifacts" ]] \
    || { refuse CI_ARTIFACT_ORIGIN_INVALID "$origin"; exit 1; }
  bash "$REPO_ROOT/ops/ci/artifact-check.sh" "$lane" "$expected_commit" "$origin" atomic >/dev/null
done
log "CI / required: six jobs, isolated clean observations, and semantic artifact checks passed"
