#!/usr/bin/env bash
# Validate one unsigned CI observation and every sanitized artifact it names.
# An expected commit binds the observation to the checked-out subject. Atomic
# mode additionally requires the downloaded/staged file inventory to be exact.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/artifact-path.sh
source "$(dirname "${BASH_SOURCE[0]}")/artifact-path.sh"
# shellcheck source=ops/ci/tool-version.sh
source "$(dirname "${BASH_SOURCE[0]}")/tool-version.sh"
# shellcheck source=ops/ci/toolchain-pins.sh
source "$(dirname "${BASH_SOURCE[0]}")/toolchain-pins.sh"

lane="${1:?lane is required}"
expected_commit="${2:-}"
artifact_container="${3:-$REPO_ROOT}"
mode="${4:-local}"
[[ "$lane" =~ ^[a-z][a-z0-9-]*$ ]] \
  || { refuse CI_LANE_INVALID "$lane"; exit 1; }
[[ "$mode" == local || "$mode" == atomic ]] \
  || { refuse CI_ARTIFACT_MODE_INVALID "$mode"; exit 1; }
[[ -d "$artifact_container" && ! -L "$artifact_container" ]] \
  || { refuse CI_ARTIFACT_ROOT_MISSING "$artifact_container"; exit 1; }
artifact_container="$(cd -P -- "$artifact_container" && pwd)"
cd "$artifact_container"
[[ -d .ci-artifacts && ! -L .ci-artifacts ]] \
  || { refuse CI_ARTIFACT_ROOT_MISSING "$artifact_container/.ci-artifacts"; exit 1; }

observation=".ci-artifacts/observations/$lane.json"
if ! validate_ci_artifact_path "$observation" || [[ ! -s "$observation" ]]; then
  refuse CI_OBSERVATION_MISSING "$observation"
  exit 1
fi
validate_strict_json() {
  local path="$1"
  bash "$REPO_ROOT/ops/ci/strict-json.sh" "$path" >/dev/null 2>&1 \
    || { refuse CI_JSON_STRICT_INVALID "$path"; return 1; }
}
validate_strict_json "$observation" || exit 1
jq -e --arg lane "$lane" --arg tool_keys "$CI_TOOL_KEYS" '
  ($tool_keys | split(" ")) as $allowed_tool_keys |
  .schema_version == "bullet.ci-observation.v1" and .repository == "bullet-farm" and
  (.commit_oid | test("^[0-9a-f]{40}$")) and (.tree_oid | test("^[0-9a-f]{40}$")) and
  (.clean | type == "boolean") and (.commands | type == "array") and
  all(.commands[]; type == "string" and length > 0) and
  (.tool_versions | type == "object") and
  (((.tool_versions | keys) - $allowed_tool_keys) | length == 0) and
  all(.tool_versions[]; type == "string" and length >= 1 and length <= 160 and
    (test("[^ -~]") | not)) and
  (.outcomes | type == "array") and (.outcomes | length == 1) and
  all(.outcomes[]; type == "object" and
    (keys | sort) == (["exit_code","lane","status"] | sort)) and
  (.outcomes[0].lane | type == "string") and .outcomes[0].lane == $lane and
  (.outcomes[0].status | type == "string") and
  (.outcomes[0].exit_code | type == "number" and . == floor and . >= 0 and . <= 255) and
  ((.outcomes[0].status == "PASS" and .outcomes[0].exit_code == 0) or
   (.outcomes[0].status == "FAIL" and .outcomes[0].exit_code > 0)) and
  (.artifact_hashes | type == "array") and
  ([.artifact_hashes[].path] | unique | length) == (.artifact_hashes | length) and
  all(.artifact_hashes[]; type == "object" and
    (keys | sort) == (["path","sha256"] | sort)) and
  all(.artifact_hashes[]; (.path | type == "string") and
    (.path | test("^\\.ci-artifacts/[A-Za-z0-9_-][A-Za-z0-9._-]*(/[A-Za-z0-9_-][A-Za-z0-9._-]*)*$")) and
    (.sha256 | type == "string") and (.sha256 | test("^[0-9a-f]{64}$"))) and
  .signed == false and
  .evidence_class == "DIAGNOSTIC_ONLY" and
  (keys | sort) == (["artifact_hashes","clean","commands","commit_oid","evidence_class",
    "outcomes","repository","schema_version","signed","tool_versions","tree_oid"] | sort)
