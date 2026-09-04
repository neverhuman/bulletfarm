#!/usr/bin/env bash
# Wave 6: six Portal surfaces stay OUT_OF_PROFILE until durable ledger subjects exist.
# This script does not invent those subjects or flip a release gate.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

surfaces='cognitive-router fusion-lab quota-capacity struggle-cockpit behavior-center workspace-hygiene'
log "Wave 6 hold: surfaces still without durable ledger subjects: $surfaces"

if [[ "$mode" == --self-test ]]; then
  exit 0
fi

refuse PORTAL_SURFACES_NOT_DURABLE \
  "Cognitive Router, Fusion Lab, Quota/Capacity, Struggle, Behavior, and Workspace Hygiene remain without durable ledger subjects; do not treat Portal green as G13"
exit 1
