#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
fixtures="$(mktemp -d)"
cleanup() { rm -rf -- "$fixtures"; }
trap cleanup EXIT

expect_refusal() {
  local code="$1" output status
  shift
  set +e
  output="$(bash ops/ci/family-report-check.sh "$@" 2>&1)"
  status=$?
  set -e
  [[ "$status" -ne 0 && "$output" == *"$code"* ]] \
    || { refuse FAMILY_REPORT_NEGATIVE_FAILED "$code status=$status output=$output"; exit 1; }
}

printf '%s\n' \
  '<testsuites tests="4" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
bash ops/ci/family-report-check.sh junit "$fixtures/junit.xml" 4 0 >/dev/null
printf '%s\n' \
  '<testsuites tests="0" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="0" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="4" failures="1" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="1" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="4" failures="0" errors="0">' \
  '  <testsuite tests="4" failures="0" errors="0" disabled="1"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
bash ops/ci/family-report-check.sh junit "$fixtures/junit.xml" 4 1 >/dev/null
expect_refusal FAMILY_REPORT_OUTCOME_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' '<testsuites tests="4" failures="0" errors="0" skipped="0">' \
  'malformed' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="4" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="1" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' '<!DOCTYPE testsuites [<!ENTITY green "4">]>' \
  '<testsuites tests="4" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
python_bin="$(command -v python3 || command -v python)"
"$python_bin" -I -S - "$fixtures/junit.xml" <<'PY'
from pathlib import Path
import sys

document = '''<?xml version="1.0" encoding="UTF-16"?>
<!DOCTYPE testsuites [<!ENTITY green "0">]>
<testsuites tests="4" failures="&green;" errors="0" skipped="0">
  <testsuite tests="4" failures="&green;" errors="0" skipped="0"/>
</testsuites>'''
Path(sys.argv[1]).write_bytes(document.encode("utf-16"))
PY
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="4" tests="4" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="4" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' '<testsuites tests="4" failures="0" errors="0" skipped="0"/>' \
  >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '%s\n' \
  '<testsuites tests="5" failures="0" errors="0" skipped="0">' \
  '  <testsuite tests="4" failures="0" errors="0" skipped="0"/>' \
  '</testsuites>' >"$fixtures/junit.xml"
expect_refusal FAMILY_REPORT_XML_INVALID junit "$fixtures/junit.xml" 4 0
printf '{"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
bash ops/ci/family-report-check.sh vitest "$fixtures/vitest.json" 3 >/dev/null
printf '{"success":false,"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_JSON_INVALID vitest "$fixtures/vitest.json" 3
printf '{"success":true,"success":false,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_JSON_INVALID vitest "$fixtures/vitest.json" 3
for hostile_number in NaN Infinity -Infinity 1e9999; do
  printf '{"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0,"startTime":%s}\n' \
    "$hostile_number" >"$fixtures/vitest.json"
  expect_refusal FAMILY_REPORT_JSON_INVALID vitest "$fixtures/vitest.json" 3
done
printf '%s\n' \
  '{"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}' \
  '{"success":true}' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_JSON_INVALID vitest "$fixtures/vitest.json" 3