' "$observation" >/dev/null || { refuse CI_OBSERVATION_INVALID "$observation"; exit 1; }

while IFS= read -r tool_key; do
  tool_value="$(jq -er --arg key "$tool_key" '.tool_versions[$key]' "$observation")" \
    || { refuse CI_TOOL_VERSION_INVALID "tool_versions[$tool_key] is missing"; exit 1; }
  ci_tool_version_shape_is_valid "$tool_key" "$tool_value" \
    || { refuse CI_TOOL_VERSION_INVALID "tool_versions[$tool_key] has an invalid lexical shape"; exit 1; }
done < <(jq -r '.tool_versions | keys[]' "$observation")

if [[ -n "$expected_commit" ]]; then
  [[ "$expected_commit" =~ ^[0-9a-f]{40}$ ]] \
    || { refuse CI_COMMIT_INVALID "$expected_commit"; exit 1; }
  resolved_commit="$(git -C "$REPO_ROOT" rev-parse --verify "$expected_commit^{commit}" 2>/dev/null)" \
    || { refuse CI_COMMIT_NOT_FOUND "$expected_commit"; exit 1; }
  [[ "$resolved_commit" == "$expected_commit" ]] \
    || { refuse CI_COMMIT_INVALID "$expected_commit resolved to $resolved_commit"; exit 1; }
  expected_tree="$(git -C "$REPO_ROOT" rev-parse --verify "$expected_commit^{tree}" 2>/dev/null)" \
    || { refuse CI_TREE_INVALID "$expected_commit"; exit 1; }
  jq -e --arg commit "$expected_commit" --arg tree "$expected_tree" \
    '.commit_oid == $commit and .tree_oid == $tree and .clean == true' "$observation" >/dev/null \
    || { refuse CI_SUBJECT_INVALID "$lane"; exit 1; }
fi

status="$(jq -r '.outcomes[0].status' "$observation")"

lane_script_for() {
  case "$1" in
    source-scan) printf '%s\n' ops/ci/source-scan.sh ;;
    fast) printf '%s\n' ops/ci/fast.sh ;;
    lint) printf '%s\n' ops/ci/lint.sh ;;
    contract) printf '%s\n' ops/ci/contract.sh ;;
    security) printf '%s\n' ops/ci/security.sh ;;
    docs) printf '%s\n' ops/ci/docs.sh ;;
    required) printf '%s\n' ops/ci/required.sh ;;
    family) printf '%s\n' ops/ci/family.sh ;;
    family-contract) printf '%s\n' ops/ci/family-contract.sh ;;
    history) printf '%s\n' ops/ci/history.sh ;;
    links) printf '%s\n' ops/ci/external-links.sh ;;
    advisory) printf '%s\n' ops/ci/advisory.sh ;;
    coverage) printf '%s\n' ops/ci/coverage.sh ;;
    platform) printf '%s\n' ops/ci/platform-refusal.sh ;;
    audit) printf '%s\n' ops/ci/audit.sh ;;
    toolchain-pinned) printf '%s\n' ops/ci/toolchain-pinned.sh ;;
    *) return 1 ;;
  esac
}

lane_tools_for() {
  case "$1" in
    source-scan) printf '%s\n' 'git gitleaks' ;;
    fast) printf '%s\n' 'git rustc cargo cargo_nextest' ;;
    lint) printf '%s\n' 'git rustc cargo cargo_nextest actionlint shellcheck b3sum jsonschema' ;;
    contract) printf '%s\n' 'git rustc cargo cargo_nextest java' ;;
    security) printf '%s\n' 'git rustc cargo gitleaks cargo_deny zizmor' ;;
    docs) printf '%s\n' 'git rustc cargo docker file jsonschema' ;;
    required) printf '%s\n' 'git rustc cargo cargo_nextest actionlint shellcheck java gitleaks cargo_deny zizmor docker file jsonschema b3sum' ;;
    family|family-contract) printf '%s\n' \
      'git rustc cargo cargo_nextest node npm jsonschema rustup b3sum rustc_pinned cargo_pinned' ;;
    history) printf '%s\n' 'git gitleaks' ;;
    links) printf '%s\n' 'git lychee' ;;
    advisory) printf '%s\n' 'git rustc cargo cargo_deny' ;;
    coverage) printf '%s\n' 'git rustc cargo cargo_nextest cargo_llvm_cov' ;;
    platform) printf '%s\n' 'git rustc cargo' ;;
    audit) printf '%s\n' 'git jankurai' ;;
    toolchain-pinned) printf '%s\n' 'git rustc cargo rustup b3sum rustc_pinned cargo_pinned' ;;
    *) return 1 ;;
  esac
}

