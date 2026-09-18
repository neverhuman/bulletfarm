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
  a6a4e78bc68635a5dff3f896b95a4940adecd9858505c5bb1781ecf73e41ab3e \
  HOSTED_REQUIRED_SOURCE_DRIFT
printf '%s\n' '# unbound workflow source is not admitted' \
  >>"$workflow_test_root/workflows/scheduled.yml"
expect_source_failure "$workflow_test_root/workflows/scheduled.yml" \
  700c2b023ef5279c6e99f990575e3e7232b150b32a8ab8b1ef0004948bbc0cfc \
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

expect_devnode_failure() {
  local hostile="$1" workflow="$2" expected="${3-HOSTED_DEVNODE_LANE_FORBIDDEN}" output code
  set +e
  output="$(refuse_hosted_devnode_lane "$workflow" 2>&1)"
  code=$?
  set -e
  [[ "$code" -ne 0 && "$output" == *"$expected"* ]] \
    || { refuse WORKFLOW_DEVNODE_HOSTILE_FAILED "$hostile code=$code output=$output"; exit 1; }
  log "workflow devnode case=$hostile expected=$expected passed"
}

# A hosted job that runs the lane.
printf '%s\n' 'jobs:' '  devnode:' '    steps:' '      - run: ops/ci/devnode.sh' \
  >"$workflow_test_root/workflows/hostile-devnode.yml"
expect_devnode_failure named-lane "$workflow_test_root/workflows/hostile-devnode.yml"
# A comment is enough: the name must not appear in a hosted workflow at all,
# because a commented lane is one uncomment away from a false claim.
printf '%s\n' 'name: ci' '# TODO: re-enable the devnode lane here' \
  >"$workflow_test_root/workflows/hostile-devnode.yml"
expect_devnode_failure commented-lane "$workflow_test_root/workflows/hostile-devnode.yml"
# A matrix entry that reaches the lane through ci-local.sh names it too.
printf '%s\n' 'jobs:' '  lanes:' '    strategy:' '      matrix:' \
  '        lane: [fast, lint, devnode]' \
  >"$workflow_test_root/workflows/hostile-devnode.yml"
expect_devnode_failure matrix-lane "$workflow_test_root/workflows/hostile-devnode.yml"
rm -- "$workflow_test_root/workflows/hostile-devnode.yml"
# The guard refuses a substituted entry rather than reading through it.
ln -s ci.yml "$workflow_test_root/workflows/hostile-devnode.yml"
expect_devnode_failure symlinked-workflow \
  "$workflow_test_root/workflows/hostile-devnode.yml" WORKFLOW_ENTRY_NOT_REGULAR
rm -- "$workflow_test_root/workflows/hostile-devnode.yml"
# And admits the two real hosted workflows, which name no such lane.
refuse_hosted_devnode_lane "$workflow_test_root/workflows/ci.yml" \
  || { refuse WORKFLOW_DEVNODE_HOSTILE_FAILED "real ci.yml was refused"; exit 1; }
refuse_hosted_devnode_lane "$workflow_test_root/workflows/scheduled.yml" \
  || { refuse WORKFLOW_DEVNODE_HOSTILE_FAILED "real scheduled.yml was refused"; exit 1; }
log "workflow devnode case=real-ci passed"
log "workflow devnode case=real-scheduled passed"
# Missing/non-file inputs and a failed read are refusals, never absence of the lane.
expect_devnode_failure missing-workflow \
  "$workflow_test_root/workflows/absent.yml" WORKFLOW_ENTRY_NOT_REGULAR
mkdir "$workflow_test_root/workflows/directory.yml"
expect_devnode_failure directory-workflow \
  "$workflow_test_root/workflows/directory.yml" WORKFLOW_ENTRY_NOT_REGULAR
rmdir "$workflow_test_root/workflows/directory.yml"
(
  grep() {
    [[ "$#" -eq 3 && "$1" == -Fq && "$2" == devnode ]] || return 99
    printf '%s\n' "$read_error_stage" >>"$workflow_test_root/read-error-marker"
    return 2
  }
  read_error_stage=control
  if grep -Fq devnode "$workflow_test_root/workflows/ci.yml"; then
    refuse WORKFLOW_DEVNODE_HOSTILE_FAILED "failed-read control unexpectedly succeeded"
    exit 1
  else
    [[ "$?" -eq 2 ]] || exit 1
  fi
  read_error_stage=guard
  expect_devnode_failure read-error "$workflow_test_root/workflows/ci.yml" \
    WORKFLOW_SOURCE_READ_FAILED
)
[[ "$(<"$workflow_test_root/read-error-marker")" == $'control\nguard' ]] \
  || { refuse WORKFLOW_DEVNODE_HOSTILE_FAILED "expected one control read and one guard read"; exit 1; }
