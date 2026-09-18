#!/usr/bin/env bash
# Real Portal recorder: Playwright frames -> 1920x1080 GIF -> bullet.real-media.v1.
# Never prints credentials. Output dir is absolute 0700 under $HOME, never the
# family tree and never /tmp.
set -euo pipefail
shopt -s inherit_errexit
umask 077

PORTAL_MEDIA="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
PORTAL_HUB="$(cd -- "$PORTAL_MEDIA/../.." && pwd -P)"
PORTAL_FAMILY="$(cd -- "$PORTAL_HUB/.." && pwd -P)"
PORTAL_SELF="$PORTAL_MEDIA/record-portal.sh"
PORTAL_WIDTH=1920
PORTAL_HEIGHT=1080
PORTAL_MEMBERS=(bullet-farm bullet-git bullet-kernel bullet-portal)

die() { printf 'RECORD_PORTAL_%s: %s\n' "$1" "$2" >&2; exit 1; }
sha() { local line; line="$(sha256sum -- "$1")" || die HASH "$1"; printf '%s' "${line%% *}"; }

OUT=""
NAME="portal-operator-console"
ORIGIN="http://127.0.0.1:7420"
SESSION_FILE="${XDG_STATE_HOME:-$HOME/.local/state}/bullet/operator/session.json"
BULLET_BIN=""
FARMD_BIN=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --out-dir) OUT="$2"; shift 2 ;;
    --run-id) NAME="$2"; shift 2 ;;
    --origin) ORIGIN="$2"; shift 2 ;;
    --session-file) SESSION_FILE="$2"; shift 2 ;;
    --bullet-bin) BULLET_BIN="$2"; shift 2 ;;
    --farmd-bin) FARMD_BIN="$2"; shift 2 ;;
    *) die ARGUMENT "$1" ;;
  esac
done

