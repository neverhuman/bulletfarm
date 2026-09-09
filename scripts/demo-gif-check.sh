#!/usr/bin/env bash
# Admit committed HQ demo GIFs: geometry, brightness, redaction, no blank first frames.
set -euo pipefail
export PATH="$HOME/.local/bin:$PATH"

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
MEDIA="${DEMO_GIF_ROOT:-$HUB/docs/demo-gif}"

for tool in ffmpeg python3; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'demo-gif-check: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

python3 - "$MEDIA" <<'PY'
import re
import subprocess
import sys
from pathlib import Path

root = Path(sys.argv[1])
email = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
secret = re.compile(r"(boot_|wrk_|sk-|ghp_|gho_)[A-Za-z0-9_-]{12,}")
errors: list[str] = []

def probe(path: Path) -> dict[str, str]:
    out = subprocess.check_output(
        [
            "ffprobe", "-v", "error", "-select_streams", "v:0",
            "-show_entries", "stream=width,height,nb_frames",
            "-of", "default=noprint_wrappers=1",
            str(path),
        ],
        text=True,
    )
    values = {}
    for line in out.splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            values[key] = value
    return values

def gray_bytes(path: Path, position: str) -> bytes:
    raw = subprocess.check_output(
        [
            "ffmpeg", "-hide_banner", "-loglevel", "error",
            "-i", str(path), "-vf",
            f"{position},scale=160:90:flags=area,format=gray",
            "-frames:v", "1", "-f", "rawvideo", "-",
        ]
    )
    if not raw:
        raise RuntimeError(f"empty frame from {path}")
    return raw

def mean_luma(path: Path, position: str) -> float:
    raw = gray_bytes(path, position)
    return sum(raw) / len(raw)

def frame_stddev(path: Path, position: str) -> float:
    raw = gray_bytes(path, position)
    mean = sum(raw) / len(raw)
    return (sum((value - mean) ** 2 for value in raw) / len(raw)) ** 0.5

for name, min_w, min_h in (
    ("claude-tui", 1400, 700),
    ("codex-tui", 1400, 700),
    ("cursor-tui", 1400, 700),
    ("portal-form", 1600, 900),
):
    gif = root / name / f"{name}.gif"
    if not gif.is_file():
        errors.append(f"missing {gif}")
        continue
    info = probe(gif)
    width = int(info.get("width", "0"))
    height = int(info.get("height", "0"))
    if width < min_w or height < min_h:
        errors.append(f"{gif.name} is {width}x{height}, need >={min_w}x{min_h}")
    frames = int(info.get("nb_frames", "0") or "0")
    late_n = max(frames - 2, 1)
    late = mean_luma(gif, f"select=eq(n\\,{late_n})")
    contrast = frame_stddev(gif, f"select=eq(n\\,{late_n})")
    if contrast < 8:
        errors.append(f"{gif.name} last frame has too little contrast ({contrast:.1f})")
    if late < 12:
        errors.append(f"{gif.name} last frame is too dark ({late:.1f})")

    transcript = root / name / "transcript.txt"
    if transcript.is_file():
        text = transcript.read_text(encoding="utf-8")
        if email.search(text) or secret.search(text):
            errors.append(f"{transcript} still contains a secret or email")
        if name != "portal-form" and "bullet-farm" not in text:
            errors.append(f"{transcript} does not name bullet-farm")

    cast = root / name / "session.cast"
    if cast.is_file() and secret.search(cast.read_text(encoding="utf-8", errors="ignore")):
        errors.append(f"{cast} still contains a secret")

if errors:
    print("demo-gif-check: FAILED", file=sys.stderr)
    for item in errors:
        print(f"  {item}", file=sys.stderr)
    raise SystemExit(1)
print("demo-gif-check: ok")
PY
