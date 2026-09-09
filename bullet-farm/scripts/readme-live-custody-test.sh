#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
test_root="$(mktemp -d)"
trap 'rm -rf -- "$test_root"' EXIT
fixture="$test_root/hub"
mkdir -p "$fixture/scripts" "$test_root/forbidden"
for script in readme-live-record.sh readme-live-render.sh readme-live-check.sh readme-schema-check.sh; do
  cp "$HUB/scripts/$script" "$fixture/scripts/"
done
real_cp="$(command -v cp)"
real_mv="$(command -v mv)"
export README_TEST_MARKER="$test_root/forbidden-called"
for tool in claude codex cursor-agent docker curl npm node; do
  # shellcheck disable=SC2016 # The marker variable expands in the generated fixture.
  printf '%s\n' '#!/usr/bin/env bash' ': >"$README_TEST_MARKER"' 'exit 93' >"$test_root/forbidden/$tool"
  chmod 700 "$test_root/forbidden/$tool"
done
export PATH="$test_root/forbidden:$PATH"
passed=0

pass() { passed=$((passed + 1)); printf 'ok %s\n' "$1"; }

expect_exit() {
  local expected="$1" reason="$2" code=0
  shift 2
  "$@" >"$test_root/last.log" 2>&1 || code=$?
  if [[ "$code" != "$expected" ]]; then
    cat "$test_root/last.log" >&2
    printf 'expected exit %s; got %s\n' "$expected" "$code" >&2
    exit 1
  fi
  if [[ -n "$reason" ]] && ! grep -Fq -- "$reason" "$test_root/last.log"; then
    cat "$test_root/last.log" >&2
    printf 'missing expected refusal: %s\n' "$reason" >&2
    exit 1
  fi
}

write_capture() {
  local root="$1" name sha bytes artifacts='[]'
  mkdir -p "$root"
  printf '%s\n' 'Actual first line.' 'Actual second line.' >"$root/stdout.txt"
  : >"$root/stderr.txt"
  for name in stderr.txt stdout.txt; do
    sha="$(sha256sum "$root/$name")"; sha="${sha%% *}"
    bytes="$(stat -c '%s' "$root/$name")"
    artifacts="$(jq -c --arg path "$name" --arg sha256 "$sha" --argjson bytes "$bytes" \
      '. + [{path:$path,sha256:$sha256,bytes:$bytes}]' <<<"$artifacts")"
  done
  jq -n --argjson artifacts "$artifacts" '{
    schema_version:"bullet.readme-live-capture.v2", classification:"UNSIGNED_OPERATOR_SUPPLIED_CAPTURE",
    release_authority:false, demo_id:"codex-session", observed_at:"UNKNOWN",
    command:{argv:["codex","exec","A literal prompt"],status:"COMPLETED",exit_code:0,
      requested_model:"operator-requested-model",requested_effort:"UNKNOWN"},
    artifact_hashes:$artifacts
  }' >"$root/capture.json"
}

rehash_stdout() {
  local root="$1" digest bytes
  digest="$(sha256sum "$root/stdout.txt")"; digest="${digest%% *}"
  bytes="$(stat -c '%s' "$root/stdout.txt")"
  jq --arg digest "$digest" --argjson bytes "$bytes" \
    '(.artifact_hashes[] | select(.path == "stdout.txt")) |= (.sha256=$digest | .bytes=$bytes)' \
    "$root/capture.json" >"$test_root/updated.json"
  mv "$test_root/updated.json" "$root/capture.json"
}

normalize() { bash "$fixture/scripts/readme-live-record.sh" --from-capture "$1" --staged-root "$2"; }
check() { bash "$fixture/scripts/readme-live-check.sh" --normalized-stage "$1"; }

capture="$test_root/capture"
stage="$test_root/stage"
write_capture "$capture"
chmod 755 "$capture"
normalize "$capture" "$stage"
[[ "$(stat -c '%a' "$stage")" == 700 ]]
check "$stage"
tail -n +5 "$stage/transcript.txt" >"$test_root/reply"
cmp "$capture/stdout.txt" "$test_root/reply"
jq -e 'all(.observed[]; . == "UNKNOWN") and .capture_verified == false and
  .media_verified == false and .operator_reported.command.requested_model == "operator-requested-model" and
  .status == "NORMALIZED_CAPTURE_ONLY"' "$stage/observation.json" >/dev/null
pass actual_multiline_reply_and_unknown_facts

normalize "$capture" "$test_root/repeat"
diff -qr "$stage" "$test_root/repeat"
pass deterministic_complete_normalization

write_capture "$test_root/literal"
# shellcheck disable=SC2016 # Hostile-looking reply syntax must remain literal data.
printf '%s\n' 'Different reply: `touch forbidden` $(touch forbidden) %{pts} "quoted" café' \
  '' 'Final line without a terminator' >"$test_root/literal/stdout.txt"
