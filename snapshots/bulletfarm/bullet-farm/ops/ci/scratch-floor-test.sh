#!/usr/bin/env bash
# shellcheck disable=SC2317 # the helpers redefined below are invoked indirectly, by the code under test
# shellcheck disable=SC2030,SC2031 # each override is deliberately confined to its subshell so no case leaks into the next
# shellcheck disable=SC2016 # the source guards match ci-local.sh's literal text, which must not expand here
set -euo pipefail

# Every case below runs entirely inside a mktemp -d fixture. No real scratch
# root and no real repository is named, read or written, nothing outside the
# fixture is created or removed, and scripts/ci-local.sh is never executed: it
# cd's to the real hub and would take the real proof-custody lock.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=ops/ci/scratch-floor.sh
source "$REPO_ROOT/ops/ci/scratch-floor.sh"
# shellcheck source=ops/ci/family-custody.sh
source "$REPO_ROOT/ops/ci/family-custody.sh"

fixture="$(mktemp -d)"
cleanup() { rm -rf -- "$fixture"; }
trap cleanup EXIT

scratch="$fixture/scratch"
mirror_root="$fixture/mirror/bullet-farm"
mkdir -p "$scratch" "$mirror_root/.git"
printf '%s\n' 'ref: refs/heads/main' >"$mirror_root/.git/HEAD"

fail() { printf 'scratch-floor-test: %s\n' "$1" >&2; exit 1; }

# A refusal is exactly status 69 carrying exactly the named reason code. The
# code is matched with its trailing colon so that CI_PROOF_DISK_FLOOR can never
# be satisfied by CI_PROOF_DISK_FLOOR_BOUND.
expect_refusal() {
  local code="$1" status
  shift
  set +e
  "$@" >"$fixture/refusal.out" 2>&1
  status=$?
  set -e
  [[ "$status" -eq 69 ]] || fail "expected 69 from '$*', got $status"
  grep -q -F -e "ci-local: $code:" "$fixture/refusal.out" \
    || fail "expected reason $code from '$*', got: $(cat "$fixture/refusal.out")"
  grep -q -F -e 'nothing was locked and nothing was built' "$fixture/refusal.out" \
    || fail "refusal from '$*' did not say what it left behind"
}

expect_admitted() {
  "$@" || fail "expected admission from '$*'"
}

# ------------------------------------------------------- the floor is a floor
# Free space is injected in a subshell so the boundary is exact and does not
# depend on the host's real free space. The real measurement path is proved
# separately, immediately below.
floor_kib=$((CI_SCRATCH_FLOOR_MIB * 1024))

( ci_scratch_free_kib() { printf '%s\n' "$floor_kib"; }
  expect_admitted ci_scratch_floor_check "$scratch"
  [[ "$CI_SCRATCH_EFFECTIVE_FLOOR_MIB" -eq "$CI_SCRATCH_FLOOR_MIB" ]] || fail 'floor not reported'
  [[ "$CI_SCRATCH_FREE_KIB" -eq "$floor_kib" ]] || fail 'free space not reported'
  [[ "$CI_SCRATCH_FREE_SUBJECT" == "$scratch" ]] || fail 'subject not reported' )

( ci_scratch_free_kib() { printf '%s\n' "$((floor_kib - 1))"; }
  expect_refusal CI_PROOF_DISK_FLOOR ci_scratch_floor_check "$scratch" )

( ci_scratch_free_kib() { printf '%s\n' "$((floor_kib + 1))"; }
  expect_admitted ci_scratch_floor_check "$scratch" )

( ci_scratch_free_kib() { printf '%s\n' 0; }
  expect_refusal CI_PROOF_DISK_FLOOR ci_scratch_floor_check "$scratch" )

# The scarcest subject decides, and it is the one named in the refusal: a lane
# whose target is roomy but whose TMPDIR is not must still refuse.
( ci_scratch_free_kib() {
    case "$1" in
      "$scratch") printf '%s\n' "$((floor_kib * 4))" ;;
      *) printf '%s\n' 4096 ;;
    esac
  }
  expect_refusal CI_PROOF_DISK_FLOOR ci_scratch_floor_check "$scratch" "$fixture"
  grep -q -F -e "$fixture has 4 MiB free" "$fixture/refusal.out" || fail 'scarcest subject not named' )

