#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
checker="$REPO_ROOT/ops/ci/check-links.sh"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

make_fixture() {
  local fixture="$1"
  mkdir -p "$fixture/repo/docs" "$fixture/repo/images"
  printf 'png\n' >"$fixture/repo/images/example.png"
  printf '%s\n' \
    '# Home' \
    '' \
    '[inline](docs/guide.md#details)' \
    '![image](images/example.png)' \
    '[reference][guide]' \
    '[same](#home)' \
    '[external](https://example.invalid/not-fetched)' \
    '' \
    '[guide]: docs/guide.md#details "Guide"' >"$fixture/repo/README.md"
  printf '%s\n' \
    '# Guide' \
    '' \
    '## Details' \
    '' \
    '[up](../README.md#home)' >"$fixture/repo/docs/guide.md"
}

expect_failure() {
  local name="$1" fixture="$2"
  if bash "$checker" --root "$fixture/repo" README.md docs/guide.md >/dev/null 2>&1; then
    printf '[ci] CHECK_LINKS_NEGATIVE_MISSED: %s\n' "$name" >&2
    exit 1
  fi
}

valid="$tmp/valid"
make_fixture "$valid"
bash "$checker" --root "$valid/repo" README.md docs/guide.md >/dev/null

missing_target="$tmp/missing-target"
make_fixture "$missing_target"
printf '\n[missing](docs/absent.md)\n' >>"$missing_target/repo/README.md"
expect_failure missing-target "$missing_target"

missing_fragment="$tmp/missing-fragment"
make_fixture "$missing_fragment"
printf '\n[missing](docs/guide.md#absent)\n' >>"$missing_fragment/repo/README.md"
expect_failure missing-fragment "$missing_fragment"

escape="$tmp/escape"
make_fixture "$escape"
printf '# Outside\n' >"$escape/outside.md"
printf '\n[outside](../outside.md)\n' >>"$escape/repo/README.md"
expect_failure parent-escape "$escape"

symlink_escape="$tmp/symlink-escape"
make_fixture "$symlink_escape"
printf '# Outside\n' >"$symlink_escape/outside.md"
ln -s "$symlink_escape/outside.md" "$symlink_escape/repo/docs/outside.md"
printf '\n[outside](docs/outside.md)\n' >>"$symlink_escape/repo/README.md"
expect_failure symlink-escape "$symlink_escape"

absolute="$tmp/absolute"
make_fixture "$absolute"
printf '\n[absolute](/etc/passwd)\n' >>"$absolute/repo/README.md"
expect_failure absolute-path "$absolute"

brand_claim="$tmp/brand-claim"
make_fixture "$brand_claim"
mkdir -p "$brand_claim/repo/docs/brand/mascots"
printf '# Brief\n\nGas Town is worse.\n' \
  >"$brand_claim/repo/docs/brand/mascots/01-hostile.md"
if bash "$checker" --root "$brand_claim/repo" >/dev/null 2>&1; then
  printf '[ci] BRAND_COMPETITOR_CLAIM_ACCEPTED\n' >&2
  exit 1
fi

printf '[ci] Markdown link containment and fragment negative matrix passed\n'
