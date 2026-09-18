#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

test_root="$(mktemp -d)"
cleanup() {
  rm -rf -- "$test_root" .ci-artifacts/family
  rm -f -- .ci-artifacts/observations/family-contract.json
}
trap cleanup EXIT HUP INT TERM

commit="$(git rev-parse HEAD)"
tree="$(git rev-parse 'HEAD^{tree}')"
subjects="$(jq -cn --arg commit "$commit" --arg tree "$tree" '
  {"bullet-farm":{commit_oid:("sha1:"+$commit),tree_oid:("sha1:"+$tree),clean:true},
   "bullet-git":{commit_oid:("sha1:"+$commit),tree_oid:("sha1:"+$tree),clean:true},
   "bullet-kernel":{commit_oid:("sha1:"+$commit),tree_oid:("sha1:"+$tree),clean:true},
   "bullet-portal":{commit_oid:("sha1:"+$commit),tree_oid:("sha1:"+$tree),clean:true}}
')"
junit='{"kind":"junit","tests":3,"executed":3,"failures":0,"errors":0,"skipped":0}'
vitest='{"kind":"vitest","tests":3,"passed":3,"failed":0,"pending":0,"todo":0}'
formal='{"kind":"formal","models":2,"completed_models":2,"pinned_summary_present":true,"status":"PASS"}'
formal_log='{"kind":"formal-log","models":2,"completed_without_error":2,"pinned_summary_present":true,"exit_code":0}'
reports="$(jq -cn --argjson junit "$junit" --argjson vitest "$vitest" \
  --argjson formal "$formal" --argjson formal_log "$formal_log" '
  [
    {id:"bullet-git-fast",repository:"bullet-git",summary:$junit},
    {id:"bullet-git-contract",repository:"bullet-git",summary:$junit},
    {id:"bullet-kernel-fast",repository:"bullet-kernel",summary:$junit},
    {id:"bullet-kernel-contract",repository:"bullet-kernel",summary:$junit},
    {id:"bullet-kernel-family",repository:"bullet-kernel",summary:$junit},
    {id:"bullet-portal-vitest",repository:"bullet-portal",summary:$vitest},
    {id:"bullet-portal-playwright",repository:"bullet-portal",summary:$junit},
    {id:"bullet-portal-real-farmd",repository:"bullet-portal",summary:$junit},
    {id:"bullet-farm-contract",repository:"bullet-farm",summary:$junit},
    {id:"bullet-farm-formal",repository:"bullet-farm",summary:$formal},
    {id:"bullet-farm-formal-log",repository:"bullet-farm",summary:$formal_log}
  ]
')"
candidate="$test_root/candidate.json"
jq -n --argjson subjects "$subjects" --argjson reports "$reports" --arg commit "$commit" '
  {schema_version:"bullet.family-ci-observation.v1",subjects:$subjects,reports:$reports,
   sole_writer_daemon:{repository:"bullet-git",commit_oid:("sha1:"+$commit),
     build:{cargo_locked:true,incremental:false,fresh_target:true,offline:true,toolchain:"1.97.1",
       binary_hash_verified_during_run:true}},
   tool_versions:{
     hub_rustc:"rustc 1.95.0 (123456789 2026-01-01)",
     hub_cargo:"cargo 1.95.0 (123456789 2026-01-01)",
     bullet_git_rustc:"rustc 1.97.1 (123456789 2026-01-01)",
     bullet_git_cargo:"cargo 1.97.1 (123456789 2026-01-01)",
     node:"v22.23.2",npm:"10.9.8",b3sum:"b3sum 1.8.2"},
   signed:false,evidence_class:"DIAGNOSTIC_ONLY",release_authority:false}
' >"$candidate"

first="$test_root/first.json"
second="$test_root/second.json"
bash ops/ci/family-observation.sh write "$candidate" "$first" >/dev/null
bash ops/ci/family-observation.sh write "$candidate" "$second" >/dev/null
cmp -s -- "$first" "$second" \
  || { refuse FAMILY_OBSERVATION_NONDETERMINISTIC 'two writes differ'; exit 1; }
bash ops/ci/family-observation.sh check "$first" >/dev/null
python_bin="$(command -v python3 || command -v python)"
[[ "$("$python_bin" --version 2>&1)" == 'Python 3.12.'* ]] \
  || { refuse FAMILY_OBSERVATION_TEST_PYTHON_INVALID "$python_bin"; exit 1; }