# ------------------------------------------------------ the real measurement
real_free="$(ci_scratch_free_kib "$fixture")" || fail 'the real df path failed'
[[ "$real_free" =~ ^[0-9]+$ ]] || fail "the real df path returned '$real_free'"
# A target directory a lane has not created yet is measured on its nearest
# existing ancestor rather than refused.
[[ "$(ci_scratch_existing_ancestor "$fixture/absent/target/debug")" == "$fixture" ]] \
  || fail 'the ancestor walk did not reach the fixture'
[[ "$(ci_scratch_existing_ancestor "$scratch")" == "$scratch" ]] \
  || fail 'the ancestor walk moved an existing directory'
# The path handed to df is the resolved ancestor, not the absent subject.
( ci_scratch_free_kib() {
    [[ "$1" == "$fixture" ]] || fail "the ancestor walk handed df '$1'"
    printf '%s\n' "$((floor_kib * 2))"
  }
  expect_admitted ci_scratch_floor_check "$fixture/absent/target" )

expect_refusal CI_PROOF_DISK_FLOOR_SUBJECT ci_scratch_floor_check
expect_refusal CI_PROOF_DISK_FLOOR_SUBJECT ci_scratch_floor_check relative/path
expect_refusal CI_PROOF_DISK_FLOOR_SUBJECT ci_scratch_floor_check "$scratch" relative/path
( ci_scratch_free_kib() { return 1; }
  expect_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED ci_scratch_floor_check "$scratch" )
( ci_scratch_free_kib() { printf '%s\n' 'not-a-number'; }
  expect_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED ci_scratch_floor_check "$scratch" )
( ci_scratch_free_kib() { printf '%s\n' '8388608 ; rm -rf /'; }
  expect_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED ci_scratch_floor_check "$scratch" )

# The real measurement path against a hostile df: a shim on PATH stands in for
# a df that fails, prints a header only, or answers with something that is not
# a number. Each one is unreadable free space, never an admission.
shim="$fixture/bin"
mkdir -p "$shim"
for shape in fail header garbage negative; do
  case "$shape" in
    fail) printf '%s\n' '#!/usr/bin/env bash' 'exit 1' >"$shim/df" ;;
    header) printf '%s\n' '#!/usr/bin/env bash' \
      "printf '%s\\n' 'Filesystem 1024-blocks Used Available Capacity Mounted on'" >"$shim/df" ;;
    garbage) printf '%s\n' '#!/usr/bin/env bash' \
      "printf '%s\\n' 'head' '/dev/x 1 2 lots 98% /'" >"$shim/df" ;;
    negative) printf '%s\n' '#!/usr/bin/env bash' \
      "printf '%s\\n' 'head' '/dev/x 1 2 -5 98% /'" >"$shim/df" ;;
  esac
  chmod 755 "$shim/df"
  ( PATH="$shim:$PATH"
    ci_scratch_free_kib "$scratch" >/dev/null 2>&1 \
      && fail "a $shape df was accepted as a measurement"
    expect_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED ci_scratch_floor_check "$scratch" )
done
# A well-formed df shim is read correctly, so the shim harness itself is honest.
printf '%s\n' '#!/usr/bin/env bash' \
  "printf '%s\\n' 'head' '/dev/x 1 2 33554432 98% /'" >"$shim/df"
chmod 755 "$shim/df"
( PATH="$shim:$PATH"
  [[ "$(ci_scratch_free_kib "$scratch")" -eq 33554432 ]] || fail 'the df shim harness cannot read a good answer'
  expect_admitted ci_scratch_floor_check "$scratch" )
# ------------------------------------------------------------- override bounds
# The override may tune the policy and may never disable it. Every rejected
# value refuses outright; none of them silently falls back to the default, which
# is proved by injecting free space far above the default floor so that a
# fallback would have been admitted.
for rejected in 0 1 2047 262145 9999999 '' ' 8192' '8192 ' '08192' '+8192' '-1' '8192abc' 'abc' '2048+1' '0x2000' '8192
16384'; do
  ( export CI_SCRATCH_FLOOR_MIB_OVERRIDE="$rejected"
    ci_scratch_free_kib() { printf '%s\n' "$((floor_kib * 16))"; }
    expect_refusal CI_PROOF_DISK_FLOOR_BOUND ci_scratch_floor_check "$scratch" )
done

