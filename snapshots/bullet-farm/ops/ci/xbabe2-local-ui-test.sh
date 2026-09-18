#!/usr/bin/env bash
# Dispatcher component tests use fake children only; no browser/TUI qualification.
set -euo pipefail
repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
umask 077
fixture="$(mktemp -d)"
cleanup() {
  local status=$?
  printf '[ci] retained local UI component fixture: %s (exit %s)\n' "$fixture" "$status" >&2
}
trap cleanup EXIT
mkdir -p "$fixture/family/bullet-farm/scripts" "$fixture/bin"
cp "$repo/scripts/xbabe2-local-ui-ci.sh" "$fixture/family/bullet-farm/scripts/"
cat >"$fixture/bin/hostname" <<'HOST'
#!/usr/bin/env bash
printf '%s\n' "${FIXTURE_HOST:-xbabe2}"
HOST
cat >"$fixture/bin/tuiwright" <<'TOOL'
#!/usr/bin/env bash
printf 'unqualified tool probe\n' >>"$FIXTURE_TOOL_LOG"
exit 0
TOOL
cat >"$fixture/bin/uname" <<'SYSTEM'
#!/usr/bin/env bash
[[ "$*" == -s ]] || exit 94
printf '%s\n' "${FIXTURE_SYSTEM:-Linux}"
SYSTEM
chmod 700 "$fixture/bin/hostname" "$fixture/bin/tuiwright" "$fixture/bin/uname"
passed=0
run_case() {
  local expected="$1" selector="$2"; shift 2
  local status=0
  case_log="$fixture/route-$((passed + 1)).log"
  env -u CI -u GITHUB_ACTIONS PATH="$fixture/bin:$PATH" XBABE2_LOCAL_UI_CI=1 \
    XBABE2_ALLOW_ANY_HOST=1 FIXTURE_TOOL_LOG="$fixture/tool-called" \
    BULLET_TUIWRIGHT_CARGO_BIN="$fixture/admitted tools/cargo" \
    BULLET_TUIWRIGHT_RUSTC_BIN="$fixture/admitted tools/rustc" \
    BULLET_TUIWRIGHT_BULLET_BIN="$fixture/admitted tools/bullet" \
    BULLET_TUIWRIGHT_CARGO_SHA256="$(printf '1%.0s' {1..64})" \
    BULLET_TUIWRIGHT_RUSTC_SHA256="$(printf '2%.0s' {1..64})" \
    BULLET_TUIWRIGHT_BULLET_SHA256="$(printf '3%.0s' {1..64})" \
    BULLET_TUIWRIGHT_SOURCE_SHA256="$(printf '4%.0s' {1..64})" \
    BULLET_TUIWRIGHT_OUTPUT="$fixture/component output" FIXTURE_ROOT="$fixture" "$@" \
    bash "$fixture/family/bullet-farm/scripts/xbabe2-local-ui-ci.sh" "$selector" \
    >"$case_log" 2>&1 || status=$?
  [[ "$status" == "$expected" ]] || { cat "$case_log" >&2; return 1; }
  [[ ! -e "$fixture/tool-called" ]]
  if grep -Eq ': PASS|skip registry hit' "$case_log"; then return 1; fi
  ((passed += 1))
  printf 'route-%s\t%s\t%s\n' "$passed" "$selector" "$status" >>"$fixture/results.tsv"
}
run_case 78 all
grep -Fq TUIWRIGHT_SUITE_UNAVAILABLE "$case_log"
run_case 78 tuiwright
run_case 78 all CI=true
run_case 78 all GITHUB_ACTIONS=1
run_case 78 all FIXTURE_HOST=not-xbabe2
run_case 78 all FIXTURE_SYSTEM=Darwin
run_case 78 all XBABE2_LOCAL_UI_CI=0
run_case 78 rendered
grep -Fq PLAYWRIGHT_SUITE_UNAVAILABLE "$case_log"
portal="$fixture/family/bullet-portal"
mkdir -p "$portal/scripts" "$portal/ops/ci"
touch "$portal/ops/ci/rendered.sh"
cat >"$portal/scripts/ci-local.sh" <<'CHILD'
#!/usr/bin/env bash
[[ "$*" == rendered ]] || exit 91
printf '%s\n' "$*" >>"$FIXTURE_CHILD_LOG"
exit "${FIXTURE_CHILD_STATUS:-0}"
CHILD
run_case 24 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=24
[[ "$(cat "$fixture/selected")" == rendered ]]
run_case 0 rendered FIXTURE_CHILD_LOG="$fixture/selected" FIXTURE_CHILD_STATUS=0
run_case 0 --list
grep -Fq 'tuiwright: Kernel scripts/ci-local.sh operator-tui' "$case_log"
run_case 2 invalid
run_case 78 all CI=false
run_case 78 all CI=
run_case 78 all GITHUB_ACTIONS=false
run_case 78 all GITHUB_ACTIONS=
kernel="$fixture/family/bullet-kernel"
mkdir -p "$kernel/scripts" "$kernel/ops/ci"
touch "$kernel/ops/ci/operator-tui.sh"
cat >"$kernel/scripts/ci-local.sh" <<'CHILD'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" == operator-tui ]] || exit 92
[[ "$BULLET_TUIWRIGHT_CARGO_BIN" == "$FIXTURE_ROOT/admitted tools/cargo" ]]
[[ "$BULLET_TUIWRIGHT_RUSTC_BIN" == "$FIXTURE_ROOT/admitted tools/rustc" ]]
[[ "$BULLET_TUIWRIGHT_BULLET_BIN" == "$FIXTURE_ROOT/admitted tools/bullet" ]]
[[ "$BULLET_TUIWRIGHT_CARGO_SHA256" == "$(printf '1%.0s' {1..64})" ]]
[[ "$BULLET_TUIWRIGHT_RUSTC_SHA256" == "$(printf '2%.0s' {1..64})" ]]
[[ "$BULLET_TUIWRIGHT_BULLET_SHA256" == "$(printf '3%.0s' {1..64})" ]]
[[ "$BULLET_TUIWRIGHT_SOURCE_SHA256" == "$(printf '4%.0s' {1..64})" ]]
[[ "$BULLET_TUIWRIGHT_OUTPUT" == "$FIXTURE_ROOT/component output" ]]
printf '%s\n' "$*" >>"$FIXTURE_CHILD_LOG"
exit "${FIXTURE_TUI_STATUS:-0}"
CHILD
run_case 78 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-unavailable" FIXTURE_TUI_STATUS=78
[[ "$(cat "$fixture/tui-unavailable")" == operator-tui ]]
run_case 31 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-failed" FIXTURE_TUI_STATUS=31
run_case 0 tuiwright FIXTURE_CHILD_LOG="$fixture/tui-only"
[[ "$(cat "$fixture/tui-only")" == operator-tui ]]
run_case 31 all FIXTURE_CHILD_LOG="$fixture/all-tui-failed" FIXTURE_TUI_STATUS=31
[[ "$(cat "$fixture/all-tui-failed")" == operator-tui ]]
run_case 24 all FIXTURE_CHILD_LOG="$fixture/all-rendered-failed" FIXTURE_CHILD_STATUS=24
[[ "$(cat "$fixture/all-rendered-failed")" == $'operator-tui\nrendered' ]]
run_case 0 all FIXTURE_CHILD_LOG="$fixture/all-complete"
[[ "$(cat "$fixture/all-complete")" == $'operator-tui\nrendered' ]]
printf '[ci] local UI dispatcher: %s refusal/routing cases passed; no browser or terminal qualification\n' "$passed"

