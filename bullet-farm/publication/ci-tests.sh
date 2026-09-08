#!/usr/bin/env bash
# Real Git, publication verifier and pinned scanner fixtures. No Cargo or remote forge.
set -euo pipefail
wrapper="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/ci.sh"
required="${wrapper%/*}/ci-required.sh"
test_binary="${BULLET_PUBLICATION_TEST_BIN:?absolute built bullet-publish fixture subject required}"
[[ "$test_binary" == /* && -f "$test_binary" && -x "$test_binary" && ! -L "$test_binary" ]]
scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
export GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_NOSYSTEM=1 GIT_NO_REPLACE_OBJECTS=1
export GIT_TERMINAL_PROMPT=0
repo="$scratch/aggregate"
mkdir "$repo"
git -C "$repo" init -q
git -C "$repo" config user.name 'Publication fixture'
git -C "$repo" config user.email fixture@bullet.invalid
printf 'public fixture\n' >"$repo/README.md"
mkdir -p "$repo/bullet-farm/publication"
printf '[extend]\nuseDefault = true\n' >"$repo/bullet-farm/publication/gitleaks.toml"
git -C "$repo" add README.md bullet-farm/publication/gitleaks.toml
git -C "$repo" commit -qm fixture
sha="$(git -C "$repo" rev-parse HEAD)"
completed="$scratch/completed-wrapper-tests.log"
: >"$completed"

pass_case() {
  printf 'publication wrapper case: %s ... ok\n' "$1" | tee -a "$completed"
}

require_refusal() {
  local code="$1"
  shift
  if "$@" >"$scratch/refusal.log" 2>&1; then
    printf 'expected refusal: %s\n' "$code" >&2
    cat "$scratch/refusal.log" >&2
    exit 1
  fi
  grep -Fq -- "$code" "$scratch/refusal.log" || {
    printf 'missing refusal: %s\n' "$code" >&2
    cat "$scratch/refusal.log" >&2
    exit 1
  }
}

expect_refusal() {
  local identity="$1"
  shift
  require_refusal "$@"
  pass_case "$identity"
}

invoke() {
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$repo" GITHUB_SHA="$sha" \
    RUNNER_TEMP="$scratch/runner" bash "$wrapper" "$@"
}

# Reviewed nextest publication subset, retained as an explicit fixture so removing
# a real test cannot silently rewrite the expected successful run in this test.
cat >"$scratch/identities" <<'IDENTITIES'
publication::observation::tests::hosted_artifact_inventory_refuses_extra_missing_empty_or_symbolic_bytes
publication::observation::tests::hosted_observation_preserves_event_and_member_identities_without_release_authority
publication::pull_request::tests::creation_receipt_reopens_same_pr_after_restart_closed_and_merged
publication::pull_request::tests::existing_human_changed_head_marker_and_duplicate_prs_refuse_creation
publication::pull_request::tests::github_api_child_has_private_configuration_and_no_inherited_credentials
publication::pull_request::tests::github_executable_refuses_relative_symlink_and_wrong_digest_subjects
publication::pull_request::tests::preexisting_closed_bot_pr_is_adopted_but_author_readback_drift_refuses
publication::pull_request::tests::receipt_substitution_and_deleted_review_ref_do_not_create_another_pr
publication::pull_request::tests::remote_drift_and_changed_intent_refuse_before_creation
publication::pull_request::tests::response_loss_reconciles_one_attempt_and_never_reposts_unknown_outcome
publication::scan::tests::actual_pinned_scanner_detects_binary_and_chunk_boundary_canaries
publication::scan::tests::actual_scan_receipt_survives_restart_and_rejects_report_tampering
publication::scan::tests::cat_file_frames_preserve_nul_newline_and_binary_bytes
publication::scan::tests::object_and_total_limits_refuse_without_truncation
publication::scan::tests::scanner_config_requires_pinned_defaults_and_refuses_external_extends
publication::scan::tests::scanner_pin_refuses_substituted_executable
publication::scan::tests::scanner_report_requires_empty_structural_array
publication::scan::tests::synthetic_root_retains_deleted_history_and_commit_metadata
publication::tests::deterministic_publication_preserves_all_exact_source_trees_and_templates
publication::tests::durable_requests_refuse_changed_inputs_and_same_tree_wrong_parent
publication::tests::publication_manifest_refuses_duplicates_unknowns_paths_and_noncanonical_bytes
publication::tests::publication_refuses_dirty_hidden_flags_and_symlinked_checkouts
publication::tests::publication_refuses_history_and_endpoint_substitution
publication::tests::publication_rejects_template_and_member_subtree_drift
publication::transport_tests::actual_prepare_recovers_interruption_after_refs_and_rejects_changed_request
publication::transport_tests::atomic_source_publication_reconstructs_real_checkouts_and_reconciles_response_loss
publication::transport_tests::conflicting_or_stale_remote_ref_cannot_partially_publish
publication::transport_tests::git_basic_auth_encoding_and_unrelated_child_custody_are_exact
publication::transport_tests::reconstructed_source_ref_drift_is_refused_before_checkout
publication::ci_render::tests::preview_admits_exact_sources_without_claiming_existing_root_integrity
publication::ci_render::tests::authentic_v1_roots_keep_original_request_and_commit_bytes
publication::ci_render::tests::generated_roots_refuse_workflow_template_and_path_drift
publication::ci_render::tests::generated_topology_preserves_dependencies_matrices_events_and_real_producer
publication::ci_render::tests::tree_rendering_works_before_shallow_source_commits_are_available
IDENTITIES
sed 's/^/test /; s/$/ ... ok/' "$scratch/identities" >"$scratch/valid-tests.log"
summary='test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 300 filtered out; finished in 1.23s'
printf '%s\n' "$summary" >>"$scratch/valid-tests.log"
bash "$wrapper" test-inventory "$scratch/valid-tests.log"
pass_case rust_inventory_exact
tac "$scratch/valid-tests.log" | sed 's/300 filtered out/0 filtered out/; s/1.23s$/20.001s/' \
  >"$scratch/reordered-tests.log"
bash "$wrapper" test-inventory "$scratch/reordered-tests.log"
pass_case rust_inventory_reordered

for spec in \
  'missing_test|1d' \
  'renamed_test|1s/hosted_artifact_inventory/renamed_inventory/' \
  'failed_test|1s/ ... ok$/ ... FAILED/' \
  'ignored_test|1s/ ... ok$/ ... ignored/' \
  'skipped_test|1s/ ... ok$/ ... skipped/' \
  'malformed_test|1s/ ... ok$/ malformed/' \
  'foreign_test|1s/publication::/unrelated::/' \
  'missing_summary|/^test result/d' \
  'wrong_pass_count|/^test result/s/34 passed/33 passed/' \
  'zero_pass_count|/^test result/s/34 passed/0 passed/' \
  'ignored_summary|/^test result/s/0 ignored/1 ignored/' \
  'malformed_duration|/^test result/s/1.23s/1..23s/' \
  'failed_summary|/^test result/s/ok\./FAILED./'; do
  identity="${spec%%|*}"
  change="${spec#*|}"
  sed "$change" "$scratch/valid-tests.log" >"$scratch/invalid-tests.log"
  expect_refusal "rust_inventory_$identity" PUBLICATION_TEST_INVENTORY_INVALID \
    bash "$wrapper" test-inventory "$scratch/invalid-tests.log"
done
awk 'NR == 1 {first = $0} NR == 2 {$0 = first} {print}' "$scratch/valid-tests.log" \
  >"$scratch/duplicate-tests.log"
expect_refusal rust_inventory_duplicate_test PUBLICATION_TEST_INVENTORY_INVALID \
  bash "$wrapper" test-inventory "$scratch/duplicate-tests.log"
for spec in \
  "duplicate_summary|$summary" \
  'extra_test|test publication::unexpected ... ok' \
  'extra_failed_test|test unrelated::unexpected ... FAILED' \
  'extra_malformed_test|test malformed'; do
  identity="${spec%%|*}"
  extra="${spec#*|}"
  cat "$scratch/valid-tests.log" >"$scratch/extra-tests.log"
  printf '%s\n' "$extra" >>"$scratch/extra-tests.log"
  expect_refusal "rust_inventory_$identity" PUBLICATION_TEST_INVENTORY_INVALID \
    bash "$wrapper" test-inventory "$scratch/extra-tests.log"
done
ln -s "$scratch/valid-tests.log" "$scratch/symlink-tests.log"
: >"$scratch/empty-tests.log"
for spec in "symlink_log|$scratch/symlink-tests.log" "empty_log|$scratch/empty-tests.log" \
  "missing_log|$scratch/missing-tests.log" "directory_log|$scratch"; do
  identity="${spec%%|*}"
  invalid="${spec#*|}"
  expect_refusal "rust_inventory_$identity" PUBLICATION_TEST_INVENTORY_INVALID \
    bash "$wrapper" test-inventory "$invalid"
done
expect_refusal rust_inventory_missing_argument PUBLICATION_CI_USAGE bash "$wrapper" test-inventory
expect_refusal rust_inventory_extra_argument PUBLICATION_CI_USAGE \
  bash "$wrapper" test-inventory "$scratch/valid-tests.log" extra

mkdir "$scratch/runner"
expect_refusal usage_missing_command PUBLICATION_CI_USAGE bash "$wrapper"
expect_refusal usage_unknown_command PUBLICATION_CI_USAGE bash "$wrapper" arbitrary
expect_refusal usage_source_scan_extra_argument PUBLICATION_CI_USAGE bash "$wrapper" source-scan extra
expect_refusal runner_not_github_actions PUBLICATION_DISPOSABLE_CI_REQUIRED \
  env GITHUB_ACTIONS=false bash "$wrapper" source-scan
expect_refusal runner_event_sha_mismatch PUBLICATION_EVENT_SHA_MISMATCH \
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$repo" \
  GITHUB_SHA=0000000000000000000000000000000000000000 \
  RUNNER_TEMP="$scratch/runner" bash "$wrapper" source-scan
expect_refusal runner_event_sha_invalid PUBLICATION_EVENT_SHA_INVALID \
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$repo" GITHUB_SHA=main \
  RUNNER_TEMP="$scratch/runner" bash "$wrapper" source-scan
printf 'dirty\n' >"$repo/untracked.txt"
expect_refusal runner_dirty_checkout PUBLICATION_CHECKOUT_DIRTY invoke source-scan
rm "$repo/untracked.txt"
ln -s "$scratch/runner" "$scratch/runner-link"
expect_refusal runner_temp_symlink PUBLICATION_RUNNER_TEMP_INVALID \
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$repo" GITHUB_SHA="$sha" \
  RUNNER_TEMP="$scratch/runner-link" bash "$wrapper" source-scan
ln -s "$repo" "$scratch/aggregate-link"
expect_refusal runner_checkout_symlink PUBLICATION_CHECKOUT_INVALID \
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$scratch/aggregate-link" GITHUB_SHA="$sha" \
  RUNNER_TEMP="$scratch/runner" bash "$wrapper" source-scan
expect_refusal source_scan_receipt_missing PUBLICATION_SOURCE_SCAN_REQUIRED invoke prove
mkdir "$scratch/runner/bullet-publication-report" "$scratch/runner/bullet-publication-private"
printf '{"success":true}\n' >"$scratch/runner/bullet-publication-report/source-scan.json"
expect_refusal source_scan_receipt_malformed PUBLICATION_SOURCE_SCAN_REQUIRED invoke prove
rm -r "$scratch/runner/bullet-publication-report" "$scratch/runner/bullet-publication-private"
invoke source-scan
[[ "$(cat "$scratch/runner/bullet-publication-report/source-scan.json")" == '[]' ]]
[[ ! -e "$scratch/runner/bullet-publication-cargo" ]]
pass_case source_scan_clean
expect_refusal source_scan_repeated 'File exists' invoke source-scan
mkdir "$scratch/canary-runner"
printf 'aws_access_key_id = %s%s\n' 'AKIA' '6QWERTYUIOPASDFG' >"$repo/canary.txt"
git -C "$repo" add canary.txt
git -C "$repo" commit -qm 'intentional scanner fixture'
require_refusal PUBLICATION_SOURCE_SCAN_FAILED \
  env GITHUB_ACTIONS=true GITHUB_WORKSPACE="$repo" \
  GITHUB_SHA="$(git -C "$repo" rev-parse HEAD)" RUNNER_TEMP="$scratch/canary-runner" \
  bash "$wrapper" source-scan
[[ ! -e "$scratch/canary-runner/bullet-publication-report/source-scan.json" ]]
[[ "$(find "$scratch/canary-runner/bullet-publication-report" -type f | wc -l)" == 1 ]]
pass_case source_scan_canary

# Exercise the complete production member-run path with the real verifier and
# exact workflow pins. These ordinary temporary repositories contain no provider
# identity and never mutate the canonical family or access a remote forge.
canonical="${wrapper%/publication/ci.sh}"
runner="$scratch/member-runner"
family="$runner/bullet-publication-family"
aggregate="$scratch/member-aggregate"
mkdir "$runner" "$family" "$aggregate" "$runner/bullet-tools" \
  "$runner/bullet-publication-target" "$runner/bullet-publication-target/debug"
cp "$test_binary" "$runner/bullet-publication-target/debug/bullet-publish"
cp "$(realpath -e "$(command -v gitleaks)")" "$runner/bullet-tools/gitleaks"
cp "$canonical/publication/root/repos.manifest.toml" "$family/repos.manifest.toml"
for member in bullet-farm bullet-kernel bullet-git bullet-portal; do
  subject="$family/$member"
  git init -q --template= "$subject"
  git -C "$subject" config user.name 'Publication member fixture'
  git -C "$subject" config user.email fixture@bullet.invalid
  mkdir -p "$subject/.github/workflows"
  for workflow in ci.yml scheduled.yml; do
    cp "$canonical/../$member/.github/workflows/$workflow" "$subject/.github/workflows/$workflow"
  done
  printf '.ci-artifacts/\n.ci-upload/\ntarget/\n' >"$subject/.gitignore"
  if [[ "$member" == bullet-farm ]]; then
    cp -R "$canonical/publication" "$subject/publication"
    cp "$canonical/.node-version" "$canonical/.npm-version" "$subject/"
    mkdir "$subject/scripts" "$subject/ops" "$subject/ops/ci"
    for script in ci-local.sh ci-doctor.sh ci-observation.sh; do
      cp "$canonical/scripts/$script" "$subject/scripts/$script"
    done
    for script in lib.sh artifact-path.sh artifact-check.sh tool-version.sh toolchain-pins.sh \
      rust-toolchain-boundary.sh strict-json.sh stage-artifacts.sh family-custody.sh scratch-floor.sh source-scan.sh; do
      cp "$canonical/ops/ci/$script" "$subject/ops/ci/$script"
    done
  fi
  git -C "$subject" add .
  git -C "$subject" commit -qm 'actual scanner fixture sources'
done
git -C "$aggregate" init -q --template=
git -C "$aggregate" config user.name 'Publication aggregate fixture'
git -C "$aggregate" config user.email fixture@bullet.invalid
for member in bullet-farm bullet-kernel bullet-git bullet-portal; do
  commit="$(git -C "$family/$member" rev-parse HEAD)"
  git -C "$aggregate" fetch -q --no-tags "$family/$member" "$commit"
  git -C "$aggregate" read-tree --prefix="$member/" -u "$commit"
done
cp -R "$family/bullet-farm/publication/root/." "$aggregate/"
printf '%s\n' "$("$test_binary" inspect "$family")" >"$aggregate/publication.json"
git -C "$aggregate" add .
git -C "$aggregate" commit -qm 'temporary exact source subjects before root generation'
# Expected roots are independently checked only after the actual renderer output
# is materialized. A preview does not admit existing workflow bytes or execution.
require_refusal PUBLICATION_ROOT_MODE_DRIFT "$test_binary" verify "$aggregate"
"$test_binary" ci-root-files "$aggregate" >"$scratch/expected-roots.json"
"$test_binary" ci-root-files "$aggregate" >"$scratch/repeated-roots.json"
cmp "$scratch/expected-roots.json" "$scratch/repeated-roots.json"
jq -e '.purpose == "EXPECTED_ROOT_FILES_ONLY" and .execution_evidence == false and (.files | length == 8)' \
  "$scratch/expected-roots.json" >/dev/null
while IFS= read -r path; do
  mkdir -p "$(dirname "$aggregate/$path")"
  jq -jr --arg path "$path" '.files[$path]' "$scratch/expected-roots.json" >"$aggregate/$path"
done < <(jq -r '.files | keys[]' "$scratch/expected-roots.json")
git -C "$aggregate" add .
git -C "$aggregate" commit -qm 'actual generated version two aggregate'
"$test_binary" verify "$aggregate"
"$test_binary" ci-plan "$aggregate" >"$scratch/v2-plan.json"
jq -e '.plan.job_definition_count == 53 and .plan.invocation_count == 55 and .plan.execution_evidence == false' \
  "$scratch/v2-plan.json" >/dev/null
# Reconstruct fresh ordinary member checkouts from only the aggregate objects,
# with commit and tree read-back against the emitted publication manifest.
reconstructed="$scratch/independently-reconstructed"
mkdir "$reconstructed"
cp "$aggregate/repos.manifest.toml" "$reconstructed/repos.manifest.toml"
for member in bullet-farm bullet-kernel bullet-git bullet-portal; do
  commit="$(jq -r --arg member "$member" '.members[$member].commit' "$aggregate/publication.json")"
  tree="$(jq -r --arg member "$member" '.members[$member].tree' "$aggregate/publication.json")"
  git init -q --template= "$reconstructed/$member"
  git -C "$reconstructed/$member" fetch -q --no-tags "$aggregate" "$commit"
  git -C "$reconstructed/$member" checkout -q --detach "$commit"
  [[ "$(git -C "$reconstructed/$member" rev-parse HEAD)" == "$commit" ]]
  [[ "$(git -C "$reconstructed/$member" rev-parse 'HEAD^{tree}')" == "$tree" ]]
done
printf '%s\n' "$("$test_binary" inspect "$reconstructed")" >"$scratch/reconstructed-manifest.json"
cmp "$aggregate/publication.json" "$scratch/reconstructed-manifest.json"
pass_case actual_v2_roots
aggregate_sha="$(git -C "$aggregate" rev-parse HEAD)"
: >"$runner/outputs"
hosted=(env "PATH=$runner/bullet-tools:/usr/bin:/bin" GITHUB_ACTIONS=true
  "GITHUB_WORKSPACE=$aggregate" "GITHUB_SHA=$aggregate_sha" "GITHUB_WORKFLOW_SHA=$aggregate_sha"
  GITHUB_EVENT_NAME=pull_request GITHUB_RUN_ID=123 GITHUB_RUN_ATTEMPT=2
  GITHUB_WORKFLOW_REF=neverhuman/bulletfarm/.github/workflows/publication.yml@refs/pull/7/merge
  "RUNNER_TEMP=$runner" "GITHUB_OUTPUT=$runner/outputs")
"${hosted[@]}" GITHUB_JOB=publication_integrity bash "$required" member-run
member_digest="$(sed -n 's/^member_completion_sha256=//p' "$runner/outputs")"
[[ "$member_digest" =~ ^[0-9a-f]{64}$ ]]
member_report="$runner/bullet-publication-member-report"
jq -e '.completed == true and .exit_code == 0 and .tool_closure_admitted == false' "$member_report/completion.json" >/dev/null
jq -e '.execution_evidence == false and .member_observation.outcomes[0].status == "PASS"' "$member_report/validation.json" >/dev/null
[[ "$(git -C "$aggregate" rev-parse HEAD)" == "$aggregate_sha" ]]
printf 'unadmitted helper\n' >"$runner/bullet-tools/git"
require_refusal PUBLICATION_REQUIRED_INVENTORY "${hosted[@]}" GITHUB_JOB=publication_integrity bash "$required" member-run
rm "$runner/bullet-tools/git"

# The bootstrap report is a test fixture with the independently enumerated
# 34 Rust and 48 shell identities; its observation is emitted by the real CLI.
bootstrap_report="$runner/bullet-publication-report"
mkdir "$bootstrap_report"
printf '[build]\njobs=2\n' >"$bootstrap_report/cargo-config.toml"
cp "$scratch/valid-tests.log" "$bootstrap_report/publication-tests.log"
for name in reconstruct status toolchain verify; do printf 'fixture %s\n' "$name" >"$bootstrap_report/$name.log"; done
mv "$bootstrap_report/status.log" "$bootstrap_report/status.txt"
mv "$bootstrap_report/toolchain.log" "$bootstrap_report/toolchain.txt"
printf '[]\n' >"$bootstrap_report/source-scan.json"
cp "$completed" "$bootstrap_report/wrapper-tests.log"
for identity in final_success final_failure final_cancelled final_skipped final_neutral final_malformed final_empty; do
  printf 'publication wrapper case: %s ... ok\n' "$identity" >>"$bootstrap_report/wrapper-tests.log"
done
printf 'publication wrapper fixtures: 48 passed; 0 failed; 0 skipped\n' >>"$bootstrap_report/wrapper-tests.log"
"${hosted[@]}" GITHUB_JOB=publication_integrity "$test_binary" ci-observe "$aggregate" "$family" "$bootstrap_report"
bootstrap_digest="$(sha256sum "$bootstrap_report/observation.json" | cut -d ' ' -f 1)"
downloads="$runner/bullet-publication-downloaded"
mkdir "$downloads"
cp -R "$bootstrap_report" "$downloads/publication-bootstrap-123-2"
cp -R "$member_report" "$downloads/publication-hub-source-scan-123-2"
final_command=("${hosted[@]}" GITHUB_JOB=publication_required
  "BOOTSTRAP_COMPLETION_SHA256=$bootstrap_digest" "MEMBER_COMPLETION_SHA256=$member_digest"
  bash "$required" required)
env INTEGRITY_RESULT=success "${final_command[@]}"

# The actual consumed final check rejects a plausible PASS without completion,
# response-loss/stale-attempt subjects, altered scripts and extra artifact bytes.
downloaded_member="$downloads/publication-hub-source-scan-123-2"
for mutation in missing empty symbolic extra duplicate changed_validator changed_command incomplete; do
  cp -R "$downloaded_member" "$scratch/member-good"
  completion="$downloaded_member/completion.json"
  expected="$member_digest"
  case "$mutation" in
    missing) rm "$completion" ;;
    empty) : >"$completion" ;;
    symbolic) rm "$completion"; ln -s "$member_report/completion.json" "$completion" ;;
    extra) printf 'unadmitted\n' >"$downloaded_member/extra.json" ;;
    duplicate) sed -i '1s/{/{"scope":"duplicate",/' "$completion" ;;
    changed_validator)
      jq '.validation.script_sha256 = ("0" * 64)' "$downloaded_member/validation.json" >"$scratch/changed.json"
      mv "$scratch/changed.json" "$downloaded_member/validation.json"
      digest="$(sha256sum "$downloaded_member/validation.json" | cut -d ' ' -f 1)"
      jq --arg digest "$digest" '.validation_sha256 = $digest' "$completion" >"$scratch/changed.json"
      mv "$scratch/changed.json" "$completion"
      expected="$(sha256sum "$completion" | cut -d ' ' -f 1)"
      ;;
    changed_command|incomplete)
      field='.command = ["true"]'
      [[ "$mutation" != incomplete ]] || field='.completed = false'
      jq "$field" "$completion" >"$scratch/changed.json"
      mv "$scratch/changed.json" "$completion"
      expected="$(sha256sum "$completion" | cut -d ' ' -f 1)"
      ;;
  esac
  if "${hosted[@]}" GITHUB_JOB=publication_required INTEGRITY_RESULT=success \
    "BOOTSTRAP_COMPLETION_SHA256=$bootstrap_digest" "MEMBER_COMPLETION_SHA256=$expected" \
    bash "$required" required >"$scratch/member-refusal.log" 2>&1; then
    printf 'accepted hostile member artifact: %s\n' "$mutation" >&2; exit 1
  fi
  grep -Eq 'PUBLICATION_|STRICT_JSON_' "$scratch/member-refusal.log"
  rm -r "$downloaded_member"
  mv "$scratch/member-good" "$downloaded_member"
done
require_refusal PUBLICATION_COMPLETION_REQUIRED "${hosted[@]}" GITHUB_JOB=publication_required \
  INTEGRITY_RESULT=success "BOOTSTRAP_COMPLETION_SHA256=$bootstrap_digest" \
  MEMBER_COMPLETION_SHA256= bash "$required" required
require_refusal PUBLICATION_REQUIRED_UPLOAD_INVENTORY "${hosted[@]}" GITHUB_JOB=publication_required \
  INTEGRITY_RESULT=success GITHUB_RUN_ATTEMPT=3 "BOOTSTRAP_COMPLETION_SHA256=$bootstrap_digest" \
  "MEMBER_COMPLETION_SHA256=$member_digest" bash "$required" required
printf 'changed\n' >>"$downloads/publication-bootstrap-123-2/verify.log"
require_refusal PUBLICATION_BOOTSTRAP_ARTIFACT_CHANGED env INTEGRITY_RESULT=success "${final_command[@]}"
cp "$bootstrap_report/verify.log" "$downloads/publication-bootstrap-123-2/verify.log"
env INTEGRITY_RESULT=success "${final_command[@]}"

# A pre-existing valid observation cannot substitute for an actual failed scan.
printf 'aws_access_key_id = %s%s\n' 'AKIA' '6QWERTYUIOPASDFG' >"$family/bullet-farm/canary.txt"
git -C "$family/bullet-farm" add canary.txt
git -C "$family/bullet-farm" commit -qm 'intentional scanner canary'
cat >"$scratch/run-scan.sh" <<'SCAN'
source "$1"
publication_run_scan "$2" "$3" "$4" "$5"
SCAN
require_refusal PUBLICATION_MEMBER_SCAN_FAILED bash "$scratch/run-scan.sh" \
  "$required" "$family/bullet-farm" "$(git -C "$family/bullet-farm" rev-parse HEAD)" \
  "$runner/failed-member-private" "$runner/bullet-tools/gitleaks"
pass_case final_success
for result in failure cancelled skipped neutral malformed ''; do
  expect_refusal "final_${result:-empty}" 'publication integrity did not succeed' \
    env INTEGRITY_RESULT="$result" "${final_command[@]}"
done
printf 'publication wrapper fixtures: 48 passed; 0 failed; 0 skipped\n' >>"$completed"
bash "$wrapper" wrapper-inventory "$completed"
printf 'publication wrapper fixtures: 48 passed; 0 failed; 0 skipped\n'
