#!/usr/bin/env bash
# Bind the aggregator process itself to the exact clean hosted checkout.
set -euo pipefail

root="${1:?repository root is required}"
expected_commit="${2:?expected commit is required}"
[[ "$expected_commit" =~ ^[0-9a-f]{40}$ ]] || {
  printf '[ci] CI_COMMIT_INVALID: %s\n' "$expected_commit" >&2
  exit 1
}
[[ -d "$root/.git" && ! -L "$root/.git" ]] || {
  printf '[ci] CI_AGGREGATOR_CHECKOUT_INVALID: %s\n' "$root" >&2
  exit 1
}
canonical_root="$(cd -P -- "$root" && pwd)"

assert_subject() {
  local head tree expected_tree
  head="$(git -C "$canonical_root" rev-parse --verify HEAD)"
  tree="$(git -C "$canonical_root" rev-parse --verify 'HEAD^{tree}')"
  expected_tree="$(git -C "$canonical_root" rev-parse --verify "$expected_commit^{tree}" 2>/dev/null)" || {
    printf '[ci] CI_COMMIT_NOT_FOUND: %s\n' "$expected_commit" >&2
    return 1
  }
  [[ "$head" == "$expected_commit" && "$tree" == "$expected_tree" ]] || {
    printf '[ci] CI_AGGREGATOR_SUBJECT_INVALID: expected %s, found %s/%s\n' \
      "$expected_commit" "$head" "$tree" >&2
    return 1
  }
  [[ -z "$(git -C "$canonical_root" status --porcelain=v1 --untracked-files=all)" ]] || {
    printf '[ci] CI_AGGREGATOR_CHECKOUT_DIRTY: %s\n' "$canonical_root" >&2
    return 1
  }
}

assert_subject
assert_subject
printf '[ci] aggregator checkout subject passed: %s\n' "$expected_commit"
