#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/advisory-db.sh
source "$(dirname "${BASH_SOURCE[0]}")/advisory-db.sh"
cd "$REPO_ROOT"
refresh_advisory_database
cargo deny --locked check advisories
log "scheduled advisory lane passed"
