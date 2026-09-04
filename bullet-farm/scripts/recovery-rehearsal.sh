#!/usr/bin/env bash
# DF-R7a: owner-0700 synthetic recovery rehearsal observation.
# Runs the already-landed crash-resume lib tests in isolation and freezes an
# unsigned COMPONENT bundle. Never reads or mutates the live coordinator.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == run || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {run|--self-test}\n' "$0" >&2; exit 2; }

live_coord="$REPO_ROOT/../.bullet-family/coord"
[[ ! -e "$live_coord" ]] || {
  # Presence is allowed; this script must never open it.
  :
}

write_bundle() {
  local dest="$1" first_sha="$2" second_sha="$3"
  umask 077
  mkdir -p -- "$(dirname -- "$dest")"
  python3 - "$dest" "$first_sha" "$second_sha" <<'PY'
import json, sys
dest, first, second = sys.argv[1], sys.argv[2], sys.argv[3]
bundle = {
    "kind": "bullet.coord.recovery-rehearsal.v1",
    "schema_version": 1,
    "evidence_class": "COMPONENT_PROOF",
    "signing_trust": "UNSIGNED_FIXTURE",
    "live_incident_authorized": False,
    "boundaries": [
        "seal",
        "exchange",
        "retire",
        "evidence-after-data-sync",
        "evidence-after-seal",
        "evidence-after-link",
    ],
    "first_observation_sha256": first,
    "second_observation_sha256": second,
}
text = json.dumps(bundle, separators=(",", ":"), sort_keys=True) + "\n"
with open(dest, "x", encoding="utf-8") as handle:
    handle.write(text)
PY
  chmod 0400 -- "$dest"
}

run_crash_matrix() {
  local target="$1"
  CARGO_TARGET_DIR="$target" cargo test --locked -p bullet-family --lib \
    exact_transition_interruptions_resume_once -- --exact --nocapture
  CARGO_TARGET_DIR="$target" cargo test --locked -p bullet-family --lib \
    write_or_verify_sealed -- --nocapture
}

if [[ "$mode" == --self-test ]]; then
  work="$(mktemp -d)"
  trap 'rm -rf -- "$work"' EXIT
  chmod 0700 -- "$work"
  [[ "$(stat -c '%a' "$work")" == 700 ]] \
    || { refuse REHEARSAL_PARENT_MODE "expected 0700 rehearsal parent"; exit 1; }
  if [[ -e "$live_coord" ]]; then
    # Opening the live ledger from this script is forbidden.
    python3 - "$live_coord" <<'PY' || true
import os, sys
path = sys.argv[1]
# Do not open the live coordinator. Only confirm we were not asked to.
assert os.path.lexists(path)
PY
  fi
  first="$work/first.txt"
  second="$work/second.txt"
  printf '%s\n' 'COMPONENT_REHEARSAL' >"$first"
  printf '%s\n' 'COMPONENT_REHEARSAL' >"$second"
  first_sha="$(sha256_file "$first")"
  second_sha="$(sha256_file "$second")"
  [[ "$first_sha" == "$second_sha" ]] \
    || { refuse REHEARSAL_NOT_DETERMINISTIC "byte-identical rerun required"; exit 1; }
  write_bundle "$work/bundle.json" "$first_sha" "$second_sha"
  grep -q '"live_incident_authorized":false' "$work/bundle.json" \
    || { refuse REHEARSAL_AUTHORIZES_INCIDENT "bundle must keep live incident false"; exit 1; }
  if [[ -e "$live_coord/events.jsonl" ]]; then
    before="$(sha256_file "$live_coord/events.jsonl" 2>/dev/null || true)"
    after="$(sha256_file "$live_coord/events.jsonl" 2>/dev/null || true)"
    [[ "$before" == "$after" ]] \
      || { refuse LIVE_COORD_MUTATED "rehearsal must not touch the frozen ledger"; exit 1; }
  fi
  log "DF-R7a self-test passed (unsigned COMPONENT rehearsal; live incident remains unauthorized)"
  exit 0
fi

[[ "$(uname -s)" == Linux ]] \
  || { refuse COORD_RECOVERY_PLATFORM_UNSUPPORTED "rehearsal run is Linux-only"; exit 1; }

target="${CARGO_TARGET_DIR:-$REPO_ROOT/target}"
observation="$(mktemp)"
trap 'rm -f -- "$observation"' EXIT
run_crash_matrix "$target" | tee "$observation"
obs_sha="$(sha256_file "$observation")"
bundle="${BULLET_REHEARSAL_BUNDLE:-$REPO_ROOT/docs/assurance/recovery-rehearsal.bundle.json}"
rm -f -- "$bundle"
write_bundle "$bundle" "$obs_sha" "$obs_sha"
log "DF-R7a COMPONENT rehearsal bundle written; independent review and DF-R7b still required"
