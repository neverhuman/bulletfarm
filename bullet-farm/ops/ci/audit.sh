#!/usr/bin/env bash
# Jankurai audit lane. Writes artifacts for hosted CI and ratchets upward only.
# The score gate is the committed policy's minimum_score, passed explicitly as
# --fail-under so the CLI verdict and the policy document cannot drift apart.
# A missing or ambiguous minimum_score fails closed; the number may only rise.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

AUDIT_POLICY=agent/audit-policy.toml
bash scripts/ci-doctor.sh audit
require_file "$AUDIT_POLICY"
minimum_score="$(sed -nE 's/^minimum_score[[:space:]]*=[[:space:]]*([0-9]+)[[:space:]]*(#.*)?$/\1/p' "$AUDIT_POLICY")"
[[ "$(grep -c . <<<"$minimum_score")" -eq 1 && "$minimum_score" =~ ^[0-9]+$ && "$minimum_score" -le 100 ]] \
  || { refuse AUDIT_POLICY_MINIMUM_SCORE_INVALID "$AUDIT_POLICY must declare exactly one integer minimum_score"; exit 1; }
mkdir -p .jankurai
rm -f .jankurai/repo-score.json .jankurai/repo-score.md .jankurai/repair-queue.jsonl
log "audit lane: jankurai audit (committed policy ${AUDIT_POLICY}, fail-under ${minimum_score})"
jankurai audit . --full --no-score-history --policy "$AUDIT_POLICY" --fail-under "$minimum_score" \
  --json .jankurai/repo-score.json --md .jankurai/repo-score.md \
  --repair-queue-jsonl .jankurai/repair-queue.jsonl
[[ -f .jankurai/repo-score.json && -f .jankurai/repo-score.md && -f .jankurai/repair-queue.jsonl ]] || {
  echo "[ci] audit artifacts missing" >&2
  exit 1
}
log "audit lane passed"
