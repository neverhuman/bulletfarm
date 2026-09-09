#!/usr/bin/env bash
set -euo pipefail
shopt -s inherit_errexit
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
# shellcheck source=scripts/readme-live-check.sh
source "$HUB/scripts/readme-live-check.sh"
LIVE_DEMOS=(claude-session codex-session cursor-session portal-ui)
LIVE_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
LIVE_TOOLS=(awk bash cat cmp cp diff dirname docker env find grep iconv id jq mkdir mktemp mv
  python3 realpath rm sha256sum sort stat timeout tr uname)
LIVE_LOCK=''
LIVE_PIN="$HUB/.config/readme-live-tools.sha256"

render_tool_receipt() {
  local expected entry name path digest bytes actual selected receipt_sha
  [[ -f "$LIVE_PIN" && ! -L "$LIVE_PIN" && "$(realpath -e -- "$LIVE_PIN")" == "$LIVE_PIN" &&
    "$(stat -c '%h' "$LIVE_PIN")" == 1 && "$(stat -c '%s' "$LIVE_PIN")" == 65 ]] || live_die INVALID_TOOL_PIN
  IFS= read -r receipt_sha <"$LIVE_PIN" || live_die INVALID_TOOL_PIN
  [[ "$receipt_sha" =~ ^[0-9a-f]{64}$ ]] || live_die INVALID_TOOL_PIN
  [[ "$LIVE_LOCK" == /* && "$LIVE_LOCK" != "$HUB/"* && -f "$LIVE_LOCK" && ! -L "$LIVE_LOCK" &&
    "$(realpath -e -- "$LIVE_LOCK")" == "$LIVE_LOCK" &&
    "$(stat -c '%h' "$LIVE_LOCK")" == 1 && "$(stat -c '%s' "$LIVE_LOCK")" -le 4194304 ]] || live_die TOOL_RECEIPT_UNAVAILABLE
  entry="$(sha256sum "$LIVE_LOCK")"; entry="${entry%% *}"
  [[ "$entry" == "$receipt_sha" ]] || live_die TOOL_RECEIPT_HASH_MISMATCH
  live_strict_json "$LIVE_LOCK"
  expected="$(printf '%s\n' "${LIVE_TOOLS[@]}" | jq -Rsc 'split("\n")[:-1] | sort')"
  jq -e --arg image "$LIVE_IMAGE" --argjson names "$expected" '
    def file: (keys | sort) == ["bytes","path","sha256"] and
      (.path | type == "string" and startswith("/") and (test("[\u0000-\u001f\\\\]") | not)) and
      (.sha256 | test("^[0-9a-f]{64}$")) and (.bytes | type == "number" and . > 0 and floor == .);
    (keys | sort) == ["ffmpeg_version","image","image_id","platform","runtime_files","schema_version","tools","vhs_version"] and
    .schema_version == "bullet.readme-live-tools.v2" and .platform == "linux-amd64" and .image == $image and
    (.image_id | test("^sha256:[0-9a-f]{64}$")) and
    .vhs_version == "vhs version v0.11.0 (c6af91a)" and
    (.ffmpeg_version | startswith("ffmpeg version 7.1.3-0+deb13u1 ")) and
    (.tools | type == "array" and length <= 64 and all(.[];
      (keys | sort) == ["bytes","name","path","sha256"] and (del(.name) | file))) and
    ([.tools[].name] | sort) == $names and
    (.runtime_files | type == "array" and length > 0 and length <= 4096 and all(.[]; file)) and
    ([.runtime_files[].path] | length) == ([.runtime_files[].path] | unique | length)
  ' "$LIVE_LOCK" >/dev/null || live_die INVALID_TOOL_RECEIPT
  [[ "$(uname -s):$(uname -m)" == Linux:x86_64 ]] || live_die UNSUPPORTED_RENDER_PLATFORM
  jq -r '.tools[] | [.name,.path,.sha256,.bytes] | @tsv' "$LIVE_LOCK" >"$LIVE_TMP/tools.tsv"
  while IFS=$'\t' read -r name path digest bytes; do
    selected="$(command -v "$name")" || live_die TOOL_UNAVAILABLE
    [[ "$(realpath -e -- "$selected")" == "$path" ]] || live_die TOOL_SELECTION_DRIFT
  done <"$LIVE_TMP/tools.tsv"
  jq -r '[.tools[] | del(.name)] + .runtime_files | .[] | [.path,.sha256,.bytes] | @tsv' \
    "$LIVE_LOCK" >"$LIVE_TMP/tool-files.tsv"
  while IFS=$'\t' read -r path digest bytes; do
    [[ -f "$path" && ! -L "$path" && "$(realpath -e -- "$path")" == "$path" &&
      "$(stat -c '%s' "$path")" == "$bytes" ]] || live_die TOOL_FILE_DRIFT
    actual="$(sha256sum -- "$path")"; actual="${actual%% *}"
    [[ "$actual" == "$digest" ]] || live_die TOOL_FILE_DRIFT
  done <"$LIVE_TMP/tool-files.tsv"
  actual="$(sha256sum "$LIVE_LOCK")"; actual="${actual%% *}"
  [[ "$actual" == "$entry" ]] || live_die TOOL_RECEIPT_HASH_MISMATCH
  jq -cS --arg receipt_sha256 "$entry" '{image,image_id,platform,vhs_version,ffmpeg_version,
    tool_receipt_sha256:$receipt_sha256,width:1200,height:675,fps:12,max_bytes:3145728,
    max_seconds:30,font:"JetBrains Mono",locale:"C.UTF-8",timezone:"UTC",epoch:1787616000}' "$LIVE_LOCK"
}

render_docker() {
  timeout --kill-after=2 10 env -u DOCKER_HOST -u DOCKER_CONTEXT -u DOCKER_CONFIG -u DOCKER_TLS -u DOCKER_TLS_VERIFY \
    -u DOCKER_CERT_PATH -u DOCKER_API_VERSION docker --host unix:///var/run/docker.sock \
    --config "$LIVE_TMP/docker-config" "$@"
}

render_tool() {
  local output="$1" entrypoint="$2" code=0
  shift 2
  container="bullet-readme-${LIVE_TMP##*/}-$((tool_sequence++))"
  local -a args=(run --rm --name "$container" --platform linux/amd64 --network none --pull never
    --read-only --cap-drop ALL --security-opt no-new-privileges --memory 1g --cpus 2 --pids-limit 128
    --tmpfs '/tmp:rw,nosuid,nodev,size=134217728' --user "$(id -u):$(id -g)"
    --env HOME=/tmp --env LANG=C.UTF-8 --env LC_ALL=C.UTF-8 --env TZ=UTC --env SOURCE_DATE_EPOCH=1787616000
    --volume "$output:/out:rw" --volume "$render_input:/in:ro" --workdir /out)
  [[ -z "$entrypoint" ]] || args+=(--entrypoint "$entrypoint")
  timeout --kill-after=5 120 env -u DOCKER_HOST -u DOCKER_CONTEXT -u DOCKER_CONFIG -u DOCKER_TLS \
    -u DOCKER_TLS_VERIFY -u DOCKER_CERT_PATH -u DOCKER_API_VERSION docker \
    --host unix:///var/run/docker.sock --config "$LIVE_TMP/docker-config" "${args[@]}" "$LIVE_IMAGE" "$@" || code=$?
  if (( code != 0 )); then
    render_docker rm -f "$container" >/dev/null 2>&1 || true
    live_die RENDER_TOOL_FAILED
  fi
  container=''
}

render_environment() {
  local version
  render_docker image inspect "$LIVE_IMAGE" >"$LIVE_TMP/image.json" || live_die IMAGE_UNAVAILABLE
  jq -e --arg image "$LIVE_IMAGE" --arg id "$(jq -r '.image_id' "$LIVE_LOCK")" '
    length == 1 and .[0].Id == $id and .[0].Os == "linux" and .[0].Architecture == "amd64" and
    (.[0].RepoDigests | index($image)) != null' "$LIVE_TMP/image.json" >/dev/null || live_die IMAGE_DRIFT
  render_tool "$LIVE_TMP" '' --version >"$LIVE_TMP/version"
  version="$(cat "$LIVE_TMP/version")"
  [[ "$version" == "$(jq -r '.vhs_version' "$LIVE_LOCK")" ]] || live_die IMAGE_VERSION_DRIFT
  render_tool "$LIVE_TMP" /usr/bin/ffmpeg -version >"$LIVE_TMP/version"
  version="$(cat "$LIVE_TMP/version")"
  [[ "${version%%$'\n'*}" == "$(jq -r '.ffmpeg_version' "$LIVE_LOCK")" ]] || live_die IMAGE_VERSION_DRIFT
}

render_layout() {
  local transcript="$1" output="$2" line code offset=0 entries='[]' text length newline
  LC_ALL=C tr -d '\12\40-\176' <"$transcript" >"$LIVE_TMP/layout-invalid"
  [[ ! -s "$LIVE_TMP/layout-invalid" ]] || live_die UNSUPPORTED_RENDER_GLYPH
  while true; do
    line=''; code=0
    IFS= read -r line || code=$?
    [[ "$code" == 0 || -n "$line" ]] || break
    while true; do
      text="${line:0:80}"
      length=${#text}; newline=false
      line="${line:length}"
      [[ -n "$line" || "$code" != 0 ]] || newline=true
      entries="$(jq -c --arg text "$text" --argjson offset "$offset" --argjson length "$length" \
        --argjson newline "$newline" '. + [{offset:$offset,length:$length,newline:$newline,text:$text}]' <<<"$entries")"
      offset=$((offset + length)); [[ "$newline" == false ]] || offset=$((offset + 1))
      [[ -n "$line" ]] || break
    done
  done <"$transcript"
  jq -e 'length > 0 and length <= 12' <<<"$entries" >/dev/null || live_die RENDER_LAYOUT_TOO_LONG
  jq -S '.' <<<"$entries" >"$output"
  jq -jr '.[] | .text, (if .newline then "\n" else "" end)' "$output" >"$LIVE_TMP/reassembled"
  cmp -s "$transcript" "$LIVE_TMP/reassembled" || live_die LOSSY_LAYOUT
}

collection_shape() {
  local root="$1" mode="$2" demo name captures='{}' sha
  local -a names
  live_root_subject "$root" >/dev/null
  collection_files=(capture-collection.json)
  [[ "$mode" != rendered ]] || collection_files+=(collection.json)
  : >"$LIVE_TMP/expected-tree"
  for demo in "${LIVE_DEMOS[@]}"; do
    live_profile "$root/$demo"
    [[ "$(jq -r '.demo_id' "$root/$demo/capture.json")" == "$demo" ]] || live_die DEMO_ID_DRIFT
    if [[ "$mode" == raw ]]; then
      live_inventory "$root/$demo" "${LIVE_CAPTURE_FILES[@]}"
      live_capture "$root/$demo"
      names=("${LIVE_CAPTURE_FILES[@]}")
    else
      live_stage "$root/$demo" "$sources" rendered
      live_strict_json "$root/$demo/layout.json" "$root/$demo/render.json"
      render_layout "$root/$demo/transcript.txt" "$LIVE_TMP/expected-layout.json"
      cmp -s "$root/$demo/layout.json" "$LIVE_TMP/expected-layout.json" || live_die LAYOUT_DRIFT
      render_manifest "$root/$demo" "$demo" "$LIVE_TMP/expected-render.json"
      cmp -s "$root/$demo/render.json" "$LIVE_TMP/expected-render.json" || live_die RENDER_MANIFEST_DRIFT
      names=("${LIVE_STAGE_FILES[@]}")
    fi
    printf '%s\td\n' "$demo" >>"$LIVE_TMP/expected-tree"
    for name in "${names[@]}"; do collection_files+=("$demo/$name"); done
    sha="$(sha256sum "$root/$demo/capture.json")"; sha="${sha%% *}"
    captures="$(jq -c --arg demo "$demo" --arg sha "$sha" '. + {($demo):$sha}' <<<"$captures")"
  done
  for name in "${collection_files[@]}"; do
    printf '%s\tf\n' "$name" >>"$LIVE_TMP/expected-tree"
    [[ -f "$root/$name" && ! -L "$root/$name" && "$(stat -c '%h' "$root/$name")" == 1 ]] || live_die UNSAFE_ARTIFACT
  done
  LC_ALL=C sort -o "$LIVE_TMP/expected-tree" "$LIVE_TMP/expected-tree"
  find -P "$root" -mindepth 1 -printf '%P\t%y\n' | LC_ALL=C sort >"$LIVE_TMP/actual-tree"
  cmp -s "$LIVE_TMP/expected-tree" "$LIVE_TMP/actual-tree" || live_die COLLECTION_INVENTORY_DRIFT
  [[ "$(stat -c '%s' "$root/capture-collection.json")" -le 65536 ]] || live_die UNSAFE_ARTIFACT
  live_strict_json "$root/capture-collection.json"
  jq -e --argjson captures "$captures" '(keys | sort) == ["classification","demos","release_authority","schema_version"] and
    .schema_version == "bullet.readme-live-inputs.v2" and .classification == "UNSIGNED_OPERATOR_SUPPLIED_CAPTURE" and
    .release_authority == false and .demos == $captures' "$root/capture-collection.json" >/dev/null || live_die CAPTURE_COLLECTION_DRIFT
  if [[ "$mode" == rendered ]]; then
    [[ "$(stat -c '%s' "$root/collection.json")" -le 65536 ]] || live_die UNSAFE_ARTIFACT
    live_strict_json "$root/collection.json"
    collection_manifest "$root" "$LIVE_TMP/expected-collection.json"
    cmp -s "$root/collection.json" "$LIVE_TMP/expected-collection.json" || live_die COLLECTION_MANIFEST_DRIFT
  fi
}

collection_manifest() {
  local root="$1" output="$2" file artifacts filtered=()
  for file in "${collection_files[@]}"; do [[ "$file" == collection.json ]] || filtered+=("$file"); done
  artifacts="$(live_files "$root" "${filtered[@]}")"
  jq -nS --argjson artifacts "$artifacts" --argjson sources "$sources" --argjson renderer "$renderer" '{
    schema_version:"bullet.readme-live-collection.v2",classification:"UNSIGNED_OPERATOR_SUPPLIED_MEDIA",
    release_authority:false,capture_verified:false,production_proof:false,status:"OFFLINE_RENDER_COMPLETE",
    demos:["claude-session","codex-session","cursor-session","portal-ui"],
    generation:{sources:$sources,renderer:$renderer},artifact_hashes:$artifacts
  }' >"$output"
}

render_probe() {
  local root="$1" demo="$2" count last
  render_tool "$root" /usr/bin/ffprobe -v error -count_frames -select_streams v:0 \
    -show_entries stream=width,height,r_frame_rate,nb_read_frames:format=duration -of json "/out/$demo.gif" >"$LIVE_TMP/probe.json"
  jq -e '.streams | length == 1 and .[0].width == 1200 and .[0].height == 675 and
    .[0].r_frame_rate == "12/1" and (.[0].nb_read_frames | tonumber | . > 0 and . <= 360 and floor == .)' \
    "$LIVE_TMP/probe.json" >/dev/null || live_die INVALID_RENDER_PROBE
  jq -e '.format.duration | tonumber | . > 0 and . <= 30' "$LIVE_TMP/probe.json" >/dev/null || live_die INVALID_RENDER_PROBE
  count="$(jq -r '.streams[0].nb_read_frames' "$LIVE_TMP/probe.json")"; last=$((count - 1))
  render_tool "$root" /usr/bin/ffmpeg -v error -i "/out/$demo.gif" -f framemd5 - >"$LIVE_TMP/frame-digests"
  awk '!/^#/' "$LIVE_TMP/frame-digests" >"$root/frames.framemd5"
  [[ -s "$root/frames.framemd5" ]] || live_die EMPTY_FRAME_DIGEST
  render_tool "$root" /usr/bin/ffmpeg -v error -y -i "/out/$demo.gif" \
    -vf "select=eq(n\\,$last)" -frames:v 1 /out/fallback.png
}

render_demo() {
  local root="$1" demo="$2" filter='[0:v]' index count y start frame
  rm -rf -- "$render_input"
  mkdir "$render_input"
  render_layout "$root/transcript.txt" "$root/layout.json"
  count="$(jq 'length' "$root/layout.json")"
  if [[ "$demo" == portal-ui ]]; then
    for frame in "${LIVE_PORTAL_FRAMES[@]}"; do
      cp -- "$root/$frame" "$render_input/$frame"
      render_tool "$root" /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=codec_name,width,height \
        -of json "/in/$frame" >"$LIVE_TMP/png.json"
      jq -e '.streams | length == 1 and .[0].codec_name == "png" and .[0].width == 1200 and .[0].height == 675' \
        "$LIVE_TMP/png.json" >/dev/null || live_die INVALID_PORTAL_FRAME
      printf "file '/in/%s'\nduration 1.5\n" "$frame" >>"$render_input/frames.txt"
    done
    printf "file '/in/%s'\n" "${LIVE_PORTAL_FRAMES[4]}" >>"$render_input/frames.txt"
    filter='[0:v]scale=1200:520:force_original_aspect_ratio=decrease:flags=lanczos,pad=1200:675:(ow-iw)/2:0:color=0x0b1020,fps=12,'
  fi
  for ((index=0; index<count; index++)); do
    jq -jr --argjson index "$index" '.[$index].text' "$root/layout.json" >"$render_input/line-$index.txt"
    y=$((32 + index * 48)); start="gte(t,$((index + 1)) * 0.5)"
    if [[ "$demo" == portal-ui ]]; then y=$((535 + index * 30)); start=1; fi
    filter+="drawtext=fontfile=/usr/share/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf:textfile=/in/line-$index.txt:expansion=none:fontcolor=0xe5e7eb:fontsize=22:x=28:y=$y:enable='$start',"
  done
  filter="${filter%,},split[a][b];[a]palettegen=stats_mode=full[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle[v]"
  local -a media_input=(-f lavfi -i 'color=c=0x0b1020:s=1200x675:r=12:d=8')
  [[ "$demo" != portal-ui ]] || media_input=(-f concat -safe 0 -i /in/frames.txt)
  render_tool "$root" /usr/bin/ffmpeg -v error -y "${media_input[@]}" -filter_complex "$filter" \
    -map '[v]' -map_metadata -1 -loop 0 "/out/$demo.gif"
  [[ -s "$root/$demo.gif" && "$(stat -c '%s' "$root/$demo.gif")" -le 3145728 ]] || live_die INVALID_RENDER_SIZE
  render_probe "$root" "$demo"
  render_manifest "$root" "$demo" "$root/render.json"
}

render_manifest() {
  local root="$1" demo="$2" output="$3" artifacts normalization
  artifacts="$(live_files "$root" "$demo.gif" fallback.png frames.framemd5 layout.json)"
  normalization="$(sha256sum "$root/manifest.json")"; normalization="${normalization%% *}"
  jq -nS --arg demo "$demo" --arg normalization "$normalization" --argjson artifacts "$artifacts" --argjson renderer "$renderer" '{
    schema_version:"bullet.readme-live-render.v2",demo_id:$demo,classification:"UNSIGNED_OPERATOR_SUPPLIED_MEDIA",
    release_authority:false,capture_verified:false,normalization_sha256:$normalization,
    renderer:$renderer,artifact_hashes:$artifacts
  }' >"$output"
}

