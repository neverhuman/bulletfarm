#!/usr/bin/env bash
# Sourced hostile fixtures for the complete hosted-workflow source contract.

workflow_test_root="$(mktemp -d)"
cleanup_workflow_tests() { rm -rf -- "$workflow_test_root"; }
trap cleanup_workflow_tests EXIT
mkdir "$workflow_test_root/workflows"
cp .github/workflows/ci.yml "$workflow_test_root/workflows/ci.yml"
cp .github/workflows/scheduled.yml "$workflow_test_root/workflows/scheduled.yml"

expect_inventory_failure() {
  local hostile="$1" expected_reason="${2-WORKFLOW_INVENTORY_DRIFT}" root="${3-$workflow_test_root/workflows}" output code
  set +e
  output="$(validate_workflow_inventory "$root" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$expected_reason"* ]] \
    || { refuse WORKFLOW_INVENTORY_HOSTILE_FAILED "$hostile code=$code output=$output"; exit 1; }
}

printf '%s\n' 'name: hostile broad upload' >"$workflow_test_root/workflows/exfil.yml"
expect_inventory_failure extra-yml
rm -- "$workflow_test_root/workflows/exfil.yml"
printf '%s\n' 'name: hostile broad upload' >"$workflow_test_root/workflows/exfil.yaml"
expect_inventory_failure extra-yaml
rm -- "$workflow_test_root/workflows/exfil.yaml"
printf '%s\n' 'name: hidden hostile broad upload' >"$workflow_test_root/workflows/.exfil.yml"
expect_inventory_failure hidden-yml
rm -- "$workflow_test_root/workflows/.exfil.yml"
printf '%s\n' 'name: hidden hostile broad upload' >"$workflow_test_root/workflows/.exfil.yaml"
expect_inventory_failure hidden-yaml
rm -- "$workflow_test_root/workflows/.exfil.yaml"
ln -s ci.yml "$workflow_test_root/workflows/exfil.yml"
expect_inventory_failure symlink
rm -- "$workflow_test_root/workflows/exfil.yml"
mkdir "$workflow_test_root/workflows/exfil.yaml"
expect_inventory_failure directory
rmdir "$workflow_test_root/workflows/exfil.yaml"
mv "$workflow_test_root/workflows/ci.yml" "$workflow_test_root/ci.yml"
ln -s ../ci.yml "$workflow_test_root/workflows/ci.yml"
expect_inventory_failure admitted-symlink WORKFLOW_ENTRY_NOT_REGULAR
rm -- "$workflow_test_root/workflows/ci.yml"
mv "$workflow_test_root/ci.yml" "$workflow_test_root/workflows/ci.yml"
mv "$workflow_test_root/workflows/scheduled.yml" "$workflow_test_root/scheduled.yml"
mkdir "$workflow_test_root/workflows/scheduled.yml"
expect_inventory_failure admitted-directory WORKFLOW_ENTRY_NOT_REGULAR
rmdir "$workflow_test_root/workflows/scheduled.yml"
mv "$workflow_test_root/scheduled.yml" "$workflow_test_root/workflows/scheduled.yml"
ln -s workflows "$workflow_test_root/workflows-link"
expect_inventory_failure root-symlink WORKFLOW_DIRECTORY_INVALID "$workflow_test_root/workflows-link"
rm -- "$workflow_test_root/workflows-link"

expect_source_failure() {
  local workflow="$1" expected="$2" reason="$3" output code
  set +e
  output="$(validate_workflow_source "$workflow" "$expected" "$reason" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$reason"* ]] \
    || { refuse WORKFLOW_SOURCE_HOSTILE_FAILED "$reason code=$code output=$output"; exit 1; }
}

printf '%s\n' '# unbound workflow source is not admitted' \
  >>"$workflow_test_root/workflows/ci.yml"
expect_source_failure "$workflow_test_root/workflows/ci.yml" \
  34d0ca2ca91bfc59a9ce7ff6952b8b55ce45827ef02423dc65acdc0ce4c88b74 \
  HOSTED_REQUIRED_SOURCE_DRIFT
printf '%s\n' '# unbound workflow source is not admitted' \
  >>"$workflow_test_root/workflows/scheduled.yml"
expect_source_failure "$workflow_test_root/workflows/scheduled.yml" \
  328a29fb58b76c55f5c315c3b698489b0b6c7c751032f3e8f9e24908e3df93df \
  HOSTED_SCHEDULED_SOURCE_DRIFT
