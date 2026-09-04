#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

readonly runbook='docs/runbooks/dogfood.md'
readonly decision='docs/decisions/0015-dogfood-track.md'

require_claim() {
  local file="$1" claim="$2"
  grep -Fq -- "$claim" "$file" \
    || { refuse DOGFOOD_DOC_TRUTH_MISSING "$file: $claim"; return 1; }
}

refuse_claim() {
  local file="$1" claim="$2"
  if grep -Fq -- "$claim" "$file"; then
    refuse DOGFOOD_DOC_STALE_CLAIM "$file: $claim"
    return 1
  fi
}

require_claim "$runbook" 'The current board is **diagnostic and blocked**'
require_claim "$runbook" "\`bullet dogfood read-only\`"
require_claim "$runbook" 'Exit 78 is neutral, never PASS'
require_claim "$runbook" "\`live_admission_enabled=true\` is refused by the dogfood path"
require_claim "$runbook" 'Do not run Fresh Genesis'
require_claim "$runbook" 'proposal pending reviewed amendment and ratification'

require_claim "$decision" 'Landed component surfaces and remaining gap'
require_claim "$decision" "\`DOGFOOD_RUN\` v0 template"
require_claim "$decision" "\`bullet-family check dogfood --json\`"
require_claim "$decision" "\`bullet dogfood read-only\` for Claude"
require_claim "$decision" "\`live_admission_enabled=true\` is refused by the dogfood validator"
require_claim "$decision" '**Operating HOLD:** do not run Fresh Genesis'
require_claim "$decision" 'that label is not accepted authority'

require_claim 'src/check/dogfood.rs' "const KIND: &'static str = \"DOGFOOD_RUN\""
require_claim 'src/check/profiles/graph.rs' '"NOT_A_RELEASE_PROFILE"'
require_claim 'crates/bullet-wire/src/policy/live.rs' '"DOGFOOD_REFUSES_LIVE_ADMISSION"'

refuse_claim "$decision" "\`DOGFOOD_RUN\` and \`dogfood-local-v0\` appear in no Rust source"
refuse_claim "$decision" 'No dogfood operational record exists yet'
refuse_claim "$decision" 'scripts/dogfood-board.py`, which always exits 0'
refuse_claim "$runbook" 'A valid board exits 0 with'

log 'dogfood documentation truth ratchet passed'