generate_collection() {
  local input="$1" output="$2" demo
  mkdir "$output"
  cp -- "$input/capture-collection.json" "$output/capture-collection.json"
  for demo in "${LIVE_DEMOS[@]}"; do
    bash "$HUB/scripts/readme-live-record.sh" --from-capture "$input/$demo" --staged-root "$output/$demo" >/dev/null
    render_demo "$output/$demo" "$demo"
  done
  # Shape validation requires the final manifest, whose closed set is fixed here.
  collection_files=(capture-collection.json)
  for demo in "${LIVE_DEMOS[@]}"; do
    live_profile "$output/$demo"
    for file in "${LIVE_STAGE_FILES[@]}" "$demo.gif" fallback.png frames.framemd5 layout.json render.json; do
      collection_files+=("$demo/$file")
    done
  done
  collection_manifest "$output" "$output/collection.json"
  collection_shape "$output" rendered
}

if [[ "${BASH_SOURCE[0]}" != "$0" ]]; then return 0; fi
if [[ "$#" == 0 ]]; then echo 'readme-live-render: MEDIA_GENERATION_UNQUALIFIED (explicit inputs required)' >&2; exit 78; fi
if [[ "$#" == 2 && "$1" == --from-normalized ]]; then
  bash "$HUB/scripts/readme-live-check.sh" --normalized-stage "$2" >/dev/null
  echo 'readme-live-render: MEDIA_GENERATION_UNQUALIFIED (complete four-demo inputs required)' >&2; exit 78
