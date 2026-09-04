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
  52c7841a5a4c6ab4a2321fce6b18755552da49d5b0e065c7be4925ad0880a1fb \
  HOSTED_REQUIRED_SOURCE_DRIFT
printf '%s\n' '# unbound workflow source is not admitted' \
  >>"$workflow_test_root/workflows/scheduled.yml"
expect_source_failure "$workflow_test_root/workflows/scheduled.yml" \
  e284e768a81d1ff83a5e453c3b81fbd3df0762e172367378af845e87502b422f \
  HOSTED_SCHEDULED_SOURCE_DRIFT

expect_audit_neutral_failure() {
  local hostile="$1" reason="$2" output code
  set +e
  output="$(validate_scheduled_audit_neutral "$workflow_test_root/workflows/scheduled.yml" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$reason"* ]] \
    || { refuse SCHEDULED_AUDIT_HOSTILE_FAILED "$hostile code=$code output=$output"; exit 1; }
  cp .github/workflows/scheduled.yml "$workflow_test_root/workflows/scheduled.yml"
}

cp .github/workflows/scheduled.yml "$workflow_test_root/workflows/scheduled.yml"
validate_scheduled_audit_neutral "$workflow_test_root/workflows/scheduled.yml" || exit 1
sed -i '0,/^            exit 78$/{s//            exit 0/}' "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure green-neutral SCHEDULED_AUDIT_NEUTRAL_DRIFT
sed -i '/^            echo "::error::AUDITOR_UNAVAILABLE_HOSTED/d' "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure untyped-neutral SCHEDULED_AUDIT_NEUTRAL_DRIFT
sed -i '/^      - name: Resolve the pinned auditor or refuse neutral$/,/^          }$/d' \
  "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure missing-neutral-step SCHEDULED_AUDIT_NEUTRAL_DRIFT
sed -i 's|^        run: bash scripts/ci-local.sh audit$|        run: true # bash scripts/ci-local.sh audit|' \
  "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure commented-lane SCHEDULED_AUDIT_LANE_EXECUTION_DRIFT
sed -i 's|^        run: bash scripts/ci-local.sh audit$|        if: ${{ always() }}\n&|' \
  "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure conditional-lane SCHEDULED_AUDIT_LANE_EXECUTION_DRIFT
sed -i '/^  audit:$/,$d' "$workflow_test_root/workflows/scheduled.yml"
expect_audit_neutral_failure missing-job SCHEDULED_JOB_MISSING
