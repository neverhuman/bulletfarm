#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
[[ "$(git rev-parse --is-shallow-repository)" == false ]] \
  || { refuse SHALLOW_HISTORY "full-history scanning requires fetch-depth 0"; exit 1; }
gitleaks git --redact --no-banner .
log "full-history secret scan passed"
