#!/usr/bin/env bash
# The uploaded JUnit is intentionally a structural summary, not a copy of raw
# test output. Prove that sensitive output and volatile metadata cannot
# enter it and that all counters come from explicit arguments.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
canary='sensitive-junit-output-canary-7f42'
write_junit_summary sanitizer-test 7 1 2 3
report=.ci-artifacts/junit/sanitizer-test.xml
grep -Fq '<testsuites tests="7" failures="1" errors="2" skipped="3">' "$report"
if grep -Eq "$canary|system-out|system-err|timestamp=|uuid=|classname=|testcase" "$report"; then
  refuse JUNIT_SANITIZER_LEAK "output or volatile metadata survived"
  exit 1
fi
rm -f "$report"
log "structural JUnit sanitizer canary passed"