[[ "$("$python_bin" -I -S -c \
  'import pathlib,sys; print(pathlib.Path(sys.argv[1]).read_bytes()[-1:] == b"}")' \
  "$first")" == True ]] \
  || { refuse FAMILY_OBSERVATION_CANONICAL_BYTES_INVALID "$first"; exit 1; }
jq -e '
  (.observation_id | test("^blake3:[0-9a-f]{64}$")) and
  ([.. | objects | keys[]] | any(. == "sha256" or . == "duration" or . == "timestamp") | not) and
  .signed == false and .evidence_class == "DIAGNOSTIC_ONLY" and .release_authority == false
' "$first" >/dev/null \
  || { refuse FAMILY_OBSERVATION_NONAUTHORITY_GUARD_FAILED "$first"; exit 1; }

expect_refusal() {
  local expected="$1" document="$2" output
  if output="$(bash ops/ci/family-observation.sh check "$document" 2>&1)"; then
    refuse FAMILY_OBSERVATION_HOSTILE_ADMITTED "$expected"; exit 1
  fi
  [[ "$output" == *"$expected"* ]] \
    || { refuse FAMILY_OBSERVATION_WRONG_REFUSAL "$expected"; exit 1; }
}

jq -cS '.observation_id="blake3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"' \
  "$first" | "$python_bin" -I -S -c \
    'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().rstrip(b"\n"))' \
  >"$test_root/wrong-identity.json"
expect_refusal FAMILY_OBSERVATION_IDENTITY_MISMATCH "$test_root/wrong-identity.json"
jq . "$first" >"$test_root/pretty.json"
expect_refusal FAMILY_OBSERVATION_NONCANONICAL "$test_root/pretty.json"
jq '.tool_versions.node="v22.23.3"' "$first" >"$test_root/node-drift.json"
expect_refusal FAMILY_OBSERVATION_SCHEMA_INVALID "$test_root/node-drift.json"
jq '.tool_versions.npm="10.9.9"' "$first" >"$test_root/npm-drift.json"
expect_refusal FAMILY_OBSERVATION_SCHEMA_INVALID "$test_root/npm-drift.json"
jq '.subjects["bullet-git"].commit_oid="ffffffffffffffffffffffffffffffffffffffff"' \
  "$first" >"$test_root/untagged-oid.json"
expect_refusal FAMILY_OBSERVATION_SCHEMA_INVALID "$test_root/untagged-oid.json"
jq -cS '.subjects["bullet-git"].tree_oid="sha1:ffffffffffffffffffffffffffffffffffffffff"' \
  "$first" | "$python_bin" -I -S -c \
    'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().rstrip(b"\n"))' \
  >"$test_root/nonhub-substitution.json"
expect_refusal FAMILY_OBSERVATION_IDENTITY_MISMATCH "$test_root/nonhub-substitution.json"
jq '.reports[0].summary={kind:"vitest",tests:3,passed:3,failed:0,pending:0,todo:0}' \
  "$first" >"$test_root/wrong-kind.json"
expect_refusal FAMILY_OBSERVATION_SEMANTICS_INVALID "$test_root/wrong-kind.json"
jq '.reports[0].summary.raw_log="credential-shaped output"' \
  "$first" >"$test_root/raw-field.json"
expect_refusal FAMILY_OBSERVATION_SCHEMA_INVALID "$test_root/raw-field.json"
jq '.reports[0] as $first | .reports[0]=.reports[1] | .reports[1]=$first' \
  "$first" >"$test_root/reordered.json"
expect_refusal FAMILY_OBSERVATION_SEMANTICS_INVALID "$test_root/reordered.json"
jq '.signed=true' "$first" >"$test_root/signed.json"
expect_refusal FAMILY_OBSERVATION_SCHEMA_INVALID "$test_root/signed.json"
jq '.reports[0].summary.executed=2 | del(.observation_id)' "$first" \
  >"$test_root/counter-candidate.json"
if bash ops/ci/family-observation.sh write "$test_root/counter-candidate.json" \
  "$test_root/counter.json" >/dev/null 2>&1; then
  refuse FAMILY_OBSERVATION_COUNTER_GUARD_FAILED junit; exit 1
fi
{
  printf '%s' '{"schema_version":"hostile-duplicate",'
  sed 's/^{//' "$first"
} >"$test_root/duplicate.json"
expect_refusal FAMILY_OBSERVATION_JSON_INVALID "$test_root/duplicate.json"