for accepted in 2048 8192 9999 262144; do
  ( export CI_SCRATCH_FLOOR_MIB_OVERRIDE="$accepted"
    ci_scratch_free_kib() { printf '%s\n' "$((accepted * 1024))"; }
    expect_admitted ci_scratch_floor_check "$scratch"
    [[ "$CI_SCRATCH_EFFECTIVE_FLOOR_MIB" -eq "$accepted" ]] \
      || fail "override $accepted was not the effective floor" )
done

# An accepted override really is enforced, not merely echoed: one KiB below the
# raised floor refuses even though the default floor would have admitted, and
# one KiB below a lowered floor refuses too.
( export CI_SCRATCH_FLOOR_MIB_OVERRIDE=16384
  ci_scratch_free_kib() { printf '%s\n' "$((16384 * 1024 - 1))"; }
  expect_refusal CI_PROOF_DISK_FLOOR ci_scratch_floor_check "$scratch" )
( export CI_SCRATCH_FLOOR_MIB_OVERRIDE=2048
  ci_scratch_free_kib() { printf '%s\n' "$((2048 * 1024 - 1))"; }
  expect_refusal CI_PROOF_DISK_FLOOR ci_scratch_floor_check "$scratch" )
( export CI_SCRATCH_FLOOR_MIB_OVERRIDE=2048
  ci_scratch_free_kib() { printf '%s\n' "$((2048 * 1024))"; }
  expect_admitted ci_scratch_floor_check "$scratch" )

# ------------------------------------ refusal precedes every acquisition/build
# This mirrors scripts/ci-local.sh's run_observed using the real sourced helpers
# against a fixture repository; the source guards further down pin the real file
# to the same shape.
mirror_run_observed() {
  local lane="$1" status record=''
  ci_scratch_floor_check "$mirror_root/target" "$scratch" || return $?
  ci_proof_acquire "$mirror_root" bullet-farm standalone "$lane" record || return $?
  printf 'built\n' >"$fixture/mirror-built"
  status=0
  ci_proof_release "$mirror_root" bullet-farm "$record" standalone || return $?
  return "$status"
}

set +e
( ci_scratch_free_kib() { printf '%s\n' 1024; }; mirror_run_observed fast ) >"$fixture/mirror.out" 2>&1
mirror_status=$?
set -e
[[ "$mirror_status" -eq 69 ]] || fail "a below-floor lane returned $mirror_status, expected 69"
[[ ! -e "$fixture/mirror-built" ]] || fail 'a below-floor lane built something'
[[ ! -e "$mirror_root/.git/bullet-ci.lock.d" ]] || fail 'a below-floor lane acquired the proof lock'
grep -q -F -e 'ci-local: CI_PROOF_DISK_FLOOR:' "$fixture/mirror.out" || fail 'no typed refusal from the lane'

# A refused floor never becomes success, even when the refusal is the only thing
# that went wrong: L2's precedence rule applied to this gate.
set +e
( export CI_SCRATCH_FLOOR_MIB_OVERRIDE=262144
  mirror_run_observed fast ) >"$fixture/mirror-bound.out" 2>&1
mirror_status=$?
set -e
[[ "$mirror_status" -eq 69 ]] || fail "a raised floor returned $mirror_status, expected 69"
[[ ! -e "$mirror_root/.git/bullet-ci.lock.d" ]] || fail 'a refused lane left a lock behind'

set +e
( ci_scratch_free_kib() { printf '%s\n' "$((floor_kib * 2))"; }; mirror_run_observed fast ) \
  >"$fixture/mirror-ok.out" 2>&1
mirror_status=$?
set -e
[[ "$mirror_status" -eq 0 ]] || fail "an above-floor lane returned $mirror_status"
[[ -f "$fixture/mirror-built" ]] || fail 'an admitted lane did not build'
[[ ! -e "$mirror_root/.git/bullet-ci.lock.d" ]] || fail 'an admitted lane leaked its lock'

# ------------------------------------------------ source guards on ci-local.sh
ci_local="$REPO_ROOT/scripts/ci-local.sh"
grep -q -F -e 'source "$REPO_ROOT/ops/ci/scratch-floor.sh"' "$ci_local" \
  || fail 'ci-local.sh does not source the floor helper'
[[ "$(grep -c -F -e 'ci_scratch_floor_check ' "$ci_local")" -eq 1 ]] \
  || fail 'the floor check is not called exactly once'
