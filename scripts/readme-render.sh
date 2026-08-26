#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="${README_MEDIA_ROOT:-$HUB/docs/readme-media}"
SNAPSHOT="$MEDIA/snapshot.json"
VHS_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
SOURCE_EPOCH=1787616000
VHS_VERSION_OUTPUT='vhs version v0.11.0 (c6af91a)'
gif_root=""
render_tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$render_tmp"
}
trap cleanup EXIT

if [[ "$#" -gt 0 ]]; then
  if [[ "$#" -ne 2 || "$1" != "--gif-root" ]]; then
    echo "usage: $0 [--gif-root ABSOLUTE_DIRECTORY]" >&2
    exit 2
  fi
  gif_root="$2"
  [[ "$gif_root" == /* ]] || {
    echo "readme-render: --gif-root must be absolute" >&2
    exit 2
  }
  mkdir -p "$gif_root"
fi

bash "$HUB/scripts/readme-input-check.sh" "$MEDIA" >/dev/null
render_input="$render_tmp/input"
mkdir -p "$render_input/docs/readme-media/component-preview" \
  "$render_input/docs/readme-media/provider-safety"
cp "$MEDIA/playback.sh" "$render_input/docs/readme-media/playback.sh"
for demo in component-preview provider-safety; do
  cp "$MEDIA/$demo/$demo.tape" "$MEDIA/$demo/transcript.txt" \
    "$render_input/docs/readme-media/$demo/"
done
expected_input="$render_tmp/expected-input"
actual_input="$render_tmp/actual-input"
printf '%s\n' \
  docs/readme-media/playback.sh \
  docs/readme-media/component-preview/component-preview.tape \
  docs/readme-media/component-preview/transcript.txt \
  docs/readme-media/provider-safety/provider-safety.tape \
  docs/readme-media/provider-safety/transcript.txt | LC_ALL=C sort >"$expected_input"
find "$render_input" -type f -print | sed "s#^$render_input/##" | LC_ALL=C sort >"$actual_input"
cmp "$expected_input" "$actual_input" || {
  echo "readme-render: minimal executable-input inventory drift" >&2
  exit 1
}
if find "$render_input" -type l -print -quit | grep -q .; then
  echo "readme-render: minimal executable-input tree contains a symlink" >&2
  exit 1
fi
bash "$HUB/scripts/readme-input-check.sh" "$render_input/docs/readme-media" >/dev/null

for tool in cmp cp docker find jq sha256sum stat; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-render: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
docker image inspect "$VHS_IMAGE" >/dev/null 2>&1 || {
  echo "readme-render: pinned VHS v0.11.0 image is absent; preload the exact digest explicitly" >&2
  exit 1
}
vhs_version="$(docker run --rm --network none --pull never "$VHS_IMAGE" --version)"
[[ "$vhs_version" == "$VHS_VERSION_OUTPUT" ]] || {
  printf 'readme-render: expected %s, found %s\n' "$VHS_VERSION_OUTPUT" "$vhs_version" >&2
  exit 1
}
ffmpeg_version="$(docker run --rm --network none --pull never --entrypoint /usr/bin/ffmpeg "$VHS_IMAGE" -version | head -n 1)"
[[ "$ffmpeg_version" == "ffmpeg version 7.1.3-0+deb13u1 "* ]] || {
  printf 'readme-render: unexpected pinned FFmpeg: %s\n' "$ffmpeg_version" >&2
  exit 1
}

run_pinned_tool() {
  local output_dir="$1"
  local entrypoint="$2"
  shift 2
  docker run --rm \
    --network none \
    --pull never \
    --user "$(id -u):$(id -g)" \
    --env HOME=/tmp \
    --env LANG=C.UTF-8 \
    --env LC_ALL=C.UTF-8 \
    --env TZ=UTC \
    --env SOURCE_DATE_EPOCH="$SOURCE_EPOCH" \
    --volume "$output_dir:/out:rw" \
    --volume "$render_tmp:/render-input:ro" \
    --workdir /out \
    --entrypoint "$entrypoint" \
    "$VHS_IMAGE" "$@"
}

render_gif() {
  local demo="$1"
  local output_dir="$2"
  local transcript="$render_input/docs/readme-media/$demo/transcript.txt"
  local output raw normalized line_dir filter line
  local index start_centiseconds start_seconds y
  output="$output_dir/$demo.gif"
  raw="$output_dir/$demo.vhs.gif"
  normalized="$output_dir/$demo.normalized.gif"
  mkdir -p "$output_dir"
  rm -f "$raw" "$normalized"
  local -a docker_args=(
    run --rm
    --network none
    --pull never
    --user "$(id -u):$(id -g)"
    --env HOME=/tmp
    --env LANG=C.UTF-8
    --env LC_ALL=C.UTF-8
    --env TZ=UTC
    --env SOURCE_DATE_EPOCH="$SOURCE_EPOCH"
    --volume "$render_input:/work:ro"
    --volume "$output_dir:/out:rw"
    --workdir /work
  )
  docker "${docker_args[@]}" "$VHS_IMAGE" --quiet --output "/out/$demo.vhs.gif" \
    "/work/docs/readme-media/$demo/$demo.tape"
  [[ -s "$raw" ]] || {
    printf 'readme-render: VHS did not emit %s\n' "$raw" >&2
    exit 1
  }

  line_dir="$render_tmp/$demo"
  mkdir -p "$line_dir"
  filter='[0:v]'
  index=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '%s\n' "$line" >"$line_dir/line-$index.txt"
    start_centiseconds=$(((index + 1) * 50))
    printf -v start_seconds '%d.%02d' \
      "$((start_centiseconds / 100))" "$((start_centiseconds % 100))"
    y=$((36 + index * 48))
    filter+="drawtext=fontfile=/usr/share/fonts/jetbrains-mono/JetBrainsMono-Regular.ttf:textfile=/render-input/$demo/line-$index.txt:fontcolor=0xe5e7eb:fontsize=22:x=38:y=$y:enable='gte(t,$start_seconds)',"
    index=$((index + 1))
  done <"$transcript"
  (( index > 0 )) || {
    printf 'readme-render: empty transcript for %s\n' "$demo" >&2
    exit 1
  }
  filter="${filter%,},split[a][b];[a]palettegen=stats_mode=full[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle[v]"
  run_pinned_tool "$output_dir" /usr/bin/ffmpeg \
    -v error -y -f lavfi -i 'color=c=0x0b1020:s=1200x675:r=12:d=8' \
    -filter_complex "$filter" -map '[v]' -map_metadata -1 -loop 0 \
    "/out/$demo.normalized.gif"
  [[ -s "$normalized" ]] || {
    printf 'readme-render: pinned FFmpeg did not normalize %s\n' "$output" >&2
    exit 1
  }
  mv "$normalized" "$output"
  rm -f "$raw"
}

write_manifest() {
  local demo="$1"
  local directory="$2"
  local frames="$directory/frames.framemd5"
  local frame_count last_frame artifacts file path sha bytes observed_at
  local snapshot_sha playback_sha schema_sha checker_sha input_checker_sha recorder_sha renderer_sha schema_checker_sha

  observed_at="$(jq -er '.observed_at' "$SNAPSHOT")"
  snapshot_sha="$(sha256sum "$SNAPSHOT" | awk '{print $1}')"
  playback_sha="$(sha256sum "$MEDIA/playback.sh" | awk '{print $1}')"
  schema_sha="$(sha256sum "$HUB/docs/schemas/bullet.readme-demo.v1.schema.json" | awk '{print $1}')"
  checker_sha="$(sha256sum "$HUB/scripts/readme-check.sh" | awk '{print $1}')"
  input_checker_sha="$(sha256sum "$HUB/scripts/readme-input-check.sh" | awk '{print $1}')"
  recorder_sha="$(sha256sum "$HUB/scripts/readme-record.sh" | awk '{print $1}')"
  renderer_sha="$(sha256sum "$HUB/scripts/readme-render.sh" | awk '{print $1}')"
  schema_checker_sha="$(sha256sum "$HUB/scripts/readme-schema-check.sh" | awk '{print $1}')"

  frame_count="$(run_pinned_tool "$directory" /usr/bin/ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of default=nokey=1:noprint_wrappers=1 "/out/$demo.gif")"
  [[ "$frame_count" =~ ^[1-9][0-9]*$ ]] || {
    printf 'readme-render: invalid frame count for %s: %s\n' "$demo" "$frame_count" >&2
    exit 1
  }
  last_frame=$((frame_count - 1))
  run_pinned_tool "$directory" /usr/bin/ffmpeg -v error -y -i "/out/$demo.gif" \
    -vf "select=eq(n\\,$last_frame)" -frames:v 1 /out/fallback.png
  run_pinned_tool "$directory" /usr/bin/ffmpeg -v error -i "/out/$demo.gif" \
    -f framemd5 - | awk '!/^#/' >"$frames"

  artifacts='[]'
  for file in "$demo.tape" transcript.txt observation.json fallback.png "$demo.gif" frames.framemd5; do
    path="$directory/$file"
    [[ -f "$path" ]]
    sha="$(sha256sum "$path" | awk '{print $1}')"
    bytes="$(stat -c '%s' "$path")"
    artifacts="$(jq -c --arg path "$file" --arg sha256 "$sha" --argjson bytes "$bytes" '. + [{path: $path, sha256: $sha256, bytes: $bytes}]' <<<"$artifacts")"
  done

  jq -n --arg demo_id "$demo" --arg observed_at "$observed_at" --arg image "$VHS_IMAGE" \
    --arg snapshot_sha "$snapshot_sha" --arg playback_sha "$playback_sha" \
    --arg schema_sha "$schema_sha" --arg checker_sha "$checker_sha" \
    --arg input_checker_sha "$input_checker_sha" --arg renderer_sha "$renderer_sha" \
    --arg recorder_sha "$recorder_sha" \
    --arg schema_checker_sha "$schema_checker_sha" \
    --argjson artifacts "$artifacts" '
    {
      schema_version: "bullet.readme-demo.v1",
      document_type: "manifest",
      demo_id: $demo_id,
      observed_at: $observed_at,
      classification: "UNSIGNED_COMPONENT_OBSERVATION",
      release_authority: false,
      live_provider_spawned: false,
      network: "disabled-during-render",
      renderer: {
        name: "VHS",
        version: "0.11.0",
        image: $image,
        ffmpeg_version: "7.1.3-0+deb13u1",
        canonicalization: "committed-transcript-drawtext",
        width: 1200,
        height: 675,
        frames_per_second: 12,
        maximum_duration_seconds: 30,
        maximum_gif_bytes: 3145728,
        font: "JetBrains Mono",
        locale: "C.UTF-8",
        timezone: "UTC",
        source_date_epoch: 1787616000,
        cursor_blink: false
      },
      generation: {
        name: "bullet-farm-readme-render",
        version: 1,
        command: "bash scripts/readme-render.sh",
        inputs: [
          {path: "docs/readme-media/playback.sh", sha256: $playback_sha},
          {path: "docs/readme-media/snapshot.json", sha256: $snapshot_sha},
          {path: "docs/schemas/bullet.readme-demo.v1.schema.json", sha256: $schema_sha},
          {path: "scripts/readme-check.sh", sha256: $checker_sha},
          {path: "scripts/readme-input-check.sh", sha256: $input_checker_sha},
          {path: "scripts/readme-record.sh", sha256: $recorder_sha},
          {path: "scripts/readme-render.sh", sha256: $renderer_sha},
          {path: "scripts/readme-schema-check.sh", sha256: $schema_checker_sha}
        ]
      },
      artifact_hashes: $artifacts
    }
  ' >"$directory/manifest.json"
}

if [[ -n "$gif_root" ]]; then
  for demo in component-preview provider-safety; do
    render_gif "$demo" "$gif_root"
  done
  echo "readme-render: rendered verification GIFs to $gif_root"
  exit 0
fi

stage="$render_tmp/staged/readme-media"
mkdir -p "$stage"
cp -a "$MEDIA/." "$stage/"
for demo in component-preview provider-safety; do
  directory="$stage/$demo"
  verification="$render_tmp/determinism/$demo"
  mkdir -p "$directory"
  cp "$MEDIA/$demo/$demo.tape" "$MEDIA/$demo/transcript.txt" \
    "$MEDIA/$demo/observation.json" "$directory/"
  render_gif "$demo" "$directory"
  render_gif "$demo" "$verification"
  cmp "$directory/$demo.gif" "$verification/$demo.gif" || {
    printf 'readme-render: staged render is not byte-deterministic: %s\n' "$demo" >&2
    exit 1
  }
  write_manifest "$demo" "$directory"
  while IFS=$'\t' read -r relative expected_sha expected_bytes; do
    path="$directory/$relative"
    [[ -f "$path" && "$(sha256sum "$path" | awk '{print $1}')" == "$expected_sha" \
      && "$(stat -c '%s' "$path")" == "$expected_bytes" ]] || {
      printf 'readme-render: staged artifact verification failed: %s\n' "$path" >&2
      exit 1
    }
  done < <(jq -r '.artifact_hashes[] | [.path,.sha256,.bytes] | @tsv' "$directory/manifest.json")
done

PATH="$HUB/.ci-tools/readme-jsonschema/bin:$PATH" \
  bash "$HUB/scripts/readme-check.sh" --staged-root "$stage" >/dev/null

# All generation work completed before publication. Each file replacement is
# atomic and the manifest is published last, so an interrupted publication is
# detectably invalid rather than a falsely self-consistent mixed generation.
for demo in component-preview provider-safety; do
  directory="$stage/$demo"
  destination="$MEDIA/$demo"
  for file in fallback.png "$demo.gif" frames.framemd5 manifest.json; do
    temporary="$destination/$file.tmp.$$"
    cp "$directory/$file" "$temporary"
    mv "$temporary" "$destination/$file"
  done
done
echo "readme-render: rendered committed media with pinned VHS and network disabled"
