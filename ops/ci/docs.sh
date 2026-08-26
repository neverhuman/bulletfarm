#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

log "docs lane: rustdoc, doctests, links, release truth, and README media"
readonly rustdoc_config='build.rustdocflags=["-Dwarnings"]'
cargo --config "$rustdoc_config" doc --locked --workspace --all-features --no-deps
cargo --config "$rustdoc_config" test --locked --workspace --doc
bash ops/ci/check-links.sh
bash scripts/release-truth.sh check

doctor_report="$(mktemp)"
cleanup() { rm -f -- "$doctor_report"; }
trap cleanup EXIT
doctor_status=0
cargo run --locked --quiet --bin bullet-family -- doctor --json >"$doctor_report" || doctor_status=$?
[[ "$doctor_status" -eq 3 ]] \
  || { refuse DOCTOR_STATUS_DRIFT "expected BLOCKED exit 3, found $doctor_status"; exit 1; }
bash ops/ci/strict-json.sh "$doctor_report" >/dev/null \
  || { refuse DOCTOR_JSON_INVALID "$doctor_report"; exit 1; }
jq -e '.status == "BLOCKED"' "$doctor_report" >/dev/null \
  || { refuse DOCTOR_VERDICT_DRIFT "doctor JSON is not BLOCKED"; exit 1; }
bash scripts/readme-check.sh
log "docs lane passed (release remains BLOCKED)"
