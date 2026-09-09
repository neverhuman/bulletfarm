#!/usr/bin/env bash
set -euo pipefail
# These are tool-contract fixtures, not real image or provider evidence.
SOURCE_HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
test_root="$(mktemp -d)"
trap 'rm -rf -- "$test_root"' EXIT
mkdir "$test_root/tmp"
export TMPDIR="$test_root/tmp"
fixture="$test_root/hub"
mkdir -p "$fixture/scripts" "$fixture/.config" "$fixture/bin" "$fixture/docs"
for file in readme-live-record.sh readme-live-render.sh readme-live-check.sh readme-schema-check.sh; do
  cp "$SOURCE_HUB/scripts/$file" "$fixture/scripts/"
done
receipt="$test_root/private-receipt.json"
pin="$fixture/.config/readme-live-tools.sha256"
export README_FAKE_MARKER="$test_root/docker-called"
export README_FORBIDDEN_MARKER="$test_root/provider-called"
for tool in claude codex cursor-agent curl npm node; do
  # shellcheck disable=SC2016 # Intentional deferred fixture variable.
  printf '%s\n' '#!/usr/bin/env bash' ': >"$README_FORBIDDEN_MARKER"' 'exit 93' >"$fixture/bin/$tool"
  chmod 700 "$fixture/bin/$tool"
done
cat >"$fixture/bin/docker" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
printf 'fixture call\n' >>"$README_FAKE_MARKER"
[[ -z "${DOCKER_HOST:-}${DOCKER_CONTEXT:-}${DOCKER_CONFIG:-}${DOCKER_TLS_VERIFY:-}" ]] || exit 94
while [[ "${1:-}" == --host || "${1:-}" == --config ]]; do shift 2; done
if [[ "$1" == rm ]]; then exit 0; fi
if [[ "$1" == image ]]; then
  if [[ -n "${README_FAKE_MUTATE:-}" && ! -e "$README_FAKE_ONCE" ]]; then
    : >"$README_FAKE_ONCE"
    printf '\n' >>"$README_FAKE_MUTATE"
  fi
  if [[ -n "${README_FAKE_COLLISION:-}" && ! -e "$README_FAKE_COLLISION" ]]; then
    mkdir "$README_FAKE_COLLISION"
    printf 'collision sentinel\n' >"$README_FAKE_COLLISION/sentinel"
  fi
  printf '[{"Id":"sha256:%064d","Os":"linux","Architecture":"amd64","RepoDigests":["%s"]}]\n' 1 "${@: -1}"
  exit 0
fi
out=''; input=''; entry=''; previous=''
for arg in "$@"; do
  [[ "$arg" != *:/out:rw ]] || out="${arg%:/out:rw}"
  [[ "$arg" != *:/in:ro ]] || input="${arg%:/in:ro}"
  [[ "$previous" != --entrypoint ]] || entry="$arg"
  previous="$arg"
done
if [[ -z "$entry" ]]; then echo "${README_FAKE_VHS_VERSION:-vhs version v0.11.0 (c6af91a)}"; exit 0; fi
if [[ " $* " == *' -version '* ]]; then echo "${README_FAKE_FFMPEG_VERSION:-ffmpeg version 7.1.3-0+deb13u1 fixture-only}"; exit 0; fi
if [[ "$entry" == /usr/bin/ffprobe ]]; then
  if [[ "${@: -1}" == *.png ]]; then
    printf '{"streams":[{"codec_name":"png","width":%s,"height":675}]}\n' "${README_FAKE_PNG_WIDTH:-1200}"
  else
    printf '{"streams":[{"width":1200,"height":675,"r_frame_rate":"12/1","nb_read_frames":"%s"}],"format":{"duration":"8.0"}}\n' "${README_FAKE_FRAMES:-96}"
  fi
  exit 0
