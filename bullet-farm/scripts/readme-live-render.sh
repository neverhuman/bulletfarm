#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="$HUB/docs/readme-live-media"
CLI_DEMOS=(claude-session codex-session cursor-session)
PORTAL_DEMO=portal-ui
ALL_DEMOS=("${CLI_DEMOS[@]}" "$PORTAL_DEMO")
VHS_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
SOURCE_EPOCH=1787616000
VHS_VERSION_OUTPUT='vhs version v0.11.0 (c6af91a)'
MAX_GIF_BYTES=3145728

for tool in docker jq sha256sum stat; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-live-render: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

vhs_version="$(docker run --rm --network none --pull never "$VHS_IMAGE" --version)"
[[ "$vhs_version" == "$VHS_VERSION_OUTPUT" ]] || {
  printf 'readme-live-render: unexpected VHS version: %s\n' "$vhs_version" >&2
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

render_cli_gif() {
  local demo="$1"
  local output_dir="$MEDIA/$demo"
  local transcript="$output_dir/transcript.txt"
  local line_dir="$render_tmp/$demo"
  local filter index start_centiseconds start_seconds y line
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
  (( index > 0 && index <= 12 )) || {
    printf 'readme-live-render: transcript line count out of range for %s\n' "$demo" >&2
    exit 1
  }
  filter="${filter%,},split[a][b];[a]palettegen=stats_mode=full[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle[v]"
  run_pinned_tool "$output_dir" /usr/bin/ffmpeg \
    -v error -y -f lavfi -i 'color=c=0x0b1020:s=1200x675:r=12:d=8' \
    -filter_complex "$filter" -map '[v]' -map_metadata -1 -loop 0 \
    "/out/$demo.gif"
}

render_portal_gif() {
  local output_dir="$MEDIA/$PORTAL_DEMO"
  local frames="$output_dir/frames"
  local list="$render_tmp/portal-frames.txt"
  local frame
  : >"$list"
  for frame in \
    01-control-tower.png \
    02-shift-brief.png \
    03-fleet.png \
    04-mission-graph.png \
    05-control-tower-return.png
  do
    [[ -f "$frames/$frame" ]] || {
      printf 'readme-live-render: missing portal frame %s\n' "$frame" >&2
      exit 1
    }
    printf "file '/render-input/portal-ui/frames/%s'\nduration 1.5\n" "$frame" >>"$list"
  done
  printf "file '/render-input/portal-ui/frames/05-control-tower-return.png'\n" >>"$list"
  mkdir -p "$render_tmp/portal-ui/frames"
  cp -f "$frames"/*.png "$render_tmp/portal-ui/frames/"
  run_pinned_tool "$output_dir" /usr/bin/ffmpeg \
    -v error -y -f concat -safe 0 -i /render-input/portal-frames.txt \
    -vf "scale=1200:675:flags=lanczos,fps=12,split[a][b];[a]palettegen=stats_mode=full[p];[b][p]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle" \
    -map_metadata -1 -loop 0 "/out/$PORTAL_DEMO.gif"
}

write_sidecar() {
  local demo="$1"
  local directory="$MEDIA/$demo"
  local frames="$directory/frames.framemd5"
  local artifacts file path sha bytes observed_at
  local recorder_sha renderer_sha checker_sha
  observed_at="$(jq -er '.observed_at' "$directory/observation.json")"
  recorder_sha="$(sha256sum "$HUB/scripts/readme-live-record.sh" | awk '{print $1}')"
  renderer_sha="$(sha256sum "$HUB/scripts/readme-live-render.sh" | awk '{print $1}')"
  checker_sha="$(sha256sum "$HUB/scripts/readme-live-check.sh" | awk '{print $1}')"

  run_pinned_tool "$directory" /usr/bin/ffmpeg -v error -i "/out/$demo.gif" \
    -f framemd5 - | awk '!/^#/' >"$frames"
  local frame_count last_frame
  frame_count="$(run_pinned_tool "$directory" /usr/bin/ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of default=nokey=1:noprint_wrappers=1 "/out/$demo.gif")"
  last_frame=$((frame_count - 1))
  run_pinned_tool "$directory" /usr/bin/ffmpeg -v error -y -i "/out/$demo.gif" \
    -vf "select=eq(n\\,$last_frame)" -frames:v 1 /out/fallback.png

  artifacts='[]'
  for file in transcript.txt observation.json fallback.png "$demo.gif" frames.framemd5; do
    path="$directory/$file"
    [[ -f "$path" ]]
    sha="$(sha256sum "$path" | awk '{print $1}')"
    bytes="$(stat -c '%s' "$path")"
    artifacts="$(jq -c --arg path "$file" --arg sha256 "$sha" --argjson bytes "$bytes" \
      '. + [{path: $path, sha256: $sha256, bytes: $bytes}]' <<<"$artifacts")"
  done
  if [[ "$demo" == "$PORTAL_DEMO" ]]; then
    for file in frames/01-control-tower.png frames/02-shift-brief.png \
      frames/03-fleet.png frames/04-mission-graph.png frames/05-control-tower-return.png
    do
      path="$directory/$file"
      [[ -f "$path" ]]
      sha="$(sha256sum "$path" | awk '{print $1}')"
      bytes="$(stat -c '%s' "$path")"
      artifacts="$(jq -c --arg path "$file" --arg sha256 "$sha" --argjson bytes "$bytes" \
        '. + [{path: $path, sha256: $sha256, bytes: $bytes}]' <<<"$artifacts")"
    done
  fi

  jq -n --arg demo_id "$demo" --arg observed_at "$observed_at" --arg image "$VHS_IMAGE" \
    --arg recorder_sha "$recorder_sha" --arg renderer_sha "$renderer_sha" \
    --arg checker_sha "$checker_sha" --argjson artifacts "$artifacts" \
    --argjson gif_bytes "$(stat -c '%s' "$directory/$demo.gif")" '
    {
      schema_version: "bullet.readme-live-demo.v1",
      document_type: "manifest",
      demo_id: $demo_id,
      observed_at: $observed_at,
      classification: "UNSIGNED_OPERATOR_AUTHENTICATED_OBSERVATION",
      release_authority: false,
      live_provider_spawned: false,
      bullet_live_admission: "disabled",
      renderer: {
        name: "VHS",
        version: "0.11.0",
        image: $image,
        ffmpeg_version: "7.1.3-0+deb13u1",
        canonicalization: (if $demo_id == "portal-ui" then "committed-portal-frames" else "committed-transcript-drawtext" end),
        width: 1200,
        height: 675,
        frames_per_second: 12,
        maximum_duration_seconds: 30,
        maximum_gif_bytes: 3145728,
        font: "JetBrains Mono",
        locale: "C.UTF-8",
        timezone: "UTC",
        source_date_epoch: 1787616000
      },
      generation: {
        name: "bullet-farm-readme-live-render",
        version: 1,
        command: "bash scripts/readme-live-render.sh",
        inputs: [
          {path: "scripts/readme-live-record.sh", sha256: $recorder_sha},
          {path: "scripts/readme-live-render.sh", sha256: $renderer_sha},
          {path: "scripts/readme-live-check.sh", sha256: $checker_sha}
        ]
      },
      artifact_hashes: $artifacts,
      gif_bytes: $gif_bytes
    }
  ' >"$directory/manifest.json"
  (( $(stat -c '%s' "$directory/$demo.gif") <= MAX_GIF_BYTES ))
}

render_tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$render_tmp"
}
trap cleanup EXIT

for demo in "${CLI_DEMOS[@]}"; do
  render_cli_gif "$demo"
done
render_portal_gif
for demo in "${ALL_DEMOS[@]}"; do
  write_sidecar "$demo"
done
echo "readme-live-render: wrote four operator-authenticated README GIFs"
