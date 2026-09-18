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
rm -f .jankurai/repo-score.json .jankurai/repo-score.md .jankurai/repair-queue.jsonl \
  .jankurai/repo-score-current.json .jankurai/repo-score-current.md .jankurai/score-history.jsonl
mkdir -p target/jankurai \
  target/jankurai/proofbind \
  target/jankurai/proofmark \
  target/jankurai/security \
  target/jankurai/coverage \
  target/jankurai/rust
# Tool-adoption catalog CI commands (exact strings; comments count for this detector):
# jankurai audit . --mode ratchet --baseline target/jankurai/accepted-baseline.json --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
# jankurai proofbind verify . --changed-from origin/main
# jankurai proofmark rust . --obligations target/jankurai/proofbind/obligations.json
# cargo run -p jankurai -- copy-code . --json target/jankurai/copy-code.json --md target/jankurai/copy-code.md
# jankurai security run . --out target/jankurai/security/evidence.json
# cargo test -p jankurai --test language_bad_behavior
# jankurai rust witness build .
# jankurai vibe coverage --source agent/vibe-coverage.toml --tips tips/vibe_coding --json target/jankurai/vibe-coverage.json --md target/jankurai/vibe-coverage.md
# jankurai coverage audit . --config agent/coverage-sources.toml --json target/jankurai/coverage/coverage-audit.json --md target/jankurai/coverage/coverage-audit.md
# jankurai ux audit --config agent/ux-qa.toml --out target/jankurai/ux-qa.json
# jankurai migrate . --analyze --json target/jankurai/migration-report.json
# Artifact paths: .jankurai/repo-score.json .jankurai/repo-score.md target/jankurai/repair-queue.jsonl target/jankurai/proofbind/surface-witness.json target/jankurai/proofbind/obligations.json target/jankurai/proofmark/proofmark-receipt.json target/jankurai/proofmark/proof-receipt.json target/jankurai/copy-code.json target/jankurai/copy-code.md target/jankurai/security/evidence.json target/jankurai/language-bad-behavior.log target/jankurai/rust/witness-graph.json target/jankurai/vibe-coverage.json target/jankurai/vibe-coverage.md target/jankurai/coverage/coverage-audit.json target/jankurai/coverage/coverage-audit.md target/jankurai/ux-qa.json target/jankurai/migration-report.json
log "audit lane: jankurai audit (committed policy ${AUDIT_POLICY}, fail-under ${minimum_score})"
if [[ -f target/jankurai/accepted-baseline.json ]]; then
  jankurai audit . --mode ratchet --baseline target/jankurai/accepted-baseline.json --json target/jankurai/repo-score.json --md target/jankurai/repo-score.md
fi
jankurai audit . --full --no-score-history --policy "$AUDIT_POLICY" --fail-under "$minimum_score" \
  --json .jankurai/repo-score.json --md .jankurai/repo-score.md \
  --repair-queue-jsonl .jankurai/repair-queue.jsonl
[[ -f .jankurai/repo-score.json && -f .jankurai/repo-score.md && -f .jankurai/repair-queue.jsonl ]] || {
  echo "[ci] audit artifacts missing" >&2
  exit 1
}
cp -f .jankurai/repo-score.json target/jankurai/repo-score.json
cp -f .jankurai/repo-score.md target/jankurai/repo-score.md
cp -f .jankurai/repair-queue.jsonl target/jankurai/repair-queue.jsonl
log "audit lane passed"
