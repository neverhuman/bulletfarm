#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="$HUB/docs/readme-live-media"
CLI_DEMOS=(claude-session codex-session cursor-session)
PORTAL_DEMO=portal-ui
ALL_DEMOS=("${CLI_DEMOS[@]}" "$PORTAL_DEMO")
MAX_GIF_BYTES=3145728
VHS_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
SOURCE_EPOCH=1787616000

for tool in cmp docker file find jq rg sha256sum stat; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-live-check: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

expected="$MEDIA/.expected-inventory"
actual="$MEDIA/.actual-inventory"
# those would dirty media; use tmp
tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

printf '%s\n' \
  README.md \
  claude-session/claude-session.gif \
  claude-session/fallback.png \
  claude-session/frames.framemd5 \
  claude-session/manifest.json \
  claude-session/observation.json \
  claude-session/transcript.txt \
  codex-session/codex-session.gif \
  codex-session/fallback.png \
  codex-session/frames.framemd5 \
  codex-session/manifest.json \
  codex-session/observation.json \
  codex-session/transcript.txt \
  cursor-session/cursor-session.gif \
  cursor-session/fallback.png \
  cursor-session/frames.framemd5 \
  cursor-session/manifest.json \
  cursor-session/observation.json \
  cursor-session/transcript.txt \
  portal-ui/fallback.png \
  portal-ui/frames.framemd5 \
  portal-ui/frames/01-control-tower.png \
  portal-ui/frames/02-shift-brief.png \
  portal-ui/frames/03-fleet.png \
  portal-ui/frames/04-mission-graph.png \
  portal-ui/frames/05-control-tower-return.png \
  portal-ui/manifest.json \
  portal-ui/observation.json \
  portal-ui/portal-ui.gif \
  portal-ui/transcript.txt | LC_ALL=C sort >"$tmp/expected"
find "$MEDIA" -type f -print | sed "s#^$MEDIA/##" | LC_ALL=C sort >"$tmp/actual"
cmp "$tmp/expected" "$tmp/actual" || {
  echo "readme-live-check: live media inventory drift" >&2
  diff -u "$tmp/expected" "$tmp/actual" >&2 || true
  exit 1
}
if find "$MEDIA" -type l -print -quit | grep -q .; then
  echo "readme-live-check: media symlinks are forbidden" >&2
  exit 1
fi

secret_pat='(sk-|ghp_|gho_|xox[baprs]-|AKIA[0-9A-Z]{16}|Bearer [A-Za-z0-9._-]+|[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,})'
for demo in "${ALL_DEMOS[@]}"; do
  if rg -n --pcre2 "$secret_pat" "$MEDIA/$demo/transcript.txt" "$MEDIA/$demo/observation.json"; then
    printf 'readme-live-check: redaction failed for %s\n' "$demo" >&2
    exit 1
  fi
  jq -e --arg demo "$demo" '
    .schema_version == "bullet.readme-live-demo.v1"
    and .document_type == "observation"
    and .demo_id == $demo
    and .release_authority == false
    and .live_provider_spawned == false
    and .bullet_live_admission == "disabled"
    and .operator_authenticated == true
  ' "$MEDIA/$demo/observation.json" >/dev/null
done

run_pinned_media_tool() {
  local entrypoint="$1"
  shift
  docker run --rm \
    --network none \
    --pull never \
    --user "$(id -u):$(id -g)" \
    --env HOME=/tmp \
    --env LANG=C.UTF-8 \
    --env LC_ALL=C.UTF-8 \
    --env TZ=UTC \
    --env SOURCE_DATE_EPOCH="$SOURCE_EPOCH" \
    --volume "$MEDIA:/media:ro" \
    --volume "$tmp:/out:rw" \
    --workdir /media \
    --entrypoint "$entrypoint" \
    "$VHS_IMAGE" "$@"
}

