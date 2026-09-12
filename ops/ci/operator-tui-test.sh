#!/usr/bin/env bash
# Component route/refusal tests only. This does not execute Cargo or Tuiwright.
set -euo pipefail
umask 077
harness=''
if [[ $# -ne 0 ]]; then
  [[ $# -eq 2 && $1 == --harness && $2 == /* && -f $2 && -x $2 && ! -L $2 ]] || exit 2
  harness="$2"
fi
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
evidence="$(mktemp -d "${TMPDIR:-/tmp}/bullet-operator-tui-route.XXXXXXXX")"
fixture="$evidence/repository"
mkdir -p "$fixture/scripts" "$fixture/ops/ci" "$fixture/.git" "$fixture/ops/qualification"
: >"$fixture/.git/HEAD"
cp "$root/scripts/ci-local.sh" "$fixture/scripts/"
cp "$root/ops/ci/"{operator-tui.sh,operator-tui-hosted.sh,lib.sh,inventory.sh} "$fixture/ops/ci/"
cp -R "$root/ops/qualification/tuiwright" "$fixture/ops/qualification/"
sha256sum "$root/scripts/ci-local.sh" "$root/ops/ci/operator-tui.sh" \
  "$root/ops/ci/operator-tui-test.sh" >"$evidence/source.sha256"
trap 'printf "operator-tui-test: evidence=%s\n" "$evidence"' EXIT

fail() { printf 'OPERATOR_TUI_ROUTE_TEST_FAILED: %s\n' "$*" >&2; exit 1; }
assert_refusal() {
  local name="$1" expected_status="$2" reason="$3" status=0
  shift 3
  env -u CI -u GITHUB_ACTIONS -u CARGO_TARGET_DIR -u BULLET_CI_CARGO_TARGET_DIR \
    -u BULLET_CI_CARGO_TARGET_ID -u BULLET_CI_PROOF_CUSTODY \
    -u BULLET_TUIWRIGHT_CARGO_BIN -u BULLET_TUIWRIGHT_CARGO_SHA256 \
    -u BULLET_TUIWRIGHT_RUSTC_BIN -u BULLET_TUIWRIGHT_RUSTC_SHA256 \
    -u BULLET_TUIWRIGHT_BULLET_BIN -u BULLET_TUIWRIGHT_BULLET_SHA256 \
    -u BULLET_TUIWRIGHT_PROFILE -u BULLET_TUIWRIGHT_WORKFLOW_SHA256 \
    -u BULLET_TUIWRIGHT_OUTPUT -u BULLET_TUIWRIGHT_SOURCE_SHA256 "$@" \
    >"$evidence/$name.stdout" 2>"$evidence/$name.stderr" || status=$?
  [[ "$status" -eq "$expected_status" ]] || fail "$name exit=$status expected=$expected_status"
  grep -Fq "$reason" "$evidence/$name.stderr" || fail "$name missing $reason"
  ! grep -Fq 'OPERATOR_TUI_COMPONENT_PASS' "$evidence/$name.stdout" || fail "$name claimed component pass"
  printf '%s\t%s\t%s\n' "$name" "$status" "$reason" >>"$evidence/refusals.tsv"
}

# Presence is what matters, including empty and false strings. Exercise the
# copied canonical dispatcher so a missing route is a failure, never a skip.
for variable in CI GITHUB_ACTIONS; do
  for value in '' false true; do
    assert_refusal "$variable-${value:-empty}" 78 OPERATOR_TUI_HOSTED_RUN_REFUSED \
      "$variable=$value" bash "$fixture/scripts/ci-local.sh" operator-tui
  done
done
# Hosted selection is explicit; malformed metadata refuses before any compiler.
assert_refusal hosted-helper-direct 78 OPERATOR_TUI_WRAPPER_REQUIRED \
  bash "$fixture/ops/ci/operator-tui-hosted.sh"
assert_refusal unknown-profile 78 OPERATOR_TUI_PROFILE_UNSUPPORTED \
  BULLET_TUIWRIGHT_PROFILE=unknown bash "$fixture/scripts/ci-local.sh" operator-tui
assert_refusal hosted-without-context 78 OPERATOR_TUI_HOSTED_CONTEXT_INVALID \
  BULLET_TUIWRIGHT_PROFILE=hosted-component bash "$fixture/scripts/ci-local.sh" operator-tui
hosted=(BULLET_TUIWRIGHT_BULLET_BIN=/missing BULLET_TUIWRIGHT_PROFILE=hosted-component CI=true GITHUB_ACTIONS=true RUNNER_OS=Linux)
if [[ "$(uname -s)" == Linux ]]; then
  for variable in CI GITHUB_ACTIONS RUNNER_OS; do
    assert_refusal "hosted-invalid-$variable" 78 OPERATOR_TUI_HOSTED_CONTEXT_INVALID \
      "${hosted[@]}" "$variable=false" bash "$fixture/scripts/ci-local.sh" operator-tui
  done
  assert_refusal hosted-missing-subjects 1 OPERATOR_TUI_HOSTED_SUBJECT_INVALID \
    "${hosted[@]}" GITHUB_RUN_ID= GITHUB_RUN_ATTEMPT= bash "$fixture/scripts/ci-local.sh" operator-tui
  # Fresh test repository, not a checkout/worktree of any product repository.
  rm "$fixture/.git/HEAD"
  git -c init.defaultBranch=main init -q "$fixture"
  mkdir -p "$fixture/.github/workflows"
  cp "$root/.github/workflows/ci.yml" "$fixture/.github/workflows/"
  printf '.ci-artifacts/\n' >"$fixture/.gitignore"
  git -C "$fixture" add .
  git -C "$fixture" -c user.name=fixture -c user.email=fixture@example.invalid \
    -c core.hooksPath=/dev/null commit -qm 'Synthetic admission refusal fixture'
  fixture_commit="$(git -C "$fixture" rev-parse HEAD)"
  workflow_sha="$(sha256sum "$fixture/.github/workflows/ci.yml" | cut -d ' ' -f1)"
  subjects=("GITHUB_WORKSPACE=$fixture" GITHUB_JOB=operator-tui GITHUB_EVENT_NAME=push GITHUB_RUN_ID=12 GITHUB_RUN_ATTEMPT=1
    "GITHUB_SHA=$fixture_commit" "GITHUB_WORKFLOW_SHA=$fixture_commit"
    GITHUB_REPOSITORY=fixture/component GITHUB_REF=refs/heads/main
    GITHUB_WORKFLOW_REF=fixture/component/.github/workflows/ci.yml@refs/heads/main
    "BULLET_TUIWRIGHT_WORKFLOW_SHA256=$workflow_sha")
  assert_refusal hosted-source-build-route 1 OPERATOR_TUI_RUNNER_TEMP_INVALID \
    "${hosted[@]}" "${subjects[@]}" RUNNER_TEMP=/missing env -u BULLET_TUIWRIGHT_BULLET_BIN \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  mkdir "$evidence/bullet-operator-tui-12-1"
  printf 'historical receipt\n' >"$evidence/bullet-operator-tui-12-1/receipt.json"
  assert_refusal hosted-occupied-stage 1 OPERATOR_TUI_OUTPUT_NOT_FRESH \
    "${hosted[@]}" "${subjects[@]}" "RUNNER_TEMP=$evidence" env -u BULLET_TUIWRIGHT_BULLET_BIN \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  [[ "$(<"$evidence/bullet-operator-tui-12-1/receipt.json")" == 'historical receipt' ]] || fail 'historical stage overwritten'
  # This pure source check does not acquire a compiler target. Importing lib.sh
  # would try to adopt the canonical parent proof target in this unrelated fixture.
  # shellcheck disable=SC2016 # Evaluated literally in the child shell.
  source_check='REPO_ROOT="$1"; refuse() { printf "%s: %s\n" "$1" "$2" >&2; return 1; }; source "$1/ops/ci/operator-tui-hosted.sh"; cd "$REPO_ROOT"; hosted_verify_source'
  CARGO_TARGET_DIR="$evidence/unrelated-parent-target" \
    bash -c "$source_check" _ "$fixture" || fail 'clean raw source rejected'
  git -C "$fixture" update-index --assume-unchanged ops/ci/inventory.sh
  printf '\n# hidden changed bytes\n' >>"$fixture/ops/ci/inventory.sh"
  [[ -z "$(git -C "$fixture" status --porcelain)" ]] || fail 'masked fixture not masked'
  assert_refusal hidden-source-bytes 1 OPERATOR_TUI_RAW_SOURCE_CHANGED \
    bash -c "$source_check" _ "$fixture"
  git -C "$fixture" update-index --no-assume-unchanged ops/ci/inventory.sh
  git -C "$fixture" checkout HEAD -- ops/ci/inventory.sh
  for event in push pull_request merge_group; do
    assert_refusal "hosted-admitted-$event" 1 OPERATOR_TUI_BINARY_INPUT_INVALID \
      "${hosted[@]}" "${subjects[@]}" "GITHUB_EVENT_NAME=$event" \
      bash "$fixture/scripts/ci-local.sh" operator-tui
  done
  for invalid in GITHUB_RUN_ID=0 GITHUB_RUN_ATTEMPT=01 GITHUB_EVENT_NAME=schedule \
    GITHUB_SHA=bad GITHUB_WORKFLOW_SHA=bad GITHUB_REPOSITORY=bad \
    GITHUB_WORKFLOW_REF=fixture/other/.github/workflows/ci.yml@refs/heads/main; do
    assert_refusal "hosted-${invalid%%=*}" 1 OPERATOR_TUI_HOSTED_SUBJECT_INVALID \
      "${hosted[@]}" "${subjects[@]}" "$invalid" bash "$fixture/scripts/ci-local.sh" operator-tui
  done
  for invalid in GITHUB_JOB=other GITHUB_WORKSPACE=/tmp; do
    assert_refusal "hosted-${invalid%%=*}" 1 OPERATOR_TUI_HOSTED_WORKSPACE_INVALID \
      "${hosted[@]}" "${subjects[@]}" "$invalid" bash "$fixture/scripts/ci-local.sh" operator-tui
  done
  zeros40="$(printf '%040d' 0)"; zeros64="$(printf '%064d' 0)"
  assert_refusal hosted-wrong-commit 1 OPERATOR_TUI_HOSTED_SOURCE_CHANGED \
    "${hosted[@]}" "${subjects[@]}" "GITHUB_SHA=$zeros40" bash "$fixture/scripts/ci-local.sh" operator-tui
  assert_refusal hosted-wrong-workflow 1 OPERATOR_TUI_HOSTED_WORKFLOW_CHANGED \
    "${hosted[@]}" "${subjects[@]}" "BULLET_TUIWRIGHT_WORKFLOW_SHA256=$zeros64" \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  assert_refusal hosted-absent-workflow-commit 1 OPERATOR_TUI_HOSTED_WORKFLOW_CHANGED \
    "${hosted[@]}" "${subjects[@]}" "GITHUB_WORKFLOW_SHA=$zeros40" \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  printf 'uncommitted source\n' >"$fixture/dirty"
  assert_refusal hosted-dirty-source 1 OPERATOR_TUI_HOSTED_SOURCE_CHANGED \
    "${hosted[@]}" "${subjects[@]}" bash "$fixture/scripts/ci-local.sh" operator-tui
fi
# Direct execution cannot bypass wrapper-owned target admission. This case is
# host-dependent only in refusal reason and invokes no qualified child.
if [[ "$(uname -s)" == Linux && "$(</proc/sys/kernel/hostname)" == xbabe2 ]]; then
  assert_refusal direct-without-custody 78 OPERATOR_TUI_WRAPPER_REQUIRED bash "$fixture/ops/ci/operator-tui.sh"
else
  assert_refusal off-host 78 OPERATOR_TUI_HOST_NOT_ADMITTED bash "$fixture/scripts/ci-local.sh" operator-tui
fi

# Inspect the closed executor path on every host. The full suite runs only on
# its admitted host and is never replaced by these diagnostic checks.
grep -Fq 'operator-tui) bash ops/ci/operator-tui.sh ;;' "$fixture/scripts/ci-local.sh" || fail 'canonical route absent'
# shellcheck disable=SC2016 # Match literal source, not test environment expansions.
grep -Fq '"$cargo_bin" build --frozen --release' "$fixture/ops/ci/operator-tui.sh" || fail 'frozen build absent'
# shellcheck disable=SC2016 # Match literal source, not test environment expansions.
grep -Fq '"$output/harness" "$profile" --bullet "$bullet_bin" --sha256 "$bullet_sha"' "$fixture/ops/ci/operator-tui.sh" || fail 'real component invocation absent'
# shellcheck disable=SC2016 # Match literal source, not test environment expansions.
grep -Fq '.selected == $selected[0] and [.completed[].id] == $selected[0]' "$fixture/ops/ci/operator-tui.sh" || fail 'selection/completion equality absent'

# Admission-negative fixtures: explicit host observation is real, never spoofed
# to execute a positive suite. No selected input can reach a compiler here.
if [[ "$(uname -s)" == Linux && "$(</proc/sys/kernel/hostname)" == xbabe2 ]]; then
  native="$(realpath -e -- "$(type -P true)")"
  native_sha="$(sha256sum "$native" | cut -d ' ' -f1)"
  options=("BULLET_TUIWRIGHT_CARGO_BIN=$native" "BULLET_TUIWRIGHT_CARGO_SHA256=$native_sha"
    "BULLET_TUIWRIGHT_RUSTC_BIN=$native" "BULLET_TUIWRIGHT_RUSTC_SHA256=$native_sha"
    "BULLET_TUIWRIGHT_BULLET_BIN=$native" "BULLET_TUIWRIGHT_BULLET_SHA256=$native_sha")
  zeros="$(printf '%064d' 0)"
  assert_refusal missing-inputs 1 OPERATOR_TUI_BINARY_INPUT_INVALID \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  assert_refusal wrong-native-digest 1 OPERATOR_TUI_BINARY_DIGEST_MISMATCH \
    "${options[@]}" "BULLET_TUIWRIGHT_CARGO_SHA256=$zeros" bash "$fixture/scripts/ci-local.sh" operator-tui
  printf '#!/bin/sh\nprintf executed >"%s"\n' "$evidence/must-not-execute" >"$evidence/probe.sh"
  chmod 0500 "$evidence/probe.sh"
  probe_sha="$(sha256sum "$evidence/probe.sh" | cut -d ' ' -f1)"
  assert_refusal script-probe-refused 1 OPERATOR_TUI_NATIVE_BINARY_REQUIRED \
    "${options[@]}" "BULLET_TUIWRIGHT_CARGO_BIN=$evidence/probe.sh" \
    "BULLET_TUIWRIGHT_CARGO_SHA256=$probe_sha" bash "$fixture/scripts/ci-local.sh" operator-tui
  [[ ! -e "$evidence/must-not-execute" ]] || fail 'script probe executed'
  mkdir "$evidence/prior-observation"
  printf 'historical receipt bytes\n' >"$evidence/prior-observation/manifest.json"
  assert_refusal stale-receipt 1 OPERATOR_TUI_OUTPUT_NOT_FRESH "${options[@]}" \
    "BULLET_TUIWRIGHT_OUTPUT=$evidence/prior-observation" bash "$fixture/scripts/ci-local.sh" operator-tui
  [[ "$(<"$evidence/prior-observation/manifest.json")" == 'historical receipt bytes' ]] || fail 'prior receipt overwritten'
  assert_refusal wrong-source 1 OPERATOR_TUI_SOURCE_DIGEST_MISMATCH "${options[@]}" \
    "BULLET_TUIWRIGHT_OUTPUT=$evidence/wrong-source" "BULLET_TUIWRIGHT_SOURCE_SHA256=$zeros" \
    bash "$fixture/scripts/ci-local.sh" operator-tui
  suite_sha="$(cd "$fixture" && find ops/qualification/tuiwright -type f -print0 \
    | LC_ALL=C sort -z | xargs -0 sha256sum | sha256sum | cut -d ' ' -f1)"
  # A native zero-exit noncompiler cannot qualify the lane: no fresh compiled
  # harness exists. This intentionally uses true, never Cargo or a fake suite.
  assert_refusal zero-exit-without-build 1 OPERATOR_TUI_BUILD_ARTIFACT_MISSING "${options[@]}" \
    "BULLET_TUIWRIGHT_OUTPUT=$evidence/zero-exit" "BULLET_TUIWRIGHT_SOURCE_SHA256=$suite_sha" \
    bash "$fixture/scripts/ci-local.sh" operator-tui
fi
printf 'operator-tui-test: PASS (route/refusals only; actual Tuiwright unexecuted)\n'

# This opt-in runs the real compiled Rust validator, never a shell stand-in.
# Its expected refusals stop before any PTY/fixture launch or evidence creation.
if [[ -n "$harness" ]]; then
  harness_sha="$(sha256sum "$harness" | cut -d ' ' -f1)"
  native_args=(hosted-component --bullet /missing --sha256 "$zeros64" --output "$evidence/must-not-exist")
  native_subjects=("${subjects[@]}" "GITHUB_WORKSPACE=$root")
  assert_refusal native-default-hosted 1 TUIWRIGHT_HOSTED_RUN_REFUSED \
    CI=true "$harness" component --bullet /missing --sha256 "$zeros64" --output "$evidence/must-not-exist"
  for invalid in CI=false GITHUB_ACTIONS=false RUNNER_OS=false; do
    assert_refusal "native-${invalid%%=*}" 1 TUIWRIGHT_HOSTED_CONTEXT_INVALID \
      "${hosted[@]}" "${native_subjects[@]}" "$invalid" "$harness" "${native_args[@]}"
  done
  for invalid in GITHUB_RUN_ID=0 GITHUB_RUN_ATTEMPT=01 GITHUB_EVENT_NAME=schedule \
    GITHUB_SHA=bad GITHUB_WORKFLOW_SHA=bad GITHUB_REPOSITORY=bad \
    GITHUB_WORKFLOW_REF=wrong BULLET_TUIWRIGHT_WORKFLOW_SHA256=bad; do
    assert_refusal "native-${invalid%%=*}" 1 TUIWRIGHT_HOSTED_SUBJECT_INVALID \
      "${hosted[@]}" "${native_subjects[@]}" "$invalid" "$harness" "${native_args[@]}"
  done
  for invalid in GITHUB_JOB=other GITHUB_WORKSPACE=/tmp; do
    assert_refusal "native-${invalid%%=*}" 1 TUIWRIGHT_HOSTED_WORKSPACE_INVALID \
      "${hosted[@]}" "${native_subjects[@]}" "$invalid" "$harness" "${native_args[@]}"
  done
  for event in push pull_request merge_group; do
    assert_refusal "native-exact-source-$event" 1 TUIWRIGHT_HOSTED_SOURCE_CHANGED \
      "${hosted[@]}" "${native_subjects[@]}" "GITHUB_EVENT_NAME=$event" "$harness" "${native_args[@]}"
  done
  [[ ! -e "$evidence/must-not-exist" && "$(sha256sum "$harness" | cut -d ' ' -f1)" == "$harness_sha" ]] \
    || fail 'native refusal altered artifact subjects'
  printf 'operator-tui-test: actual Rust profile refusals PASS; component scenarios NOT_RUN\n'
fi