mkdir -p .ci-artifacts/family .ci-artifacts/observations
cp "$first" .ci-artifacts/family/subjects.json
CI_COMMAND_COUNT=2 bash scripts/ci-observation.sh family-contract 0 \
  'bash scripts/ci-doctor.sh family-contract' 'bash ops/ci/family-contract.sh' \
  .ci-artifacts/family/subjects.json >/dev/null
# The meta-test itself runs from a dirty source tree and may inherit optional
# ambient Node/npm binaries. Bind the synthetic outer observation to the clean,
# pinned family subject that the real family lane requires.
jq '.clean=true |
  .tool_versions.node="v22.23.2" |
  .tool_versions.npm="10.9.8"' \
  .ci-artifacts/observations/family-contract.json \
  >"$test_root/outer.json"
mv "$test_root/outer.json" .ci-artifacts/observations/family-contract.json
bash ops/ci/artifact-check.sh family-contract >/dev/null

jq --arg wrong "sha1:$(printf 'f%.0s' {1..40})" '.subjects["bullet-farm"].commit_oid=$wrong |
  del(.observation_id)' "$first" >"$test_root/substituted-candidate.json"
bash ops/ci/family-observation.sh write "$test_root/substituted-candidate.json" \
  .ci-artifacts/family/subjects.json >/dev/null
new_sha="$(sha256_file .ci-artifacts/family/subjects.json)"
jq --arg digest "$new_sha" '
  (.artifact_hashes[] | select(.path == ".ci-artifacts/family/subjects.json").sha256)=$digest
' .ci-artifacts/observations/family-contract.json >"$test_root/outer.json"
mv "$test_root/outer.json" .ci-artifacts/observations/family-contract.json
if output="$(bash ops/ci/artifact-check.sh family-contract 2>&1)" \
  || [[ "$output" != *CI_FAMILY_SUBJECT_INVALID* ]]; then
  refuse FAMILY_OBSERVATION_OUTER_SUBSTITUTION_GUARD_FAILED subject; exit 1
fi

family_source="$(<ops/ci/family.sh)"
dollar='$'
source_offset() {
  local needle="$1" prefix
  [[ "$family_source" == *"$needle"* ]] || return 1
  prefix="${family_source%%"$needle"*}"
  printf '%s\n' "${#prefix}"
}
pre_hash_offset="$(source_offset "source_hash_before=\"${dollar}(sha256_file \"${dollar}report\")\"")"
copy_offset="$(source_offset "cp -P -- \"${dollar}report\" \"${dollar}snapshot\"")"
snapshot_equality_offset="$(source_offset 'FAMILY_REPORT_CHANGED_DURING_SNAPSHOT')"
snapshot_parser_offset="$(source_offset "family-report-check.sh junit \"${dollar}snapshot\"")"
post_parse_equality_offset="$(source_offset 'FAMILY_REPORT_CHANGED_DURING_PARSE')"
publication_offset="$(source_offset 'family-observation.sh write')"
final_source_recheck_offset="$(source_offset 'FAMILY_REPORT_CHANGED_AFTER_HASH')"
[[ "$family_source" == *"env -i HOME=\"${dollar}{HOME:?}\" PATH=\"${dollar}PATH\" LC_ALL=C TZ=UTC"* \
  && "$family_source" == *'bash --noprofile --norc -c'* \
  && "$family_source" == *'rustup run 1.97.1 cargo build --locked'* \
  && "$family_source" == *'CARGO_INCREMENTAL=0 CARGO_TARGET_DIR='* \
  && "$family_source" == *'CARGO_NET_OFFLINE=true'* \
  && "$pre_hash_offset" -lt "$copy_offset" \
  && "$copy_offset" -lt "$snapshot_equality_offset" \
  && "$snapshot_equality_offset" -lt "$snapshot_parser_offset" \
  && "$snapshot_parser_offset" -lt "$post_parse_equality_offset" \
  && "$post_parse_equality_offset" -lt "$publication_offset" \
  && "$publication_offset" -lt "$final_source_recheck_offset" \
  && "$family_source" == *'assert_family_subjects after-observation-publication'* ]] \
  || { refuse FAMILY_GIT_TOOLCHAIN_ISOLATION_GUARD_FAILED stage-2; exit 1; }

log 'family observation schema, identity, determinism, substitution, and toolchain-isolation guards passed'