for demo in "${ALL_DEMOS[@]}"; do
  directory="$MEDIA/$demo"
  gif="$directory/$demo.gif"
  [[ "$(file --brief --mime-type "$gif")" == "image/gif" ]]
  [[ "$(file --brief --mime-type "$directory/fallback.png")" == "image/png" ]]
  width="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=width -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  height="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=height -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  rate="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  duration="$(run_pinned_media_tool /usr/bin/ffprobe -v error -show_entries format=duration -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  bytes="$(stat -c '%s' "$gif")"
  [[ "$width" == 1200 && "$height" == 675 && "$rate" == "12/1" ]]
  awk -v duration="$duration" 'BEGIN { exit !(duration > 0 && duration <= 30) }'
  (( bytes <= MAX_GIF_BYTES ))
  run_pinned_media_tool /usr/bin/ffmpeg -v error -i "/media/$demo/$demo.gif" \
    -f framemd5 - | awk '!/^#/' >"$tmp/$demo.framemd5"
  cmp "$tmp/$demo.framemd5" "$directory/frames.framemd5"
  while IFS= read -r entry; do
    relative="$(jq -r '.path' <<<"$entry")"
    [[ "$relative" != /* && "$relative" != *..* ]]
    path="$directory/$relative"
    expected_sha="$(jq -r '.sha256' <<<"$entry")"
    expected_bytes="$(jq -r '.bytes' <<<"$entry")"
    actual_sha="$(sha256sum "$path" | awk '{print $1}')"
    actual_bytes="$(stat -c '%s' "$path")"
    [[ "$actual_sha" == "$expected_sha" && "$actual_bytes" == "$expected_bytes" ]] || {
      printf 'readme-live-check: artifact drift for %s\n' "$path" >&2
      exit 1
    }
  done < <(jq -c '.artifact_hashes[]' "$directory/manifest.json")
done

grep -Fq '](docs/readme-live-media/claude-session/claude-session.gif)' "$HUB/README.md"
grep -Fq '](docs/readme-live-media/codex-session/codex-session.gif)' "$HUB/README.md"
grep -Fq '](docs/readme-live-media/cursor-session/cursor-session.gif)' "$HUB/README.md"
grep -Fq '](docs/readme-live-media/portal-ui/portal-ui.gif)' "$HUB/README.md"
grep -Fq 'just readme-live-record' "$HUB/README.md"
grep -Fq 'operator-authenticated' "$HUB/README.md"

# Hosted CI never re-records. A second render must reproduce the committed GIFs.
verify="$tmp/re-render"
mkdir -p "$verify"
# Render into a copy, then compare GIFs. The renderer writes beside transcripts,
# so copy the media tree and run against README_LIVE_MEDIA_ROOT if supported.
# The renderer is hard-wired to the hub path; compare by re-invoking after copy
# is not available. Instead, hash-check plus a second in-place render into tmp
# by copying MEDIA and pointing a wrapper is enough: invoke renderer against a
# disposable hub clone is too heavy. Re-run ffmpeg drawtext locally via the
# renderer by temporarily swapping? Refuse: copy hub scripts + media into tmp
# tree and run renderer there, then cmp GIFs.
clone="$tmp/hub"
mkdir -p "$clone/scripts" "$clone/docs"
cp -a "$HUB/scripts/readme-live-record.sh" "$HUB/scripts/readme-live-render.sh" \
  "$HUB/scripts/readme-live-check.sh" "$clone/scripts/"
cp -a "$MEDIA" "$clone/docs/readme-live-media"
bash "$clone/scripts/readme-live-render.sh"
for demo in "${ALL_DEMOS[@]}"; do
  cmp "$MEDIA/$demo/$demo.gif" "$clone/docs/readme-live-media/$demo/$demo.gif" || {
    printf 'readme-live-check: second render drifted for %s\n' "$demo" >&2
    exit 1
  }
done

echo "readme-live-check: four operator-authenticated README GIFs passed"