truncate -s -1 "$test_root/literal/stdout.txt"
rehash_stdout "$test_root/literal"
normalize "$test_root/literal" "$test_root/literal-stage"
tail -n +5 "$test_root/literal-stage/transcript.txt" >"$test_root/reply"
cmp "$test_root/literal/stdout.txt" "$test_root/reply"
if cmp -s "$stage/transcript.txt" "$test_root/literal-stage/transcript.txt"; then
  echo 'different replies produced the same transcript' >&2
  exit 1
fi
pass distinct_metacharacter_reply_is_literal

for tool in record render check; do
  expect_exit 78 UNQUALIFIED bash "$fixture/scripts/readme-live-$tool.sh"
done
expect_exit 78 MEDIA_GENERATION_UNQUALIFIED bash "$fixture/scripts/readme-live-render.sh" --from-normalized "$stage"
[[ ! -e "$README_TEST_MARKER" ]]
pass normal_media_paths_refuse_without_external_execution

for mutation in 'del(.artifact_hashes)' '.artifact_hashes=[]' '.artifact_hashes={}' \
  '.artifact_hashes += [.artifact_hashes[0]]' '.artifact_hashes[0].path="../stdout.txt"' \
  '.artifact_hashes[0].path="/stdout.txt"' '.artifact_hashes[0].path="frames\\stdout.txt"' \
  '.artifact_hashes[0].sha256="bad"' '.generation.inputs=[]' \
  '.generation.inputs += [.generation.inputs[0]]' '.generation.inputs[0].sha256=("0"*64)' \
  '.generation.inputs[0].path="scripts/unknown.sh"' '.extra=true'; do
  cp -a "$stage" "$test_root/hostile"
  jq "$mutation" "$stage/manifest.json" >"$test_root/hostile/manifest.json"
  expect_exit 1 '' check "$test_root/hostile"
  rm -rf "$test_root/hostile"
done
pass closed_artifact_and_current_input_sets

for mutation in '.command.status="FAILED"' '.command.exit_code=1' '.command.status="INTERRUPTED"' \
  '.command.actual_model="invented"' '.release_authority=true' '.demo_id="portal-ui"' \
  '.observed_at="2026-02-30T00:00:00Z"'; do
  cp -a "$capture" "$test_root/hostile"
  jq "$mutation" "$capture/capture.json" >"$test_root/hostile/capture.json"
  expect_exit 1 UNSUPPORTED_OR_INCOMPLETE_CAPTURE normalize "$test_root/hostile" "$test_root/not-published"
  [[ ! -e "$test_root/not-published" ]]
  rm -rf "$test_root/hostile"
done
pass failed_interrupted_and_unmeasured_claims_refused

for malformed in duplicate trailing nonfinite; do
  cp -a "$capture" "$test_root/hostile"
  case "$malformed" in
    duplicate) sed 's/"release_authority": false/"release_authority": true, "release_authority": false/' \
      "$capture/capture.json" >"$test_root/hostile/capture.json" ;;
    trailing) printf '{}\n' >>"$test_root/hostile/capture.json" ;;
    nonfinite) sed 's/"exit_code": 0/"exit_code": NaN/' "$capture/capture.json" >"$test_root/hostile/capture.json" ;;
  esac
  expect_exit 1 INVALID_JSON normalize "$test_root/hostile" "$test_root/not-published"
  rm -rf "$test_root/hostile"
done
pass strict_json_refuses_duplicate_trailing_and_nonfinite

for invalid in empty control del utf8 oversized secret; do
  cp -a "$capture" "$test_root/hostile"
  case "$invalid" in
    empty) : >"$test_root/hostile/stdout.txt"; reason=EMPTY_REPLY ;;
    control) printf 'reply\000hidden\n' >"$test_root/hostile/stdout.txt"; reason=CONTROL_CHARACTER ;;
    del) printf 'reply\177hidden\n' >"$test_root/hostile/stdout.txt"; reason=CONTROL_CHARACTER ;;
    utf8) printf '\377\n' >"$test_root/hostile/stdout.txt"; reason=INVALID_UTF8 ;;
    oversized) head -c 65537 /dev/zero >"$test_root/hostile/stdout.txt"; reason=UNSAFE_ARTIFACT ;;
    secret) printf '%s\n' 'Authorization: Bearer fake-sensitive-value' >"$test_root/hostile/stdout.txt"; reason=REDACTION_REQUIRED ;;
  esac
  rehash_stdout "$test_root/hostile"
  expect_exit 1 "$reason" normalize "$test_root/hostile" "$test_root/not-published"
  rm -rf "$test_root/hostile"
done
pass bounded_text_and_redaction_refusals

cp -a "$capture" "$test_root/escaped"
sed 's/operator-requested-model/\\u0041uthorization: Bearer fake-sensitive-value/' \
  "$capture/capture.json" >"$test_root/escaped/capture.json"
expect_exit 1 REDACTION_REQUIRED normalize "$test_root/escaped" "$test_root/not-published"
pass decoded_metadata_is_scanned