fi
[[ "${README_FAKE_RENDER_FAIL:-0}" == 0 ]] || exit 95
if [[ " $* " == *' framemd5 '* ]]; then
  for gif in "$out"/*.gif; do hash="$(sha256sum "$gif")"; done
  printf '0, 0, 0, 1, 1, %s\n' "${hash%% *}"
elif [[ "${@: -1}" == /out/fallback.png ]]; then
  for gif in "$out"/*.gif; do hash="$(sha256sum "$gif")"; done
  printf 'FIXTURE-PNG %s\n' "${hash%% *}" >"$out/fallback.png"
else
  [[ " $* " == *'expansion=none'* ]] || exit 96
  [[ -d "$out" && -d "$input" ]]
  target="${@: -1}"; target="$out/${target##*/}"
  hashes=''
  for file in "$input"/*; do
    [[ "${file##*/}" != frames.txt ]] || continue
    hash="$(sha256sum "$file")"; hashes+="${file##*/}:${hash%% *};"
  done
  printf 'FIXTURE-GIF %s\n' "$hashes" >"$target"
  if [[ "${README_FAKE_NONDETERMINISTIC:-0}" == 1 ]]; then
    calls="$(cat "$README_FAKE_MARKER")"
    printf 'nondeterministic %s\n' "${#calls}" >>"$target"
  fi
fi
SH
chmod 700 "$fixture/bin/docker"
export PATH="$fixture/bin:$PATH"
passed=0
pass() { passed=$((passed + 1)); printf 'ok %s\n' "$1"; }
expect_failure() {
  local reason="$1" code=0
  shift
  "$@" >"$test_root/last.log" 2>&1 || code=$?
  if [[ "$code" == 0 ]] || ! grep -Fq "$reason" "$test_root/last.log"; then
    cat "$test_root/last.log" >&2; printf 'missing failure: %s (exit %s)\n' "$reason" "$code" >&2; exit 1
  fi
}
hash_records() {
  local root="$1" name hash size records='[]'
  shift
  for name in "$@"; do
    hash="$(sha256sum "$root/$name")"; hash="${hash%% *}"; size="$(stat -c '%s' "$root/$name")"
    records="$(jq -c --arg path "$name" --arg sha256 "$hash" --argjson bytes "$size" \
      '. + [{path:$path,sha256:$sha256,bytes:$bytes}]' <<<"$records")"
  done
  printf '%s\n' "$records"
}
tools='[]'
for name in awk bash cat cmp cp diff dirname docker env find grep iconv id jq mkdir mktemp mv python3 realpath rm sha256sum sort stat timeout tr uname; do
  path="$(realpath -e "$(command -v "$name")")"; digest="$(sha256sum "$path")"; digest="${digest%% *}"
  tools="$(jq -c --arg name "$name" --arg path "$path" --arg sha256 "$digest" --argjson bytes "$(stat -c '%s' "$path")" \
    '. + [{name:$name,path:$path,sha256:$sha256,bytes:$bytes}]' <<<"$tools")"
done
runtime="$(realpath -e /usr/lib/x86_64-linux-gnu/libc.so.6)"
digest="$(sha256sum "$runtime")"; digest="${digest%% *}"
jq -n --argjson tools "$tools" --arg path "$runtime" --arg sha256 "$digest" \
  --argjson bytes "$(stat -c '%s' "$runtime")" '{
  schema_version:"bullet.readme-live-tools.v2",platform:"linux-amd64",
  image:"ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93",
  image_id:("sha256:"+("0"*63)+"1"),vhs_version:"vhs version v0.11.0 (c6af91a)",
  ffmpeg_version:"ffmpeg version 7.1.3-0+deb13u1 fixture-only",tools:$tools,
  runtime_files:[{path:$path,sha256:$sha256,bytes:$bytes}]
}' >"$receipt"
cp "$receipt" "$test_root/original-tools.json"
pin_receipt() { local hash; hash="$(sha256sum "$receipt")"; printf '%s\n' "${hash%% *}" >"$pin"; }
pin_receipt
chmod 600 "$receipt"
cp "$pin" "$test_root/original-pin"

captures="$test_root/captures"
mkdir "$captures"
for demo in claude-session codex-session cursor-session; do
  mkdir "$captures/$demo"
  case "$demo" in claude-session) cli=claude ;; codex-session) cli=codex ;; cursor-session) cli=cursor-agent ;; esac
  printf '%s\n' 'The four Bullet Farm member repositories are bullet-farm, bullet-kernel, bullet-git, and bullet-portal.' >"$captures/$demo/stdout.txt"
  [[ "$(stat -c '%s' "$captures/$demo/stdout.txt")" == 104 ]]
  : >"$captures/$demo/stderr.txt"
  records="$(hash_records "$captures/$demo" stdout.txt stderr.txt)"
  jq -n --arg demo "$demo" --arg cli "$cli" --argjson artifacts "$records" '{
    schema_version:"bullet.readme-live-capture.v2",classification:"UNSIGNED_OPERATOR_SUPPLIED_CAPTURE",
    release_authority:false,demo_id:$demo,observed_at:"UNKNOWN",
    command:{argv:[$cli,"operator-supplied capture"],status:"COMPLETED",exit_code:0,
      requested_model:"UNKNOWN",requested_effort:"UNKNOWN"},artifact_hashes:$artifacts
  }' >"$captures/$demo/capture.json"