# Call order, not file order: run_observed_locked is defined above run_observed,
# so the guard reads the one function that decides what happens first.
run_observed_body="$(sed -n '/^run_observed() {$/,/^}$/p' "$ci_local")"
[[ -n "$run_observed_body" ]] || fail 'run_observed is no longer a top-level function'
floor_index="$(grep -n -F -e 'ci_scratch_floor_check ' <<<"$run_observed_body" | cut -d: -f1)"
acquire_index="$(grep -n -F -e 'acquire_proof_lock "$lane"' <<<"$run_observed_body" | cut -d: -f1)"
locked_index="$(grep -n -F -e 'run_observed_locked "$@"' <<<"$run_observed_body" | cut -d: -f1)"
[[ -n "$floor_index" && -n "$acquire_index" && -n "$locked_index" ]] \
  || fail 'run_observed no longer checks the floor, acquires and runs the lane'
[[ "$floor_index" -lt "$acquire_index" ]] || fail 'the floor check does not precede acquisition'
[[ "$acquire_index" -lt "$locked_index" ]] || fail 'acquisition does not precede the locked body'
# The doctor, and therefore every build, is reachable only through the locked
# body that acquisition guards.
[[ "$(grep -c -F -e 'bash scripts/ci-doctor.sh "$lane"' "$ci_local")" -eq 1 ]] \
  || fail 'the doctor is invoked from more than one place'
locked_body="$(sed -n '/^run_observed_locked() {$/,/^}$/p' "$ci_local")"
grep -q -F -e 'bash scripts/ci-doctor.sh "$lane"' <<<"$locked_body" \
  || fail 'the doctor escaped run_observed_locked'
grep -q -F -e 'ci_scratch_floor_check "$REPO_ROOT/target" "${TMPDIR:-/tmp}" || return $?' "$ci_local" \
  || fail 'the floor check does not return its status unchanged'

# L2's four custody traps must survive this lane untouched, and the lane arms
# must all still dispatch through the one run_observed that carries the gate.
[[ "$(grep -c -F -e 'trap ' "$ci_local")" -eq 4 ]] || fail 'the four custody traps changed'
for handler in 'trap ci_proof_custody_exit EXIT' "trap 'ci_proof_custody_signal 129' HUP" \
  "trap 'ci_proof_custody_signal 130' INT" "trap 'ci_proof_custody_signal 143' TERM"; do
  grep -q -F -e "$handler" "$ci_local" || fail "a custody trap was lost: $handler"
done
case_block="$(sed -n '/^case "\$lane" in$/,/^esac$/p' "$ci_local")"
arm_total="$(grep -c -E '^  [a-z][a-z-]*\)' <<<"$case_block")"
arm_dispatch="$(grep -c -E '^  [a-z][a-z-]*\) run_observed ' <<<"$case_block")"
[[ "$arm_total" -eq 17 && "$arm_dispatch" -eq 17 ]] \
  || fail "lane arms bypass run_observed ($arm_dispatch of $arm_total)"

# ------------------------------------------------- the helper never mutates
# This lane is admission only. A disabled deletion path is still a deletion
# path someone will enable later, so the helper is pinned to having none.
helper="$REPO_ROOT/ops/ci/scratch-floor.sh"
# Comment-only lines are prose about mutation, not mutation; the guard reads the
# code. The orchestrator's own check is the stricter whole-file form and is
# transcribed in the handoff.
helper_code="$(grep -v -E -e '^[[:space:]]*#' "$helper")"
mutation="$(grep -n -E -e '\brm\b|rmdir|chmod|chown|unlink|mkdir|truncate|mv |tee |>>' <<<"$helper_code" || true)"
[[ -z "$mutation" ]] || fail "the floor helper gained a mutation: $mutation"
for verb in mktemp 'du ' 'find ' rsync 'install ' 'dd '; do
  [[ "$(grep -c -F -e "$verb" <<<"$helper_code")" -eq 0 ]] || fail "the floor helper gained '$verb'"
done
# The helper redirects onto stderr and onto /dev/null, and never into a path.
[[ "$(grep -c -E -e '>[[:space:]]*("|'"'"'|\$|/[^d])' <<<"$helper_code")" -eq 0 ]] \
  || fail 'the floor helper redirects into a path'
grep -q -F -e '>&2' <<<"$helper_code" || fail 'the floor helper stopped writing its refusal to stderr'

printf '[ci] scratch floor admission fixture passed\n'
