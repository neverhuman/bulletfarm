#!/usr/bin/env bash
# Jankurai audit lane. Writes artifacts for hosted CI and ratchets upward only.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

AUDIT_POLICY=agent/audit-policy.toml
bash scripts/ci-doctor.sh audit
require_file "$AUDIT_POLICY"
mkdir -p .jankurai
rm -f .jankurai/repo-score.json .jankurai/repo-score.md .jankurai/repair-queue.jsonl
log "audit lane: jankurai audit (committed policy ${AUDIT_POLICY})"
jankurai audit . --full --no-score-history --policy "$AUDIT_POLICY" \
  --json .jankurai/repo-score.json --md .jankurai/repo-score.md \
  --repair-queue-jsonl .jankurai/repair-queue.jsonl
[[ -f .jankurai/repo-score.json && -f .jankurai/repo-score.md && -f .jankurai/repair-queue.jsonl ]] || {
  echo "[ci] audit artifacts missing" >&2
  exit 1
}
log "audit lane passed"
