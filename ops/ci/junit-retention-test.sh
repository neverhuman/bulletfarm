#!/usr/bin/env bash
# Exercise actual partition dispatch and retention with a synthetic test runner.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
origin="$REPO_ROOT"
umask 077
fixture="$(mktemp -d)"
cleanup() {
  local status=$?
  if (( status == 0 )); then rm -rf -- "$fixture";
  else printf '[ci] retained failed JUnit fixture: %s\n' "$fixture" >&2; fi
}
trap cleanup EXIT
make_case() {
  REPO_ROOT="$fixture/$1"
  mkdir -p "$REPO_ROOT/ops/ci"
  cp "$origin/ops/ci/junit-retention.sh" "$origin/ops/ci/sanitize-junit.sh" "$REPO_ROOT/ops/ci/"
  prepare_junit_store fast fast
  printf 'original failed raw report\n' >"$REPO_ROOT/target/nextest/fast/junit.xml"
  printf 'original published failure\n' >"$REPO_ROOT/.ci-artifacts/junit/fast.xml"
}
partition_count() { printf '1\n'; }
cargo() {
  printf 'launched\n' >>"$REPO_ROOT/launched"
  cat >"$REPO_ROOT/target/nextest/fast/junit.xml" <<'XML'
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="fixture" tests="1" failures="0" errors="0">
  <testsuite name="suite" tests="1" failures="0" errors="0">
    <testcase name="case" classname="suite" time="0.1"/>
  </testsuite>
</testsuites>
XML
  return "${RUNNER_STATUS:-0}"
}
assert_retained() {
  local raw published
  raw="$(find "$REPO_ROOT/.ci-artifacts/junit/history" -name raw.xml -print -quit)"
  published="$(find "$REPO_ROOT/.ci-artifacts/junit/history" -name published.xml -print -quit)"
  [[ "$(cat "$raw")" == 'original failed raw report' ]]
  [[ "$(cat "$published")" == 'original published failure' ]]
  (cd "$(dirname "$raw")"; sha256sum --check SHA256SUMS)
}
CI_CARGO_TARGET_ADMITTED=false
make_case successful-retry
run_partition_tests fast fast 1 fixture
assert_retained
[[ -s "$REPO_ROOT/.ci-artifacts/junit/fast.xml" ]]
make_case failed-new-run
RUNNER_STATUS=42
status=0
(run_partition_tests fast fast 1 fixture) || status=$?
[[ "$status" == 42 ]]
assert_retained
unset RUNNER_STATUS
make_case first-run
rm -- "$REPO_ROOT/target/nextest/fast/junit.xml" "$REPO_ROOT/.ci-artifacts/junit/fast.xml"
run_partition_tests fast fast 1 fixture
[[ ! -e "$REPO_ROOT/.ci-artifacts/junit/history" ]]
for hostile in raw-symlink raw-hardlink public-mode history-symlink; do
  make_case "$hostile"
  case "$hostile" in
    raw-symlink) mv "$REPO_ROOT/target/nextest/fast/junit.xml" "$REPO_ROOT/outside"
      ln -s "$REPO_ROOT/outside" "$REPO_ROOT/target/nextest/fast/junit.xml" ;;
    raw-hardlink) ln "$REPO_ROOT/target/nextest/fast/junit.xml" "$REPO_ROOT/peer" ;;
    public-mode) chmod 644 "$REPO_ROOT/.ci-artifacts/junit/fast.xml" ;;
    history-symlink) mkdir "$REPO_ROOT/outside"; ln -s "$REPO_ROOT/outside" "$REPO_ROOT/.ci-artifacts/junit/history" ;;
  esac
  status=0
  (run_partition_tests fast fast 1 fixture) >"$REPO_ROOT/result.log" 2>&1 || status=$?
  [[ "$status" != 0 && ! -e "$REPO_ROOT/launched" ]]
  [[ "$(cat "$REPO_ROOT/target/nextest/fast/junit.xml")" == 'original failed raw report' ]]
  [[ "$(cat "$REPO_ROOT/.ci-artifacts/junit/fast.xml")" == 'original published failure' ]]
done
make_case partial-move-failure
# Invoked by retain_junit_reports in the sourced production helper.
# shellcheck disable=SC2317
mv() { if [[ "$3" == */junit/fast.xml ]]; then return 73; fi; command mv "$@"; }
status=0
(run_partition_tests fast fast 1 fixture) >"$REPO_ROOT/result.log" 2>&1 || status=$?
unset -f mv
[[ "$status" != 0 && ! -e "$REPO_ROOT/launched" ]]
[[ "$(cat "$REPO_ROOT/.ci-artifacts/junit/fast.xml")" == 'original published failure' ]]
raw="$(find "$REPO_ROOT/.ci-artifacts/junit/history" -name raw.xml -print -quit)"
[[ "$(cat "$raw")" == 'original failed raw report' ]]
run_partition_tests fast fast 1 fixture
assert_retained
printf '[ci] JUnit predecessor retention: 8 cases passed, including partial failure before launch\n'
