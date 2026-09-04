#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
log "family-contract compatibility alias: ordered family proof"
exec bash ops/ci/family.sh
