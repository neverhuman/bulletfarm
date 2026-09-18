#!/usr/bin/env bash
set -uo pipefail
SC=/tmp/claude-1000/-home-ubuntu-bullet/8fd51d96-4d62-4539-9449-2f0695d91d78/scratchpad
STAMP=$(date -u +%Y%m%dT%H%M%SZ)
OUT=$SC/e2e/$STAMP; mkdir -p "$OUT"; chmod 700 "$OUT"
export BULLET_TXN_PROVIDER=claude
# Dogfood ON bulletfarm: the tree under review is Bullet Farm's own kernel.
export BULLET_TXN_SOURCE_SNAPSHOT=${BULLET_TXN_SOURCE_SNAPSHOT:-/home/ubuntu/bullet/bullet-kernel}
export BULLET_TXN_OBJECTIVE=${BULLET_TXN_OBJECTIVE:-"Read this Bullet Farm kernel checkout and satisfy the repository gate"}
export BULLET_DOGFOOD_DATA_DIR=/home/ubuntu/bullet-dogfood-2
export BULLET_DOGFOOD_POLICY=/home/ubuntu/bullet-dogfood-2/policy/policy.json
export BULLET_DOGFOOD_BINDING=/home/ubuntu/bullet-dogfood-2/policy/binding.json
export BULLET_DOGFOOD_ENROLLMENT=/home/ubuntu/bullet-dogfood-2/policy/enrollments/claude.json
export BULLET_DOGFOOD_ISSUER=dogfood-local
export BULLET_DOGFOOD_KEY_ID=dogfood-runner-1
export BULLET_DOGFOOD_EXECUTABLE=/usr/lib/bullet/providers/claude/2.1.266/bin/claude
export BULLET_DOGFOOD_RECEIPT=$OUT/dogfood-receipt.json
export BULLET_DOGFOOD_MAX_BUDGET_USD=0.75
export BULLET_DOGFOOD_WALL_TIMEOUT_SECS=600
CRED_SRC="$HOME/.claude/.credentials.json"
CRED_DIGEST="$(b3sum "$CRED_SRC" | cut -d' ' -f1)"
export BULLET_DOGFOOD_CREDENTIALS="$CRED_SRC,.claude/.credentials.json,$CRED_DIGEST"
export BULLET_DOGFOOD_CAPTURE_DIR=$OUT/capture
mkdir -p "$OUT/capture"
# The proof root's ancestors must be euid-owned and not group/other-writable,
# which rules out anything under the world-writable /tmp.
PROOF=$HOME/bullet-e2e/$STAMP
mkdir -p "$HOME/bullet-e2e"; chmod 700 "$HOME/bullet-e2e"
mkdir -p "$PROOF/data"; chmod 700 "$PROOF" "$PROOF/data"
export BULLET_DATA_DIR=$PROOF/data
export TRANSACTION_OFFLINE_RECEIPT=$PROOF/COMPONENT_PROOF.receipt.json
export TRANSACTION_OFFLINE_ARTIFACT_ROOT=$PROOF/artifacts
bash /home/ubuntu/bullet/bullet-farm/scripts/proof-transaction-offline.sh > "$OUT/stdout.log" 2> "$OUT/stderr.log"
echo "RC=$?" | tee "$OUT/rc.txt"
echo "OUT=$OUT"