validate_tool_version() {
  local key="$1" value
  value="$(jq -r --arg key "$key" '.tool_versions[$key] // empty' "$observation")"
  [[ -n "$value" ]] || { refuse CI_TOOL_VERSION_MISSING "tool_versions[$key]"; return 1; }
  case "$key" in
    git) [[ "$value" == "git version "* ]] ;;
    rustc) [[ "$value" == "rustc 1.95.0 "* ]] ;;
    cargo) [[ "$value" == "cargo 1.95.0 "* ]] ;;
    cargo_nextest) [[ "$value" == "cargo-nextest 0.9.137 "* ]] ;;
    actionlint) [[ "$value" == 1.7.8 ]] ;;
    shellcheck) [[ "$value" == 0.10.0 ]] ;;
    java) [[ "$value" == "openjdk version \"21."* || "$value" == "java version \"21."* ]] ;;
    gitleaks) [[ "$value" == 8.21.2 ]] ;;
    cargo_deny) [[ "$value" == "cargo-deny 0.19.8" ]] ;;
    zizmor) [[ "$value" == "zizmor 1.25.2" ]] ;;
    docker) [[ "$value" == "Docker version "* ]] ;;
    file) [[ "$value" == file-* || "$value" == "file "* ]] ;;
    node) [[ "$value" == "v$PINNED_NODE_VERSION" ]] ;;
    npm) [[ "$value" == "$PINNED_NPM_VERSION" ]] ;;
    lychee) [[ "$value" == "lychee 0.24.0" ]] ;;
    cargo_llvm_cov) [[ "$value" == "cargo-llvm-cov 0.8.7" ]] ;;
    jankurai) [[ "$value" == "jankurai 1.6.11" ]] ;;
    rustup) [[ "$value" == "rustup 1.29.0 "* ]] ;;
    b3sum) [[ "$value" == "b3sum 1.8.2" ]] ;;
    rustc_pinned) [[ "$value" == "rustc 1.97.1 "* ]] ;;
    cargo_pinned) [[ "$value" == "cargo 1.97.1 "* ]] ;;
    python) [[ "$value" == "Python 3.12."* ]] ;;
    jsonschema) [[ "$value" == 4.26.0 ]] ;;
    *) refuse CI_TOOL_KEY_UNKNOWN "tool_versions[$key]"; return 1 ;;
  esac || { refuse CI_TOOL_VERSION_INVALID "tool_versions[$key] has an inadmissible version"; return 1; }
}

# The strict parser is part of every observation's semantic admission, so its
# exact major/minor runtime is required and bound even for artifact-free lanes.
validate_tool_version python

if [[ "$lane" != observation-test ]]; then
  lane_script="$(lane_script_for "$lane")" \
    || { refuse CI_ARTIFACT_LANE_UNSUPPORTED "$lane"; exit 1; }
  lane_tool_list="$(lane_tools_for "$lane")" \
    || { refuse CI_ARTIFACT_LANE_UNSUPPORTED "$lane"; exit 1; }
  expected_doctor="bash scripts/ci-doctor.sh $lane"
  expected_lane="bash $lane_script"
  if [[ "$status" == PASS ]]; then
    jq -e --arg doctor "$expected_doctor" --arg command "$expected_lane" \
      '.commands == [$doctor,$command]' "$observation" >/dev/null \
      || { refuse CI_OBSERVATION_COMMAND_INVALID "$lane"; exit 1; }
    for tool in $lane_tool_list; do
      validate_tool_version "$tool"
    done
  else
    jq -e --arg doctor "$expected_doctor" --arg command "$expected_lane" \
      '.commands == [$doctor] or .commands == [$doctor,$command]' "$observation" >/dev/null \
      || { refuse CI_OBSERVATION_COMMAND_INVALID "$lane FAIL observation"; exit 1; }
  fi
fi