[[ -n "$OUT" && "$OUT" == /* && ! -e "$OUT" ]] || die OUT_DIR "absolute absent directory required"
case "$OUT" in
  /tmp|/*/tmp/*|"$PORTAL_FAMILY"|"$PORTAL_FAMILY"/*) die OUT_DIR "must sit under home, not family or tmp" ;;
esac
[[ "$OUT" == "$HOME"/* ]] || die OUT_DIR "must sit under home"
[[ -n "$BULLET_BIN" && -x "$BULLET_BIN" && -n "$FARMD_BIN" && -x "$FARMD_BIN" ]] || die BINARIES "clean bullet and farmd required"
[[ -f "$SESSION_FILE" && ! -L "$SESSION_FILE" ]] || die SESSION "operator session file required"
[[ "$NAME" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$ ]] || die NAME "$NAME"

mkdir -m 700 -- "$OUT"
FRAMES="$OUT/frames"
mkdir -m 700 -- "$FRAMES"
export BULLET_RECORD_PORTAL_ORIGIN="$ORIGIN"
export BULLET_RECORD_SESSION_FILE="$SESSION_FILE"
export BULLET_RECORD_FRAME_DIR="$FRAMES"
export BULLET_RECORD_PORTAL_ROOT="$PORTAL_FAMILY/bullet-portal"

NODE="${NODE:-}"
if [[ -z "$NODE" ]]; then
  NODE="$(command -v node)"
fi
[[ -x "$NODE" ]] || die TOOL_UNAVAILABLE node
PLAYWRIGHT="$PORTAL_FAMILY/bullet-portal/node_modules/playwright"
[[ -d "$PLAYWRIGHT" ]] || die TOOL_UNAVAILABLE playwright

( cd "$PORTAL_FAMILY/bullet-portal" && "$NODE" "$PORTAL_MEDIA/narrate-operator-portal.mjs" ) \
  || die NARRATION "portal walk failed"

[[ -f "$FRAMES/observation.json" && -f "$FRAMES/frames.json" ]] || die NARRATION "observation missing"

FFMPEG="$(command -v ffmpeg)"
FFPROBE="$(command -v ffprobe)"
PYTHON="$(command -v python3)"
[[ -x "$FFMPEG" && -x "$FFPROBE" && -x "$PYTHON" ]] || die TOOL_UNAVAILABLE ffmpeg

# High-contrast GIF, no dither, exact 1920x1080, delays from frames.json (>=2cs).
mapfile -t HOLDS < <(jq -r '.[].hold_cs' "$FRAMES/frames.json")
mapfile -t FILES < <(jq -r '.[].file' "$FRAMES/frames.json")
((${#FILES[@]} >= 1 && ${#FILES[@]} <= 64)) || die FRAMES "need 1..64 frames"
LIST="$FRAMES/frames.ffconcat"
{
  printf 'ffconcat version 1.0\n'
  idx=0
  for file in "${FILES[@]}"; do
    hold="${HOLDS[$idx]}"
    ((hold >= 2)) || die FRAME_DELAY "$hold"
    printf "file '%s'\n" "$file"
    awk -v cs="$hold" 'BEGIN { printf "duration %.2f\n", cs/100 }'
    idx=$((idx + 1))
  done
  printf "file '%s'\n" "${FILES[-1]}"
} >"$LIST"

( cd "$FRAMES" && "$FFMPEG" -nostdin -hide_banner -loglevel error -y -f concat -safe 0 -i frames.ffconcat \
  -vf "scale=${PORTAL_WIDTH}:${PORTAL_HEIGHT}:flags=neighbor,split[s0][s1];[s0]palettegen=max_colors=64:stats_mode=full[p];[s1][p]paletteuse=dither=none" \
  "$OUT/$NAME.gif" ) || die RENDER gif

"$FFMPEG" -nostdin -hide_banner -loglevel error -y -i "$OUT/$NAME.gif" \
  -f framemd5 "$OUT/$NAME.frames.framemd5" || die RENDER framemd5
"$FFMPEG" -nostdin -hide_banner -loglevel error -y -i "$OUT/$NAME.gif" \
  -c:v ffv1 -an "$OUT/$NAME.master.nut" || die RENDER ffv1

HOST="$(uname -n)"; HOST="${HOST%%.*}"
members_json() {
  local member directory oid dirty
  for member in "${PORTAL_MEMBERS[@]}"; do
    directory="$PORTAL_FAMILY/$member"
    oid="$(GIT_OPTIONAL_LOCKS=0 git -C "$directory" rev-parse HEAD)"
    if [[ -n "$(GIT_OPTIONAL_LOCKS=0 git -C "$directory" status --porcelain)" ]]; then
      dirty=true
    else
      dirty=false
    fi
    printf '%s\t%s\t%s\n' "$member" "$oid" "$dirty"
  done | jq -R -s -c 'split("\n") | map(select(length > 0) | split("\t"))
    | map({key: .[0], value: {oid: .[1], dirty: (.[2] == "true")}}) | from_entries'
}

GIF_FRAMES="$("$FFPROBE" -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of csv=p=0 -- "$OUT/$NAME.gif")"
[[ "$GIF_FRAMES" =~ ^[0-9]+$ ]] || die FRAMES "ffprobe frame count"
export GIF_FRAMES

python3 - "$OUT" "$NAME" "$PORTAL_SELF" "$BULLET_BIN" "$FARMD_BIN" "$HOST" \
  "$(members_json)" "$FFMPEG" "$PYTHON" <<'PY'
import hashlib, json, os, subprocess, sys
from datetime import datetime, timezone
from pathlib import Path

out, name, recorder, bullet, farmd, host, members, ffmpeg, python = sys.argv[1:10]
out = Path(out)

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

def version(cmd, arg="--version"):
    text = subprocess.check_output([cmd, arg], text=True, stderr=subprocess.STDOUT)
    return text.split()[1] if text.split() else "0.0.0"

run_id = name

def ident(prefix, label):
    return prefix + hashlib.sha256(f"bullet.operator-console {run_id} {label}".encode()).hexdigest()

gif = out / f"{name}.gif"
obs = out / "frames" / "observation.json"
receipt = {
    "schema": "bullet.portal-render.v1",
    "name": name,
    "frames": json.loads((out / "frames" / "frames.json").read_text()),
}
receipt_path = out / "render-receipt.json"
receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
run = {
    "schema": "bullet.record-run.v1",
    "run_id": run_id,
    "command_id": ident("cmd_", "command"),
    "attempt_id": ident("atm_", "attempt"),
    "candidate_id": None,
    "receipt": {"algorithm": "sha256", "digest": ident("", "receipt")},
    "provider": {"name": "none", "runtime_version": "0.0.0"},
    "claims": ["local-observation", "no-gate-cleared", "real-recording"],
}
(out / "run.json").write_text(json.dumps(run, indent=2, sort_keys=True) + "\n")
manifest = {
    "schema": "bullet.real-media.v1",
    "kind": "portal",
    "name": name,
    "recorded_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "host": host,
    "recorder": {"path": "scripts/media/record-portal.sh", "sha256": sha(recorder)},
    "render_tools": {
        "agg": None,
        "ffmpeg": {"sha256": sha(ffmpeg), "version": version(ffmpeg, "-version")},
        "python": {"sha256": sha(python), "version": f"{sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}"},
    },
    "binaries": {"bullet": {"sha256": sha(bullet)}, "bullet-farmd": {"sha256": sha(farmd)}},
    "members": json.loads(members),
    "run": {"command_id": run["command_id"], "attempt_id": run["attempt_id"], "candidate_id": None},
    "receipt": run["receipt"],
    "provider": run["provider"],
    "account": "REDACTED",
    "master": {
        "frames_framemd5_sha256": sha(out / f"{name}.frames.framemd5"),
        "ffv1_sha256": sha(out / f"{name}.master.nut"),
    },
    "gif": {"sha256": sha(gif), "bytes": gif.stat().st_size},
    "canvas": {
        "method": "NONE",
        "native_width": 1920,
        "native_height": 1080,
        "offset_x": 0,
        "offset_y": 0,
        "background_rgb": None,
    },
    "capture_sha256": sha(obs),
    "render_receipt_sha256": sha(receipt_path),
    "portal": {"frames": int(os.environ["GIF_FRAMES"])},
    "claims": run["claims"],
    "release_eligible": False,
}
(out / f"{name}.manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
print(f"wrote {name}.gif bytes={manifest['gif']['bytes']}")
PY

# Copy observation next to the published pair using a home-free name.
cp -- "$FRAMES/observation.json" "$OUT/$NAME.observation.json"
bash "$PORTAL_HUB/scripts/readme-real-check.sh" "$OUT" "$NAME"
printf 'record-portal: PASS %s\n' "$NAME"
