#!/usr/bin/env bash
# Prove that the pinned scanner reports a genuine secret-shaped finding. The
# value is assembled at runtime so the repository itself remains clean.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
canary_root="$(mktemp -d)"
cleanup() { rm -rf -- "$canary_root"; }
trap cleanup EXIT
printf 'aws_access_key_id = %s%s\n' 'AKIA' '6QWERTYUIOPASDFG' >"$canary_root/canary.txt"
report="$canary_root/findings.json"
set +e
gitleaks detect --source "$canary_root" --no-git --redact --no-banner \
  --exit-code 42 --report-format json --report-path "$report" >/dev/null 2>&1
status=$?
set -e
[[ "$status" -eq 42 ]] \
  || { refuse SECRET_CANARY_NOT_DETECTED "expected gitleaks finding exit 42, found $status"; exit 1; }
bash ops/ci/strict-json.sh "$report" >/dev/null \
  || { refuse SECRET_CANARY_JSON_INVALID "$report"; exit 1; }
jq -e '
  length == 1 and .[0].RuleID == "aws-access-token" and
  (.[0].File | endswith("/canary.txt")) and .[0].StartLine == 1
' "$report" >/dev/null \
  || { refuse SECRET_CANARY_FINDING_INVALID "wrong rule, file, line, or finding count"; exit 1; }
log "secret canary produced the exact structured AWS finding"
