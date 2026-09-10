#!/usr/bin/env bash
# Tracked real-TUI recorder: PTY capture -> tracked render -> 1920x1080 canvas
# -> bullet.real-media.v1 manifest -> scripts/readme-real-check.sh.
#
# It replaces the local wrappers media/dogfood/{run-real-e2e.sh,session.sh} and
# fixes the five defects media/dogfood/README.md lists against them:
#   * no per-session scratchpad path is baked in; --out-dir is explicit, private
#     and checked before anything is written to it
#   * the narration's exit status is propagated instead of discarded
#   * exactly one run directory and one proof directory are created and handed
#     to the narration, so nothing selects "the latest" of anything
#   * the native capture result, the raw PTY byte stream and the transcript are
#     retained next to the published GIF
#   * every published artifact is re-checked by the stage-two gate before this
#     script reports success
#
# This script records. It establishes no product completion, no provider
# qualification and no release authority. See scripts/media/README.md for the
# manifest schema, the run.json contract, the geometry math and what a
# recording made with it may and may not claim.
set -euo pipefail
shopt -s inherit_errexit
umask 077

TUI_MEDIA="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
TUI_HUB="$(cd -- "$TUI_MEDIA/../.." && pwd -P)"
TUI_FAMILY="$(cd -- "$TUI_HUB/.." && pwd -P)"
TUI_SELF="$TUI_MEDIA/record-tui.sh"
TUI_WIDTH=1920
TUI_HEIGHT=1080
# agg 1.5.0 defaults. scripts/lib/demo-gif-render.py invokes agg without
# --font-size, --line-height, --font-family or --fps-cap, so the recorder does
# not get to choose them; it only declares them, and readme-real-check.sh
# refuses the manifest if they do not re-derive the rendered geometry.
TUI_FONT_SIZE=14
TUI_LINE_HEIGHT=1.4
TUI_FONT_FAMILY=agg-default-chain
TUI_THEME=github-light
TUI_RENDERER=fontdue
TUI_MEMBERS=(bullet-farm bullet-git bullet-kernel bullet-portal)

tui_die() { printf 'RECORD_TUI_%s: %s\n' "$1" "$2" >&2; exit 1; }
tui_say() { printf 'record-tui: %s\n' "$*"; }

tui_usage() {
  cat <<'USAGE'
usage:
  record-tui.sh --out-dir DIR --narration SCRIPT --bullet-bin PATH --farmd-bin PATH
                [--cols N] [--rows N] [--run-id ID] [--max-seconds N]
  record-tui.sh --self-test --out-dir DIR [--cols N] [--rows N] [--run-id ID]

--out-dir       absolute, must not exist yet, created 0700, under $HOME, never
                inside the source family and never under /tmp
--narration     absolute path to an executable script run inside the PTY; it
                must write run.json (see scripts/media/README.md)
--cols/--rows   terminal grid; defaults 225x54 render at 1913x1078 with the
                tracked renderer's agg defaults and are centred on 1920x1080
--bullet-bin    the exact bullet binary the recorded session runs
--farmd-bin     the exact bullet-farmd binary the recorded session runs
--run-id        optional; also names the published artifacts
--max-seconds   PTY capture-loop limit, 1..600 (default 120)
--self-test     record a harmless synthetic narration instead; declares
                claims ["self-test"] and names no product binary
USAGE
}

tui_sha() { local line; line="$(sha256sum -- "$1")" || tui_die HASH "$1"; printf '%s' "${line%% *}"; }

