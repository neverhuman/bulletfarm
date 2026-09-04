#!/usr/bin/env bash
# DF-R7b: compare live incident inputs to a reviewed rehearsal bundle.
# This script never mutates coordinator state, never chmods the frozen store,
# and never executes recover-rollover / adopt. Independent human APPROVE is
# required even to claim a comparison PASS; this checkout has none.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

usage() {
  printf 'usage: %s {compare|--self-test} [--rehearsal BUNDLE] [--live-source PATH] [--approval PATH]\n' "$0" >&2
  exit 2
}

mode="${1:-}"
[[ -n "$mode" ]] || usage
shift || true
rehearsal=""
live_source=""
approval=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --rehearsal) rehearsal="${2:-}"; shift 2 ;;
    --live-source) live_source="${2:-}"; shift 2 ;;
    --approval) approval="${2:-}"; shift 2 ;;
    *) usage ;;
  esac
done

frozen_default="$(cd "$REPO_ROOT/.." && pwd -P)/.bullet-family/coord/events.jsonl"

refuse_execution() {
  refuse "$1" "$2"
  exit 1
}

if [[ "$mode" == --self-test ]]; then
  work="$(mktemp -d)"
  trap 'rm -rf -- "$work"' EXIT
  chmod 0700 -- "$work"
  printf '%s\n' '{"kind":"bullet.coord.recovery-rehearsal.v1","live_incident_authorized":false}' >"$work/bundle.json"
  printf '%s\n' 'not-an-approval' >"$work/approval.json"
  printf '%s\n' 'synthetic-live' >"$work/live.jsonl"
  status=0
  bash "$REPO_ROOT/scripts/recovery-incident-compare.sh" compare \
    --rehearsal "$work/bundle.json" \
    --live-source "$work/live.jsonl" \
    --approval "$work/approval.json" >"$work/self.out" 2>"$work/self.err" || status=$?
  [[ "$status" -ne 0 ]] || refuse_execution R7B_UNEXPECTED_SUCCESS "compare must refuse without independent APPROVE"
  grep -q 'INDEPENDENT_APPROVAL_ABSENT\|INDEPENDENT_APPROVAL_INVALID' "$work/self.err" \
    || refuse_execution R7B_REASON_DRIFT "expected independent-approval refusal"
  if [[ -e "$frozen_default" ]]; then
    before="$(sha256_file "$frozen_default")"
  fi
  status=0
  bash "$REPO_ROOT/scripts/recovery-incident-compare.sh" compare \
    --rehearsal "$work/bundle.json" \
    --live-source "$frozen_default" >"$work/live.out" 2>"$work/live.err" || status=$?
  [[ "$status" -ne 0 ]] || refuse_execution R7B_LIVE_EXECUTED "live frozen source must not authorize mutation"
  grep -q 'LIVE_INCIDENT_EXECUTION_FORBIDDEN' "$work/live.err" \
    || refuse_execution R7B_LIVE_REASON_DRIFT "expected LIVE_INCIDENT_EXECUTION_FORBIDDEN"
  if [[ -n "${before:-}" ]]; then
    after="$(sha256_file "$frozen_default")"
    [[ "$before" == "$after" ]] || refuse_execution LIVE_COORD_MUTATED "compare must not rewrite the frozen ledger"
  fi
  log "DF-R7b self-test passed (execution remains forbidden; frozen ledger untouched)"
  exit 0
fi

[[ "$mode" == compare ]] || usage
[[ -n "$rehearsal" && -f "$rehearsal" && ! -L "$rehearsal" ]] \
  || refuse_execution REHEARSAL_BUNDLE_ABSENT "reviewed rehearsal bundle is required"
[[ -n "$live_source" ]] \
  || refuse_execution LIVE_SOURCE_ABSENT "exact live source path is required"

if [[ "$live_source" == "$frozen_default" || "$live_source" == *"/.bullet-family/coord/"* ]]; then
  refuse_execution LIVE_INCIDENT_EXECUTION_FORBIDDEN \
    "frozen real incident remains unauthorized; agents must not chmod, recover, or adopt it"
fi

if [[ -z "$approval" || ! -f "$approval" || -L "$approval" ]]; then
  refuse_execution INDEPENDENT_APPROVAL_ABSENT \
    "DF-R7b requires an independent human APPROVE document; none is admitted"
fi

if ! grep -q '"decision":"APPROVE"' "$approval"; then
  refuse_execution INDEPENDENT_APPROVAL_INVALID \
    "approval is not a closed independent APPROVE document"
fi

refuse_execution LIVE_INCIDENT_EXECUTION_FORBIDDEN \
  "even a well-formed compare cannot execute recover-rollover or adopt from this script"
