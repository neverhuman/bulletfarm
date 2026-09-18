#!/usr/bin/env python3
"""Private screenshot-sequence/cast derivatives; never public or release evidence."""

import argparse
import hashlib
from fractions import Fraction
import json
import math
import os
from pathlib import Path
import re
import resource
import signal
import stat
import struct
import subprocess
import sys
import time

MAX_FILE, MAX_TOTAL, MAX_PIXELS = 64 * 1024**2, 256 * 1024**2, 4096 * 2160
SCHEMA = "bullet.private-render.v1"


def need(ok, reason):
    if not ok:
        raise ValueError(reason)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path, limit=MAX_FILE):
    need(path.is_absolute() and path.resolve(strict=True) == path, "NONCANONICAL_FILE")
    info = path.lstat()
    need(stat.S_ISREG(info.st_mode) and info.st_nlink == 1 and info.st_size <= limit, "UNSAFE_FILE")
    return path.read_bytes()


def js(data):
    def pairs(items):
        result = {}
        for key, value in items:
            need(key not in result, "DUPLICATE_JSON_KEY")
            result[key] = value
        return result
    return json.loads(data, object_pairs_hook=pairs,
                      parse_constant=lambda _: need(False, "NONFINITE_JSON"))


def private(path):
    need(path.is_absolute() and path.resolve(strict=True) == path, "NONCANONICAL_DIRECTORY")
    info = path.stat()
    need(stat.S_ISDIR(info.st_mode) and info.st_uid == os.getuid() and info.st_mode & 0o077 == 0,
         "PRIVATE_DIRECTORY_REQUIRED")


def inventory(root):
    private(root)
    found = {}
    for directory, dirs, files in os.walk(root, followlinks=False):
        for name in dirs:
            private(Path(directory) / name)
        for name in files:
            path = Path(directory) / name
            data = read(path)
            need(path.stat().st_uid == os.getuid() and path.stat().st_mode & 0o077 == 0, "PRIVATE_FILE_REQUIRED")
            found[str(path.relative_to(root))] = {"sha256": sha(data), "bytes": len(data)}
            need(len(found) <= 512 and sum(v["bytes"] for v in found.values()) <= MAX_TOTAL, "ARTIFACT_LIMIT")
    return found


def put(path, data):
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def put_json(path, value):
    put(path, (json.dumps(value, sort_keys=True, indent=2) + "\n").encode())


def implementation():
    scripts = Path(__file__).resolve().parent.parent
    return {str(p): sha(read(p)) for p in [scripts / "demo-gif-render.sh", scripts / "demo-gif-check.sh",
                                          Path(__file__).resolve()]}


def tool(path, expected):
    need(path is not None and expected is not None and re.fullmatch("[a-f0-9]{64}", expected), "TOOL_HASH_REQUIRED")
    need(os.access(path, os.X_OK) and sha(read(path)) == expected, "TOOL_HASH_MISMATCH")
    return {"path": str(path), "sha256": expected}


def fonts():
    # Observed subset only: agg always loads system fonts and fallbacks.
    result, total = {}, 0
    for root in [Path("/usr/share/fonts"), Path("/usr/local/share/fonts"), Path("/etc/fonts"),
                 Path.home() / ".fonts", Path.home() / ".local/share/fonts", Path.home() / ".config/fontconfig"]:
        result[str(root)] = {"present": root.exists()}
        if root.is_dir():
            for path in sorted(root.rglob("*")):
                if path.is_file():
                    total += path.stat().st_size
                    need(len(result) < 4096 and path.stat().st_size <= MAX_FILE and total <= MAX_TOTAL,
                         "FONT_INVENTORY_LIMIT")
                    result[str(path)] = {"resolved": str(path.resolve()), "sha256": sha(path.read_bytes())}
    return result


