#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "contract lane: canonical wire partition and generated drift"
run_partition contract contract "$WIRE_FILTER" "$WIRE_EXPECTED_TESTS"
cargo run --locked --quiet -p bullet-wire --bin bullet-contract -- check --root "$REPO_ROOT"
prepare_ci_directory "$REPO_ROOT" .ci-artifacts/contracts \
  || { refuse CONTRACT_ARTIFACT_ROOT_INVALID .ci-artifacts/contracts; exit 1; }
cp -- contracts/v1alpha1/bundle-manifest.json .ci-artifacts/contracts/bundle-manifest.json
cmp contracts/v1alpha1/bundle-manifest.json .ci-artifacts/contracts/bundle-manifest.json \
  || { refuse CONTRACT_MANIFEST_COPY_INVALID bundle-manifest.json; exit 1; }
bash ops/ci/strict-json.sh .ci-artifacts/contracts/bundle-manifest.json >/dev/null \
  || { refuse CONTRACT_MANIFEST_JSON_INVALID bundle-manifest.json; exit 1; }
jq -e '
  (keys | sort) == (["authority_golden_hash","bundle_hash","catalog_hash",
    "generated_client_hash","generated_clients","generator","invariant_registry_hash",
    "launch_grant_golden_hash","policy_snapshot_hash","record_count","schema_version"] | sort) and
  .schema_version == "v1alpha1" and
  .generator == "bullet-wire-contract-tool-v1alpha1" and
  (.record_count | type == "number" and . == floor and . > 0) and
  (.generated_clients | keys | sort) == ["rust","typescript"] and
  ([.authority_golden_hash,.bundle_hash,.catalog_hash,.generated_client_hash,
    .invariant_registry_hash,.launch_grant_golden_hash,.policy_snapshot_hash,
    .generated_clients.rust,.generated_clients.typescript] |
    all(type == "string" and test("^[0-9a-f]{64}$")))
' .ci-artifacts/contracts/bundle-manifest.json >/dev/null \
  || { refuse CONTRACT_MANIFEST_INVALID bundle-manifest.json; exit 1; }
log "contract lane: exactly two pinned bounded formal models"
mkdir -p .ci-artifacts/formal
formal_raw="$(mktemp)"
cleanup() { rm -f -- "$formal_raw"; }
trap cleanup EXIT
formal_status=0
set +e
bash formal/model-check.sh 2>&1 | tee "$formal_raw"
formal_status=${PIPESTATUS[0]}
set -e
completed_models="$(grep -Fc 'Model checking completed. No error has been found.' "$formal_raw" || true)"
final_summary="$(grep -E -c 'formal-check: 2/2 models match .*' "$formal_raw" || true)"
formal_complete=false
formal_exit="$formal_status"
formal_result=FAIL
if (( formal_status == 0 && completed_models == 2 && final_summary == 1 )); then
  formal_complete=true
  formal_exit=0
  formal_result=PASS
elif (( formal_exit == 0 )); then
  formal_exit=1
fi
{
  printf 'schema=bullet.formal-log.v1\n'
  printf 'models=2\n'
  printf 'completed_without_error=%s\n' "$completed_models"
  printf 'pinned_summary_present=%s\n' "$final_summary"
  printf 'exit_code=%s\n' "$formal_exit"
  printf 'classification=DIAGNOSTIC_ONLY\n'
} >.ci-artifacts/formal/contract.log
jq -n --arg status "$formal_result" \
  --argjson exit_code "$formal_exit" \
  --argjson completed_models "$completed_models" \
  --argjson final_summary "$final_summary" \
  '{schema_version:"bullet.formal-summary.v1",models:2,completed_models:$completed_models,
    pinned_summary_present:($final_summary == 1),status:$status,exit_code:$exit_code,
    signed:false,evidence_class:"DIAGNOSTIC_ONLY"}' \
  >.ci-artifacts/formal/contract.json
if [[ "$formal_complete" != true ]]; then
  refuse FORMAL_PROOF_INCOMPLETE "exit=$formal_status completed=$completed_models summary=$final_summary"
  exit 1
fi
bash formal/model-check-concurrency-test.sh
log "contract lane: ordered family proof custody"
bash ops/ci/family-custody-test.sh
log "contract lane passed"