for changed in stdout.txt transcript.txt observation.json; do
  cp -a "$stage" "$test_root/hostile"
  printf 'changed\n' >>"$test_root/hostile/$changed"
  expect_exit 1 '' check "$test_root/hostile"
  rm -rf "$test_root/hostile"
done
pass changed_capture_observation_and_transcript_refused

printf 'outside sentinel\n' >"$test_root/outside"
outside_sha="$(sha256sum "$test_root/outside")"
for invalid in symlink fifo extra hardlink; do
  cp -a "$stage" "$test_root/hostile"
  case "$invalid" in
    symlink) rm "$test_root/hostile/stdout.txt"; ln -s "$test_root/outside" "$test_root/hostile/stdout.txt" ;;
    fifo) rm "$test_root/hostile/stdout.txt"; mkfifo "$test_root/hostile/stdout.txt" ;;
    extra) mkdir "$test_root/hostile/extra" ;;
    hardlink) rm "$test_root/hostile/stdout.txt"; ln "$test_root/outside" "$test_root/hostile/stdout.txt" ;;
  esac
  expect_exit 1 '' check "$test_root/hostile"
  rm -rf "$test_root/hostile"
done
ln -s "$test_root" "$test_root/ancestor"
expect_exit 1 UNSAFE_ROOT check "$test_root/ancestor/stage"
[[ "$(sha256sum "$test_root/outside")" == "$outside_sha" ]]
pass unsafe_paths_and_outside_targets_preserved

expect_exit 1 UNSAFE_OR_EXISTING_DESTINATION normalize "$capture" "$stage"
check "$stage"
pass existing_generation_collision_preserved

mkdir "$test_root/failing-bin"
printf '%s\n' '#!/usr/bin/env bash' 'exit 92' >"$test_root/failing-bin/jq"
chmod 700 "$test_root/failing-bin/jq"
expect_exit 92 '' env PATH="$test_root/failing-bin:$PATH" bash "$fixture/scripts/readme-live-record.sh" \
  --from-capture "$capture" --staged-root "$test_root/not-published"
rm "$test_root/failing-bin/jq"
printf '%s\n' '#!/usr/bin/env bash' 'exit 2' >"$test_root/failing-bin/grep"
chmod 700 "$test_root/failing-bin/grep"
expect_exit 1 SCANNER_FAILED env PATH="$test_root/failing-bin:$PATH" bash "$fixture/scripts/readme-live-record.sh" \
  --from-capture "$capture" --staged-root "$test_root/not-published"
[[ ! -e "$test_root/not-published" ]]
pass parser_and_scanner_errors_cannot_be_success

mkdir "$test_root/gate-bin"
# shellcheck disable=SC2016 # The fixture wrapper expands only these test variables.
printf '%s\n' '#!/usr/bin/env bash' 'set -euo pipefail' \
  '"$README_TEST_REAL_CP" "$@"' \
  'if [[ ! -e "$README_TEST_ONCE" ]]; then' \
  '  : >"$README_TEST_ONCE"' \
  '  printf "source changed\n" >>"$README_TEST_CHANGE"' 'fi' >"$test_root/gate-bin/cp"
chmod 700 "$test_root/gate-bin/cp"
for changed in "$fixture/scripts/readme-live-render.sh" "$stage/stdout.txt"; do
  cp "$changed" "$test_root/original"
  expect_exit 1 INPUT_CHANGED env README_TEST_REAL_CP="$real_cp" README_TEST_ONCE="$test_root/once" \
    README_TEST_CHANGE="$changed" PATH="$test_root/gate-bin:$PATH" \
    bash "$fixture/scripts/readme-live-check.sh" --normalized-stage "$stage"
  cp "$test_root/original" "$changed"
  rm "$test_root/once"
done
pass source_and_input_changes_after_snapshot_refused

rm "$test_root/gate-bin/cp"
# shellcheck disable=SC2016 # A colliding destination is created by a test-only tool gate.
printf '%s\n' '#!/usr/bin/env bash' 'set -euo pipefail' \
  'mkdir "$README_TEST_DESTINATION"' \
  'printf "collision sentinel\n" >"$README_TEST_DESTINATION/sentinel"' \
  'exec "$README_TEST_REAL_MV" "$@"' >"$test_root/gate-bin/mv"
chmod 700 "$test_root/gate-bin/mv"
expect_exit 1 PUBLICATION_INCOMPLETE env README_TEST_REAL_MV="$real_mv" \
  README_TEST_DESTINATION="$test_root/collision" PATH="$test_root/gate-bin:$PATH" \
  bash "$fixture/scripts/readme-live-record.sh" --from-capture "$capture" --staged-root "$test_root/collision"
grep -Fxq 'collision sentinel' "$test_root/collision/sentinel"
expect_exit 1 INVENTORY_DRIFT check "$test_root/collision"
pass late_collision_cannot_replace_or_validate_destination

[[ ! -e "$README_TEST_MARKER" && ! -e forbidden ]]
printf 'readme-live-custody-test: %s grouped cases passed (data normalization only)\n' "$passed"