done
# The retained stdout-only form makes no process/argv/stderr claim; completed fixture stays paired.
for demo in claude-session codex-session; do
  rm "$captures/$demo/stderr.txt"
  records="$(hash_records "$captures/$demo" stdout.txt)"
  jq --argjson records "$records" '.command={status:"UNKNOWN",exit_code:null,argv:null,
    requested_model:"UNKNOWN",requested_effort:"UNKNOWN"} | .artifact_hashes=$records' \
    "$captures/$demo/capture.json" >"$test_root/unknown-capture.json"
  mv "$test_root/unknown-capture.json" "$captures/$demo/capture.json"
done
frames=(01-control-tower.png 02-shift-brief.png 03-fleet.png 04-mission-graph.png 05-control-tower-return.png)
mkdir "$captures/portal-ui"
for frame in "${frames[@]}"; do printf 'UNSIGNED FIXTURE FRAME %s\n' "$frame" >"$captures/portal-ui/$frame"; done
records="$(hash_records "$captures/portal-ui" "${frames[@]}")"
jq -n --argjson artifacts "$records" '{schema_version:"bullet.readme-portal-capture.v2",
  classification:"UNSIGNED_OPERATOR_SUPPLIED_CAPTURE",release_authority:false,demo_id:"portal-ui",
  observed_at:"UNKNOWN",artifact_hashes:$artifacts}' >"$captures/portal-ui/capture.json"
demos='{}'
for demo in claude-session codex-session cursor-session portal-ui; do
  digest="$(sha256sum "$captures/$demo/capture.json")"; digest="${digest%% *}"
  demos="$(jq -c --arg demo "$demo" --arg sha "$digest" '. + {($demo):$sha}' <<<"$demos")"
done
jq -n --argjson demos "$demos" '{schema_version:"bullet.readme-live-inputs.v2",
  classification:"UNSIGNED_OPERATOR_SUPPLIED_CAPTURE",release_authority:false,demos:$demos}' \
  >"$captures/capture-collection.json"
generate() { bash "$fixture/scripts/readme-live-render.sh" --from-captures "$1" --staged-root "$2" --tools-receipt "$receipt"; }
verify() { bash "$fixture/scripts/readme-live-check.sh" --rendered-collection "$1" --tools-receipt "$receipt"; }
output="$test_root/output"
[[ ! -e "$fixture/.config/readme-live-tools.json" ]]
expect_failure MEDIA_GENERATION_UNQUALIFIED bash "$fixture/scripts/readme-live-check.sh"
cp "$receipt" "$fixture/.config/local-receipt.json"
ln -s "$receipt" "$test_root/receipt-link"
ln -s "$test_root" "$test_root/parent-link"
for selected in relative.json "$test_root/missing.json" "$fixture/.config/local-receipt.json" \
  "$test_root/receipt-link" "$test_root/parent-link/private-receipt.json"; do
  expect_failure TOOL_RECEIPT_UNAVAILABLE bash "$fixture/scripts/readme-live-render.sh" \
    --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$selected"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
done
rm "$fixture/.config/local-receipt.json" "$test_root/receipt-link" "$test_root/parent-link"
cp "$receipt" "$test_root/changed-receipt.json"
printf '\n' >>"$test_root/changed-receipt.json"
expect_failure TOOL_RECEIPT_HASH_MISMATCH bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$test_root/changed-receipt.json"
for bad in missing invalid nonhex digest symlink hardlink oversized; do
  case "$bad" in
    missing) rm "$pin" ;;
    invalid) printf '%064d' 0 >"$pin" ;;
    nonhex) printf '%s\n' "$(printf 'g%.0s' {1..64})" >"$pin" ;;
    digest) printf '%064d\n' 0 >"$pin" ;;
    symlink) rm "$pin"; ln -s "$test_root/original-pin" "$pin" ;;
    hardlink) ln "$pin" "$test_root/pin-peer" ;;
    oversized) truncate -s 66 "$pin" ;;
  esac
  reason=INVALID_TOOL_PIN; [[ "$bad" != digest ]] || reason=TOOL_RECEIPT_HASH_MISMATCH
  expect_failure "$reason" generate "$captures" "$test_root/not-published"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
  [[ "$bad" != hardlink ]] || rm "$test_root/pin-peer"
  rm -f "$pin"; cp "$test_root/original-pin" "$pin"
