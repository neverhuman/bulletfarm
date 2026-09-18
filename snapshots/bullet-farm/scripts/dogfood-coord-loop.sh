#!/usr/bin/env bash
# DF-DOG0: first coordination-dogfood loop.
# Uses recovered schema-2 coord only. Typed unavailable/recovery states are honest stops.
# Never chmods the live ledger, never invents a second coordinator, never commits.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

family="$(cd "$REPO_ROOT/.." && pwd -P)"
events="$family/.bullet-family/coord/events.jsonl"

coord_status_class() {
  local output_file="$1" frozen_codes
  frozen_codes='COORD_IO_FAILED|COORD_NOT_INITIALIZED'
  frozen_codes+='|COORD_RECOVERY_REQUIRED|COORD_RECOVERY_IN_PROGRESS'
  if grep -Eq "^bullet-family: (${frozen_codes}): " "$output_file"; then
    printf 'COORD_FROZEN\n'
  else
    printf 'COORD_STATUS_FAILED\n'
  fi
}

if [[ "$mode" == --self-test ]]; then
  [[ -e "$events" ]] || { refuse COORD_LEDGER_ABSENT "frozen events.jsonl missing"; exit 1; }
  fixture="$(mktemp)"
  trap 'rm -f -- "$fixture"' EXIT
  for code in COORD_RECOVERY_REQUIRED COORD_RECOVERY_IN_PROGRESS; do
    printf 'bullet-family: %s: fixture\n' "$code" >"$fixture"
    [[ "$(coord_status_class "$fixture")" == COORD_FROZEN ]] || {
      refuse DOG0_SELF_TEST_FAILED "$code did not map to COORD_FROZEN"
      exit 1
    }
  done
  for line in 'bullet-family: COORD_RECOVERY_REQUIRED_EXTRA: fixture' 'Permission denied'; do
    printf '%s\n' "$line" >"$fixture"
    [[ "$(coord_status_class "$fixture")" == COORD_STATUS_FAILED ]] || {
      refuse DOG0_SELF_TEST_FAILED "non-typed or near-match text mapped to COORD_FROZEN"
      exit 1
    }
  done
  log "DF-DOG0 self-test: COORD_RECOVERY_REQUIRED and COORD_RECOVERY_IN_PROGRESS" \
    "map to COORD_FROZEN; no coord command executed"
  exit 0
fi

status=0
output="$(mktemp)"
trap 'rm -f -- "$output"' EXIT
if [[ -x "$REPO_ROOT/target/debug/bullet-family" ]]; then
  bin="$REPO_ROOT/target/debug/bullet-family"
else
  refuse COORD_BINARY_UNAVAILABLE "build bullet-family before DF-DOG0 check"
  exit 1
fi

"$bin" --root "$family" coord status --json >"$output" 2>&1 || status=$?
if [[ "$(coord_status_class "$output")" == COORD_FROZEN ]]; then
  refuse COORD_FROZEN \
    "DF-DOG0 cannot start: machine coord is frozen; do not chmod or recover the live ledger from this script"
  exit 1
fi
if [[ "$status" -ne 0 ]]; then
  refuse COORD_STATUS_FAILED "coord status exited $status"
  exit 1
fi

refuse DOG0_NOT_EXECUTED \
  "coord answered, but DF-DOG0 still requires recovered schema-2 claim/heartbeat/handoff/receipt/restart read-back; this script does not commit"
exit 1
