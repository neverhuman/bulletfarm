#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
require_tool gitleaks || exit 1
require_exact_output "8.21.2" gitleaks version
gitleaks detect --source . --no-git --redact --no-banner
log "current source and lockfile secret scan passed"