printf '{"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID vitest "$fixtures/vitest.json" 4
expect_refusal FAMILY_REPORT_OUTCOME_INVALID vitest "$fixtures/vitest.json" 2
printf '{"success":true,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID vitest "$fixtures/vitest.json" 3
printf '{"schema_version":"bullet.formal-summary.v1","models":2,"completed_models":2,"pinned_summary_present":true,"status":"FAIL","status":"PASS","exit_code":0,"signed":false,"evidence_class":"DIAGNOSTIC_ONLY"}\n' >"$fixtures/formal.json"
expect_refusal FAMILY_REPORT_JSON_INVALID formal-json "$fixtures/formal.json"
printf '{"schema_version":"bullet.formal-summary.v1","models":2,"completed_models":2,"pinned_summary_present":true,"status":"PASS","status":"FAIL","exit_code":0,"signed":false,"evidence_class":"DIAGNOSTIC_ONLY"}\n' >"$fixtures/formal.json"
expect_refusal FAMILY_REPORT_JSON_INVALID formal-json "$fixtures/formal.json"
printf '{"schema_version":"bullet.formal-summary.v1","models":2,"completed_models":2,"pinned_summary_present":true,"status":"PASS","exit_code":0,"signed":false,"evidence_class":"DIAGNOSTIC_ONLY"}\n' >"$fixtures/formal.json"
bash ops/ci/family-report-check.sh formal-json "$fixtures/formal.json" >/dev/null
printf '{"success":true,"numTotalTests":"3","numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID vitest "$fixtures/vitest.json" 3
printf '{"success":true,"numTotalTests":0,"numPassedTests":0,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_OUTCOME_INVALID vitest "$fixtures/vitest.json" 3
report_root="$(printf '\044reports')"
printf 'node ops/ci/assert-report.mjs vitest "%s/vitest.json" 120\n' "$report_root" >"$fixtures/fast.sh"
printf 'node ops/ci/assert-report.mjs vitest "%s/coverage-tests.json" 120\n' "$report_root" >"$fixtures/coverage.sh"
bash ops/ci/family-report-check.sh vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh" >/dev/null
printf 'node ops/ci/assert-report.mjs vitest "%s/coverage-tests.json" 121\n' "$report_root" >"$fixtures/coverage.sh"
expect_refusal FAMILY_VITEST_SOURCE_DRIFT vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh"
printf 'node ops/ci/assert-report.mjs vitest "%s/coverage-tests.json" 119\n' "$report_root" >"$fixtures/coverage.sh"
expect_refusal FAMILY_VITEST_SOURCE_DRIFT vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh"
printf 'node ops/ci/assert-report.mjs vitest "%s/coverage-tests.json" 0\n' "$report_root" >"$fixtures/coverage.sh"
expect_refusal FAMILY_VITEST_SOURCE_INVALID vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh"
printf 'node ops/ci/assert-report.mjs vitest "%s/coverage-tests.json" many\n' "$report_root" >"$fixtures/coverage.sh"
expect_refusal FAMILY_VITEST_SOURCE_INVALID vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh"
: >"$fixtures/coverage.sh"
expect_refusal FAMILY_VITEST_SOURCE_MISSING vitest-source-pair "$fixtures/fast.sh" "$fixtures/coverage.sh"
family_source="$(<ops/ci/family.sh)"
[[ "$(grep -Fc 'bash scripts/sync-family-contracts.sh check' <<<"$family_source")" -eq 1 \
  && "$(grep -Fc 'bash ops/ci/contract.sh' <<<"$family_source")" -eq 1 \
  && "$(grep -Fc 'bash ops/ci/required.sh' <<<"$family_source")" -eq 0 ]] \
  || { refuse FAMILY_FINAL_STAGE_DUPLICATE "drift/contract order"; exit 1; }
[[ "$family_source" != *'bullet-farm|.ci-artifacts/junit/fast.xml'* ]]
after_sync="${family_source#*bash scripts/sync-family-contracts.sh check}"
[[ "$after_sync" != "$family_source" && "$after_sync" == *'bash ops/ci/contract.sh'* ]] \
  || { refuse FAMILY_FINAL_STAGE_ORDER_INVALID "contract before drift"; exit 1; }
lane_network() {
  sed -n "/^name = \"$1\"$/,/^\[\[lane\]\]/p" agent/proof-lanes.toml \
    | sed -nE 's/^requires_network = (true|false)$/\1/p'
}
for lane in security required family; do
  [[ "$(lane_network "$lane")" == true ]] \
    || { refuse PROOF_LANE_NETWORK_TRANSITIVITY_INVALID "$lane"; exit 1; }
done
grep -Fq 'bash ops/ci/security.sh' ops/ci/required.sh
grep -Fq 'scripts/ci-local.sh required' ops/ci/family.sh
printf '{"success":true,"numTotalTests":3,"numPassedTests":3,"numFailedTests":0,"numPendingTests":0,"numTodoTests":0}\n' >"$fixtures/vitest.json"
mv "$fixtures/vitest.json" "$fixtures/target.json"
ln -s "$fixtures/target.json" "$fixtures/vitest.json"
expect_refusal FAMILY_REPORT_MISSING vitest "$fixtures/vitest.json" 3
log "family report malformed, strict JSON, zero, failure, skip, count, and symlink matrix passed"