done
pass explicit_external_receipt_and_source_pin_admission
DOCKER_HOST=forbidden DOCKER_CONTEXT=forbidden DOCKER_CONFIG=forbidden generate "$captures" "$output"
[[ -s "$README_FAKE_MARKER" && ! -e "$README_FORBIDDEN_MARKER" ]]
for demo in claude-session codex-session cursor-session; do
  jq -jr '.[] | .text,(if .newline then "\n" else "" end)' "$output/$demo/layout.json" >"$test_root/unwrapped"
  cmp "$output/$demo/transcript.txt" "$test_root/unwrapped"
  jq -e 'all(.[]; .length <= 80) and length == 6' "$output/$demo/layout.json" >/dev/null
done
jq -e 'all(.observed[]; . == "UNKNOWN") and .capture_verified == false' "$output/portal-ui/observation.json" >/dev/null
pass complete_four_demo_generation_and_lossless104_byte_reply

for demo in claude-session codex-session; do
  [[ ! -e "$output/$demo/stderr.txt" ]]
  jq -e '.operator_reported.command == {status:"UNKNOWN",exit_code:null,argv:null,
    requested_model:"UNKNOWN",requested_effort:"UNKNOWN"}' "$output/$demo/observation.json" >/dev/null
done
jq -e '.operator_reported.command.status == "COMPLETED" and .operator_reported.command.exit_code == 0' \
  "$output/cursor-session/observation.json" >/dev/null
for filter in '.command.argv=["claude"]' '.command.exit_code=0' '.command.status="COMPLETED"' \
  '.command.status="FAILED"' '.command.status="INTERRUPTED"' '.command.requested_model="inferred"' \
  '.command.requested_effort="inferred"'; do
  cp -a "$captures/claude-session" "$test_root/unknown-hostile"
  jq "$filter" "$captures/claude-session/capture.json" >"$test_root/unknown-hostile/capture.json"
  expect_failure '' bash "$fixture/scripts/readme-live-record.sh" --from-capture "$test_root/unknown-hostile" \
    --staged-root "$test_root/not-published"
  [[ ! -e "$test_root/not-published" ]]
  rm -rf "$test_root/unknown-hostile"
done
cp -a "$captures/claude-session" "$test_root/unknown-hostile"
: >"$test_root/unknown-hostile/stderr.txt"
expect_failure INVENTORY_DRIFT bash "$fixture/scripts/readme-live-record.sh" --from-capture "$test_root/unknown-hostile" \
  --staged-root "$test_root/not-published"
rm -rf "$test_root/unknown-hostile"
pass exact_unknown_stdout_only_and_reported_completed_forms


verify "$output"
cp -a "$output" "$fixture/docs/readme-live-media"
bash "$fixture/scripts/readme-live-check.sh" --tools-receipt "$receipt"
expect_failure TOOL_RECEIPT_REQUIRED bash "$fixture/scripts/readme-live-check.sh"
pass complete_verifier_and_normal_docs_endpoint

for filter in '.artifact_hashes=[]' '.artifact_hashes += [.artifact_hashes[0]]' '.demos |= .[0:3]' \
  '.generation.sources[0].sha256=("0"*64)' '.generation.renderer.tool_receipt_sha256=("0"*64)'; do
  cp -a "$output" "$test_root/hostile"
  jq "$filter" "$output/collection.json" >"$test_root/hostile/collection.json"
  rm -f "$README_FAKE_MARKER"
  expect_failure COLLECTION_MANIFEST_DRIFT verify "$test_root/hostile"
  [[ ! -e "$README_FAKE_MARKER" ]]
  rm -rf "$test_root/hostile"
done
pass closed_collection_and_stale_source_receipt_refusal

for file in fallback.png frames.framemd5 layout.json render.json; do
  cp -a "$output" "$test_root/hostile"
  printf 'changed\n' >>"$test_root/hostile/codex-session/$file"
  rm -f "$README_FAKE_MARKER"
  expect_failure '' verify "$test_root/hostile"
  [[ ! -e "$README_FAKE_MARKER" ]]
  rm -rf "$test_root/hostile"