# Preserve the reviewed Head refusal regression in this canonical focused suite.
head_passed=0
hub="$fixture/family/bullet-farm"
mkdir -p "$hub/scripts" "$hub/ops/ci" "$fixture/bin" "$fixture/state/receipts"
cp "$repo/scripts/xbabe2-head.sh" "$hub/scripts/"
cp "$repo/ops/ci/lib.sh" "$repo/ops/ci/artifact-path.sh" \
  "$repo/ops/ci/rust-toolchain-boundary.sh" "$hub/ops/ci/"
cat >"$fixture/bin/hostname" <<'HOST'
#!/usr/bin/env bash
printf '%s\n' "${FIXTURE_HOST:-xbabe2}"
HOST
chmod 700 "$fixture/bin/hostname"
# These are the old cache-key inputs for absent member checkouts. No candidate
# or transaction is created; the deliberately fabricated file must never pass.
key="$(printf 'missing\nmissing\nhead-doors\n27\nnone' | sha256sum)"
key="${key%% *}"
receipt="$fixture/state/receipts/$key"
run_head_case() {
  local name="$1" expected="$2" status=0
  shift 2
  env -u CI -u GITHUB_ACTIONS PATH="$fixture/bin:$PATH" \
    JANKURAI_CHECKOUT="$fixture/absent-jankurai" \
    BULLET_XBABE2_HEAD_STATE="$fixture/state" "$@" \
    bash "$hub/scripts/xbabe2-head.sh" >"$fixture/$name.log" 2>&1 || status=$?
  [[ "$status" == 78 ]] || {
    printf '[ci] HEAD_REFUSAL_REGRESSION: %s exit=%s\n' "$name" "$status" >&2
    cat "$fixture/$name.log" >&2
    return 1
  }
  grep -Fq "$expected" "$fixture/$name.log"
  if grep -Eq 'skip registry hit|: PASS' "$fixture/$name.log"; then return 1; fi
  ((head_passed += 1))
  printf 'head-%s\t%s\t%s\n' "$head_passed" "$name" "$status" >>"$fixture/results.tsv"
}
run_head_case absent-receipt HEAD_RUNTIME_BINDING_REQUIRED
printf 'this is not an executed or admitted proof\n' >"$receipt"
chmod 600 "$receipt"
before="$(sha256sum "$receipt")"
run_head_case forged-private-receipt HEAD_RUNTIME_BINDING_REQUIRED
[[ "$(sha256sum "$receipt")" == "$before" ]]
mv "$receipt" "$fixture/retained-receipt"
ln -s "$fixture/retained-receipt" "$receipt"
run_head_case symlink-receipt HEAD_RUNTIME_BINDING_REQUIRED
[[ -L "$receipt" ]]
[[ "$(cat "$fixture/retained-receipt")" == 'this is not an executed or admitted proof' ]]
run_head_case ci-refusal XBABE2_PROVIDER_PROOF_UNAVAILABLE CI=true
run_head_case actions-refusal XBABE2_PROVIDER_PROOF_UNAVAILABLE GITHUB_ACTIONS=true
run_head_case other-host XBABE2_PROVIDER_PROOF_UNAVAILABLE FIXTURE_HOST=foreign
run_head_case other-system XBABE2_PROVIDER_PROOF_UNAVAILABLE FIXTURE_SYSTEM=Darwin
run_head_case ci-empty XBABE2_PROVIDER_PROOF_UNAVAILABLE CI=
run_head_case actions-empty XBABE2_PROVIDER_PROOF_UNAVAILABLE GITHUB_ACTIONS=
run_head_case ci-false XBABE2_PROVIDER_PROOF_UNAVAILABLE CI=false
run_head_case actions-false XBABE2_PROVIDER_PROOF_UNAVAILABLE GITHUB_ACTIONS=false
printf '[ci] Head proof: %s receipt/environment refusals passed; no live qualification\n' "$head_passed"
