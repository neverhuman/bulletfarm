#!/usr/bin/env bash
# One real Hub source scan under the bootstrap job. Unsigned diagnostics only.
# Supporting tool hashes record a cooperative interval, not an admitted closure.
set -euo pipefail

publication_refuse() { printf '%s\n' "$1" >&2; return 1; }
publication_hash() { sha256sum "$1" | cut -d ' ' -f 1; }
publication_file() {
  local path="$1" limit="${2:-1048576}" size
  [[ -f "$path" && ! -L "$path" ]] || { publication_refuse PUBLICATION_REQUIRED_FILE; return 1; }
  size="$(wc -c <"$path")"
  [[ "$size" -gt 0 && "$size" -le "$limit" ]] || publication_refuse PUBLICATION_REQUIRED_SIZE
}
publication_directory() {
  [[ "$1" == /* && -d "$1" && ! -L "$1" && "$(realpath -e "$1")" == "$1" ]] \
    || publication_refuse PUBLICATION_REQUIRED_DIRECTORY
}
publication_inventory() {
  local root="$1" expected="$2" depth="${3:-8}" actual
  publication_directory "$root" || return 1
  actual="$(timeout 10s find "$root" -mindepth 1 -maxdepth "$depth" -printf '%y %P\n' | head -c 4097)" \
    || { publication_refuse PUBLICATION_REQUIRED_INVENTORY; return 1; }
  [[ "${#actual}" -le 4096 && "$(LC_ALL=C sort <<<"$actual")" == "$(LC_ALL=C sort <<<"$expected")" ]] \
    || publication_refuse PUBLICATION_REQUIRED_INVENTORY
}
publication_json() {
  publication_file "$1" || return 1
  bash "$PUBLICATION_HUB/ops/ci/strict-json.sh" "$1" >/dev/null \
    || publication_refuse PUBLICATION_REQUIRED_JSON
}
publication_host() {
  local job="$1"
  [[ "${GITHUB_ACTIONS:-}" == true && "${GITHUB_JOB:-}" == "$job" \
    && "${GITHUB_SHA:-}" =~ ^[0-9a-f]{40}$ && "${GITHUB_WORKFLOW_SHA:-}" == "$GITHUB_SHA" \
    && "${GITHUB_RUN_ID:-}" =~ ^[1-9][0-9]{0,19}$ \
    && "${GITHUB_RUN_ATTEMPT:-}" =~ ^[1-9][0-9]{0,19}$ ]] \
    || { publication_refuse PUBLICATION_REQUIRED_CONTEXT; return 1; }
  publication_directory "${GITHUB_WORKSPACE:-}" || return 1
  publication_directory "${RUNNER_TEMP:-}" || return 1
  [[ -d "$GITHUB_WORKSPACE/.git" && ! -L "$GITHUB_WORKSPACE/.git" \
    && "$(git -C "$GITHUB_WORKSPACE" rev-parse HEAD)" == "$GITHUB_SHA" \
    && -z "$(git -C "$GITHUB_WORKSPACE" status --porcelain=v1 --untracked-files=all)" ]] \
    || { publication_refuse PUBLICATION_REQUIRED_SUBJECT; return 1; }
  PUBLICATION_HUB="$GITHUB_WORKSPACE/bullet-farm"
  local prefix='neverhuman/bulletfarm/.github/workflows/publication.yml@' ref
  [[ "${GITHUB_WORKFLOW_REF:-}" == "$prefix"* && "${#GITHUB_WORKFLOW_REF}" -le 1024 ]] \
    || { publication_refuse PUBLICATION_REQUIRED_WORKFLOW; return 1; }
  ref="${GITHUB_WORKFLOW_REF#"$prefix"}"
  git -C "$GITHUB_WORKSPACE" check-ref-format "$ref" \
    || { publication_refuse PUBLICATION_REQUIRED_WORKFLOW; return 1; }
  case "${GITHUB_EVENT_NAME:-}" in
    pull_request) [[ "$ref" =~ ^refs/pull/[1-9][0-9]*/merge$ ]] ;;
    push) [[ "$ref" == refs/heads/main ]] ;;
    workflow_dispatch) [[ "$ref" == refs/heads/?* ]] ;;
    *) return 1 ;;
  esac || publication_refuse PUBLICATION_REQUIRED_WORKFLOW
}

