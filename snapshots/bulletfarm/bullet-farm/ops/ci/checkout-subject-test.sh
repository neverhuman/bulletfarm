#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
checker="$REPO_ROOT/ops/ci/checkout-subject.sh"
test_root="$(mktemp -d)"
trap 'rm -rf -- "$test_root"' EXIT
fixture="$test_root/repository"
mkdir "$fixture"
git -C "$fixture" init -q
git -C "$fixture" config user.name ci-fixture
git -C "$fixture" config user.email ci-fixture@example.invalid
printf 'subject\n' >"$fixture/subject.txt"
git -C "$fixture" add subject.txt
git -C "$fixture" commit -qm subject
commit="$(git -C "$fixture" rev-parse HEAD)"
bash "$checker" "$fixture" "$commit" >/dev/null

printf 'dirty\n' >>"$fixture/subject.txt"
if bash "$checker" "$fixture" "$commit" >/dev/null 2>&1; then
  echo '[ci] CI_AGGREGATOR_DIRTY_NEGATIVE_MISSED' >&2
  exit 1
fi
git -C "$fixture" restore subject.txt
if bash "$checker" "$fixture" 0000000000000000000000000000000000000000 >/dev/null 2>&1; then
  echo '[ci] CI_AGGREGATOR_COMMIT_NEGATIVE_MISSED' >&2
  exit 1
fi
printf '[ci] aggregator checkout subject negative matrix passed\n'
