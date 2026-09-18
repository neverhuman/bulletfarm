#!/usr/bin/env bash
# Workspace line-coverage floor for the Hub coverage lane.
#
# Reads the sanitized Cobertura report that ops/ci/coverage.sh writes
# (.ci-artifacts/coverage/cobertura.xml, after coverage-sanitize.sh) and refuses
# when measured line coverage (lines-covered / lines-valid on the root element)
# is below COVERAGE_LINE_FLOOR. The floor is a ratchet: it may only rise. It was
# introduced at measured-1 (72.44 % lines on 2026-08-26 -> 71). A missing,
# symlinked, empty, malformed, or internally inconsistent report is a refusal,
# never a pass.
#
# The optional argument names another report and exists only so hostile
# fixtures can be proven; the coverage lane always calls this with no argument.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

COVERAGE_LINE_FLOOR=71
report="${1:-.ci-artifacts/coverage/cobertura.xml}"

require_tool awk || exit 1
require_tool grep || exit 1
[[ "$COVERAGE_LINE_FLOOR" =~ ^[0-9]+$ && "$COVERAGE_LINE_FLOOR" -le 100 ]] \
  || { refuse COVERAGE_FLOOR_INVALID "$COVERAGE_LINE_FLOOR"; exit 1; }
[[ -f "$report" && ! -L "$report" && -s "$report" ]] \
  || { refuse COVERAGE_ARTIFACT_MISSING "$report"; exit 1; }

root_count="$({ grep -Eo '<coverage[[:space:]]' "$report" || true; } | wc -l | tr -d ' ')"
[[ "$root_count" -eq 1 ]] \
  || { refuse COVERAGE_ARTIFACT_INVALID "expected one <coverage> root, found $root_count"; exit 1; }
root_line="$(grep -m1 '<coverage[[:space:]]' "$report")"

root_attribute() {
  local name="$1" matches count value
  matches="$(grep -oE "[[:space:]]${name}=\"[^\"]*\"" <<<"$root_line" || true)"
  count="$(grep -c . <<<"$matches")"
  [[ "$count" -eq 1 ]] \
    || { refuse COVERAGE_ARTIFACT_INVALID "expected one $name attribute, found $count"; return 1; }
  value="${matches#*=\"}"
  printf '%s\n' "${value%\"}"
}
line_count="$(root_attribute lines-valid)" || exit 1
line_covered="$(root_attribute lines-covered)" || exit 1
line_rate="$(root_attribute line-rate)" || exit 1
[[ "$line_count" =~ ^[1-9][0-9]*$ && "$line_covered" =~ ^[0-9]+$ \
  && "$line_covered" -le "$line_count" && "$line_rate" =~ ^(0|1)(\.[0-9]+)?$ ]] \
  || { refuse COVERAGE_ARTIFACT_INVALID "lines=$line_covered/$line_count line-rate=$line_rate"; exit 1; }

# Recompute the percent from the counters and require the report to agree
# with its own line-rate, so a hand-edited rate cannot outrun its counters.
measured="$(awk -v count="$line_count" -v covered="$line_covered" \
  'BEGIN { printf "%.2f", (covered * 100) / count }')"
awk -v measured="$measured" -v rate="$line_rate" 'BEGIN {
  delta = measured - rate * 100
  if (delta < 0) delta = -delta
  exit (delta <= 0.01) ? 0 : 1
}' || { refuse COVERAGE_ARTIFACT_INVALID "line-rate $line_rate disagrees with $line_covered/$line_count"; exit 1; }

log "workspace line coverage measured=${measured}% (${line_covered}/${line_count} lines) floor=${COVERAGE_LINE_FLOOR}%"
awk -v measured="$measured" -v floor="$COVERAGE_LINE_FLOOR" \
  'BEGIN { exit (measured + 0 >= floor + 0) ? 0 : 1 }' \
  || { refuse COVERAGE_BELOW_FLOOR "measured ${measured}% < floor ${COVERAGE_LINE_FLOOR}% lines"; exit 1; }
log "coverage floor passed"
