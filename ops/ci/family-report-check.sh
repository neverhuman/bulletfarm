#!/usr/bin/env bash
# Validate one dependency-ordered family report and emit a normalized summary
# with no test names, paths, timestamps, or captured output.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

kind="${1:-}"
report="${2:-}"
[[ -n "$kind" && -f "$report" && ! -L "$report" && -s "$report" ]] \
  || { refuse FAMILY_REPORT_MISSING "${report:-missing}"; exit 1; }

validate_json_report() {
  bash "$REPO_ROOT/ops/ci/strict-json.sh" "$1" >/dev/null 2>&1 \
    || { refuse FAMILY_REPORT_JSON_INVALID "$1"; return 1; }
}

run_report_python_312() {
  local env_executable env_sha256 python_executable python_sha256 status
  env_executable="$(resolved_executable env)" || return 1
  env_sha256="$(sha256_file "$env_executable")" || return 1
  python_executable="$(resolve_python_312 "$env_executable")" || return 1
  python_sha256="$(sha256_file "$python_executable")" || return 1
  verify_resolved_tool "$env_executable" "$env_sha256" env || return 1
  verify_resolved_tool "$python_executable" "$python_sha256" Python || return 1
  if "$env_executable" -i HOME="${HOME:-/}" \
    PATH="${python_executable%/*}:/usr/bin:/bin" LC_ALL=C TZ=UTC PYTHONIOENCODING=utf-8 \
    "$python_executable" -I -S "$@"; then
    status=0
  else
    status=$?
  fi
  verify_resolved_tool "$python_executable" "$python_sha256" Python || return 1
  verify_resolved_tool "$env_executable" "$env_sha256" env || return 1
  return "$status"
}

case "$kind" in
  vitest-source-pair)
    [[ "$#" -eq 3 && -f "$3" && ! -L "$3" && -s "$3" ]] \
      || { refuse FAMILY_VITEST_SOURCE_MISSING "fast/coverage source"; exit 1; }
    source_count() {
      local source="$1" report_name="$2"
      local -a values=()
      # The declaration being parsed contains the literal shell variable name `$reports`.
      # shellcheck disable=SC2016
      mapfile -t values < <(sed -nE \
        's#^node ops/ci/assert-report\.mjs vitest "\$reports/'"$report_name"'" ([0-9]+)$#\1#p' \
        "$source")
      [[ "${#values[@]}" -eq 1 && "${values[0]}" =~ ^[1-9][0-9]*$ ]] \
        || { refuse FAMILY_VITEST_SOURCE_INVALID "$source:$report_name"; return 1; }
      printf '%s\n' "${values[0]}"
    }
    fast_count="$(source_count "$2" vitest.json)" || exit 1
    coverage_count="$(source_count "$3" coverage-tests.json)" || exit 1
    [[ "$fast_count" -eq "$coverage_count" ]] \
      || { refuse FAMILY_VITEST_SOURCE_DRIFT "fast=$fast_count coverage=$coverage_count"; exit 1; }
    printf '%s\n' "$fast_count"
    ;;
  junit)
    [[ "$#" -eq 4 ]] || { refuse FAMILY_REPORT_USAGE "junit FILE TESTS SKIPPED"; exit 2; }
    expected_tests="$3"
    expected_skipped="$4"
    [[ "$expected_tests" =~ ^[1-9][0-9]*$ && "$expected_skipped" =~ ^[0-9]+$ ]] \
      || { refuse FAMILY_REPORT_EXPECTATION_INVALID "$expected_tests/$expected_skipped"; exit 1; }
    max_xml_bytes=$((16 * 1024 * 1024))
    report_bytes="$(wc -c <"$report")"
    [[ "$report_bytes" =~ ^[1-9][0-9]*$ && "$report_bytes" -le "$max_xml_bytes" ]] \
      || { refuse FAMILY_REPORT_XML_INVALID "size=$report_bytes"; exit 1; }
    counters="$(run_report_python_312 - "$report" "$max_xml_bytes" <<'PY'
import sys
import xml.etree.ElementTree as ET

path, maximum = sys.argv[1], int(sys.argv[2])
data = open(path, "rb").read(maximum + 1)
if not data or len(data) > maximum:
    raise SystemExit(2)
try:
    document = data.decode("utf-8")
except UnicodeDecodeError:
    raise SystemExit(2)
if "<!DOCTYPE" in document or "<!ENTITY" in document:
    raise SystemExit(2)
try:
    root = ET.fromstring(document)
except (ET.ParseError, UnicodeError, ValueError):
    raise SystemExit(2)
if root.tag != "testsuites":
    raise SystemExit(2)

MAX_SAFE_INTEGER = 9_007_199_254_740_991

def counter(element, name, required=True):
    raw = element.get(name)
    if raw is None:
        if required:
            raise ValueError(name)
        return None
    if not raw.isascii() or not raw.isdecimal():
        raise ValueError(name)
    value = int(raw, 10)
    if value > MAX_SAFE_INTEGER:
        raise ValueError(name)
    return value

