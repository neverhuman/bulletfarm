#!/usr/bin/env bash
# Render high-resolution demo GIFs from recorded casts and Portal video.
# No provider credentials are used. agg is fetched into a cache directory.
set -euo pipefail
export PATH="$HOME/.local/bin:$PATH"

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
MEDIA="${DEMO_GIF_ROOT:-$HUB/docs/demo-gif}"
CACHE="${BULLET_DEMO_GIF_CACHE:-$HOME/.cache/bullet-demo-gif}"
AGG_BIN="${DEMO_GIF_AGG:-$CACHE/bin/agg}"
THEME_FILE="$MEDIA/theme.json"
FONT_DIR="/usr/share/fonts/truetype/dejavu"
THEME="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["agg"])' "$THEME_FILE")"

for tool in ffmpeg python3 curl; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'demo-gif-render: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

install_agg() {
  mkdir -p "$CACHE/bin"
  if [[ -x "$AGG_BIN" ]]; then
    return 0
  fi
  curl -fsSL -o "$AGG_BIN" \
    https://github.com/asciinema/agg/releases/download/v1.5.0/agg-x86_64-unknown-linux-gnu
  chmod +x "$AGG_BIN"
}

palette_gif() {
  local input="$1"
  local output="$2"
  local width="$3"
  local fps="$4"
  local work
  work="$(mktemp -d)"
  ffmpeg -y -hide_banner -loglevel error -i "$input" \
    -vf "fps=${fps},scale=${width}:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=256:stats_mode=full[p];[s1][p]paletteuse=dither=none:diff_mode=rectangle" \
    "$work/full.gif"
  if command -v gifsicle >/dev/null 2>&1; then
    gifsicle -O3 --no-extensions --no-warnings -o "$output" "$work/full.gif"
  else
    mv "$work/full.gif" "$output"
  fi
  rm -rf "$work"
}

install_agg
[[ -x "$AGG_BIN" ]] || {
  echo "demo-gif-render: agg binary is missing" >&2
  exit 1
}
[[ -n "$THEME" && -f "$THEME_FILE" ]] || {
  echo "demo-gif-render: theme.json is missing or has no agg palette" >&2
  exit 1
}

render_cast() {
  local name="$1"
  local dest="$MEDIA/$name"
  [[ -f "$dest/session.cast" ]] || {
    printf 'demo-gif-render: missing %s/session.cast\n' "$name" >&2
    exit 1
  }
  "$AGG_BIN" \
    --font-dir "$FONT_DIR" \
    --font-family "DejaVu Sans Mono" \
    --font-size 20 \
    --line-height 1.35 \
    --fps-cap 15 \
    --idle-time-limit 0.7 \
    --last-frame-duration 2.4 \
    --renderer fontdue \
    --theme "$THEME" \
    "$dest/session.cast" \
    "$dest/${name}.gif"
    last_n="$(ffprobe -v error -select_streams v:0 -show_entries stream=nb_frames \
      -of default=noprint_wrappers=1:nokey=1 "$dest/${name}.gif")"
    last_n=$((last_n - 1))
    ffmpeg -y -hide_banner -loglevel error -i "$dest/${name}.gif" \
      -vf "select=eq(n\\,${last_n})" -vsync 0 -frames:v 1 "$dest/fallback.png"
}

for demo in claude-tui codex-tui cursor-tui; do
  echo "demo-gif-render: $demo"
  render_cast "$demo"
done

echo "demo-gif-render: portal-form"
[[ -f "$MEDIA/portal-form/portal-form.mp4" ]] || {
  echo "demo-gif-render: missing portal-form.mp4" >&2
  exit 1
}
palette_gif "$MEDIA/portal-form/portal-form.mp4" "$MEDIA/portal-form/portal-form.gif" 1920 12
cp "$MEDIA/portal-form/frames/08-control-tower-return.png" "$MEDIA/portal-form/fallback.png"

python3 - "$MEDIA" <<'PY'
import hashlib
import json
from pathlib import Path
root = Path(__import__("sys").argv[1])
manifest = {"schema_version": "bullet.demo-gif.v1", "demos": {}}
for name in ("claude-tui", "codex-tui", "cursor-tui", "portal-form"):
    gif = root / name / f"{name}.gif"
    data = gif.read_bytes()
    manifest["demos"][name] = {
        "gif": str(gif.relative_to(root)),
        "bytes": len(data),
        "sha256": hashlib.sha256(data).hexdigest(),
    }
(root / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
print("demo-gif-render: wrote", root / "manifest.json")
PY
