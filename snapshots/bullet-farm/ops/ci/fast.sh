#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "fast lane: Hub metadata and bullet-family component partition"
for path in README.md AGENTS.md repos.manifest.toml family.lock agent/owner-map.json agent/test-map.json; do
  require_file "$path" || exit 1
done
cargo run --locked --quiet --bin bullet-family -- hub check
bash ops/ci/setup-refusal.sh
run_partition fast fast "$HUB_FILTER" "$HUB_EXPECTED_TESTS"
log "fast lane passed"