publication_tools() {
  local scanner="$1" name path result='{}'
  [[ "$scanner" == /* && -f "$scanner" && ! -L "$scanner" && -x "$scanner" \
    && "$(publication_hash "$scanner")" == 50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359 \
    && "$("$scanner" version)" == 8.21.2 ]] \
    || { publication_refuse PUBLICATION_MEMBER_SCANNER_INVALID; return 1; }
  [[ "$(/usr/bin/python3 --version)" == 'Python 3.12.'* ]] \
    || { publication_refuse PUBLICATION_MEMBER_PYTHON_INVALID; return 1; }
  # Fixed PATH plus fixed names; no caller-provided executable or argv map.
  for name in awk bash cat cmp cp cut df dirname env find git grep head id jq mkdir mv \
    python3 realpath rm rmdir sed sha256sum sort timeout tr wc xargs; do
    path="$(PATH=/usr/bin:/bin builtin type -P "$name")" || return 1
    path="$(realpath -e "$path")"
    [[ -f "$path" && -x "$path" ]] || return 1
    result="$(jq -c --arg key "$name" --arg path "$path" --arg sha "$(publication_hash "$path")" \
      '. + {($key):{path:$path,sha256:$sha}}' <<<"$result")"
  done
  jq -c --arg path "$scanner" --arg sha "$(publication_hash "$scanner")" \
    '. + {gitleaks:{path:$path,sha256:$sha}}' <<<"$result"
}

publication_run_scan() {
  local member="$1" commit="$2" scratch="$3" scanner="$4" status
  publication_directory "$member" || return 1
  [[ "$(git -C "$member" rev-parse HEAD)" == "$commit" \
    && -z "$(git -C "$member" status --porcelain=v1 --untracked-files=all)" ]] \
    || { publication_refuse PUBLICATION_MEMBER_SUBJECT; return 1; }
  mkdir "$scratch" "$scratch/home" "$scratch/tmp" || return 1
  # Private raw output can contain scanner finding descriptions; never upload it.
  if (ulimit -f 16384; cd "$member" && timeout --signal=TERM --kill-after=10s 600s \
    env -i "HOME=$scratch/home" "TMPDIR=$scratch/tmp" "PATH=${scanner%/*}:/usr/bin:/bin" \
    LC_ALL=C TZ=UTC GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null \
    GIT_NO_REPLACE_OBJECTS=1 GIT_NO_LAZY_FETCH=1 GIT_TERMINAL_PROMPT=0 \
    /usr/bin/bash scripts/ci-local.sh source-scan) >"$scratch/raw.log" 2>&1; then
    status=0
  else
    status=$?
  fi
  [[ "$status" -eq 0 ]] || { publication_refuse PUBLICATION_MEMBER_SCAN_FAILED; return "$status"; }
  (ulimit -f 16384; cd "$member" && timeout --signal=TERM --kill-after=5s 60s \
    env -i "HOME=$scratch/home" "PATH=${scanner%/*}:/usr/bin:/bin" \
    LC_ALL=C TZ=UTC GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_NO_REPLACE_OBJECTS=1 \
    /usr/bin/bash ops/ci/stage-artifacts.sh source-scan "$commit") >"$scratch/stage.log" 2>&1 \
    || { publication_refuse PUBLICATION_MEMBER_STAGE_FAILED; return 1; }
}

publication_member() {
  publication_host publication_integrity
  umask 077
  local family="$RUNNER_TEMP/bullet-publication-family" binary="$RUNNER_TEMP/bullet-publication-target/debug/bullet-publish"
  local destination="$RUNNER_TEMP/bullet-publication-member-report" scratch="$RUNNER_TEMP/bullet-publication-member-private"
  local scanner commit tools binary_hash observation completion
  publication_directory "$family"
  publication_file "$binary" 1073741824
  [[ -x "$binary" ]] || { publication_refuse PUBLICATION_MEMBER_VERIFIER; return 1; }
  binary_hash="$(publication_hash "$binary")"
  scanner="$(command -v gitleaks)"
  [[ "$scanner" == "$RUNNER_TEMP/bullet-tools/gitleaks" ]] \
    || { publication_refuse PUBLICATION_MEMBER_SCANNER_PATH; return 1; }
  publication_inventory "$RUNNER_TEMP/bullet-tools" 'f gitleaks'
  tools="$(publication_tools "$scanner")"
  mkdir "$destination"
  "$binary" ci-job-context "$GITHUB_WORKSPACE" "$family" bullet-farm:REQUIRED:source_scan \
    >"$destination/context.json"
  publication_json "$destination/context.json"
  commit="$(jq -er '.subject.invocation.member_commit' "$destination/context.json")"
  publication_run_scan "$family/bullet-farm" "$commit" "$scratch" "$scanner"
  mkdir "$destination/member"
  cp -R "$family/bullet-farm/.ci-upload/source-scan/.ci-artifacts" "$destination/member/"
  "$binary" ci-job-observe "$GITHUB_WORKSPACE" "$family" bullet-farm:REQUIRED:source_scan \
    "$destination/member" >"$destination/validation.json"
  publication_json "$destination/validation.json"
  [[ "$(publication_tools "$scanner")" == "$tools" && "$(publication_hash "$binary")" == "$binary_hash" ]] \
    || { publication_refuse PUBLICATION_MEMBER_TOOL_CHANGED; return 1; }
  observation="$destination/member/.ci-artifacts/observations/source-scan.json"
  jq -n --arg context "$(publication_hash "$destination/context.json")" \
    --arg validation "$(publication_hash "$destination/validation.json")" \
    --arg observation "$(publication_hash "$observation")" --arg binary "$binary_hash" \
    --arg script "$(publication_hash "$PUBLICATION_HUB/publication/ci-required.sh")" --argjson tools "$tools" '
    {schema_version:"bullet.publication-bootstrap-source-scan.v1",scope:"BOOTSTRAP_PLUS_HUB_SOURCE_SCAN",
     command:["bash","scripts/ci-local.sh","source-scan"],completed:true,exit_code:0,
     context_sha256:$context,validation_sha256:$validation,member_observation_sha256:$observation,
     verifier_sha256:$binary,executor_script_sha256:$script,tool_subjects:$tools,
     tool_closure_admitted:false,signed:false,release_authority:false,evidence_class:"DIAGNOSTIC_ONLY"}' \
    >"$destination/completion.json"
  completion="$(publication_hash "$destination/completion.json")"
  publication_validate_member "$destination" "$completion"
  publication_host publication_integrity
  [[ -f "${GITHUB_OUTPUT:-}" && ! -L "$GITHUB_OUTPUT" ]] \
    || { publication_refuse PUBLICATION_MEMBER_OUTPUT_INVALID; return 1; }
  printf 'member_completion_sha256=%s\n' "$completion" >>"$GITHUB_OUTPUT"
}

publication_validate_member() {
  local directory="$1" expected="$2" completion context validation observation manifest
  [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || { publication_refuse PUBLICATION_COMPLETION_REQUIRED; return 1; }
  publication_inventory "$directory" $'f completion.json\nf context.json\nf validation.json\nd member\nd member/.ci-artifacts\nd member/.ci-artifacts/observations\nf member/.ci-artifacts/observations/source-scan.json' || return 1
  completion="$directory/completion.json"; context="$directory/context.json"
  validation="$directory/validation.json"; observation="$directory/member/.ci-artifacts/observations/source-scan.json"
  for manifest in "$completion" "$context" "$validation" "$observation"; do publication_json "$manifest" || return 1; done
  [[ "$(publication_hash "$completion")" == "$expected" ]] \
    || { publication_refuse PUBLICATION_COMPLETION_CHANGED; return 1; }
  manifest="$GITHUB_WORKSPACE/publication.json"
  publication_json "$manifest" || return 1
  jq -e --slurpfile c "$context" --slurpfile v "$validation" --slurpfile o "$observation" \
    --slurpfile m "$manifest" --arg context "$(publication_hash "$context")" \
    --arg validation "$(publication_hash "$validation")" --arg observation "$(publication_hash "$observation")" \
    --arg executor "$(publication_hash "$PUBLICATION_HUB/publication/ci-required.sh")" \
    --arg validator "$(publication_hash "$PUBLICATION_HUB/ops/ci/artifact-check.sh")" \
    --arg event "$GITHUB_SHA" --arg tree "$(git -C "$GITHUB_WORKSPACE" rev-parse 'HEAD^{tree}')" \
    --arg manifest "$(publication_hash "$manifest")" --arg run "$GITHUB_RUN_ID" --arg attempt "$GITHUB_RUN_ATTEMPT" \
    --arg name "$GITHUB_EVENT_NAME" --arg ref "$GITHUB_WORKFLOW_REF" --arg workflow "$GITHUB_WORKFLOW_SHA" \
    --arg root_workflow "$(publication_hash "$GITHUB_WORKSPACE/.github/workflows/publication.yml")" \
    --arg nested_workflow "$(publication_hash "$PUBLICATION_HUB/.github/workflows/ci.yml")" '
    def keys_are($keys): (keys|sort) == ($keys|sort);
    def sha: type == "string" and test("^[0-9a-f]{64}$");
    keys_are(["schema_version","scope","command","completed","exit_code","context_sha256",
      "validation_sha256","member_observation_sha256","verifier_sha256","executor_script_sha256",
      "tool_subjects","tool_closure_admitted","signed","release_authority","evidence_class"]) and
    .schema_version == "bullet.publication-bootstrap-source-scan.v1" and .scope == "BOOTSTRAP_PLUS_HUB_SOURCE_SCAN" and
    .command == ["bash","scripts/ci-local.sh","source-scan"] and .completed == true and .exit_code == 0 and
    .context_sha256 == $context and .validation_sha256 == $validation and .member_observation_sha256 == $observation and
    .executor_script_sha256 == $executor and (.verifier_sha256|sha) and
    .tool_closure_admitted == false and .signed == false and .release_authority == false and .evidence_class == "DIAGNOSTIC_ONLY" and
    (.tool_subjects|keys_are(["awk","bash","cat","cmp","cp","cut","df","dirname","env","find","git","grep","head","id",
      "jq","mkdir","mv","python3","realpath","rm","rmdir","sed","sha256sum","sort","timeout","tr","wc","xargs","gitleaks"])) and
    all(.tool_subjects[]; keys_are(["path","sha256"]) and (.path|type == "string" and startswith("/")) and (.sha256|sha)) and
    .tool_subjects.gitleaks.sha256 == "50b742abd7daad8bbddb6301f3017efb680632d9a5b3b4d8f137b3aac250e359" and
    ($c[0]|keys_are(["schema_version","purpose","execution_evidence","hosted","subject","root_workflow_path","root_workflow_sha256"])) and
    $c[0].schema_version == "bullet.publication-ci-job-context.v1" and
    $c[0].purpose == "BOOTSTRAP_MEMBER_DIAGNOSTIC_VALIDATION" and $c[0].execution_evidence == false and
    $c[0].hosted == {event_sha:$event,event_name:$name,run_id:$run,run_attempt:$attempt,
      workflow_ref:$ref,workflow_sha:$workflow,job:"publication_integrity"} and
    $c[0].root_workflow_path == ".github/workflows/publication.yml" and $c[0].root_workflow_sha256 == $root_workflow and
    ($c[0].subject|keys_are(["aggregate_commit","aggregate_tree","manifest_sha256","catalog_sha256","plan_sha256","invocation_key","invocation"])) and
    $c[0].subject.aggregate_commit == $event and $c[0].subject.aggregate_tree == $tree and $c[0].subject.manifest_sha256 == $manifest and
    ($c[0].subject.catalog_sha256|sha) and ($c[0].subject.plan_sha256|sha) and
    $c[0].subject.invocation_key == "bullet-farm:REQUIRED:source_scan" and
    $c[0].subject.invocation == {member:"bullet-farm",member_commit:$m[0].members["bullet-farm"].commit,
      member_tree:$m[0].members["bullet-farm"].tree,workflow_path:".github/workflows/ci.yml",workflow_sha256:$nested_workflow,
      scope:"REQUIRED",job_id:"source_scan",matrix:{},runner:"ubuntu-24.04",condition:"success()",needs:[]} and
    ($v[0]|keys_are(["schema_version","purpose","execution_evidence","context","member_observation_path","member_observation_sha256",
      "member_observation","validation","test_inventory_kind","selected_tests","completed_tests","signed","release_authority","evidence_class"])) and
    $v[0].schema_version == "bullet.publication-ci-job-observation.v1" and $v[0].purpose == "VALIDATED_MEMBER_DIAGNOSTIC_ONLY" and
    $v[0].execution_evidence == false and $v[0].context == $c[0] and $v[0].member_observation == $o[0] and
    $v[0].member_observation_path == ".ci-artifacts/observations/source-scan.json" and $v[0].member_observation_sha256 == $observation and
    $v[0].test_inventory_kind == "SOURCE_SCAN_HAS_NO_TEST_SUITE" and $v[0].selected_tests == [] and $v[0].completed_tests == [] and
    $v[0].signed == false and $v[0].release_authority == false and $v[0].evidence_class == "DIAGNOSTIC_ONLY" and
    ($v[0].validation|keys_are(["program","script_path","script_sha256","exit_code","stdout_sha256","stderr_sha256"])) and
    $v[0].validation.program == "/usr/bin/bash" and $v[0].validation.script_path == "ops/ci/artifact-check.sh" and
    $v[0].validation.exit_code == 0 and $v[0].validation.script_sha256 == $validator and
    ($v[0].validation.stdout_sha256|sha) and ($v[0].validation.stderr_sha256|sha) and
    ($o[0]|keys_are(["schema_version","repository","commit_oid","tree_oid","clean","commands","tool_versions","outcomes",
      "artifact_hashes","signed","evidence_class"])) and
    $o[0].schema_version == "bullet.ci-observation.v1" and $o[0].repository == "bullet-farm" and
    $o[0].signed == false and $o[0].evidence_class == "DIAGNOSTIC_ONLY" and
    $o[0].tool_versions.gitleaks == "8.21.2" and ($o[0].tool_versions.python|test("^Python 3[.]12[.][0-9]+$")) and
    ($o[0].tool_versions.git|startswith("git version ")) and
    $o[0].commit_oid == $m[0].members["bullet-farm"].commit and $o[0].tree_oid == $m[0].members["bullet-farm"].tree and
    $o[0].clean == true and $o[0].outcomes == [{lane:"source-scan",status:"PASS",exit_code:0}] and
    $o[0].commands == ["bash scripts/ci-doctor.sh source-scan","bash ops/ci/source-scan.sh"] and $o[0].artifact_hashes == []
  ' "$completion" >/dev/null || { publication_refuse PUBLICATION_MEMBER_COMPLETION_INVALID; return 1; }
}

publication_validate_bootstrap() {
  local directory="$1" expected="$2" observation name digest files
  [[ "$expected" =~ ^[0-9a-f]{64}$ ]] || { publication_refuse PUBLICATION_BOOTSTRAP_COMPLETION_REQUIRED; return 1; }
  files=$'cargo-config.toml\npublication-tests.log\nreconstruct.log\nsource-scan.json\nstatus.txt\ntoolchain.txt\nverify.log\nwrapper-tests.log'
  publication_inventory "$directory" $'f observation.json\nf '"${files//$'\n'/$'\nf '}" || return 1
  observation="$directory/observation.json"
  publication_json "$observation" || return 1
  [[ "$(publication_hash "$observation")" == "$expected" ]] \
    || { publication_refuse PUBLICATION_BOOTSTRAP_COMPLETION_CHANGED; return 1; }
  jq -e --arg sha "$GITHUB_SHA" --arg run "$GITHUB_RUN_ID" --arg attempt "$GITHUB_RUN_ATTEMPT" \
    --arg event "$GITHUB_EVENT_NAME" --arg ref "$GITHUB_WORKFLOW_REF" --arg workflow "$GITHUB_WORKFLOW_SHA" \
    --arg tree "$(git -C "$GITHUB_WORKSPACE" rev-parse 'HEAD^{tree}')" \
    --arg manifest "$(publication_hash "$GITHUB_WORKSPACE/publication.json")" \
    --arg workflow_hash "$(publication_hash "$GITHUB_WORKSPACE/.github/workflows/publication.yml")" \
    --slurpfile manifest_doc "$GITHUB_WORKSPACE/publication.json" --arg files "$files" '
    (keys|sort) == (["schema_version","context","aggregate_commit","aggregate_tree","manifest_sha256","workflow_path",
      "workflow_sha256","members","artifact_sha256","evidence_class","signed","release_authority"]|sort) and
    .schema_version == "bullet.publication-ci-observation.v1" and .aggregate_commit == $sha and .aggregate_tree == $tree and
    .context == {event_sha:$sha,event_name:$event,run_id:$run,run_attempt:$attempt,workflow_ref:$ref,workflow_sha:$workflow} and
    .manifest_sha256 == $manifest and .workflow_path == ".github/workflows/publication.yml" and .workflow_sha256 == $workflow_hash and
    .members == $manifest_doc[0].members and .signed == false and .release_authority == false and .evidence_class == "DIAGNOSTIC_ONLY" and
    (.artifact_sha256|keys|sort) == ($files|split("\n")|sort) and all(.artifact_sha256[]; type == "string" and test("^[0-9a-f]{64}$"))
  ' "$observation" >/dev/null || { publication_refuse PUBLICATION_BOOTSTRAP_COMPLETION_INVALID; return 1; }
  while IFS= read -r name; do
    publication_file "$directory/$name" 16777216 || return 1
    digest="$(jq -er --arg name "$name" '.artifact_sha256[$name]' "$observation")"
    [[ "$(publication_hash "$directory/$name")" == "$digest" ]] \
      || { publication_refuse PUBLICATION_BOOTSTRAP_ARTIFACT_CHANGED; return 1; }
  done <<<"$files"
  bash "$PUBLICATION_HUB/publication/ci.sh" test-inventory "$directory/publication-tests.log" || return 1
  bash "$PUBLICATION_HUB/publication/ci.sh" wrapper-inventory "$directory/wrapper-tests.log" || return 1
  publication_json "$directory/source-scan.json" || return 1
  jq -e '. == []' "$directory/source-scan.json" >/dev/null || publication_refuse PUBLICATION_BOOTSTRAP_SCAN_INVALID
}

publication_required() {
  publication_host publication_required
  [[ "${INTEGRITY_RESULT:-}" == success ]] \
    || { publication_refuse 'publication integrity did not succeed'; return 1; }
  local root="$RUNNER_TEMP/bullet-publication-downloaded" bootstrap member
  bootstrap="publication-bootstrap-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  member="publication-hub-source-scan-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  publication_directory "$root"
  # Each subtree is separately validated; unknown sibling uploads cannot merge in.
  publication_inventory "$root" "$(printf 'd %s\n' "$bootstrap" "$member")" 1 \
    || { publication_refuse PUBLICATION_REQUIRED_UPLOAD_INVENTORY; return 1; }
  publication_validate_member "$root/$member" "${MEMBER_COMPLETION_SHA256:-}"
  publication_validate_bootstrap "$root/$bootstrap" "${BOOTSTRAP_COMPLETION_SHA256:-}"
  publication_host publication_required
  printf 'publication bootstrap and one actual Hub source scan passed; full 53-job/55-invocation CI remains required\n'
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  [[ "$#" == 1 ]] || { publication_refuse PUBLICATION_REQUIRED_USAGE; exit 2; }
  # Ambient shell/Git configuration and credentials are not passed to the lane.
  for variable in "${!GIT_@}"; do unset "$variable"; done
  export LC_ALL=C TZ=UTC GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_NO_REPLACE_OBJECTS=1
  case "$1" in
    member-run) publication_member ;;
    required) publication_required ;;
    *) publication_refuse PUBLICATION_REQUIRED_USAGE; exit 2 ;;
  esac
fi