fi
mode="$1"
[[ ( "$#" == 4 && "$mode" == --verify-collection && "$3" == --tools-receipt ) ||
  ( "$#" == 6 && "$mode" == --from-captures && "$3" == --staged-root && "$5" == --tools-receipt ) ]] || {
  echo 'usage: readme-live-render.sh (--from-captures ABSOLUTE_DIRECTORY --staged-root NEW_ABSOLUTE_DIRECTORY | --verify-collection ABSOLUTE_DIRECTORY) --tools-receipt ABSOLUTE_PRIVATE_FILE' >&2; exit 2;
}
LIVE_LOCK="${!#}"
live_start
stage=''; container=''; tool_sequence=0; render_input="$LIVE_TMP/render-input"
mkdir "$render_input" "$LIVE_TMP/docker-config"
cleanup() {
  local code=$?
  [[ -z "$container" ]] || render_docker rm -f "$container" >/dev/null 2>&1 || true
  rm -rf -- "$LIVE_TMP"
  [[ -z "$stage" ]] || printf 'readme-live-render: retained render attempts at %s\n' "$stage" >&2
  return "$code"
}
trap cleanup EXIT
sources="$(live_sources)"
renderer="$(render_tool_receipt)"
input="$2"; input_subject="$(live_root_subject "$input")"
kind=raw; [[ "$mode" != --verify-collection ]] || kind=rendered
collection_shape "$input" "$kind"
input_files=("${collection_files[@]}")
input_hashes="$(live_files "$input" "${input_files[@]}")"
mkdir "$LIVE_TMP/snapshot"
cp -a --no-dereference -- "$input/." "$LIVE_TMP/snapshot/"
collection_shape "$LIVE_TMP/snapshot" "$kind"
[[ "$(live_files "$LIVE_TMP/snapshot" "${input_files[@]}")" == "$input_hashes" ]] || live_die INPUT_CHANGED
raw="$LIVE_TMP/raw"
mkdir "$raw"
cp "$LIVE_TMP/snapshot/capture-collection.json" "$raw/"
for demo in "${LIVE_DEMOS[@]}"; do
  live_profile "$LIVE_TMP/snapshot/$demo"
  mkdir "$raw/$demo"
  for file in "${LIVE_CAPTURE_FILES[@]}"; do cp "$LIVE_TMP/snapshot/$demo/$file" "$raw/$demo/"; done
  mkdir "$LIVE_TMP/layout-$demo"
  live_expected "$raw/$demo" "$LIVE_TMP/layout-$demo"
  render_layout "$LIVE_TMP/layout-$demo/transcript.txt" "$LIVE_TMP/layout-$demo/layout.json"
