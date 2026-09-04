#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

for tool in cargo-nextest cmp comm jq mktemp sort; do require_tool "$tool" || exit 1; done
packages="$(cargo metadata --locked --no-deps --format-version 1 | jq -r '.packages[].name' | LC_ALL=C sort)"
expected_packages="$(printf '%s\n' bullet-family bullet-linux-lease bullet-wire | LC_ALL=C sort)"
[[ "$packages" == "$expected_packages" ]] \
  || { refuse WORKSPACE_MEMBER_DRIFT "expected bullet-family, bullet-linux-lease, and bullet-wire only"; exit 1; }

inventory="$(mktemp -d)"
cleanup() { rm -rf "$inventory"; }
trap cleanup EXIT
list_partition() {
  local filter="$1" output="$2"
  local json="$output.json"
  cargo nextest list --locked --workspace -E "$filter" --message-format json >"$json"
  jq -r '."rust-suites" | to_entries[] | .key as $suite | .value.testcases | to_entries[]
      | select(.value."filter-match".status == "matches") | [$suite, .key] | @tsv' \
      "$json" | LC_ALL=C sort -u >"$output"
}
list_partition 'all()' "$inventory/all"
list_partition "$HUB_FILTER" "$inventory/hub"
list_partition "$WIRE_FILTER" "$inventory/wire"
assert_identity_digest() {
  local name="$1" path="$2" expected="$3" actual
  actual="$(sha256_file "$path")" || return 1
  [[ "$actual" == "$expected" ]] \
    || refuse TEST_IDENTITY_DIGEST_DRIFT "$name identity digest is $actual; expected $expected"
}
expect_identity_digest_rejection() {
  local name="$1" path="$2" expected="$3" output status
  set +e
  output="$(assert_identity_digest "$name" "$path" "$expected" 2>&1)"
  status=$?
  set -e
  [[ "$status" -eq 1 && "$output" == *TEST_IDENTITY_DIGEST_DRIFT* ]] \
    || { refuse TEST_PARTITION_HOSTILE_FAILED "$name status=$status output=$output"; exit 1; }
}
for partition in all hub wire; do
  [[ -s "$inventory/$partition" ]] \
    || { refuse ZERO_TEST_PARTITION "$partition"; exit 1; }
done
jq -r '."rust-suites" | to_entries[] | select((.value.testcases | length) == 0) | .key' \
  "$inventory/all.json" | LC_ALL=C sort >"$inventory/zero-suites"
printf '%s\n' \
  'bullet-family::bin/bullet-family' \
  'bullet-family::ci_controls' \
  'bullet-wire::bin/bullet-contract' \
  | LC_ALL=C sort >"$inventory/expected-zero-suites"
cmp -s "$inventory/zero-suites" "$inventory/expected-zero-suites" \
  || { refuse ZERO_TEST_SUITE_INVENTORY_DRIFT "only two CLI bins and cfg-disabled ci_controls may be empty on Linux"; exit 1; }
ignored_count="$(jq '[."rust-suites"[].testcases[] | select(.ignored == true)] | length' "$inventory/all.json")"
[[ "$ignored_count" -eq 0 ]] \
  || { refuse IGNORED_TESTS_FORBIDDEN "$ignored_count tests are ignored"; exit 1; }
for partition in all hub wire; do
  declared="$(jq -r '."test-count"' "$inventory/$partition.json")"
  actual="$(wc -l <"$inventory/$partition")"
  [[ "$declared" -eq "$actual" ]] \
    || { refuse TEST_LIST_COUNT_CONTRADICTION "$partition declared=$declared listed=$actual"; exit 1; }
done
[[ -z "$(comm -12 "$inventory/hub" "$inventory/wire")" ]] \
  || { refuse OVERLAPPING_TEST_PARTITIONS "Hub and wire partitions overlap"; exit 1; }
LC_ALL=C sort -u "$inventory/hub" "$inventory/wire" >"$inventory/union"
cmp -s "$inventory/all" "$inventory/union" \
  || { refuse SILENT_TEST_LOSS "Hub and wire partitions do not cover the workspace"; exit 1; }
hub_count="$(wc -l <"$inventory/hub")"
wire_count="$(wc -l <"$inventory/wire")"
total_count="$(wc -l <"$inventory/all")"
[[ "$HUB_EXPECTED_TESTS" -gt 0 && "$WIRE_EXPECTED_TESTS" -gt 0 \
    && "$((HUB_EXPECTED_TESTS + WIRE_EXPECTED_TESTS))" -eq "$TOTAL_EXPECTED_TESTS" ]] \
  || { refuse TEST_PARTITION_DECLARATION_INVALID "Hub and wire declarations must be nonzero and sum to total"; exit 1; }
[[ "$hub_count" -eq "$HUB_EXPECTED_TESTS" && "$wire_count" -eq "$WIRE_EXPECTED_TESTS" && "$total_count" -eq "$TOTAL_EXPECTED_TESTS" ]] \
  || { refuse TEST_PARTITION_DRIFT "hub=$hub_count/$HUB_EXPECTED_TESTS wire=$wire_count/$WIRE_EXPECTED_TESTS total=$total_count/$TOTAL_EXPECTED_TESTS"; exit 1; }
assert_identity_digest all "$inventory/all" "$TOTAL_EXPECTED_IDENTITIES_SHA256" || exit 1
assert_identity_digest hub "$inventory/hub" "$HUB_EXPECTED_IDENTITIES_SHA256" || exit 1
assert_identity_digest wire "$inventory/wire" "$WIRE_EXPECTED_IDENTITIES_SHA256" || exit 1
printf '%s\n' $'suite\talpha' $'suite\tbeta' >"$inventory/digest-hostile"
digest_hostile_expected="$(sha256_file "$inventory/digest-hostile")" || exit 1
printf '%s\n' $'suite\talpha' $'suite\tgamma' >"$inventory/digest-hostile"
expect_identity_digest_rejection count-neutral-substitution \
  "$inventory/digest-hostile" "$digest_hostile_expected"
log "test inventory exact: hub=$hub_count wire=$wire_count total=$total_count"
