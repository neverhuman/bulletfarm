#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "required gate: five atomic lanes, sequentially, exactly once"
# Security goes first so source and lockfiles are scanned before Cargo resolves
# or builds dependencies in the remaining lanes.
bash ops/ci/security.sh
bash ops/ci/fast.sh
bash ops/ci/lint.sh
bash ops/ci/contract.sh
bash ops/ci/docs.sh
log "required gate passed"