done
if [[ "$mode" == --from-captures ]]; then
  destination="$4"; parent="${destination%/*}"; name="${destination##*/}"
  [[ "$destination" == /* && "$destination" != "$HUB/"* && "$destination" != "$input/"* && -n "$name" && "$name" != . && "$name" != .. &&
    ! -e "$destination" && ! -L "$destination" ]] || live_die UNSAFE_OR_EXISTING_DESTINATION
  parent_subject="$(live_root_subject "$parent")"
  stage="$(mktemp -d -- "$parent/.readme-live-render.XXXXXXXXXX")"
else
  stage="$(mktemp -d)"
fi
render_environment
generate_collection "$raw" "$stage/a"
if [[ "$mode" == --from-captures ]]; then
  generate_collection "$raw" "$stage/b"
  diff -qr --no-dereference -- "$stage/a" "$stage/b" >/dev/null || live_die RECONSTRUCTION_DRIFT
else
  diff -qr --no-dereference -- "$LIVE_TMP/snapshot" "$stage/a" >/dev/null || live_die RECONSTRUCTION_DRIFT
fi
render_environment
collection_shape "$input" "$kind"
[[ "$(live_root_subject "$input")" == "$input_subject" && "$(live_files "$input" "${input_files[@]}")" == "$input_hashes" &&
  "$(live_sources)" == "$sources" && "$(render_tool_receipt)" == "$renderer" ]] || live_die INPUT_CHANGED
if [[ "$mode" == --from-captures ]]; then
  [[ "$(live_root_subject "$parent")" == "$parent_subject" ]] || live_die DESTINATION_CHANGED
  published_subject="$(live_root_subject "$stage/a")"
  mv -T --no-clobber -- "$stage/a" "$destination" || live_die PUBLICATION_INCOMPLETE
  [[ ! -e "$stage/a" && ! -L "$destination" &&
    "$(live_root_subject "$destination")" == "$published_subject" ]] || live_die PUBLICATION_INCOMPLETE
fi
echo 'readme-live-check: PASS (complete deterministic offline unsigned media; capture provenance unverified)'
