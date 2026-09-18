#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
test_root="$(mktemp -d)"
cleanup() { rm -rf -- "$test_root"; }
trap cleanup EXIT
expected_commit="$(git rev-parse HEAD)"
expected_tree="$(git rev-parse 'HEAD^{tree}')"
run_id=123456
run_attempt=2
lanes=(source-scan fast lint contract security docs)
green=(success success success success success success)
declare -A lane_scripts
lane_scripts[source-scan]=ops/ci/source-scan.sh
lane_scripts[fast]=ops/ci/fast.sh
lane_scripts[lint]=ops/ci/lint.sh
lane_scripts[contract]=ops/ci/contract.sh
lane_scripts[security]=ops/ci/security.sh
lane_scripts[docs]=ops/ci/docs.sh

write_fixture_junit() {
  local output="$1" lane="$2" tests="$3"
  printf '%s\n' \
    '<?xml version="1.0" encoding="UTF-8"?>' \
    "<testsuites tests=\"$tests\" failures=\"0\" errors=\"0\" skipped=\"0\">" \
    "  <testsuite name=\"bullet-farm-$lane\" tests=\"$tests\" failures=\"0\" errors=\"0\" skipped=\"0\"/>" \
    '</testsuites>' >"$output"
}

write_duplicate_string_member_fixture() {
  local output="$1" key="$2" first="$3" second="$4"
  printf '{"%s":"%s","%s":"%s"}\n' \
    "$key" "$first" "$key" "$second" >"$output"
}

write_nonfinite_number_fixture() {
  local output="$1"
  printf '{"record_count":NaN}\n' >"$output"
}

origin_for() {
  printf '%s/hub-%s-%s-%s\n' "$test_root" "$1" "$run_id" "$run_attempt"
}

make_fixtures() {
  local lane origin relative path digest artifacts
  local -a files
  rm -rf -- "$test_root"
  mkdir -p "$test_root"
  for lane in "${lanes[@]}"; do
    origin="$(origin_for "$lane")"
    mkdir -p "$origin/.ci-artifacts/observations"
    artifacts='[]'
    case "$lane" in
      fast)
        mkdir -p "$origin/.ci-artifacts/junit"
        write_fixture_junit "$origin/.ci-artifacts/junit/fast.xml" fast "$HUB_EXPECTED_TESTS"
        files=(.ci-artifacts/junit/fast.xml)
        ;;
      contract)
        mkdir -p "$origin/.ci-artifacts/junit" "$origin/.ci-artifacts/formal" \
          "$origin/.ci-artifacts/contracts"
        write_fixture_junit "$origin/.ci-artifacts/junit/contract.xml" contract "$WIRE_EXPECTED_TESTS"
        printf '%s\n' \
          'schema=bullet.formal-log.v1' \
          'models=2' \
          'completed_without_error=2' \
          'pinned_summary_present=1' \
          'exit_code=0' \
          'classification=DIAGNOSTIC_ONLY' >"$origin/.ci-artifacts/formal/contract.log"
        jq -n '{schema_version:"bullet.formal-summary.v1",models:2,completed_models:2,
          pinned_summary_present:true,status:"PASS",exit_code:0,signed:false,
          evidence_class:"DIAGNOSTIC_ONLY"}' >"$origin/.ci-artifacts/formal/contract.json"
        cp contracts/v1alpha1/bundle-manifest.json \
          "$origin/.ci-artifacts/contracts/bundle-manifest.json"
        files=(.ci-artifacts/junit/contract.xml .ci-artifacts/formal/contract.json \
          .ci-artifacts/formal/contract.log .ci-artifacts/contracts/bundle-manifest.json)
        ;;
      *) files=() ;;
    esac
    for relative in "${files[@]}"; do
      path="$origin/$relative"
      digest="$(sha256_file "$path")"
      artifacts="$(jq -c --arg path "$relative" --arg sha256 "$digest" \
        '. + [{path:$path,sha256:$sha256}]' <<<"$artifacts")"
    done
    jq -n --arg lane "$lane" --arg commit "$expected_commit" --arg tree "$expected_tree" \
      --arg command "bash ${lane_scripts[$lane]}" --argjson artifacts "$artifacts" '
      {schema_version:"bullet.ci-observation.v1",repository:"bullet-farm",commit_oid:$commit,
       tree_oid:$tree,clean:true,commands:[("bash scripts/ci-doctor.sh " + $lane),$command],
       tool_versions:{git:"git version 2.43.0",rustc:"rustc 1.95.0 (123456789 2026-01-01)",
         cargo:"cargo 1.95.0 (123456789 2026-01-01)",
         cargo_nextest:"cargo-nextest 0.9.137 (123456789 2026-01-01)",
         actionlint:"1.7.8",shellcheck:"0.10.0",
         b3sum:"b3sum 1.8.2",
         java:"openjdk version \"21.0.0\" 2026-01-01",gitleaks:"8.21.2",
         cargo_deny:"cargo-deny 0.19.8",zizmor:"zizmor 1.25.2",
         docker:"Docker version 28.0.0, build 1234567",file:"file-5.45",
         python:"Python 3.12.3",jsonschema:"4.26.0"},
       outcomes:[{lane:$lane,status:"PASS",exit_code:0}],artifact_hashes:$artifacts,
       signed:false,evidence_class:"DIAGNOSTIC_ONLY"}' >"$origin/.ci-artifacts/observations/$lane.json"
  done
}

