#!/usr/bin/env bash
# Wave 8 / post-GA honesty: first-GA and later profiles remain BLOCKED
# until admitted receipts exist. This script never synthesizes PASS.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

require_file docs/assurance/release-truth.generated.md
grep -q 'RELEASE DECISION: BLOCKED' docs/assurance/release-truth.generated.md \
  || { refuse RELEASE_TRUTH_DRIFT "generated page must remain BLOCKED"; exit 1; }

if [[ "$mode" == --self-test ]]; then
  log "check-release honesty self-test: generated universal page remains BLOCKED"
  exit 0
fi

refuse FIRST_GA_BLOCKED \
  "self-hosted-v1, evolution-v1, universal-v1, team-v1, and saga-v1 stay BLOCKED; OD-A/B/D/E/H/C/I/J remain OPEN"
exit 1