expected_paths='[]'
case "$lane" in
  source-scan|lint|security|docs|history|links|advisory|platform|audit|toolchain-pinned) ;;
  fast) expected_paths='[".ci-artifacts/junit/fast.xml"]' ;;
  contract) expected_paths='[".ci-artifacts/contracts/bundle-manifest.json",".ci-artifacts/formal/contract.json",".ci-artifacts/formal/contract.log",".ci-artifacts/junit/contract.xml"]' ;;
  required) expected_paths='[".ci-artifacts/contracts/bundle-manifest.json",".ci-artifacts/formal/contract.json",".ci-artifacts/formal/contract.log",".ci-artifacts/junit/contract.xml",".ci-artifacts/junit/fast.xml"]' ;;
  coverage) expected_paths='[".ci-artifacts/coverage/cobertura.xml"]' ;;
  family|family-contract) expected_paths='[".ci-artifacts/family/subjects.json"]' ;;
  observation-test) expected_paths='[".ci-artifacts/test/artifact.txt"]' ;;
  *) refuse CI_ARTIFACT_LANE_UNSUPPORTED "$lane"; exit 1 ;;
esac
while IFS=$'\t' read -r artifact_index path expected; do
  [[ -n "$path" ]] || continue
  artifact_label="artifact_hashes[$artifact_index]"
  validate_ci_artifact_path "$path" \
    || { refuse CI_ARTIFACT_PATH_INVALID "$artifact_label"; exit 1; }
  actual="$(sha256_file "$path" 2>/dev/null)" \
    || { refuse CI_ARTIFACT_READ_INVALID "$artifact_label"; exit 1; }
  [[ "$actual" == "$expected" ]] \
    || { refuse CI_ARTIFACT_HASH_MISMATCH "$artifact_label"; exit 1; }
done < <(jq -r '.artifact_hashes | to_entries[] | [.key,.value.path,.value.sha256] | @tsv' "$observation")

if [[ "$status" == PASS ]]; then
  jq -e --argjson expected "$expected_paths" \
    '([.artifact_hashes[].path] | sort) == ($expected | sort)' "$observation" >/dev/null \
    || { refuse CI_ARTIFACT_INVENTORY_INVALID "$lane"; exit 1; }
else
  jq -e '.artifact_hashes == []' "$observation" >/dev/null \
    || { refuse CI_ARTIFACT_INVENTORY_INVALID "$lane FAIL observation"; exit 1; }
fi

if [[ "$mode" == atomic ]]; then
  if find .ci-artifacts -type l -print -quit | grep -q .; then
    refuse CI_ATOMIC_ARTIFACT_SYMLINK "artifact inventory"
    exit 1
  fi
  if find .ci-artifacts -mindepth 1 ! -type d ! -type f -print -quit | grep -q .; then
    refuse CI_ATOMIC_ARTIFACT_TYPE_INVALID "artifact inventory"
    exit 1
  fi
  expected_files="$({
    printf '%s\n' "$observation"
    jq -r '.artifact_hashes[].path' "$observation"
  } | LC_ALL=C sort -u)"
  actual_files="$(find .ci-artifacts -type f -print | LC_ALL=C sort)"
  [[ "$actual_files" == "$expected_files" ]] \
    || { refuse CI_ATOMIC_ARTIFACT_INVENTORY_INVALID "$lane"; exit 1; }
fi

validate_junit_report() {
  local report="$1" expected_tests="$2" source_lane="$3" root_line tests failures errors skipped
  root_line="$(grep -m1 '<testsuites ' "$report")" \
    || { refuse CI_JUNIT_INVALID "$report"; return 1; }
  tests="$(xml_integer_attribute "$root_line" tests)" || return 1
  failures="$(xml_integer_attribute "$root_line" failures)" || return 1
  errors="$(xml_integer_attribute "$root_line" errors)" || return 1
  skipped="$(xml_integer_attribute "$root_line" skipped)" || return 1
  [[ "$tests" -eq "$expected_tests" && "$tests" -gt 0 && "$failures" -eq 0 \
    && "$errors" -eq 0 && "$skipped" -eq 0 ]] \
    || { refuse CI_JUNIT_OUTCOME_INVALID "$report tests=$tests failures=$failures errors=$errors skipped=$skipped"; return 1; }
  cmp -s -- "$report" <(printf '%s\n' \
    '<?xml version="1.0" encoding="UTF-8"?>' \
    "<testsuites tests=\"$expected_tests\" failures=\"0\" errors=\"0\" skipped=\"0\">" \
    "  <testsuite name=\"bullet-farm-$source_lane\" tests=\"$expected_tests\" failures=\"0\" errors=\"0\" skipped=\"0\"/>" \
    '</testsuites>') \
    || { refuse CI_JUNIT_SANITIZATION_INVALID "$report"; return 1; }
}