try:
    root_tests = counter(root, "tests")
    root_failures = counter(root, "failures")
    root_errors = counter(root, "errors")
    root_skipped = counter(root, "skipped", required=False)
    suites = list(root)
    if not suites or any(suite.tag != "testsuite" for suite in suites):
        raise ValueError("testsuite")
    if any(nested.tag == "testsuite" for suite in suites for nested in suite.iter() if nested is not suite):
        raise ValueError("nested testsuite")
    totals = {"tests": 0, "failures": 0, "errors": 0, "skipped": 0}
    for suite in suites:
        totals["tests"] += counter(suite, "tests")
        totals["failures"] += counter(suite, "failures")
        totals["errors"] += counter(suite, "errors")
        skipped = counter(suite, "skipped", required=False)
        disabled = counter(suite, "disabled", required=False)
        if skipped is not None and disabled is not None:
            raise ValueError("ambiguous skipped counter")
        if skipped is None and disabled is None:
            raise ValueError("missing skipped counter")
        totals["skipped"] += skipped if skipped is not None else disabled
    if (root_tests, root_failures, root_errors) != (
        totals["tests"], totals["failures"], totals["errors"]
    ):
        raise ValueError("root totals")
    if root_skipped is not None and root_skipped != totals["skipped"]:
        raise ValueError("root skipped")
except (TypeError, ValueError):
    raise SystemExit(2)

print(root_tests, root_failures, root_errors, totals["skipped"], sep="\t")
PY
)" || { refuse FAMILY_REPORT_XML_INVALID "$report"; exit 1; }
    IFS=$'\t' read -r tests failures errors skipped <<<"$counters"
    [[ "$tests" =~ ^[0-9]+$ && "$failures" =~ ^[0-9]+$ && "$errors" =~ ^[0-9]+$ \
      && "$skipped" =~ ^[0-9]+$ ]] \
      || { refuse FAMILY_REPORT_XML_INVALID "counter output"; exit 1; }
    [[ "$tests" -eq "$expected_tests" && "$tests" -gt 0 && "$failures" -eq 0 \
      && "$errors" -eq 0 && "$skipped" -eq "$expected_skipped" ]] \
      || { refuse FAMILY_REPORT_OUTCOME_INVALID "tests=$tests/$expected_tests failures=$failures errors=$errors skipped=$skipped/$expected_skipped"; exit 1; }
    executed=$((tests - skipped))
    jq -cn --argjson tests "$tests" --argjson executed "$executed" --argjson skipped "$skipped" \
      '{kind:"junit",tests:$tests,executed:$executed,failures:0,errors:0,skipped:$skipped}'
    ;;
  vitest)
    [[ "$#" -eq 3 && "$3" =~ ^[1-9][0-9]*$ ]] \
      || { refuse FAMILY_REPORT_USAGE "vitest FILE TESTS"; exit 2; }
    validate_json_report "$report" || exit 1
    jq -ce --argjson expected "$3" '
      select(.success == true and .numTotalTests == $expected and .numTotalTests > 0 and
        .numPassedTests == $expected and .numFailedTests == 0 and
        .numPendingTests == 0 and .numTodoTests == 0) |
      {kind:"vitest",tests:.numTotalTests,passed:.numPassedTests,failed:0,pending:0,todo:0}
    ' "$report" || { refuse FAMILY_REPORT_OUTCOME_INVALID "$report"; exit 1; }
    ;;
  formal-json)
    [[ "$#" -eq 2 ]] || { refuse FAMILY_REPORT_USAGE "formal-json FILE"; exit 2; }
    validate_json_report "$report" || exit 1
    jq -ce '
      select(. == {schema_version:"bullet.formal-summary.v1",models:2,completed_models:2,
        pinned_summary_present:true,status:"PASS",exit_code:0,signed:false,
        evidence_class:"DIAGNOSTIC_ONLY"}) |
      {kind:"formal",models:2,completed_models:2,pinned_summary_present:true,status:"PASS"}
    ' "$report" || { refuse FAMILY_REPORT_OUTCOME_INVALID "$report"; exit 1; }
    ;;
  formal-log)
    [[ "$#" -eq 2 ]] || { refuse FAMILY_REPORT_USAGE "formal-log FILE"; exit 2; }
    expected="$(printf '%s\n' \
      'schema=bullet.formal-log.v1' 'models=2' 'completed_without_error=2' \
      'pinned_summary_present=1' 'exit_code=0' 'classification=DIAGNOSTIC_ONLY')"
    [[ "$(<"$report")" == "$expected" ]] \
      || { refuse FAMILY_REPORT_OUTCOME_INVALID "$report"; exit 1; }
    jq -cn '{kind:"formal-log",models:2,completed_without_error:2,pinned_summary_present:true,exit_code:0}'
    ;;
  *) refuse FAMILY_REPORT_USAGE "expected vitest-source-pair|junit|vitest|formal-json|formal-log"; exit 2 ;;
esac