expect_failure() {
  local reason="$1" output status
  shift
  set +e
  output="$(bash ops/ci/aggregate.sh "$@" 2>&1)"
  status=$?
  set -e
  [[ "$status" -ne 0 && "$output" == *"$reason"* ]] \
    || { refuse AGGREGATOR_NEGATIVE_FAILED "$reason status=$status output=$output"; exit 1; }
}

aggregate_args=("$test_root" "$expected_commit" "$run_id" "$run_attempt")
make_fixtures
bash ops/ci/aggregate.sh "${aggregate_args[@]}" "${green[@]}" >/dev/null
for bad in failure skipped cancelled ''; do
  candidate=("${green[@]}")
  candidate[1]="$bad"
  expect_failure CI_JOB_NOT_SUCCESSFUL "${aggregate_args[@]}" "${candidate[@]}"
done
expect_failure CI_JOB_MISSING "${aggregate_args[@]}" "${green[@]:0:5}"

make_fixtures; rm "$(origin_for lint)/.ci-artifacts/observations/lint.json"
expect_failure CI_OBSERVATION_MISSING "${aggregate_args[@]}" "${green[@]}"
make_fixtures; docs_observation="$(origin_for docs)/.ci-artifacts/observations/docs.json"; write_duplicate_string_member_fixture "$docs_observation" repository hostile-duplicate bullet-farm
expect_failure CI_JSON_STRICT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq '.clean=false' "$(origin_for docs)/.ci-artifacts/observations/docs.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for docs)/.ci-artifacts/observations/docs.json"
expect_failure CI_SUBJECT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq '.commit_oid="2222222222222222222222222222222222222222"' "$(origin_for security)/.ci-artifacts/observations/security.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for security)/.ci-artifacts/observations/security.json"
expect_failure CI_SUBJECT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq '.tree_oid="2222222222222222222222222222222222222222"' "$(origin_for security)/.ci-artifacts/observations/security.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for security)/.ci-artifacts/observations/security.json"
expect_failure CI_SUBJECT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq '.commands=["doctor","lane"]' "$(origin_for docs)/.ci-artifacts/observations/docs.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for docs)/.ci-artifacts/observations/docs.json"
expect_failure CI_OBSERVATION_COMMAND_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq 'del(.tool_versions.cargo_nextest)' "$(origin_for fast)/.ci-artifacts/observations/fast.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for fast)/.ci-artifacts/observations/fast.json"
expect_failure CI_TOOL_VERSION_MISSING "${aggregate_args[@]}" "${green[@]}"
make_fixtures; jq '.tool_versions.zizmor="zizmor 0.0.0"' "$(origin_for security)/.ci-artifacts/observations/security.json" >"$test_root/x"; mv "$test_root/x" "$(origin_for security)/.ci-artifacts/observations/security.json"
expect_failure CI_TOOL_VERSION_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; rm "$(origin_for fast)/.ci-artifacts/junit/fast.xml"
expect_failure CI_ARTIFACT_PATH_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; printf 'tampered\n' >>"$(origin_for contract)/.ci-artifacts/formal/contract.log"
expect_failure CI_ARTIFACT_HASH_MISMATCH "${aggregate_args[@]}" "${green[@]}"
make_fixtures; printf 'extra\n' >"$(origin_for lint)/.ci-artifacts/raw.log"
expect_failure CI_ATOMIC_ARTIFACT_INVENTORY_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; mkdir "$test_root/unexpected-origin"
expect_failure CI_ARTIFACT_ORIGIN_INVENTORY_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; expect_failure CI_ARTIFACT_ORIGIN_INVENTORY_INVALID "$test_root" "$expected_commit" 999 "$run_attempt" "${green[@]}"
make_fixtures; rm "$(origin_for lint)/.ci-artifacts/observations/lint.json"; ln -s "$(origin_for source-scan)/.ci-artifacts/observations/source-scan.json" "$(origin_for lint)/.ci-artifacts/observations/lint.json"
expect_failure CI_OBSERVATION_MISSING "${aggregate_args[@]}" "${green[@]}"
make_fixtures; contract_origin="$(origin_for contract)"; jq '.completed_models=1' "$contract_origin/.ci-artifacts/formal/contract.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/formal/contract.json"; digest="$(sha256_file "$contract_origin/.ci-artifacts/formal/contract.json")"; jq --arg digest "$digest" '(.artifact_hashes[] | select(.path == ".ci-artifacts/formal/contract.json").sha256) = $digest' "$contract_origin/.ci-artifacts/observations/contract.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/observations/contract.json"
expect_failure CI_FORMAL_SUMMARY_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; contract_origin="$(origin_for contract)"; formal="$contract_origin/.ci-artifacts/formal/contract.json"; write_duplicate_string_member_fixture "$formal" status FAIL PASS; digest="$(sha256_file "$formal")"; jq --arg digest "$digest" '(.artifact_hashes[] | select(.path == ".ci-artifacts/formal/contract.json").sha256) = $digest' "$contract_origin/.ci-artifacts/observations/contract.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/observations/contract.json"
expect_failure CI_JSON_STRICT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; contract_origin="$(origin_for contract)"; jq '.generator="hostile-generator"' "$contract_origin/.ci-artifacts/contracts/bundle-manifest.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/contracts/bundle-manifest.json"; digest="$(sha256_file "$contract_origin/.ci-artifacts/contracts/bundle-manifest.json")"; jq --arg digest "$digest" '(.artifact_hashes[] | select(.path == ".ci-artifacts/contracts/bundle-manifest.json").sha256) = $digest' "$contract_origin/.ci-artifacts/observations/contract.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/observations/contract.json"
expect_failure CI_CONTRACT_MANIFEST_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; contract_origin="$(origin_for contract)"; manifest="$contract_origin/.ci-artifacts/contracts/bundle-manifest.json"; write_nonfinite_number_fixture "$manifest"; digest="$(sha256_file "$manifest")"; jq --arg digest "$digest" '(.artifact_hashes[] | select(.path == ".ci-artifacts/contracts/bundle-manifest.json").sha256) = $digest' "$contract_origin/.ci-artifacts/observations/contract.json" >"$test_root/x"; mv "$test_root/x" "$contract_origin/.ci-artifacts/observations/contract.json"
expect_failure CI_JSON_STRICT_INVALID "${aggregate_args[@]}" "${green[@]}"
make_fixtures; fast_origin="$(origin_for fast)"; printf '<system-out>credential-shaped raw output</system-out>\n' >>"$fast_origin/.ci-artifacts/junit/fast.xml"; digest="$(sha256_file "$fast_origin/.ci-artifacts/junit/fast.xml")"; jq --arg digest "$digest" '(.artifact_hashes[] | select(.path == ".ci-artifacts/junit/fast.xml").sha256) = $digest' "$fast_origin/.ci-artifacts/observations/fast.json" >"$test_root/x"; mv "$test_root/x" "$fast_origin/.ci-artifacts/observations/fast.json"
expect_failure CI_JUNIT_SANITIZATION_INVALID "${aggregate_args[@]}" "${green[@]}"
log "aggregator origin, subject, strict JSON, artifact, and semantic negative matrix passed"