validate_formal_reports() {
  local summary=.ci-artifacts/formal/contract.json
  local normalized=.ci-artifacts/formal/contract.log expected_log actual_log
  validate_strict_json "$summary" || return 1
  jq -e '
    . == {schema_version:"bullet.formal-summary.v1",models:2,completed_models:2,
      pinned_summary_present:true,status:"PASS",exit_code:0,signed:false,
      evidence_class:"DIAGNOSTIC_ONLY"}
  ' "$summary" >/dev/null || { refuse CI_FORMAL_SUMMARY_INVALID "$summary"; return 1; }
  expected_log="$(printf '%s\n' \
    'schema=bullet.formal-log.v1' \
    'models=2' \
    'completed_without_error=2' \
    'pinned_summary_present=1' \
    'exit_code=0' \
    'classification=DIAGNOSTIC_ONLY')"
  actual_log="$(<"$normalized")"
  [[ "$actual_log" == "$expected_log" ]] \
    || { refuse CI_FORMAL_LOG_INVALID "$normalized"; return 1; }
}

validate_contract_manifest() {
  local manifest=.ci-artifacts/contracts/bundle-manifest.json
  validate_strict_json "$manifest" || return 1
  jq -e '
    (keys | sort) == (["authority_golden_hash","bundle_hash","catalog_hash",
      "generated_client_hash","generated_clients","generator","invariant_registry_hash",
      "launch_grant_golden_hash","policy_snapshot_hash","record_count","schema_version"] | sort) and
    .schema_version == "v1alpha1" and
    .generator == "bullet-wire-contract-tool-v1alpha1" and
    (.record_count | type == "number" and . == floor and . > 0) and
    (.generated_clients | keys | sort) == ["rust","typescript"] and
    ([.authority_golden_hash,.bundle_hash,.catalog_hash,.generated_client_hash,
      .invariant_registry_hash,.launch_grant_golden_hash,.policy_snapshot_hash,
      .generated_clients.rust,.generated_clients.typescript] |
      all(type == "string" and test("^[0-9a-f]{64}$")))
  ' "$manifest" >/dev/null || { refuse CI_CONTRACT_MANIFEST_INVALID "$manifest"; return 1; }
}

if [[ "$status" == PASS ]]; then
  case "$lane" in
    fast) validate_junit_report .ci-artifacts/junit/fast.xml "$HUB_EXPECTED_TESTS" fast ;;
    contract)
      validate_junit_report .ci-artifacts/junit/contract.xml "$WIRE_EXPECTED_TESTS" contract
      validate_formal_reports
      validate_contract_manifest
      ;;
    required)
      validate_junit_report .ci-artifacts/junit/fast.xml "$HUB_EXPECTED_TESTS" fast
      validate_junit_report .ci-artifacts/junit/contract.xml "$WIRE_EXPECTED_TESTS" contract
      validate_formal_reports
      validate_contract_manifest
      ;;
    coverage)
      bash "$REPO_ROOT/ops/ci/coverage-sanitize.sh" check .ci-artifacts/coverage/cobertura.xml
      ;;
    family|family-contract)
      bash "$REPO_ROOT/ops/ci/family-observation.sh" check \
        "$artifact_container/.ci-artifacts/family/subjects.json" >/dev/null
      jq -e --arg commit "sha1:$(jq -r '.commit_oid' "$observation")" \
        --arg tree "sha1:$(jq -r '.tree_oid' "$observation")" '
        .subjects["bullet-farm"] == {commit_oid:$commit,tree_oid:$tree,clean:true}
      ' .ci-artifacts/family/subjects.json >/dev/null \
        || { refuse CI_FAMILY_SUBJECT_INVALID "family observation does not bind its outer Hub subject"; exit 1; }
      ;;
  esac
fi

log "artifact observation passed: $lane ($mode)"
