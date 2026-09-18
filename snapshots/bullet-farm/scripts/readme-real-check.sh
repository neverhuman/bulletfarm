#!/usr/bin/env bash
# Stage-two validator for REAL recordings (schema bullet.real-media.v1).
#
# Stage one (scripts/readme-check.sh) admits only synthetic VHS media: exactly
# 1200x675, 12 fps, <= 3 MiB, byte-identical re-render. A real capture can never
# pass it. This gate validates a real 1920x1080 recording plus its manifest and
# retained master, and refuses with one typed line per defect:
#
#   REAL_MEDIA_<CODE>: <value rejected>
#
# A terminal renderer cannot land on 1920x1080 by itself (agg's cell grid only
# reaches multiples of its cell box), so the published GIF is the native render
# centred on the target canvas by rewriting the GIF logical screen; the manifest
# declares that expansion and this gate re-derives it from the frame descriptors.
#
# It does not establish product completion, provider qualification, or release
# authority. See scripts/media/README.md for the schema and honesty rules.
set -euo pipefail
shopt -s inherit_errexit

REAL_HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
REAL_SELF="$(realpath -e -- "${BASH_SOURCE[0]}")"
REAL_MAX_GIF_BYTES=50000000
REAL_TMP=""

real_die() { printf 'REAL_MEDIA_%s: %s\n' "$1" "$2" >&2; exit 1; }

real_start() {
  local tool
  for tool in cp ffmpeg ffprobe gitleaks jq mktemp python3 realpath sha256sum stat truncate; do
    command -v "$tool" >/dev/null 2>&1 || real_die TOOL_UNAVAILABLE "$tool"
  done
  REAL_FFPROBE="$(command -v ffprobe)"
  REAL_PYTHON="$(command -v python3)"
  [[ -f "$REAL_HUB/scripts/lib/demo-gif-render.py" ]] || real_die TOOL_UNAVAILABLE demo-gif-render.py
  REAL_TMP="$(mktemp -d)"
  chmod 700 "$REAL_TMP"
}

real_cleanup() { [[ -z "$REAL_TMP" ]] || rm -rf -- "$REAL_TMP"; }

# Structural validation. Reuses the tracked renderer's strict JSON parser, safe
# file reader, sha256 and GIF header parser; the GIF block walker below mirrors
# demo-gif-render.py gif_delays with the portal-only 64-frame cap lifted.
real_validate() {
  # -B: importing the tracked renderer must not drop a __pycache__ into the
  # source tree this gate is supposed to leave alone.
  "$REAL_PYTHON" -IB - "$1" "$2" "$REAL_FFPROBE" "$REAL_HUB" "$REAL_MAX_GIF_BYTES" <<'PY'
import importlib.util
import json
import math
import re
import struct
import subprocess
import sys
from datetime import datetime
from pathlib import Path

root, name, ffprobe, hub, max_gif = sys.argv[1:6]
root, max_gif = Path(root), int(max_gif)
WIDTH, HEIGHT = 1920, 1080
MEMBERS = ["bullet-farm", "bullet-git", "bullet-kernel", "bullet-portal"]
CLAIMS = ["self-test", "real-recording", "real-provider-turn", "local-observation",
          "transaction-offline-bridge", "candidate-preserved", "known-defects-shown", "no-gate-cleared"]
MANDATORY = ["local-observation", "no-gate-cleared"]
FORBIDDEN = ["TRANSACTION_PROOF", "independent", "LIVE_PROOF", "release", "self-hosted-v1"]
HEX64 = re.compile(r"^[0-9a-f]{64}$")
HEX40 = re.compile(r"^[0-9a-f]{40}$")
ANSI = re.compile(r"\x1b(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))")
TOKEN = re.compile(r"(?<![A-Za-z0-9_])(?:boot|csrf|wrk)_[A-Za-z0-9_-]{8,}")
HEX64_ANY = re.compile(r"(?<![0-9A-Fa-f])[0-9A-Fa-f]{64}(?![0-9A-Fa-f])")
REDACTION = {  # scripts/readme-live-check.sh live_text pattern, split by name, plus /tmp/
    "home-path": r"/home/", "users-path": r"/Users/", "tmp-path": r"/tmp/",
    "authorization-header": r"Authorization:", "bearer": r"Bearer[ \t]",
    "private-key": r"BEGIN [A-Z ]*PRIVATE KEY", "sk-token": r"sk-[A-Za-z0-9_-]{16,}",
    "github-token": r"gh[pousr]_[A-Za-z0-9]{20,}", "github-pat": r"github_pat_[A-Za-z0-9_]{20,}",
    "aws-key": r"(?:AKIA|ASIA)[0-9A-Z]{16}", "slack-token": r"xox[baprs]-",
    "email": r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}",
}


def refuse(code, value):
    print(f"REAL_MEDIA_{code}: {value}", file=sys.stderr)
    sys.exit(1)


spec = importlib.util.spec_from_file_location("demo_gif_render", Path(hub) / "scripts/lib/demo-gif-render.py")
dgr = importlib.util.module_from_spec(spec)
spec.loader.exec_module(dgr)


def safe_read(path, limit=dgr.MAX_FILE):
    if not path.exists() and not path.is_symlink():
        refuse("MISSING_FILE", path.name)
    try:
        return dgr.read(path, limit)
    except (ValueError, OSError) as error:
        refuse("UNSAFE_FILE", f"{path.name}:{error}")