class Runner:
    def __init__(self, root, seconds):
        self.root, self.deadline, self.commands = root, time.monotonic() + seconds, []
        (root / "logs").mkdir(mode=0o700)

    def run(self, command):
        i = len(self.commands)
        out, err = [self.root / "logs" / f"{i:03}.{suffix}" for suffix in ["stdout", "stderr"]]
        def limits():
            resource.setrlimit(resource.RLIMIT_FSIZE, (MAX_FILE, MAX_FILE))
            resource.setrlimit(resource.RLIMIT_AS, (2 * 1024**3, 2 * 1024**3))
        need(time.monotonic() < self.deadline, "RENDER_DEADLINE")
        with out.open("xb") as stdout, err.open("xb") as stderr:
            child = subprocess.Popen(command, cwd=self.root, stdin=subprocess.DEVNULL,
                                     stdout=stdout, stderr=stderr, start_new_session=True, preexec_fn=limits,
                                     env={**os.environ, "RAYON_NUM_THREADS": "2"})
            timed_out, gone = False, False
            try:
                # Observe exit without reaping: the leader reserves this group identity.
                while os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOHANG | os.WNOWAIT) is None:
                    if time.monotonic() >= self.deadline:
                        timed_out = True
                        break
                    time.sleep(0.01)
            finally:
                try:
                    os.killpg(child.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                code = child.wait(timeout=2)
                # Never signal after reap. A remaining/reused group only causes refusal.
                try:
                    os.killpg(child.pid, 0)
                except ProcessLookupError:
                    gone = True
                self.commands.append({"argv": command, "exit": code, "timed_out": timed_out,
                                      "owned_group_gone": gone, "stdout": str(out.relative_to(self.root)),
                                      "stderr": str(err.relative_to(self.root)),
                                      "environment_overrides": {"RAYON_NUM_THREADS": "2"}})
        inventory(self.root)
        need(gone, "TOOL_GROUP_TEARDOWN_UNVERIFIED")
        need(not timed_out and code == 0, "TOOL_TIMEOUT" if timed_out else "TOOL_FAILED")
        return read(out)

    def ff(self, executable, *args):
        return self.run([str(executable), "-nostdin", "-hide_banner", "-loglevel", "error", "-threads", "1",
                         "-filter_threads", "1", "-filter_complex_threads", "1", *args])


def decoded(run, ffmpeg, path, dimensions, timing=None):
    data = run.ff(ffmpeg, "-copyts", "-protocol_whitelist", "file,pipe", "-i", str(path), "-map", "0:v:0",
                  "-fps_mode", "passthrough", "-c:v", "rawvideo", "-pix_fmt", "rgb24",
                  "-enc_time_base", "-1", "-f", "framehash", "-hash", "sha256", "-")
    frames, timebase, geometry = [], None, None
    for line in data.decode().splitlines():
        if line.startswith("#tb 0: "):
            timebase = line.removeprefix("#tb 0: ")
        if line.startswith("#dimensions 0: "):
            geometry = tuple(int(part) for part in line.removeprefix("#dimensions 0: ").split("x"))
        if not line.startswith("#"):
            need(geometry == dimensions, "DECODED_GEOMETRY_MISMATCH")
            fields = [item.strip() for item in line.split(",")]
            need(len(fields) == 6 and int(fields[4]) == dimensions[0] * dimensions[1] * 3
                 and re.fullmatch("[a-f0-9]{64}", fields[5]), "DECODED_FRAME_INVALID")
            frames.append(fields[5])
            if timing is not None:
                need(timebase is not None and re.fullmatch(r"[1-9][0-9]*/[1-9][0-9]*", timebase), "TIMEBASE_INVALID")
                timing.append({"pts": int(fields[2]), "timebase": timebase})
    need(0 < len(frames) <= 100000, "DECODED_FRAME_COUNT")
    return frames


def gif_delays(data):
    """Walk GIF blocks, never search compressed image payload for control bytes."""
    pos, pending, delays = 0, None, []
    def take(size):
        nonlocal pos
        need(pos + size <= len(data), "GIF_BLOCK_TRUNCATED")
        part, pos = data[pos:pos + size], pos + size
        return part
    def blocks():
        size = take(1)[0]
        while size:
            take(size)
            size = take(1)[0]
    header = take(13)
    need(header[:6] in [b"GIF87a", b"GIF89a"], "GIF_REQUIRED")
    if header[10] & 128:
        take(3 * 2 ** ((header[10] & 7) + 1))
    while True:
        marker = take(1)[0]
        if marker == 0x21:
            label = take(1)[0]
            if label == 0xf9:
                control = take(6)
                need(pending is None and control[0] == 4 and control[-1] == 0, "GIF_GCE_AMBIGUOUS")
                pending = struct.unpack("<H", control[2:4])[0]
            else:
                need(label in [0xfe, 0xff], "GIF_RENDERING_EXTENSION_UNSUPPORTED")
                blocks()
        elif marker == 0x2c:
            descriptor = take(9)
            if descriptor[8] & 128:
                take(3 * 2 ** ((descriptor[8] & 7) + 1))
            take(1)  # LZW code size, followed by structurally bounded data sub-blocks.
            blocks()
            need(pending is not None and len(delays) < 64, "GIF_GCE_MISSING")
            delays.append(pending)
            pending = None
        else:
            need(marker == 0x3b and pos == len(data) and pending is None and delays, "GIF_TRAILER_INVALID")
            return delays


def portal_schedule(value, measured):
    rows = value["frames"]
    need(len(rows) == len(measured), "MASTER_TIMING_COUNT")
    times = [Fraction(row["pts"]) * Fraction(row["timebase"]) for row in measured]
    need(times[0] == 0, "MASTER_TIMING_ORIGIN")
    for i, (row, observed) in enumerate(zip(rows, times)):
        expected = (Fraction(str(row["elapsed_ms"])) - Fraction(str(rows[0]["elapsed_ms"]))) / 1000
        need(abs(observed - expected) <= Fraction(i, 2_000_000), "MASTER_SOURCE_TIMING_DRIFT")
        need(i == 0 or observed > times[i - 1], "MASTER_TIMING_ORDER")
    pts = [int(time * 100 + Fraction(1, 2)) for time in times]  # Nonnegative half-up, as FFmpeg.
    gaps = [after - before for before, after in zip(pts, pts[1:])]
    need(all(2 <= gap <= 65535 for gap in gaps), "GIF_TIMING_UNREPRESENTABLE")
    return pts, gaps + [gaps[-1] if gaps else 10]


def verify_portal_timing(value, master, gif, path):
    pts, delays = portal_schedule(value, master)
    observed = [Fraction(row["pts"]) * Fraction(row["timebase"]) for row in gif]
    need(observed == [Fraction(tick, 100) for tick in pts], "GIF_SOURCE_TIMING_DRIFT")
    need(gif_delays(read(path)) == delays, "GIF_NATIVE_DELAY_DRIFT")
    return {"gif_native_delays_cs": delays,
            "final_frame_hold": {"policy": "REPEAT_LAST_GAP_OR_SINGLE_100MS", "delay_cs": delays[-1],
                                 "source_observed": False}}


def gif_geometry(path):
    data = read(path)
    need(data[:6] in [b"GIF87a", b"GIF89a"], "GIF_REQUIRED")
    width, height = struct.unpack("<HH", data[6:10])
    need(0 < width * height <= MAX_PIXELS, "GIF_GEOMETRY")
    return width, height


def capture(root, kind, expected):
    name = "observation.json" if kind == "portal" else "session.cast.result.json"
    data = read(root / name, 4 * 1024**2)
    need(sha(data) == expected, "CAPTURE_HASH_MISMATCH")
    value, files = js(data), inventory(root)
    if kind == "terminal":
        required = {"session.cast", "session.cast.raw", "transcript.txt", name}
        need(set(files) in [required, required | {"producer.exit"}], "CAPTURE_INVENTORY")
        need(value["schema_version"] == "bullet.native-capture.v1" and value["child_started"] is True
             and value["owned_group_gone"] is True and value["recorder_exit"] == 0
             and value["stop_reason"] == "CHILD_EXIT", "INCOMPLETE_CAPTURE")
        for label, filename in [("cast", "session.cast"), ("raw", "session.cast.raw"), ("transcript", "transcript.txt")]:
            need(files[filename] == value["artifacts"][label], "CAPTURE_ARTIFACT_DRIFT")
        if "producer.exit" in files:
            need(read(root / "producer.exit") == b"0\n", "INCOMPLETE_CAPTURE")
        lines = read(root / "session.cast", 16 * 1024**2).splitlines()
        need(1 < len(lines) <= 100001, "CAST_EVENT_LIMIT")
        header = js(lines[0])
        need(header["version"] == 2 and type(header["width"]) is int and type(header["height"]) is int
             and 1 <= header["width"] <= 240 and 1 <= header["height"] <= 100, "CAST_GEOMETRY")
        prior = 0
        for line in lines[1:]:
            event = js(line)
            need(isinstance(event, list) and len(event) == 3 and type(event[0]) in [float, int]
                 and prior <= event[0] <= 600 and event[1] in ["o", "i"] and isinstance(event[2], str), "CAST_EVENT")
            prior = event[0]
    else:
        need(value["schema_version"] == "bullet.portal-capture.v1" and value["capture_status"] == "CAPTURED",
             "INCOMPLETE_CAPTURE")
        rows = value["frames"]
        need(isinstance(rows, list) and 1 <= len(rows) <= 64, "SCREENSHOT_COUNT")
        need(set(files) == {"started.json", "observation.json", *(row["file"] for row in rows)}, "CAPTURE_INVENTORY")
        prior, dimensions = -1, None
        for row in rows:
            need(re.fullmatch(r"frames/[A-Za-z0-9_-]+\.png", row["file"])
                 and type(row["elapsed_ms"]) in [int, float] and math.isfinite(row["elapsed_ms"])
                 and 0 <= row["elapsed_ms"] <= 600000 and prior < row["elapsed_ms"], "SCREENSHOT_TIMING_OR_PATH")
            image = read(root / row["file"])
            need(image[:8] == b"\x89PNG\r\n\x1a\n" and image[12:16] == b"IHDR", "PNG_REQUIRED")
            size = struct.unpack(">II", image[16:24])
            need(size == (row["width"], row["height"]) and 0 < size[0] * size[1] <= MAX_PIXELS
                 and image[24] == 8 and image[25] in [2, 6], "PNG_GEOMETRY")
            need(dimensions in [None, size] and files[row["file"]] == {"sha256": row["sha256"], "bytes": row["bytes"]},
                 "SCREENSHOT_DRIFT")
            dimensions, prior = size, row["elapsed_ms"]
    need(value["bullet_live_admission"] is False, "CAPTURE_AUTHORITY_INVALID")
    return value, files


def portal(run, ffmpeg, source, value):
    rows = value["frames"]
    dimensions = rows[0]["width"], rows[0]["height"]
    pixels = dimensions[0] * dimensions[1]
    original = []
    for row in rows:
        path = source / row["file"]
        if read(path)[25] == 6:
            rgba = run.ff(ffmpeg, "-i", str(path), "-frames:v", "1", "-pix_fmt", "rgba", "-f", "rawvideo", "-")
            need(len(rgba) == pixels * 4 and all(a == 255 for a in rgba[3::4]), "TRANSPARENCY_NOT_ADMITTED")
        original.append(decoded(run, ffmpeg, path, dimensions)[0])
    concat = "ffconcat version 1.0\n"
    for i, row in enumerate(rows):
        concat += f"file source/{row['file']}\noption framerate 1000000\n"
        if i + 1 < len(rows):
            duration = int((Fraction(str(rows[i+1]["elapsed_ms"])) - Fraction(str(row["elapsed_ms"]))) * 1000 + Fraction(1, 2))
            concat += f"duration {duration // 1000000}.{duration % 1000000:06}\n"
    put(run.root / "sequence.ffconcat", concat.encode())
    master, gif = run.root / "master.nut", run.root / "derivative.gif"
    run.ff(ffmpeg, "-n", "-f", "concat", "-safe", "0", "-protocol_whitelist", "file,pipe", "-i", "sequence.ffconcat",
           "-fps_mode", "passthrough", "-c:v", "ffv1", "-level", "3", "-pix_fmt", "bgr0",
           "-enc_time_base", "1:1000000", "-f", "nut", str(master))
    master_timing, gif_timing = [], []
    master_rgb = decoded(run, ffmpeg, master, dimensions, master_timing)
    need(master_rgb == original, "MASTER_RGB_MISMATCH")
    _, delays = portal_schedule(value, master_timing)
    run.ff(ffmpeg, "-n", "-i", str(master), "-filter_complex",
           "split[a][b];[a]palettegen=reserve_transparent=0[p];[b][p]paletteuse=dither=none",
           "-fps_mode", "passthrough", "-enc_time_base", "1:100", "-final_delay", str(delays[-1]),
           "-loop", "0", str(gif))
    gif_rgb = decoded(run, ffmpeg, gif, dimensions, gif_timing)
    need(len(gif_rgb) == len(original), "GIF_FRAME_COUNT_MISMATCH")
    timing_proof = verify_portal_timing(value, master_timing, gif_timing, gif)
    return {**timing_proof, "master_rgb": "EXACT", "gif_rgb": "EXACT" if gif_rgb == original else "QUANTIZED",
            "source_rgb_sha256": original, "master_rgb_sha256": master_rgb, "gif_rgb_sha256": gif_rgb,
            "source_elapsed_ms": [row["elapsed_ms"] for row in rows],
            "master_decoded_timing": master_timing, "gif_decoded_timing": gif_timing,
            "timing": "source-bound native PTS; master cumulative half-microsecond interval rounding/GIF centiseconds; explicit display tail; no interpolated frames"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=["render", "check"])
    parser.add_argument("--kind", choices=["portal", "terminal"], required=True)
    for name in ["input", "output", "ffmpeg", "agg"]:
        parser.add_argument("--" + name, type=Path, required=name != "agg")
    for name in ["input-sha256", "ffmpeg-sha256", "agg-sha256"]:
        parser.add_argument("--" + name, required=name != "agg-sha256")
    parser.add_argument("--timeout", type=float, default=60)
    parser.add_argument("--strict-lossless", action="store_true", help="Exact original RGB pixels; casts have no RGB master")
    parser.add_argument("--strict-font-closure", action="store_true")
    args = parser.parse_args()
    os.umask(0o077)
    need(math.isfinite(args.timeout) and 0 < args.timeout <= 120, "TIMEOUT_LIMIT")
    private(args.output.parent)
    need(args.output.is_absolute() and args.output.parent.resolve() == args.output.parent
         and args.output.name not in [".", ".."] and not args.output.is_relative_to(args.input)
         and not args.output.is_relative_to(Path(__file__).resolve().parents[3]), "OUTPUT_PATH")
    args.output.mkdir(mode=0o700)
    run = Runner(args.output, args.timeout)
    def interrupted(signum, _frame):
        raise ValueError(f"RENDER_INTERRUPTED:{signum}")
    for signum in [signal.SIGINT, signal.SIGTERM]:
        signal.signal(signum, interrupted)
    try:
        need(not args.strict_font_closure, "FONT_CLOSURE_NOT_ADMITTED")
        sources = implementation()
        selected = {"ffmpeg": tool(args.ffmpeg, args.ffmpeg_sha256),
                    "python": {"path": sys.executable, "sha256": sha(read(Path(sys.executable)))}}
        observed_fonts = None
        if args.kind == "terminal":
            need(not args.strict_lossless, "NO_ORIGINAL_TERMINAL_RGB_MASTER")
            selected["agg"] = tool(args.agg, args.agg_sha256)
            observed_fonts = fonts()
        if args.mode == "render":
            value, originals = capture(args.input, args.kind, args.input_sha256)
            source = args.output / "source"
            source.mkdir(mode=0o700)
            for name in originals:
                path = source / name
                path.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                put(path, read(args.input / name))
            need(inventory(source) == originals, "COPIED_SOURCE_DRIFT")
            if args.kind == "portal":
                fidelity = portal(run, args.ffmpeg, source, value)
                need(not args.strict_lossless or fidelity["gif_rgb"] == "EXACT", "GIF_RGB_QUANTIZED")
            else:
                gif = args.output / "derivative.gif"
                run.run([str(args.agg), "--renderer", "fontdue", "--theme", "github-light", "--speed", "1",
                         "--idle-time-limit", "600", "--last-frame-duration", "0", str(source / "session.cast"), str(gif)])
                fidelity = {"gif_rgb": "UNVERIFIED_NO_ORIGINAL_RGB_MASTER", "master_rgb": "NOT_AVAILABLE",
                            "gif_rgb_sha256": decoded(run, args.ffmpeg, gif, gif_geometry(gif)),
                            "timing": "exact cast retained; agg GIF derivative timing is not original TUI evidence"}
            need(inventory(args.input) == inventory(source) == originals, "CAPTURE_CHANGED_DURING_RENDER")
            receipt = {"schema_version": SCHEMA, "kind": args.kind, "input_capture_sha256": args.input_sha256,
                       "input_files": originals, "tools": selected, "implementation": sources, "fidelity": fidelity,
                       "input_class": "PORTAL_SCREENSHOT_SEQUENCE" if args.kind == "portal" else "TERMINAL_CAST",
                       "observed_font_inventory": observed_fonts, "closure_verified": False,
                       "provenance_scope": "SUPPLIED_LOCAL_TOOL_FILES_AND_OBSERVED_INPUTS_ONLY",
                       "product_completion": "UNVERIFIED", "public_export_review": "REQUIRED", "release_eligible": False}
        else:
            data = read(args.input / "manifest.json", 4 * 1024**2)
            need(sha(data) == args.input_sha256, "MANIFEST_HASH_MISMATCH")
            receipt = js(data)
            need(receipt["schema_version"] == SCHEMA and receipt["kind"] == args.kind
                 and receipt["implementation"] == sources and receipt["tools"] == selected
                 and receipt["observed_font_inventory"] == observed_fonts, "SOURCE_OR_TOOL_DRIFT")
            need(receipt["closure_verified"] is False and receipt["release_eligible"] is False
                 and receipt["product_completion"] == "UNVERIFIED" and receipt["public_export_review"] == "REQUIRED",
                 "RENDER_AUTHORITY_INVALID")
            actual = inventory(args.input)
            actual.pop("manifest.json")
            need(actual == receipt["files"], "GENERATION_ARTIFACT_DRIFT")
            value, originals = capture(args.input / "source", args.kind, receipt["input_capture_sha256"])
            need(originals == receipt["input_files"], "GENERATION_SOURCE_DRIFT")
            if args.kind == "portal":
                dimensions = value["frames"][0]["width"], value["frames"][0]["height"]
                original = [decoded(run, args.ffmpeg, args.input / "source" / row["file"], dimensions)[0] for row in value["frames"]]
                master_timing, gif_timing = [], []
                master = decoded(run, args.ffmpeg, args.input / "master.nut", dimensions, master_timing)
                gif = decoded(run, args.ffmpeg, args.input / "derivative.gif", dimensions, gif_timing)
                need(receipt["fidelity"]["source_elapsed_ms"] == [row["elapsed_ms"] for row in value["frames"]]
                     and receipt["fidelity"]["master_decoded_timing"] == master_timing
                     and receipt["fidelity"]["gif_decoded_timing"] == gif_timing, "TIMING_RECEIPT_DRIFT")
                need(original == master == receipt["fidelity"]["source_rgb_sha256"] == receipt["fidelity"]["master_rgb_sha256"]
                     and gif == receipt["fidelity"]["gif_rgb_sha256"] and receipt["fidelity"]["master_rgb"] == "EXACT", "RGB_RECEIPT_DRIFT")
                timing_proof = verify_portal_timing(value, master_timing, gif_timing, args.input / "derivative.gif")
                need(all(receipt["fidelity"][key] == val for key, val in timing_proof.items()), "TIMING_POLICY_DRIFT")
                label = "EXACT" if gif == original else "QUANTIZED"
                need(receipt["fidelity"]["gif_rgb"] == label and (not args.strict_lossless or label == "EXACT"), "GIF_RGB_QUANTIZED")
            else:
                gif = args.input / "derivative.gif"
                need(receipt["fidelity"]["master_rgb"] == "NOT_AVAILABLE"
                     and receipt["fidelity"]["gif_rgb"] == "UNVERIFIED_NO_ORIGINAL_RGB_MASTER"
                     and decoded(run, args.ffmpeg, gif, gif_geometry(gif)) == receipt["fidelity"]["gif_rgb_sha256"], "TERMINAL_DERIVATIVE_DRIFT")
            need(inventory(args.input) == {**actual, "manifest.json": {"sha256": sha(data), "bytes": len(data)}}, "GENERATION_CHANGED")
            receipt["verified_generation_sha256"] = args.input_sha256
        need(implementation() == sources, "SOURCE_CHANGED")
        for item in selected.values():
            need(sha(read(Path(item["path"]))) == item["sha256"], "TOOL_CHANGED")
        need(observed_fonts is None or fonts() == observed_fonts, "OBSERVED_FONTS_CHANGED")
        receipt.update(commands=run.commands, files=inventory(args.output))
        put_json(args.output / ("manifest.json" if args.mode == "render" else "verification.json"), receipt)
        print("PRIVATE_RENDER_VERIFIED; PUBLIC_EXPORT_REVIEW_REQUIRED")
        return 0
    except (OSError, ValueError, KeyError, TypeError, IndexError, struct.error, subprocess.SubprocessError) as error:
        put_json(args.output / "failure.json", {"status": "FAILED", "reason": str(error), "commands": run.commands,
                                                "release_eligible": False, "public_export_review": "REQUIRED"})
        print(f"demo-gif: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