done
pass unchanged_gif_cannot_hide_other_output_drift

for filter in '.tools |= .[1:]' '.tools |= map(select(.name != "dirname"))' '.tools += [.tools[0]]' '.tools[0].sha256=("0"*64)' \
  '.runtime_files[0].sha256=("0"*64)' '.image="unexpected"' '.platform="unknown"'; do
  jq "$filter" "$test_root/original-tools.json" >"$receipt"
  pin_receipt # Fixture source selects malformed receipt to exercise downstream checks.
  rm -f "$README_FAKE_MARKER"
  expect_failure '' generate "$captures" "$test_root/not-published"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
done
cp "$test_root/original-tools.json" "$receipt"; pin_receipt
pass tool_selection_hash_platform_and_image_receipt_refusal

for bad in config-symlink receipt-hardlink oversized; do
  case "$bad" in
    config-symlink) mv "$fixture/.config" "$test_root/elsewhere-config"; ln -s "$test_root/elsewhere-config" "$fixture/.config" ;;
    receipt-hardlink) ln "$receipt" "$test_root/receipt-peer" ;;
    oversized) truncate -s 4194305 "$receipt" ;;
  esac
  rm -f "$README_FAKE_MARKER"
  reason=TOOL_RECEIPT_UNAVAILABLE; [[ "$bad" != config-symlink ]] || reason=INVALID_TOOL_PIN
  expect_failure "$reason" generate "$captures" "$test_root/not-published"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
  case "$bad" in
    config-symlink) rm "$fixture/.config"; mv "$test_root/elsewhere-config" "$fixture/.config" ;;
    receipt-hardlink) rm "$test_root/receipt-peer" ;;
    oversized) cp "$test_root/original-tools.json" "$receipt"; pin_receipt ;;
  esac
done
pass canonical_tool_receipt_ancestry_links_and_preparse_bound

jq '.image_id=("sha256:"+("2"*64))' "$test_root/original-tools.json" >"$receipt"
pin_receipt
expect_failure IMAGE_DRIFT generate "$captures" "$test_root/not-published"
cp "$test_root/original-tools.json" "$receipt"; pin_receipt
expect_failure IMAGE_VERSION_DRIFT env README_FAKE_VHS_VERSION=unqualified bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$receipt"
expect_failure IMAGE_VERSION_DRIFT env README_FAKE_FFMPEG_VERSION=unqualified bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$receipt"
[[ ! -e "$test_root/not-published" ]]
pass actual_image_and_version_readback_refusals


for bad in missing extra symlink; do
  cp -a "$captures" "$test_root/hostile"
  case "$bad" in
    missing) rm "$test_root/hostile/portal-ui/${frames[0]}" ;;
    extra) printf 'extra\n' >"$test_root/hostile/portal-ui/extra.png" ;;
    symlink) rm "$test_root/hostile/portal-ui/${frames[0]}"; ln -s "$captures/portal-ui/${frames[0]}" "$test_root/hostile/portal-ui/${frames[0]}" ;;
  esac
  rm -f "$README_FAKE_MARKER"
  expect_failure '' generate "$test_root/hostile" "$test_root/not-published"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
  rm -rf "$test_root/hostile"
done
pass exact_portal_frame_inventory_before_tool_execution

expect_failure INVALID_PORTAL_FRAME env README_FAKE_PNG_WIDTH=999 bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/bad-frame" --tools-receipt "$receipt"
expect_failure INVALID_RENDER_PROBE env README_FAKE_FRAMES=0 bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/bad-probe" --tools-receipt "$receipt"
expect_failure RENDER_TOOL_FAILED env README_FAKE_RENDER_FAIL=1 bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/bad-render" --tools-receipt "$receipt"
[[ ! -e "$test_root/bad-frame" && ! -e "$test_root/bad-probe" && ! -e "$test_root/bad-render" ]]
pass decoder_and_renderer_failures_cannot_publish

rm -f "$README_FAKE_MARKER"
expect_failure UNSAFE_OR_EXISTING_DESTINATION generate "$captures" "$output"
[[ ! -e "$README_FAKE_MARKER" ]]
[[ ! -e "$README_FORBIDDEN_MARKER" ]]
expect_failure UNSAFE_OR_EXISTING_DESTINATION generate "$captures" "$captures/nested-output"
[[ ! -e "$README_FAKE_MARKER" && ! -e "$captures/nested-output" ]]
find "$captures" -maxdepth 1 -name '.readme-live-render.*' >"$test_root/nested-stages"
[[ ! -s "$test_root/nested-stages" ]]
pass preexisting_output_collision_and_input_custody_preserved