def strict_json(data, code):
    try:
        return dgr.js(data)
    except (ValueError, RecursionError) as error:
        refuse(code, str(error)[:80])


def keys(obj, where, expected):
    if not isinstance(obj, dict):
        refuse("MANIFEST_FIELD", f"object:{where}")
    missing = sorted(set(expected) - set(obj))
    extra = sorted(set(obj) - set(expected))
    if missing:
        refuse("MANIFEST_FIELD", f"missing:{where}.{missing[0]}")
    if extra:
        refuse("MANIFEST_FIELD", f"unexpected:{where}.{extra[0]}")


def field(obj, where, check, code="MANIFEST_FIELD"):
    if not isinstance(obj, dict) or where.rsplit(".", 1)[-1] not in obj:
        refuse("MANIFEST_FIELD", f"missing:{where}")
    value = obj[where.rsplit(".", 1)[-1]]
    if not check(value):
        refuse(code, f"{where}={json.dumps(value)[:96]}")
    return value


def is_str(pattern=None, limit=128):
    return lambda v: isinstance(v, str) and 0 < len(v) <= limit and (pattern is None or re.fullmatch(pattern, v))


def rfc3339(value):
    if not isinstance(value, str) or not re.fullmatch(r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z", value):
        return False
    try:
        return datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ").strftime("%Y-%m-%dT%H:%M:%SZ") == value
    except ValueError:
        return False


def gif_delays(data):
    """GIF block walk (demo-gif-render.py gif_delays; frame cap raised for terminal recordings).

    Also returns every image descriptor rectangle, which is what proves the
    declared canvas expansion instead of taking the manifest's word for it.
    """
    pos, pending, delays, frames = 0, None, [], []

    def take(size):
        nonlocal pos
        if pos + size > len(data):
            refuse("GIF_FORMAT", "GIF_BLOCK_TRUNCATED")
        part, pos = data[pos:pos + size], pos + size
        return part

    def blocks():
        size = take(1)[0]
        while size:
            take(size)
            size = take(1)[0]

    header = take(13)
    if header[:6] not in [b"GIF87a", b"GIF89a"]:
        refuse("GIF_FORMAT", "GIF_REQUIRED")
    if header[10] & 128:
        take(3 * 2 ** ((header[10] & 7) + 1))
    while True:
        marker = take(1)[0]
        if marker == 0x21:
            label = take(1)[0]
            if label == 0xF9:
                control = take(6)
                if pending is not None or control[0] != 4 or control[-1] != 0:
                    refuse("GIF_FORMAT", "GIF_GCE_AMBIGUOUS")
                pending = struct.unpack("<H", control[2:4])[0]
            elif label in [0xFE, 0xFF]:
                blocks()
            else:
                refuse("GIF_FORMAT", f"GIF_EXTENSION_{label:02x}")
        elif marker == 0x2C:
            descriptor = take(9)
            if descriptor[8] & 128:
                take(3 * 2 ** ((descriptor[8] & 7) + 1))
            take(1)
            blocks()
            if pending is None or len(delays) >= 100000:
                refuse("GIF_FORMAT", "GIF_GCE_MISSING_OR_FRAME_LIMIT")
            delays.append(pending)
            frames.append(struct.unpack("<HHHH", descriptor[:8]))
            pending = None
        else:
            if marker != 0x3B or pos != len(data) or pending is not None or not delays:
                refuse("GIF_FORMAT", "GIF_TRAILER_INVALID")
            return delays, frames


def probe(*entries, path):
    command = [ffprobe, "-v", "error", "-select_streams", "v:0", *entries, "-of", "csv=p=0", str(path)]
    try:
        done = subprocess.run(command, capture_output=True, text=True, timeout=300, check=False)
    except (OSError, subprocess.SubprocessError) as error:
        refuse("FFPROBE", type(error).__name__)
    if done.returncode != 0:
        refuse("FFPROBE", done.stderr.strip()[:96] or f"exit {done.returncode}")
    return [line for line in done.stdout.splitlines() if line.strip()]


def scan_text(text, label, declared, cast=False):
    for pattern_name, pattern in REDACTION.items():
        match = re.search(pattern, text, re.I)
        if match:
            refuse("REDACTION_REQUIRED", f"{pattern_name}@{label}:{match.group(0)[:4]}…")
    if not cast:
        return
    match = TOKEN.search(text)
    if match:
        refuse("TOKEN_IN_CAST", f"{match.group(0)[:9]}…@{label}")
    for match in HEX64_ANY.finditer(text):
        if match.group(0).lower() not in declared:
            refuse("TOKEN_IN_CAST", f"64-hex:{match.group(0)[:8]}…@{label}")


# --- files -------------------------------------------------------------------
gif_path = root / f"{name}.gif"
manifest_path = root / f"{name}.manifest.json"
for path in (gif_path, manifest_path):
    if not path.exists() and not path.is_symlink():
        refuse("MISSING_FILE", path.name)
gif_bytes = gif_path.lstat().st_size
if gif_bytes >= max_gif:
    refuse("SIZE", f"{gif_bytes} >= {max_gif}; strictly below required")
gif_data = safe_read(gif_path)
manifest_data = safe_read(manifest_path, 1024 * 1024)
manifest_text = manifest_data.decode("utf-8", "replace")
manifest = strict_json(manifest_data, "MANIFEST_JSON")
if not isinstance(manifest, dict):
    refuse("MANIFEST_JSON", "object required")

# --- manifest ----------------------------------------------------------------
field(manifest, "schema", lambda v: v == "bullet.real-media.v1", "SCHEMA")
kind = field(manifest, "kind", lambda v: v in ["terminal", "portal"], "KIND")
top = ["schema", "kind", "name", "recorded_at", "host", "recorder", "render_tools", "binaries", "members",
       "run", "receipt", "provider", "account", "master", "gif", "canvas", "capture_sha256",
       "render_receipt_sha256", "claims", "release_eligible", kind]
keys(manifest, "manifest", top)
field(manifest, "name", lambda v: v == name and re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]{0,63}", v), "NAME")
field(manifest, "recorded_at", rfc3339, "RECORDED_AT")
field(manifest, "host", is_str(r"[A-Za-z0-9][A-Za-z0-9.-]{0,62}"), "HOST")
keys(manifest["recorder"], "recorder", ["path", "sha256"])
field(manifest["recorder"], "recorder.path", is_str(r"scripts/[A-Za-z0-9._/-]+", 256))
field(manifest["recorder"], "recorder.sha256", is_str(HEX64.pattern))
keys(manifest["render_tools"], "render_tools", ["agg", "ffmpeg", "python"])
for tool_name in ["agg", "ffmpeg", "python"]:
    tool = manifest["render_tools"][tool_name]
    if tool is None and tool_name == "agg" and kind == "portal":
        continue
    keys(tool, f"render_tools.{tool_name}", ["sha256", "version"])
    field(tool, f"render_tools.{tool_name}.sha256", is_str(HEX64.pattern))
    field(tool, f"render_tools.{tool_name}.version", is_str(r"[A-Za-z0-9._+-]{1,64}"))
keys(manifest["members"], "members", MEMBERS)
for member in MEMBERS:
    entry = manifest["members"][member]
    keys(entry, f"members.{member}", ["oid", "dirty"])
    field(entry, f"members.{member}.oid", is_str(HEX40.pattern), "MEMBER")
    field(entry, f"members.{member}.dirty", lambda v: isinstance(v, bool), "MEMBER")
keys(manifest["run"], "run", ["command_id", "attempt_id", "candidate_id"])
field(manifest["run"], "run.command_id", is_str(r"cmd_[0-9a-f]{64}"), "RUN_IDENTITY")
field(manifest["run"], "run.attempt_id", is_str(r"atm_[0-9a-f]{64}"), "RUN_IDENTITY")
field(manifest["run"], "run.candidate_id", lambda v: v is None or is_str(r"can_[0-9a-f]{64}")(v), "RUN_IDENTITY")
keys(manifest["receipt"], "receipt", ["algorithm", "digest"])
field(manifest["receipt"], "receipt.algorithm", lambda v: v in ["blake3", "sha256"], "RECEIPT")
field(manifest["receipt"], "receipt.digest", is_str(HEX64.pattern), "RECEIPT")
keys(manifest["provider"], "provider", ["name", "runtime_version"])
field(manifest["provider"], "provider.name", is_str(r"[a-z][a-z0-9-]{0,31}"), "PROVIDER")
field(manifest["provider"], "provider.runtime_version",
      is_str(r"[0-9]+(?:\.[0-9]+){0,3}(?:[-+][A-Za-z0-9.-]{1,32})?", 64), "PROVIDER")
field(manifest, "account", lambda v: v == "REDACTED", "ACCOUNT")
if re.search(REDACTION["email"], manifest_text):
    refuse("ACCOUNT", "email present in manifest")
keys(manifest["gif"], "gif", ["sha256", "bytes"])
canvas = manifest["canvas"]
keys(canvas, "canvas", ["method", "native_width", "native_height", "offset_x", "offset_y", "background_rgb"])
field(canvas, "canvas.method", lambda v: v in ["NONE", "GIF_LOGICAL_SCREEN_EXPANSION"], "CANVAS")
field(canvas, "canvas.native_width", lambda v: type(v) is int and 0 < v <= WIDTH, "CANVAS")
field(canvas, "canvas.native_height", lambda v: type(v) is int and 0 < v <= HEIGHT, "CANVAS")
field(canvas, "canvas.offset_x", lambda v: type(v) is int and v >= 0, "CANVAS")
field(canvas, "canvas.offset_y", lambda v: type(v) is int and v >= 0, "CANVAS")
field(canvas, "canvas.background_rgb", lambda v: v is None or is_str(r"[0-9a-f]{6}")(v), "CANVAS")
native = canvas["native_width"], canvas["native_height"]
if (canvas["offset_x"], canvas["offset_y"]) != ((WIDTH - native[0]) // 2, (HEIGHT - native[1]) // 2):
    refuse("CANVAS", f"offset {canvas['offset_x']},{canvas['offset_y']} does not centre {native[0]}x{native[1]}")
if (canvas["method"] == "NONE") != (native == (WIDTH, HEIGHT)):
    refuse("CANVAS", f"method {canvas['method']} with native {native[0]}x{native[1]}")
if (canvas["method"] == "NONE") != (canvas["background_rgb"] is None):
    refuse("CANVAS", f"method {canvas['method']} with background {canvas['background_rgb']}")
field(manifest, "capture_sha256", is_str(HEX64.pattern))
field(manifest, "render_receipt_sha256", is_str(HEX64.pattern))
field(manifest, "release_eligible", lambda v: v is False, "AUTHORITY")
claims = field(manifest, "claims", lambda v: isinstance(v, list) and 0 < len(v) <= len(CLAIMS)
               and all(isinstance(c, str) for c in v), "CLAIMS")
for claim in claims:
    for needle in FORBIDDEN:
        if needle.lower() in claim.lower():
            refuse("FORBIDDEN_CLAIM", f"{needle} in {claim[:40]}")
for claim in claims:
    if claim not in CLAIMS:
        refuse("UNKNOWN_CLAIM", claim[:64])
if len(set(claims)) != len(claims):
    refuse("CLAIMS", "duplicate")
if "self-test" in claims and claims != ["self-test"]:
    refuse("CLAIMS", "self-test must be the only claim")
if "self-test" not in claims:
    for claim in MANDATORY:
        if claim not in claims:
            refuse("CLAIMS", f"missing:{claim}")
# A self-test recording runs no product binary and must not name one; every
# other recording must name the exact binaries the recorded session executed.
if claims == ["self-test"]:
    if manifest["binaries"] is not None:
        refuse("BINARIES", "a self-test recording must declare binaries null")
else:
    if not isinstance(manifest["binaries"], dict):
        refuse("BINARIES", f"binaries={json.dumps(manifest['binaries'])[:32]} outside a self-test recording")
    keys(manifest["binaries"], "binaries", ["bullet", "bullet-farmd"])
    for binary in ["bullet", "bullet-farmd"]:
        keys(manifest["binaries"][binary], f"binaries.{binary}", ["sha256"])
        field(manifest["binaries"][binary], f"binaries.{binary}.sha256", is_str(HEX64.pattern), "BINARIES")
if kind == "terminal":
    keys(manifest["master"], "master", ["cast_sha256"])
    field(manifest["master"], "master.cast_sha256", is_str(HEX64.pattern), "MASTER")
    term = manifest["terminal"]
    keys(term, "terminal", ["cols", "rows", "font_family", "font_size", "line_height", "theme", "renderer"])
    field(term, "terminal.cols", lambda v: isinstance(v, int) and 1 <= v <= 240)
    field(term, "terminal.rows", lambda v: isinstance(v, int) and 1 <= v <= 100)
    field(term, "terminal.font_family", is_str(r"[A-Za-z0-9 ._-]{1,64}"))
    field(term, "terminal.font_size", lambda v: isinstance(v, int) and 6 <= v <= 96)
    field(term, "terminal.line_height", lambda v: type(v) in [int, float] and math.isfinite(v) and 0.5 <= v <= 3)
    field(term, "terminal.theme", is_str(r"[a-z0-9-]{1,32}"))
    field(term, "terminal.renderer", lambda v: v in ["fontdue", "resvg"])
    # agg lays out (rows + 1) cell boxes of font_size * line_height and (cols + 2)
    # advance widths, so the declared grid has to re-derive the native height and
    # imply a believable monospace advance. See scripts/media/README.md.
    if round((term["rows"] + 1) * term["font_size"] * term["line_height"]) != native[1]:
        refuse("GEOMETRY_MATH", f"(rows+1)*font_size*line_height != native height {native[1]}")
    advance = native[0] / ((term["cols"] + 2) * term["font_size"])
    if not 0.3 <= advance <= 1.2:
        refuse("GEOMETRY_MATH", f"cell advance {advance:.4f} of font_size is not monospace-plausible")
else:
    keys(manifest["master"], "master", ["frames_framemd5_sha256", "ffv1_sha256"])
    field(manifest["master"], "master.frames_framemd5_sha256", is_str(HEX64.pattern), "MASTER")
    field(manifest["master"], "master.ffv1_sha256", is_str(HEX64.pattern), "MASTER")
    keys(manifest["portal"], "portal", ["frames"])
    field(manifest["portal"], "portal.frames", lambda v: isinstance(v, int) and 1 <= v <= 64)

# --- GIF ---------------------------------------------------------------------
if manifest["gif"]["sha256"] != dgr.sha(gif_data) or manifest["gif"]["bytes"] != gif_bytes:
    refuse("GIF_DRIFT", f"{dgr.sha(gif_data)[:16]}…/{gif_bytes}")
try:
    header_geometry = dgr.gif_geometry(gif_path)
except ValueError as error:
    refuse("GIF_FORMAT", str(error))
probed = probe("-show_entries", "stream=width,height", path=gif_path)
if len(probed) != 1 or header_geometry != (WIDTH, HEIGHT) or probed[0] != f"{WIDTH},{HEIGHT}":
    refuse("GEOMETRY", "x".join(str(v) for v in header_geometry))
delays, descriptors = gif_delays(gif_data)
box = (canvas["offset_x"], canvas["offset_y"], *native)
if descriptors[0] != box:
    refuse("CANVAS", f"first frame {descriptors[0]} is not the declared native area {box}")
for index, (left, upper, wide, high) in enumerate(descriptors):
    if left < box[0] or upper < box[1] or left + wide > box[0] + box[2] or upper + high > box[1] + box[3]:
        refuse("CANVAS", f"frame {index} at {left},{upper} {wide}x{high} leaves the declared native area")
pts = []
for line in probe("-show_entries", "frame=pts_time", path=gif_path):
    try:
        pts.append(float(line.split(",")[0]))
    except ValueError:
        refuse("FRAMES", f"pts:{line[:32]}")
if len(pts) != len(delays):
    refuse("FRAMES", f"native={len(delays)} decoded={len(pts)}")
for index, delay in enumerate(delays):
    if delay < 2:
        refuse("FRAME_DELAY", f"frame {index} delay {delay}cs")
for index in range(1, len(pts)):
    if not pts[index] > pts[index - 1]:
        refuse("FRAME_TIMING", f"frame {index} pts {pts[index]} <= {pts[index - 1]}")
duration = sum(delays) / 100
if not 0 < duration <= 900:
    refuse("DURATION", f"{duration}s")
if kind == "portal" and manifest["portal"]["frames"] != len(delays):
    refuse("FRAMES", f"portal.frames={manifest['portal']['frames']} native={len(delays)}")

# --- master and text screening ---------------------------------------------
declared = {m.group(0).lower() for m in HEX64_ANY.finditer(manifest_text)}
scan_text(manifest_text, f"{name}.manifest.json", declared)
if kind == "terminal":
    cast_path = root / f"{name}.cast"
    cast_data = safe_read(cast_path, 16 * 1024 ** 2)
    if dgr.sha(cast_data) != manifest["master"]["cast_sha256"]:
        refuse("MASTER_DRIFT", f"{name}.cast {dgr.sha(cast_data)[:16]}…")
    try:
        cast_text = cast_data.decode("utf-8")
    except UnicodeDecodeError as error:
        refuse("CAST_UTF8", f"offset {error.start}")
    lines = cast_text.splitlines()
    if not 1 < len(lines) <= 100001:
        refuse("CAST_EVENT", f"lines={len(lines)}")
    header = strict_json(lines[0].encode(), "CAST_HEADER")
    if not (isinstance(header, dict) and header.get("version") == 2
            and header.get("width") == term["cols"] and header.get("height") == term["rows"]):
        refuse("CAST_GEOMETRY", f"{header.get('width')}x{header.get('height')} vs {term['cols']}x{term['rows']}")
    prior, plain = 0, []
    for number, line in enumerate(lines[1:], 2):
        event = strict_json(line.encode(), "CAST_EVENT")
        if not (isinstance(event, list) and len(event) == 3 and type(event[0]) in [int, float]
                and math.isfinite(event[0]) and prior <= event[0] <= 36000 and event[1] in ["o", "i"]
                and isinstance(event[2], str)):
            refuse("CAST_EVENT", f"line {number}")
        prior = event[0]
        plain.append(event[2])
    blob = "".join(plain)
    stripped = ANSI.sub("", blob)
    printable = sum(1 for ch in stripped if ch.isprintable())
    if printable < 500:
        refuse("CAST_ENTROPY", f"printable={printable}<500")
    if not re.search(r"\x1b\[[0-9]*;[0-9]*H", blob):
        refuse("CAST_ENTROPY", "zero cursor-position CSI")
    if not re.search(r"\x1b\[[0-9;]*[34]8;2;", blob):
        refuse("CAST_ENTROPY", "zero 38;2/48;2 SGR")
    scan_text(stripped, f"{name}.cast", declared, cast=True)
    scan_text(cast_text, f"{name}.cast(raw)", declared, cast=True)
else:
    framemd5_path = root / f"{name}.frames.framemd5"
    framemd5 = safe_read(framemd5_path, 4 * 1024 ** 2)
    if dgr.sha(framemd5) != manifest["master"]["frames_framemd5_sha256"]:
        refuse("MASTER_DRIFT", f"{name}.frames.framemd5 {dgr.sha(framemd5)[:16]}…")
    try:
        framemd5_text = framemd5.decode("ascii")
    except UnicodeDecodeError as error:
        refuse("MASTER", f"framemd5 non-ascii at {error.start}")
    if len([line for line in framemd5_text.splitlines() if line and not line.startswith("#")]) != len(delays):
        refuse("MASTER", "framemd5 row count differs from GIF frame count")
    scan_text(framemd5_text, f"{name}.frames.framemd5", declared, cast=True)
    master_path = root / f"{name}.master.nut"
    if master_path.exists() or master_path.is_symlink():
        if dgr.sha(safe_read(master_path)) != manifest["master"]["ffv1_sha256"]:
            refuse("MASTER_DRIFT", f"{name}.master.nut")
print(f"kind={kind} frames={len(delays)} duration={duration}s bytes={gif_bytes} "
      f"manifest_sha256={dgr.sha(manifest_data)}")
PY
}

# Secret scan with gitleaks over private copies (a hostile .gitleaks.toml or
# .gitleaksignore inside the media directory cannot influence the scan).
real_gitleaks() {
  local root="$1" name="$2" scan="$REAL_TMP/gitleaks-scan" report="$REAL_TMP/gitleaks.json" suffix code=0 rules
  mkdir -m 700 "$scan"
  for suffix in manifest.json cast frames.framemd5; do
    [[ ! -f "$root/$name.$suffix" ]] || cp -- "$root/$name.$suffix" "$scan/"
  done
  gitleaks detect --no-git --no-banner --redact --exit-code 9 -s "$scan" -i "$scan" \
    -f json -r "$report" >/dev/null 2>"$REAL_TMP/gitleaks.err" || code=$?
  if [[ "$code" == 9 ]]; then
    rules="$(jq -r '[.[].RuleID] | unique | join(",")' "$report" 2>/dev/null || echo unknown-rule)"
    real_die SECRET_FINDING "gitleaks:$rules"
  fi
  [[ "$code" == 0 ]] || real_die SCANNER_FAILED "gitleaks exit $code"
}

real_check() {
  local root="$1" name="$2" summary
  [[ "$root" == /* && -d "$root" && ! -L "$root" && "$root" == "$(realpath -e -- "$root")" ]] ||
    real_die INPUT "$root"
  [[ "$name" =~ ^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$ ]] || real_die NAME "$name"
  summary="$(real_validate "$root" "$name")"
  real_gitleaks "$root" "$name"
  printf 'readme-real-check: PASS %s %s\n' "$name" "$summary"
}

# --- self-test ---------------------------------------------------------------
real_fixture_gif() {  # path geometry
  ffmpeg -nostdin -hide_banner -loglevel error -y -f lavfi -i "color=c=0x0d1117:size=$2:rate=2" \
    -frames:v 2 "$1"
}

real_refresh() {  # dir name: rebind gif and cast hashes after a byte mutation
  local dir="$1" name="$2" gif_sha cast_sha bytes
  gif_sha="$(sha256sum -- "$dir/$name.gif")"; gif_sha="${gif_sha%% *}"
  cast_sha="$(sha256sum -- "$dir/$name.cast")"; cast_sha="${cast_sha%% *}"
  bytes="$(stat -c '%s' -- "$dir/$name.gif")"
  jq -S --arg gif "$gif_sha" --arg cast "$cast_sha" --argjson bytes "$bytes" \
    '.gif = {sha256: $gif, bytes: $bytes} | .master.cast_sha256 = $cast' \
    "$dir/$name.manifest.json" >"$dir/$name.manifest.json.new"
  mv -- "$dir/$name.manifest.json.new" "$dir/$name.manifest.json"
}

real_fixture() {  # dir name
  local dir="$1" name="$2" zero oid hex1 hex2 hex3
  zero="$(printf '%064d' 0)"
  oid="$(printf '%040d' 1)"
  hex1="$(printf 'readme-real-check self-test command' | sha256sum)"; hex1="${hex1%% *}"
  hex2="$(printf 'readme-real-check self-test attempt' | sha256sum)"; hex2="${hex2%% *}"
  hex3="$(printf 'readme-real-check self-test receipt' | sha256sum)"; hex3="${hex3%% *}"
  mkdir -m 700 "$dir"
  real_fixture_gif "$dir/$name.gif" 1920x1080
  printf '%s\n' \
    '{"version": 2, "width": 126, "height": 29, "timestamp": 1788960494, "title": "self-test"}' \
    '[0.01, "o", "\u001b[2J\u001b[1;1H\u001b[38;2;255;196;77m\u001b[48;2;8;16;31mreadme-real-check self-test fixture\u001b[0m\r\n"]' \
    '[0.5, "o", "HOLD-honest dark board  pad-for-entropy-gate  xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\r\n"]' \
    '[0.8, "o", "done\r\n"]' >"$dir/$name.cast"
  jq -nS --arg name "$name" --arg zero "$zero" --arg oid "$oid" \
    --arg cmd "cmd_$hex1" --arg atm "atm_$hex2" --arg receipt "$hex3" '{
      schema: "bullet.real-media.v1", kind: "terminal", name: $name,
      recorded_at: "2026-09-10T00:00:00Z", host: "selftest-host",
      recorder: {path: "scripts/media/record-tui.sh", sha256: $zero},
      render_tools: {agg: {sha256: $zero, version: "1.5.0"}, ffmpeg: {sha256: $zero, version: "6.1.1"},
        python: {sha256: $zero, version: "3.12.3"}},
      binaries: null,
      members: {"bullet-farm": {oid: $oid, dirty: false}, "bullet-git": {oid: $oid, dirty: false},
        "bullet-kernel": {oid: $oid, dirty: false}, "bullet-portal": {oid: $oid, dirty: false}},
      run: {command_id: $cmd, attempt_id: $atm, candidate_id: null},
      receipt: {algorithm: "sha256", digest: $receipt},
      provider: {name: "self-test", runtime_version: "0.0.0"}, account: "REDACTED",
      master: {cast_sha256: $zero}, gif: {sha256: $zero, bytes: 0},
      canvas: {method: "NONE", native_width: 1920, native_height: 1080,
        offset_x: 0, offset_y: 0, background_rgb: null},
      capture_sha256: $zero, render_receipt_sha256: $zero,
      terminal: {cols: 126, rows: 29, font_family: "Liberation Mono", font_size: 25, line_height: 1.44,
        theme: "github-dark", renderer: "fontdue"},
      claims: ["self-test"], release_eligible: false
    }' >"$dir/$name.manifest.json"
  real_refresh "$dir" "$name"
}

real_case() {  # fresh mutable copy of the fixture
  rm -rf -- "$REAL_CASE"
  cp -a -- "$REAL_FIXTURE" "$REAL_CASE"
}

real_manifest_edit() {  # jq program applied to the case manifest
  jq -S "$1" "$REAL_CASE/$REAL_NAME.manifest.json" >"$REAL_CASE/$REAL_NAME.manifest.json.new"
  mv -- "$REAL_CASE/$REAL_NAME.manifest.json.new" "$REAL_CASE/$REAL_NAME.manifest.json"
}

real_cast_append() {  # text appended as one more output event, hashes rebound
  printf '[0.9, "o", %s]\n' "$(jq -cn --arg t "$1" '$t')" >>"$REAL_CASE/$REAL_NAME.cast"
  real_refresh "$REAL_CASE" "$REAL_NAME"
}

real_expect_refusal() {  # code label
  local code="$1" label="$2" out
  if out="$(bash "$REAL_SELF" "$REAL_CASE" "$REAL_NAME" 2>&1)"; then
    printf 'readme-real-check: self-test "%s" was accepted instead of refused\n' "$label" >&2
    exit 1
  fi
  grep -q "^REAL_MEDIA_$code:" <<<"$out" || {
    printf 'readme-real-check: self-test "%s" expected REAL_MEDIA_%s, got: %s\n' "$label" "$code" "$out" >&2
    exit 1
  }
  printf '  refused  %-26s %s\n' "$label" "$(head -n 1 <<<"$out")"
  REAL_REFUSALS=$((REAL_REFUSALS + 1))
}

real_expect_pass() {  # label
  local out
  out="$(bash "$REAL_SELF" "$REAL_CASE" "$REAL_NAME" 2>&1)" || {
    printf 'readme-real-check: self-test "%s" was refused: %s\n' "$1" "$out" >&2
    exit 1
  }
  printf '  accepted %-26s %s\n' "$1" "$out"
  REAL_PASSES=$((REAL_PASSES + 1))
}

real_self_test() {
  local rnd
  REAL_NAME=selftest
  REAL_FIXTURE="$REAL_TMP/fixture"
  REAL_CASE="$REAL_TMP/case"
  REAL_PASSES=0
  REAL_REFUSALS=0
  real_fixture "$REAL_FIXTURE" "$REAL_NAME"
  echo 'readme-real-check: self-test (synthetic 1920x1080 two-frame GIF; proves the gate, not any product)'
  real_case
  real_expect_pass 'valid fixture'
  real_case; real_cast_append "declared id: $(jq -r '.run.command_id' "$REAL_CASE/$REAL_NAME.manifest.json")"
  real_expect_pass 'declared 64-hex in cast'

  real_case; real_fixture_gif "$REAL_CASE/$REAL_NAME.gif" 1280x720; real_refresh "$REAL_CASE" "$REAL_NAME"
  real_expect_refusal GEOMETRY 'wrong geometry 1280x720'
  real_case; truncate -s "$REAL_MAX_GIF_BYTES" "$REAL_CASE/$REAL_NAME.gif"
  real_expect_refusal SIZE 'gif exactly at byte limit'
  real_case; truncate -s $((REAL_MAX_GIF_BYTES + 1)) "$REAL_CASE/$REAL_NAME.gif"
  real_expect_refusal SIZE 'oversize gif'
  real_case; real_manifest_edit 'del(.host)'
  real_expect_refusal MANIFEST_FIELD 'missing field host'
  real_case; real_manifest_edit '.notes = "free text"'
  real_expect_refusal MANIFEST_FIELD 'unexpected field'
  real_case; real_manifest_edit '.schema = "bullet.real-media.v0"'
  real_expect_refusal SCHEMA 'wrong schema'
  real_case; real_manifest_edit '.kind = "webcam"'
  real_expect_refusal KIND 'wrong kind'
  real_case; real_manifest_edit '.name = "other"'
  real_expect_refusal NAME 'name mismatch'
  real_case; real_manifest_edit '.recorded_at = "2026-09-10 00:00:00"'
  real_expect_refusal RECORDED_AT 'non-RFC3339 recorded_at'
  real_case; real_manifest_edit '.run.command_id = "cmd_short"'
  real_expect_refusal RUN_IDENTITY 'malformed command id'
  real_case; real_manifest_edit '.receipt.algorithm = "md5"'
  real_expect_refusal RECEIPT 'unsupported receipt digest'
  real_case; real_manifest_edit '.provider.runtime_version = "latest"'
  real_expect_refusal PROVIDER 'non-numeric runtime version'
  real_case; real_manifest_edit '.account = "operator@example.com"'
  real_expect_refusal ACCOUNT 'email as account label'
  real_case; real_manifest_edit 'del(.members["bullet-git"])'
  real_expect_refusal MANIFEST_FIELD 'missing member repo'
  real_case; real_manifest_edit '.members["bullet-kernel"].oid = "HEAD"'
  real_expect_refusal MEMBER 'non-oid member commit'
  real_case; real_manifest_edit '.release_eligible = true'
  real_expect_refusal AUTHORITY 'release_eligible true'
  real_case; real_manifest_edit '.claims += ["TRANSACTION_PROOF"]'
  real_expect_refusal FORBIDDEN_CLAIM 'forbidden claim (proof)'
  real_case; real_manifest_edit '.claims = ["local-observation", "no-gate-cleared", "Independent Evidence"]'
  real_expect_refusal FORBIDDEN_CLAIM 'forbidden claim (independent)'
  real_case; real_manifest_edit '.claims = ["local-observation", "no-gate-cleared", "verified-by-operator"]'
  real_expect_refusal UNKNOWN_CLAIM 'claim outside allowlist'
  real_case; real_manifest_edit '.claims = ["real-recording"]'
  real_expect_refusal CLAIMS 'mandatory claims absent'
  real_case; real_manifest_edit '.claims = ["self-test", "real-recording"]'
  real_expect_refusal CLAIMS 'self-test mixed with real'
  real_case; real_manifest_edit '.claims = ["local-observation", "no-gate-cleared"]'
  real_expect_refusal BINARIES 'real claims without binaries'
  real_case; real_manifest_edit '.binaries = {bullet: {sha256: ("0" * 64)}, "bullet-farmd": {sha256: ("0" * 64)}}'
  real_expect_refusal BINARIES 'self-test naming product binaries'
  real_case; real_manifest_edit '.terminal.rows = 40'
  real_expect_refusal GEOMETRY_MATH 'declared grid does not fit the canvas'
  real_case; real_manifest_edit '.terminal.cols = 20 | .terminal.font_size = 25'
  real_expect_refusal GEOMETRY_MATH 'implausible cell advance'
  real_case; real_manifest_edit '.canvas.native_width = 1900'
  real_expect_refusal CANVAS 'canvas offset not centred'
  real_case; real_manifest_edit '.canvas = {method: "GIF_LOGICAL_SCREEN_EXPANSION", native_width: 1900,
    native_height: 1044, offset_x: 10, offset_y: 18, background_rgb: "eceff4"} | .terminal.rows = 28'
  real_expect_refusal CANVAS 'canvas geometry not in the GIF'
  real_case; real_manifest_edit '.gif.sha256 = ("f" * 64)'
  real_expect_refusal GIF_DRIFT 'gif hash drift'
  real_case; printf '[1.0, "o", "late\\r\\n"]\n' >>"$REAL_CASE/$REAL_NAME.cast"
  real_expect_refusal MASTER_DRIFT 'cast hash drift'
  real_case; rm -- "$REAL_CASE/$REAL_NAME.cast"
  real_expect_refusal MISSING_FILE 'missing cast'
  real_case; mv -- "$REAL_CASE/$REAL_NAME.gif" "$REAL_CASE/real.gif"; ln -s real.gif "$REAL_CASE/$REAL_NAME.gif"
  real_expect_refusal UNSAFE_FILE 'symlinked gif'
  real_case; real_manifest_edit '.terminal.cols = 80'
  real_expect_refusal CAST_GEOMETRY 'cast/manifest grid mismatch'
  real_case
  printf '%s\n' \
    '{"version": 2, "width": 126, "height": 29, "timestamp": 1788960494, "title": "self-test"}' \
    '[0.3, "o", "\u001b[m\u001b[m\u001b[0m\u001b[?25l"]' \
    '[0.4, "o", "ok\r\n"]' >"$REAL_CASE/$REAL_NAME.cast"
  real_refresh "$REAL_CASE" "$REAL_NAME"
  real_expect_refusal CAST_ENTROPY 'low-entropy white-tape cast'
  rnd="$(printf 'readme-real-check self-test token' | sha256sum)"; rnd="${rnd%% *}"
  real_case; real_cast_append "boot_${rnd:0:32}"
  real_expect_refusal TOKEN_IN_CAST 'boot_ token in cast'
  real_case; real_cast_append "csrf_${rnd:0:32}"
  real_expect_refusal TOKEN_IN_CAST 'csrf_ token in cast'
  real_case; real_cast_append "nonce $rnd"
  real_expect_refusal TOKEN_IN_CAST 'undeclared 64-hex in cast'
  real_case; real_cast_append 'mail operator@example.com'
  real_expect_refusal REDACTION_REQUIRED 'email in cast'
  real_case; real_cast_append 'path /home/operator/private'
  real_expect_refusal REDACTION_REQUIRED 'home path in cast'
  real_case; real_cast_append "glpat-$(printf '%s' "$rnd" | tr '0-9' 'A-J' | head -c 20)"
  real_expect_refusal SECRET_FINDING 'gitleaks-only finding'
  real_case
  "$REAL_PYTHON" -IB - "$REAL_CASE/$REAL_NAME.gif" <<'PY'
import sys
path = sys.argv[1]
data = bytearray(open(path, "rb").read())
pos = 13 + (3 * 2 ** ((data[10] & 7) + 1) if data[10] & 128 else 0)
while data[pos] == 0x21 and data[pos + 1] != 0xF9:  # skip application/comment extensions
    pos += 2
    while data[pos]:
        pos += data[pos] + 1
    pos += 1
assert data[pos] == 0x21 and data[pos + 1] == 0xF9
data[pos + 4:pos + 6] = b"\x00\x00"  # first frame delay 0cs
open(path, "wb").write(data)
PY
  real_refresh "$REAL_CASE" "$REAL_NAME"
  real_expect_refusal FRAME_DELAY 'zero frame delay'
  printf 'readme-real-check: SELF-TEST PASS (%s accepted, %s typed refusals)\n' "$REAL_PASSES" "$REAL_REFUSALS"
}

# --- entry -------------------------------------------------------------------
if [[ "$#" == 1 && "$1" == --self-test ]]; then
  real_start
  trap real_cleanup EXIT
  real_self_test
  exit 0
fi
if [[ "$#" == 1 && "$1" == /* ]]; then
  set -- "$1" "$(find "$1" -maxdepth 1 -name '*.manifest.json' -printf '%f\n' 2>/dev/null | sed 's/\.manifest\.json$//')"
fi
[[ "$#" == 2 && "$1" == /* && "$2" != */* && -n "$2" ]] || {
  echo 'usage: readme-real-check.sh ABSOLUTE_MEDIA_DIRECTORY [NAME] | --self-test' >&2
  exit 2
}
real_start
trap real_cleanup EXIT
real_check "$1" "$2"
