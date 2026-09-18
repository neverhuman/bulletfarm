#!/usr/bin/env bash
# Wave 1: schema-3 lock generate stays refused without OD-D/E subjects.
# Agents must not mint a lock, signer policy, or Hub tag.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

require_file family.lock
grep -qx 'schema_version = "2"' family.lock \
  || { refuse LOCK_SCHEMA_DRIFT "checked-in lock must remain diagnostic schema 2"; exit 1; }

if [[ "$mode" == --self-test ]]; then
  log "Wave 1 self-test: schema-2 lock remains diagnostic; OD-D/E are OPEN; lock generate is not invoked"
  exit 0
fi

refuse SCHEMA3_LOCK_INPUTS_ABSENT \
  "OD-D/OD-E are OPEN: do not run lock generate, do not invent signers, do not tag Hub last"
exit 1