cp -a "$output" "$test_root/hostile"
printf 'changed decoded fallback\n' >>"$test_root/hostile/codex-session/fallback.png"
# Recalculate every enclosing hash as an attacker could; reconstruction must still refuse.
bash -c '
  source "$1/scripts/readme-live-render.sh"
  live_start
  trap '\''rm -rf -- "$LIVE_TMP"'\'' EXIT
  LIVE_LOCK="$3"
  sources="$(live_sources)"; renderer="$(render_tool_receipt)"
  render_manifest "$2/codex-session" codex-session "$2/codex-session/render.json"
  collection_files=()
  while IFS= read -r file; do collection_files+=("$file"); done < <(jq -r ".artifact_hashes[].path" "$2/collection.json")
  collection_manifest "$2" "$2/collection.json"
' fixture "$fixture" "$test_root/hostile" "$receipt"
rm -f "$README_FAKE_MARKER"
expect_failure RECONSTRUCTION_DRIFT verify "$test_root/hostile"
[[ -s "$README_FAKE_MARKER" ]]
rm -rf "$test_root/hostile"
pass fully_rehashed_output_requires_independent_reconstruction

for bad in unicode too-many-lines; do
  cp -a "$captures" "$test_root/hostile"
  if [[ "$bad" == unicode ]]; then printf '\303\251\n' >"$test_root/hostile/codex-session/stdout.txt"
  else printf 'line\n%.0s' {1..9} >"$test_root/hostile/codex-session/stdout.txt"; fi
  records="$(hash_records "$test_root/hostile/codex-session" stdout.txt)"
  jq --argjson records "$records" '.artifact_hashes=$records' "$captures/codex-session/capture.json" >"$test_root/hostile/codex-session/capture.json"
  digest="$(sha256sum "$test_root/hostile/codex-session/capture.json")"; digest="${digest%% *}"
  jq --arg sha "$digest" '.demos["codex-session"]=$sha' "$captures/capture-collection.json" >"$test_root/hostile/capture-collection.json"
  rm -f "$README_FAKE_MARKER"
  reason=UNSUPPORTED_RENDER_GLYPH; [[ "$bad" != too-many-lines ]] || reason=RENDER_LAYOUT_TOO_LONG
  expect_failure "$reason" generate "$test_root/hostile" "$test_root/not-published"
  [[ ! -e "$README_FAKE_MARKER" && ! -e "$test_root/not-published" ]]
  rm -rf "$test_root/hostile"
done
pass unsupported_glyph_and_overflow_refuse_without_lossy_output

for changed in "$captures/codex-session/stdout.txt" "$fixture/scripts/readme-live-render.sh" "$receipt" "$pin"; do
  cp "$changed" "$test_root/original-file"
  rm -f "$test_root/once"
  expect_failure '' env README_FAKE_MUTATE="$changed" README_FAKE_ONCE="$test_root/once" \
    bash "$fixture/scripts/readme-live-render.sh" --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$receipt"
  [[ -e "$test_root/once" && ! -e "$test_root/not-published" ]]
  cp "$test_root/original-file" "$changed"
done
pass changed_input_source_and_tool_receipt_cannot_publish

expect_failure RECONSTRUCTION_DRIFT env README_FAKE_NONDETERMINISTIC=1 bash "$fixture/scripts/readme-live-render.sh" \
  --from-captures "$captures" --staged-root "$test_root/not-published" --tools-receipt "$receipt"
[[ ! -e "$test_root/not-published" ]]
expect_failure PUBLICATION_INCOMPLETE env README_FAKE_COLLISION="$test_root/late-collision" \
  bash "$fixture/scripts/readme-live-render.sh" --from-captures "$captures" --staged-root "$test_root/late-collision" --tools-receipt "$receipt"
grep -Fxq 'collision sentinel' "$test_root/late-collision/sentinel"
expect_failure '' verify "$test_root/late-collision"
[[ ! -e "$README_FORBIDDEN_MARKER" ]]
pass nondeterminism_and_late_collision_cannot_publish_success

printf 'readme-live-render-test: %s groups passed (fake tool contracts only; no real images)\n' "$passed"