tui_value() {  # flag value...
  [[ $# -ge 2 && -n "${2-}" && "${2-}" != --* ]] || tui_die ARGUMENT "$1 needs a value"
  printf '%s' "$2"
}

tui_resolve() {  # command -> canonical absolute path
  local found
  found="$(command -v "$1")" || tui_die TOOL_UNAVAILABLE "$1"
  realpath -e -- "$found" || tui_die TOOL_UNAVAILABLE "$1"
}

# scripts/lib/demo-gif-render.py hashes every tool by reading it whole under a
# 64 MiB cap, so a large static ffmpeg build cannot be admitted at all. Pick the
# first ffmpeg on PATH that the tracked renderer can actually read, or take
# BULLET_RECORD_FFMPEG when the operator names one.
tui_select_ffmpeg() {
  local -a candidates=() entries=()
  local candidate canonical size limit=$((64 * 1024 * 1024))
  if [[ -n "${BULLET_RECORD_FFMPEG-}" ]]; then
    candidates=("$BULLET_RECORD_FFMPEG")
  else
    IFS=: read -r -a entries <<<"$PATH"
    for candidate in "${entries[@]}"; do
      candidates+=("${candidate:-.}/ffmpeg")
    done
  fi
  for candidate in "${candidates[@]}"; do
    canonical="$(realpath -e -- "$candidate" 2>/dev/null)" || continue
    [[ -f "$canonical" && ! -L "$canonical" && -x "$canonical" ]] || continue
    size="$(stat -c '%s' -- "$canonical")" || continue
    ((size <= limit)) || continue
    printf '%s' "$canonical"
    return 0
  done
  tui_die TOOL_UNAVAILABLE "no ffmpeg of at most $limit bytes, which is all the tracked renderer can hash"
}

tui_file() {  # label path -> canonical path of an existing plain executable file
  local canonical
  [[ "$2" == /* ]] || tui_die "$1" "$2 is not absolute"
  [[ -f "$2" && ! -L "$2" && -x "$2" ]] || tui_die "$1" "$2 is not a plain executable file"
  canonical="$(realpath -e -- "$2")" || tui_die "$1" "$2"
  [[ "$canonical" == "$2" ]] || tui_die "$1" "$2 is not canonical"
  printf '%s' "$canonical"
}

# --- arguments ---------------------------------------------------------------
TUI_OUT=""
TUI_NARRATION=""
TUI_COLS=225
TUI_ROWS=54
TUI_BULLET=""
TUI_FARMD=""
TUI_RUN_ID=""
TUI_MAX_SECONDS=120
TUI_SELF_TEST=0

while (($#)); do
  case "$1" in
    --out-dir) TUI_OUT="$(tui_value "$@")"; shift 2 ;;
    --narration) TUI_NARRATION="$(tui_value "$@")"; shift 2 ;;
    --cols) TUI_COLS="$(tui_value "$@")"; shift 2 ;;
    --rows) TUI_ROWS="$(tui_value "$@")"; shift 2 ;;
    --bullet-bin) TUI_BULLET="$(tui_value "$@")"; shift 2 ;;
    --farmd-bin) TUI_FARMD="$(tui_value "$@")"; shift 2 ;;
    --run-id) TUI_RUN_ID="$(tui_value "$@")"; shift 2 ;;
    --max-seconds) TUI_MAX_SECONDS="$(tui_value "$@")"; shift 2 ;;
    --self-test) TUI_SELF_TEST=1; shift ;;
    -h | --help) tui_usage; exit 0 ;;
    *) tui_usage >&2; exit 2 ;;
  esac
done

tui_check_arguments() {
  [[ -n "$TUI_OUT" ]] || { tui_usage >&2; exit 2; }
  [[ "$TUI_COLS" =~ ^[1-9][0-9]{0,2}$ && "$TUI_COLS" -le 240 ]] || tui_die COLS "$TUI_COLS"
  [[ "$TUI_ROWS" =~ ^[1-9][0-9]?$ || "$TUI_ROWS" == 100 ]] || tui_die ROWS "$TUI_ROWS"
  [[ "$TUI_MAX_SECONDS" =~ ^[1-9][0-9]{0,2}$ && "$TUI_MAX_SECONDS" -le 600 ]] ||
    tui_die MAX_SECONDS "$TUI_MAX_SECONDS"
  [[ -z "$TUI_RUN_ID" || "$TUI_RUN_ID" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,47}$ ]] || tui_die RUN_ID "$TUI_RUN_ID"
  if ((TUI_SELF_TEST)); then
    [[ -z "$TUI_NARRATION" ]] || tui_die ARGUMENT "--self-test supplies its own narration"
    [[ -z "$TUI_BULLET$TUI_FARMD" ]] ||
      tui_die ARGUMENT "--self-test runs no product binary, so it must not name one"
  else
    [[ -n "$TUI_NARRATION" ]] || { tui_usage >&2; exit 2; }
    [[ -n "$TUI_BULLET" && -n "$TUI_FARMD" ]] || { tui_usage >&2; exit 2; }
    TUI_NARRATION="$(tui_file NARRATION "$TUI_NARRATION")"
    TUI_BULLET="$(tui_file BULLET_BIN "$TUI_BULLET")"
    TUI_FARMD="$(tui_file FARMD_BIN "$TUI_FARMD")"
  fi
}

# The output tree has to be private, brand new and outside both the source
# family (the tracked renderer refuses to write there) and /tmp (world-writable
# ancestors are refused by the capture and render private-path checks).
tui_check_out_dir() {
  local parent
  [[ -n "${HOME-}" && "$HOME" == /* ]] || tui_die ENVIRONMENT "HOME is not an absolute path"
  [[ "$TUI_OUT" == /* ]] || tui_die OUT_DIR "$TUI_OUT is not absolute"
  [[ "$TUI_OUT" != */ && "$TUI_OUT" != *//* ]] || tui_die OUT_DIR "$TUI_OUT is not a plain path"
  [[ "$TUI_OUT" != *"/./"* && "$TUI_OUT" != *"/../"* && "$TUI_OUT" != */. && "$TUI_OUT" != */.. ]] ||
    tui_die OUT_DIR "$TUI_OUT is not a plain path"
  [[ "$TUI_OUT" == "$HOME"/* ]] || tui_die OUT_DIR "$TUI_OUT is not under HOME"
  [[ "$TUI_OUT" != "$TUI_FAMILY" && "$TUI_OUT" != "$TUI_FAMILY"/* ]] ||
    tui_die OUT_DIR "$TUI_OUT is inside the source family"
  [[ "$TUI_OUT" != /tmp && "$TUI_OUT" != /tmp/* ]] || tui_die OUT_DIR "$TUI_OUT is under /tmp"
  [[ ! -e "$TUI_OUT" && ! -L "$TUI_OUT" ]] || tui_die OUT_DIR "$TUI_OUT already exists"
  parent="$(dirname -- "$TUI_OUT")"
  [[ -d "$parent" ]] || tui_die OUT_DIR "$parent does not exist"
  [[ "$(realpath -e -- "$parent")" == "$parent" ]] || tui_die OUT_DIR "$parent is not canonical"
  mkdir -m 700 -- "$TUI_OUT" || tui_die OUT_DIR "$TUI_OUT could not be created privately"
  [[ "$(realpath -e -- "$TUI_OUT")" == "$TUI_OUT" ]] || tui_die OUT_DIR "$TUI_OUT is not canonical"
}

tui_layout() {
  TUI_NAME="${TUI_RUN_ID:-tui-$(date -u +%Y%m%dT%H%M%SZ)}"
  [[ "$TUI_NAME" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$ ]] || tui_die NAME "$TUI_NAME"
  TUI_CAPTURE="$TUI_OUT/capture"
  TUI_RENDER="$TUI_OUT/render"
  TUI_RUN_DIR="$TUI_OUT/run"
  TUI_PROOF_DIR="$TUI_OUT/proof"
  TUI_WORK="$TUI_OUT/work"
  TUI_RUN_JSON="$TUI_OUT/run.json"
  # $TUI_RENDER is created by the tracked renderer itself and left untouched
  # afterwards so scripts/demo-gif-check.sh can still re-verify that generation.
  mkdir -m 700 -- "$TUI_CAPTURE" "$TUI_RUN_DIR" "$TUI_PROOF_DIR" "$TUI_WORK"
}

tui_tools() {
  local tool version
  for tool in awk cp date dirname git jq od python3 realpath sha256sum stat tr; do
    command -v "$tool" >/dev/null 2>&1 || tui_die TOOL_UNAVAILABLE "$tool"
  done
  TUI_PYTHON="$(tui_resolve python3)"
  TUI_FFMPEG="$(tui_select_ffmpeg)"
  TUI_FFPROBE="$(tui_resolve ffprobe)"
  TUI_AGG="$(realpath -e -- "${BULLET_RECORD_AGG:-$HOME/.cache/bullet-demo-gif/bin/agg}")" ||
    tui_die TOOL_UNAVAILABLE agg
  [[ -f "$TUI_AGG" && -x "$TUI_AGG" ]] || tui_die TOOL_UNAVAILABLE "$TUI_AGG"
  for tool in "$TUI_HUB/scripts/demo-gif-render.sh" "$TUI_HUB/scripts/lib/demo-gif-render.py" \
    "$TUI_HUB/scripts/lib/demo-gif-pty-record.py" "$TUI_HUB/scripts/readme-real-check.sh"; do
    [[ -f "$tool" && ! -L "$tool" ]] || tui_die TOOL_UNAVAILABLE "$tool"
  done
  TUI_PYTHON_SHA="$(tui_sha "$TUI_PYTHON")"
  TUI_FFMPEG_SHA="$(tui_sha "$TUI_FFMPEG")"
  TUI_AGG_SHA="$(tui_sha "$TUI_AGG")"
  TUI_IMPL_SHA="$(tui_sha "$TUI_HUB/scripts/lib/demo-gif-render.py")"
  TUI_RECORDER_SHA="$(tui_sha "$TUI_SELF")"
  TUI_PYTHON_VERSION="$("$TUI_PYTHON" -c 'import platform; print(platform.python_version())')"
  TUI_FFMPEG_VERSION="$("$TUI_FFMPEG" -version | awk 'NR == 1 {print $3}')"
  TUI_AGG_VERSION="$("$TUI_AGG" --version | awk 'NR == 1 {print $2}')"
  for version in "$TUI_PYTHON_VERSION" "$TUI_FFMPEG_VERSION" "$TUI_AGG_VERSION"; do
    [[ "$version" =~ ^[0-9]+(\.[0-9]+){0,3}([-+][A-Za-z0-9.-]{1,32})?$ ]] || tui_die TOOL_VERSION "$version"
  done
  TUI_HOST="$(uname -n)"
  TUI_HOST="${TUI_HOST%%.*}"
  [[ "$TUI_HOST" =~ ^[A-Za-z0-9][A-Za-z0-9.-]{0,62}$ ]] || tui_die HOST "$TUI_HOST"
}

# --- recording ---------------------------------------------------------------
tui_write_wrapper() {
  cat >"$TUI_WORK/pty-launcher.sh" <<'LAUNCHER'
#!/usr/bin/env bash
# In-PTY launcher. It runs the narration, records the narration's exact exit
# status beside the capture as producer.exit (which scripts/lib/demo-gif-render.py
# re-reads and refuses unless it is 0), and exits with that same status so the
# PTY recorder reports it and record-tui.sh propagates it.
set -u
"$BULLET_RECORD_NARRATION"
launcher_status=$?
printf '%s\n' "$launcher_status" >"$BULLET_RECORD_CAPTURE_DIR/producer.exit"
exit "$launcher_status"
LAUNCHER
  chmod 700 -- "$TUI_WORK/pty-launcher.sh"
}

tui_record() {
  local status=0 inventory
  tui_write_wrapper
  export BULLET_RECORD_NARRATION="$TUI_NARRATION"
  export BULLET_RECORD_CAPTURE_DIR="$TUI_CAPTURE"
  export BULLET_RECORD_OUT_DIR="$TUI_OUT"
  export BULLET_RECORD_RUN_DIR="$TUI_RUN_DIR"
  export BULLET_RECORD_PROOF_DIR="$TUI_PROOF_DIR"
  export BULLET_RECORD_RUN_JSON="$TUI_RUN_JSON"
  export BULLET_RECORD_RUN_ID="$TUI_NAME"
  export BULLET_RECORD_COLS="$TUI_COLS"
  export BULLET_RECORD_ROWS="$TUI_ROWS"
  export BULLET_RECORD_BULLET_BIN="$TUI_BULLET"
  export BULLET_RECORD_FARMD_BIN="$TUI_FARMD"
  tui_say "recording ${TUI_COLS}x${TUI_ROWS} for at most ${TUI_MAX_SECONDS}s"
  "$TUI_PYTHON" -I "$TUI_HUB/scripts/lib/demo-gif-pty-record.py" \
    --cast "$TUI_CAPTURE/session.cast" --transcript "$TUI_CAPTURE/transcript.txt" \
    --cols "$TUI_COLS" --rows "$TUI_ROWS" --max-seconds "$TUI_MAX_SECONDS" \
    --title "bullet-farm record-tui $TUI_NAME" -- "$TUI_WORK/pty-launcher.sh" || status=$?
  printf '%s\n' "$status" >"$TUI_OUT/narration-exit.txt"
  if ((status != 0)); then
    printf 'RECORD_TUI_NARRATION_FAILED: exit %s (capture retained)\n' "$status" >&2
    exit "$status"
  fi
  [[ -f "$TUI_CAPTURE/producer.exit" && "$(cat -- "$TUI_CAPTURE/producer.exit")" == 0 ]] ||
    tui_die NARRATION_FAILED "producer.exit is not 0"
  inventory="$(cd -- "$TUI_CAPTURE" && printf '%s\n' * | sort | tr '\n' ' ')"
  [[ "$inventory" == "producer.exit session.cast session.cast.raw session.cast.result.json transcript.txt " ]] ||
    tui_die CAPTURE_INVENTORY "$inventory"
  TUI_CAPTURE_SHA="$(tui_sha "$TUI_CAPTURE/session.cast.result.json")"
}

# --- render ------------------------------------------------------------------
tui_render() {
  tui_say "rendering through scripts/demo-gif-render.sh"
  bash "$TUI_HUB/scripts/demo-gif-render.sh" "$TUI_PYTHON" "$TUI_PYTHON_SHA" "$TUI_IMPL_SHA" \
    --kind terminal --input "$TUI_CAPTURE" --input-sha256 "$TUI_CAPTURE_SHA" \
    --output "$TUI_RENDER" --ffmpeg "$TUI_FFMPEG" --ffmpeg-sha256 "$TUI_FFMPEG_SHA" \
    --agg "$TUI_AGG" --agg-sha256 "$TUI_AGG_SHA" --timeout 120 >"$TUI_WORK/render.log" 2>&1 || {
    cat -- "$TUI_WORK/render.log" >&2
    tui_die RENDER_FAILED "scripts/demo-gif-render.sh"
  }
  [[ -f "$TUI_RENDER/derivative.gif" && -f "$TUI_RENDER/manifest.json" ]] ||
    tui_die RENDER_FAILED "the render produced no derivative and receipt"
  TUI_RENDER_RECEIPT_SHA="$(tui_sha "$TUI_RENDER/manifest.json")"
}

# The native render can never be exactly 1920x1080: agg lays out (rows + 1)
# boxes of font_size * line_height and (cols + 2) advance widths, and no integer
# grid hits 1080 at line height 1.4. The published GIF is therefore the native
# render centred on the target canvas by rewriting the GIF logical screen and
# the per-frame image offsets: no pixel, palette or delay is re-encoded.
tui_canvas() {
  local source="$TUI_RENDER/derivative.gif" geometry predicted result probed
  source="$TUI_RENDER/derivative.gif"
  geometry="$("$TUI_FFPROBE" -v error -select_streams v:0 -show_entries stream=width,height \
    -of csv=p=0 -- "$source")" || tui_die FFPROBE "$source"
  TUI_NATIVE_W="${geometry%%,*}"
  TUI_NATIVE_H="${geometry##*,}"
  [[ "$TUI_NATIVE_W" =~ ^[0-9]+$ && "$TUI_NATIVE_H" =~ ^[0-9]+$ ]] || tui_die FFPROBE "$geometry"
  ((TUI_NATIVE_W <= TUI_WIDTH && TUI_NATIVE_H <= TUI_HEIGHT)) ||
    tui_die RENDER_GEOMETRY "${TUI_NATIVE_W}x${TUI_NATIVE_H} does not fit ${TUI_WIDTH}x${TUI_HEIGHT}; lower --cols/--rows"
  predicted="$("$TUI_PYTHON" -c 'import sys; print(round((int(sys.argv[1]) + 1) * int(sys.argv[2]) * float(sys.argv[3])))' \
    "$TUI_ROWS" "$TUI_FONT_SIZE" "$TUI_LINE_HEIGHT")"
  [[ "$predicted" == "$TUI_NATIVE_H" ]] ||
    tui_die RENDER_GEOMETRY "declared cell box predicts ${predicted}px, agg rendered ${TUI_NATIVE_H}px"
  TUI_OFFSET_X=$(((TUI_WIDTH - TUI_NATIVE_W) / 2))
  TUI_OFFSET_Y=$(((TUI_HEIGHT - TUI_NATIVE_H) / 2))
  if ((TUI_NATIVE_W == TUI_WIDTH && TUI_NATIVE_H == TUI_HEIGHT)); then
    TUI_CANVAS_METHOD=NONE
    TUI_CANVAS_BACKGROUND=""
    cp -- "$source" "$TUI_OUT/$TUI_NAME.gif"
  else
    TUI_CANVAS_METHOD=GIF_LOGICAL_SCREEN_EXPANSION
    "$TUI_FFMPEG" -nostdin -hide_banner -loglevel error -y -i "$source" -frames:v 1 \
      -vf crop=1:1:0:0 -pix_fmt rgb24 -f rawvideo "$TUI_WORK/background.rgb" ||
      tui_die CANVAS "the render background pixel could not be decoded"
    TUI_CANVAS_BACKGROUND="$(od -An -v -tx1 -N3 -- "$TUI_WORK/background.rgb" | tr -d ' \n')"
    [[ "$TUI_CANVAS_BACKGROUND" =~ ^[0-9a-f]{6}$ ]] || tui_die CANVAS "background $TUI_CANVAS_BACKGROUND"
    result="$("$TUI_PYTHON" -I - "$source" "$TUI_OUT/$TUI_NAME.gif" "$TUI_WIDTH" "$TUI_HEIGHT" \
      "$TUI_CANVAS_BACKGROUND" <<'PY'
"""Centre a GIF on a larger canvas without touching a single pixel.

Only the logical screen size, the background colour index, the first global
colour table entry and each frame's left/top offset change. Every image
descriptor must carry its own local colour table, or repainting the global
entry would alter frame pixels and the rewrite is refused instead.
"""
import struct
import sys

source, target, width, height, background = sys.argv[1:6]
width, height, background = int(width), int(height), bytes.fromhex(background)
data = bytearray(open(source, "rb").read())
if bytes(data[:6]) not in [b"GIF87a", b"GIF89a"]:
    raise SystemExit("CANVAS_GIF_REQUIRED")
native = struct.unpack("<HH", data[6:10])
if not 0 < native[0] <= width or not 0 < native[1] <= height:
    raise SystemExit(f"CANVAS_NATIVE_TOO_LARGE:{native[0]}x{native[1]}")
dx, dy = (width - native[0]) // 2, (height - native[1]) // 2
position = 13
if not data[10] & 128:
    raise SystemExit("CANVAS_NO_GLOBAL_COLOUR_TABLE")
position += 3 * 2 ** ((data[10] & 7) + 1)
frames = 0
while True:
    marker = data[position]
    position += 1
    if marker == 0x21:
        position += 1
        while data[position]:
            position += data[position] + 1
        position += 1
    elif marker == 0x2C:
        left, top, wide, high = struct.unpack("<HHHH", data[position:position + 8])
        if left + wide > native[0] or top + high > native[1]:
            raise SystemExit(f"CANVAS_FRAME_OUTSIDE_SCREEN:{frames}")
        flags = data[position + 8]
        if not flags & 128:
            raise SystemExit(f"CANVAS_FRAME_WITHOUT_LOCAL_COLOUR_TABLE:{frames}")
        struct.pack_into("<HH", data, position, left + dx, top + dy)
        position += 9 + 3 * 2 ** ((flags & 7) + 1) + 1
        while data[position]:
            position += data[position] + 1
        position += 1
        frames += 1
    elif marker == 0x3B:
        if position != len(data) or not frames:
            raise SystemExit("CANVAS_TRAILER_INVALID")
        break
    else:
        raise SystemExit(f"CANVAS_BLOCK_{marker:02x}")
data[11] = 0
data[13:16] = background
struct.pack_into("<HH", data, 6, width, height)
with open(target, "xb") as stream:
    stream.write(data)
print(f"{dx} {dy} {frames}")
PY
    )" || tui_die CANVAS "the logical screen could not be rewritten"
    [[ "$result" == "$TUI_OFFSET_X $TUI_OFFSET_Y "* ]] || tui_die CANVAS "offsets $result"
  fi
  probed="$("$TUI_FFPROBE" -v error -select_streams v:0 -show_entries stream=width,height \
    -of csv=p=0 -- "$TUI_OUT/$TUI_NAME.gif")" || tui_die FFPROBE "$TUI_NAME.gif"
  [[ "$probed" == "$TUI_WIDTH,$TUI_HEIGHT" ]] || tui_die CANVAS "published geometry $probed"
  tui_say "canvas ${TUI_NATIVE_W}x${TUI_NATIVE_H} -> ${TUI_WIDTH}x${TUI_HEIGHT} at +${TUI_OFFSET_X}+${TUI_OFFSET_Y}"
}

# --- manifest ----------------------------------------------------------------
tui_read_run_json() {
  local file="$TUI_RUN_JSON"
  [[ -f "$file" && ! -L "$file" ]] || tui_die RUN_JSON "the narration wrote no run.json"
  jq -e '.schema == "bullet.record-run.v1"' "$file" >/dev/null 2>&1 ||
    tui_die RUN_JSON "schema is not bullet.record-run.v1"
  jq -e '.claims | type == "array" and length > 0 and length <= 8 and all(.[]; type == "string")' \
    "$file" >/dev/null 2>&1 || tui_die RUN_JSON "claims is not a short array of strings"
  TUI_COMMAND_ID="$(jq -r '.command_id // ""' "$file")"
  TUI_ATTEMPT_ID="$(jq -r '.attempt_id // ""' "$file")"
  TUI_CANDIDATE_ID="$(jq -r 'if .candidate_id == null then "" else (.candidate_id // "") end' "$file")"
  TUI_RECEIPT_ALGORITHM="$(jq -r '.receipt.algorithm // ""' "$file")"
  TUI_RECEIPT_DIGEST="$(jq -r '.receipt.digest // ""' "$file")"
  TUI_PROVIDER_NAME="$(jq -r '.provider.name // ""' "$file")"
  TUI_PROVIDER_VERSION="$(jq -r '.provider.runtime_version // ""' "$file")"
  TUI_CLAIMS="$(jq -cS '.claims' "$file")"
  [[ "$TUI_COMMAND_ID" =~ ^cmd_[0-9a-f]{64}$ ]] || tui_die RUN_JSON "command_id $TUI_COMMAND_ID"
  [[ "$TUI_ATTEMPT_ID" =~ ^atm_[0-9a-f]{64}$ ]] || tui_die RUN_JSON "attempt_id $TUI_ATTEMPT_ID"
  [[ -z "$TUI_CANDIDATE_ID" || "$TUI_CANDIDATE_ID" =~ ^can_[0-9a-f]{64}$ ]] ||
    tui_die RUN_JSON "candidate_id $TUI_CANDIDATE_ID"
  [[ "$TUI_RECEIPT_ALGORITHM" == blake3 || "$TUI_RECEIPT_ALGORITHM" == sha256 ]] ||
    tui_die RUN_JSON "receipt.algorithm $TUI_RECEIPT_ALGORITHM"
  [[ "$TUI_RECEIPT_DIGEST" =~ ^[0-9a-f]{64}$ ]] || tui_die RUN_JSON "receipt.digest is not a 64-hex digest"
  [[ "$TUI_PROVIDER_NAME" =~ ^[a-z][a-z0-9-]{0,31}$ ]] || tui_die RUN_JSON "provider.name $TUI_PROVIDER_NAME"
  [[ "$TUI_PROVIDER_VERSION" =~ ^[0-9]+(\.[0-9]+){0,3}([-+][A-Za-z0-9.-]{1,32})?$ ]] ||
    tui_die RUN_JSON "provider.runtime_version $TUI_PROVIDER_VERSION"
  # A self-test recording claims nothing else and names no product binary; every
  # other recording must claim neither more nor less than what it can support.
  if ((TUI_SELF_TEST)); then
    [[ "$TUI_CLAIMS" == '["self-test"]' ]] || tui_die CLAIMS "--self-test declared $TUI_CLAIMS"
  else
    [[ "$TUI_CLAIMS" != *'"self-test"'* ]] || tui_die CLAIMS "a real recording must not claim self-test"
  fi
}

tui_members_json() {
  local member directory oid dirty
  for member in "${TUI_MEMBERS[@]}"; do
    directory="$TUI_FAMILY/$member"
    [[ -d "$directory/.git" ]] || tui_die MEMBER "$member is not a checkout"
    oid="$(GIT_OPTIONAL_LOCKS=0 git -C "$directory" rev-parse HEAD)" || tui_die MEMBER "$member HEAD"
    if [[ -n "$(GIT_OPTIONAL_LOCKS=0 git -C "$directory" status --porcelain)" ]]; then
      dirty=true
    else
      dirty=false
    fi
    printf '%s\t%s\t%s\n' "$member" "$oid" "$dirty"
  done | jq -R -s -c 'split("\n") | map(select(length > 0) | split("\t"))
    | map({key: .[0], value: {oid: .[1], dirty: (.[2] == "true")}}) | from_entries'
}

tui_manifest() {
  local binaries members
  members="$(tui_members_json)"
  if ((TUI_SELF_TEST)); then
    binaries=null
  else
    binaries="$(jq -nc --arg bullet "$(tui_sha "$TUI_BULLET")" --arg farmd "$(tui_sha "$TUI_FARMD")" \
      '{bullet: {sha256: $bullet}, "bullet-farmd": {sha256: $farmd}}')"
  fi
  cp -- "$TUI_CAPTURE/session.cast" "$TUI_OUT/$TUI_NAME.cast"
  jq -nS \
    --arg name "$TUI_NAME" \
    --arg recorded_at "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
    --arg host "$TUI_HOST" \
    --arg recorder_sha "$TUI_RECORDER_SHA" \
    --arg agg_sha "$TUI_AGG_SHA" --arg agg_version "$TUI_AGG_VERSION" \
    --arg ffmpeg_sha "$TUI_FFMPEG_SHA" --arg ffmpeg_version "$TUI_FFMPEG_VERSION" \
    --arg python_sha "$TUI_PYTHON_SHA" --arg python_version "$TUI_PYTHON_VERSION" \
    --argjson binaries "$binaries" --argjson members "$members" \
    --arg command_id "$TUI_COMMAND_ID" --arg attempt_id "$TUI_ATTEMPT_ID" \
    --arg candidate_id "$TUI_CANDIDATE_ID" \
    --arg receipt_algorithm "$TUI_RECEIPT_ALGORITHM" --arg receipt_digest "$TUI_RECEIPT_DIGEST" \
    --arg provider_name "$TUI_PROVIDER_NAME" --arg provider_version "$TUI_PROVIDER_VERSION" \
    --arg cast_sha "$(tui_sha "$TUI_OUT/$TUI_NAME.cast")" \
    --arg gif_sha "$(tui_sha "$TUI_OUT/$TUI_NAME.gif")" \
    --argjson gif_bytes "$(stat -c '%s' -- "$TUI_OUT/$TUI_NAME.gif")" \
    --arg canvas_method "$TUI_CANVAS_METHOD" \
    --arg canvas_background "$TUI_CANVAS_BACKGROUND" \
    --argjson native_width "$TUI_NATIVE_W" --argjson native_height "$TUI_NATIVE_H" \
    --argjson offset_x "$TUI_OFFSET_X" --argjson offset_y "$TUI_OFFSET_Y" \
    --arg capture_sha "$TUI_CAPTURE_SHA" --arg receipt_sha "$TUI_RENDER_RECEIPT_SHA" \
    --argjson cols "$TUI_COLS" --argjson rows "$TUI_ROWS" \
    --arg font_family "$TUI_FONT_FAMILY" --argjson font_size "$TUI_FONT_SIZE" \
    --argjson line_height "$TUI_LINE_HEIGHT" --arg theme "$TUI_THEME" --arg renderer "$TUI_RENDERER" \
    --argjson claims "$TUI_CLAIMS" '{
      schema: "bullet.real-media.v1", kind: "terminal", name: $name,
      recorded_at: $recorded_at, host: $host,
      recorder: {path: "scripts/media/record-tui.sh", sha256: $recorder_sha},
      render_tools: {
        agg: {sha256: $agg_sha, version: $agg_version},
        ffmpeg: {sha256: $ffmpeg_sha, version: $ffmpeg_version},
        python: {sha256: $python_sha, version: $python_version}},
      binaries: $binaries, members: $members,
      run: {command_id: $command_id, attempt_id: $attempt_id,
            candidate_id: (if $candidate_id == "" then null else $candidate_id end)},
      receipt: {algorithm: $receipt_algorithm, digest: $receipt_digest},
      provider: {name: $provider_name, runtime_version: $provider_version},
      account: "REDACTED",
      master: {cast_sha256: $cast_sha},
      gif: {sha256: $gif_sha, bytes: $gif_bytes},
      canvas: {method: $canvas_method, native_width: $native_width, native_height: $native_height,
               offset_x: $offset_x, offset_y: $offset_y,
               background_rgb: (if $canvas_background == "" then null else $canvas_background end)},
      capture_sha256: $capture_sha, render_receipt_sha256: $receipt_sha,
      terminal: {cols: $cols, rows: $rows, font_family: $font_family, font_size: $font_size,
                 line_height: $line_height, theme: $theme, renderer: $renderer},
      claims: $claims, release_eligible: false
    }' >"$TUI_OUT/$TUI_NAME.manifest.json"
}

# --- self-test ---------------------------------------------------------------
tui_self_test_narration() {
  cat >"$TUI_WORK/selftest-narration.sh" <<'NARRATION'
#!/usr/bin/env bash
# Harmless narration for record-tui.sh --self-test. It calls no provider, spends
# nothing, touches no family repository, and prints nothing the stage-two
# redaction screen has to refuse: no path, no address, no undeclared digest.
set -u
bold=$'\033[1m'
dim=$'\033[38;5;245m'
green=$'\033[38;5;35m'
off=$'\033[0m'
printf '%srecord-tui --self-test%s   synthetic narration: no provider, no spend\n' "$bold" "$off"
sleep 0.7
for step in "one capture reserved" "one run directory bound" "one proof directory bound"; do
  printf '  %s->%s %s\n' "$dim" "$off" "$step"
  sleep 0.6
done
printf '  %sthis clears no gate and proves only that the recorder runs%s\n' "$green" "$off"
python3 - "$BULLET_RECORD_RUN_JSON" "$BULLET_RECORD_RUN_ID" <<'PY'
import hashlib
import json
import sys

path, run_id = sys.argv[1], sys.argv[2]


def identifier(prefix, label):
    return prefix + hashlib.sha256(f"record-tui self-test {run_id} {label}".encode()).hexdigest()


with open(path, "w", encoding="utf-8") as stream:
    json.dump({
        "schema": "bullet.record-run.v1",
        "run_id": run_id,
        "command_id": identifier("cmd_", "command"),
        "attempt_id": identifier("atm_", "attempt"),
        "candidate_id": None,
        "receipt": {"algorithm": "sha256",
                    "digest": identifier("", "receipt")},
        "provider": {"name": "self-test", "runtime_version": "0.0.0"},
        "claims": ["self-test"],
    }, stream, indent=2, sort_keys=True)
    stream.write("\n")
PY
sleep 0.5
NARRATION
  chmod 700 -- "$TUI_WORK/selftest-narration.sh"
  TUI_NARRATION="$TUI_WORK/selftest-narration.sh"
}

# --- entry -------------------------------------------------------------------
tui_check_arguments
tui_tools
tui_check_out_dir
tui_layout
((TUI_SELF_TEST == 0)) || tui_self_test_narration
tui_record
tui_read_run_json
tui_render
tui_canvas
tui_manifest
tui_say "checking through scripts/readme-real-check.sh"
bash "$TUI_HUB/scripts/readme-real-check.sh" "$TUI_OUT" "$TUI_NAME"
tui_say "recorded $TUI_NAME (gif, manifest, cast in the output directory; capture and render receipt retained)"
