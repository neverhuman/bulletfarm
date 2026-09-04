#!/usr/bin/env bash
# Current-tree secrets, a real-detection canary, the complete Rust supply-chain
# policy, and offline workflow analysis. Every check fails closed.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/advisory-db.sh
source "$(dirname "${BASH_SOURCE[0]}")/advisory-db.sh"
cd "$REPO_ROOT"

log "security lane: current tree before dependency resolution"
bash ops/ci/source-scan.sh
bash ops/ci/secret-canary.sh
require_file deny.toml
refresh_advisory_database
cargo deny --locked check licenses advisories bans sources
zizmor --offline --no-ignores --strict-collection .
log "security lane passed"
