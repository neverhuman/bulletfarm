#!/usr/bin/env python3
"""Bounded private terminal capture. Native output is not product proof."""

from __future__ import annotations

import argparse
import codecs
import errno
import fcntl
import hashlib
import json
import math
import os
import pty
import re
import select
import signal
import struct
import sys
import termios
import time
from pathlib import Path


ANSI = re.compile(r"\x1b(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))")


def redact(text: str) -> str:
    """Limited display redaction; public export still requires review."""
    text = re.sub(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", "[redacted-email]", text)
    text = re.sub(r"[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}", "[redacted-id]", text, flags=re.I)
    return re.sub(r"(boot_|wrk_)[0-9a-f]{32,}", r"\1[redacted]", text)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cast", required=True)
    parser.add_argument("--transcript", required=True)
    parser.add_argument("--cols", type=int, default=140)
    parser.add_argument("--rows", type=int, default=38)
    parser.add_argument("--inject-delay", type=float, default=0.0)
    parser.add_argument("--inject", action="append", default=[])
    parser.add_argument("--inject-gap", type=float, default=0.12)
    parser.add_argument("--idle-quit", type=float, default=0.0)
    parser.add_argument("--min-after-inject", type=float, default=0.0)
    parser.add_argument("--min-runtime", type=float, default=0.0)
    parser.add_argument("--max-seconds", type=float, default=120.0,
                        help="capture-loop limit; teardown adds at most 1.4s of polling; filesystem I/O is separate")
    parser.add_argument("--max-bytes", type=int, default=16 * 1024 * 1024)
    parser.add_argument("--max-events", type=int, default=100000)
    parser.add_argument("--title", default="")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    times = [args.inject_delay, args.inject_gap, args.idle_quit,
             args.min_after_inject, args.min_runtime, args.max_seconds]
    if (len(args.command) < 2 or args.command[0] != "--"
            or any(not math.isfinite(x) or x < 0 for x in times)
            or not 0 < args.max_seconds <= 3600
            or not 0 < args.max_bytes <= 64 * 1024 * 1024
            or not 0 < args.max_events <= 1000000
            or not 0 < args.rows <= 4096 or not 0 < args.cols <= 4096):
        parser.error("invalid command or finite capture bounds")
    return args


def reserve_files(args: argparse.Namespace) -> dict:
    paths = {"cast": Path(args.cast), "transcript": Path(args.transcript),
             "raw": Path(args.cast + ".raw"), "result": Path(args.cast + ".result.json")}
    if len(set(paths.values())) != len(paths):
        raise ValueError("capture paths must be distinct")
    files = {}
    try:
        for name, path in paths.items():
            parent = path.parent
            if (not path.is_absolute() or parent.resolve(strict=True) != parent
                    or parent.stat().st_uid != os.getuid()
                    or parent.stat().st_mode & 0o077):
                raise ValueError("capture parent must be canonical, owned, and private")
            fd = os.open(path, os.O_CREAT | os.O_EXCL | os.O_RDWR | os.O_NOFOLLOW, 0o600)
            files[name] = os.fdopen(fd, "w+b", buffering=0)
        for parent in {path.parent for path in paths.values()}:
            fd = os.open(parent, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
            try:
                os.fsync(fd)
            finally:
                os.close(fd)
        return files
    except BaseException:
        for handle in files.values():
            handle.close()
        raise  # Retain partial reservations; never reuse an attempt.


def exited(pid: int) -> bool:
    # Leave the leader unreaped so its PID/group identity cannot be reused.
    return os.waitid(os.P_PID, pid, os.WEXITED | os.WNOHANG | os.WNOWAIT) is not None


def signal_group(pid: int, sig: int) -> None:
    try:
        os.killpg(pid, sig)
    except ProcessLookupError:
        pass


def finish_group(pid: int) -> tuple[int | None, bool]:
    signal_group(pid, signal.SIGTERM)
    grace = time.monotonic() + 0.3
    while time.monotonic() < grace and not exited(pid):
        time.sleep(0.01)
    signal_group(pid, signal.SIGKILL)
    deadline = time.monotonic() + 1.0
    while time.monotonic() < deadline and not exited(pid):
        time.sleep(0.01)
    if not exited(pid):
        return None, False
    _, status = os.waitpid(pid, os.WNOHANG)
    # No signals after reap: the old group ID may now become reusable.
    try:
        os.killpg(pid, 0)
        gone = False
    except ProcessLookupError:
        gone = True
    return status, gone


def write_all(handle, data: bytes) -> None:
    view = memoryview(data)
    while view:
        count = handle.write(view)
        if count is None or count <= 0:
            raise OSError("capture write made no progress")
        view = view[count:]


def capture(args: argparse.Namespace, files: dict) -> dict:
    started = time.monotonic()
    result = {"schema_version": "bullet.native-capture.v1", "child_started": False,
              "completion": "UNVERIFIED", "bullet_live_admission": False,
              "ownership": "OWNED_PROCESS_GROUP_ONLY", "raw_private": True,
              "capture_limit_seconds": args.max_seconds, "cleanup_poll_allowance_seconds": 1.4,
              "filesystem_io_deadline": "NOT_GUARANTEED",
              "public_export_review": "REQUIRED", "command": args.command[1:],
              "started_at_unix": time.time(), "child_exit": None, "child_signal": None}
    interrupted = []
    handlers = {sig: signal.signal(sig, lambda number, _frame: interrupted.append(number))
                for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)}
    pid = fd = status = None
    total = events = 0
    reason = "CHILD_EXIT"
    decoder = codecs.getincrementaldecoder("utf-8")("replace")

    def event(kind: str, text: str) -> None:
        nonlocal events
        if events >= args.max_events:
            raise OverflowError("event limit")
        frame = [round(time.monotonic() - started, 6), kind, text]
        write_all(files["cast"], (json.dumps(frame) + "\n").encode())
        events += 1

    try:
        inputs = [x.encode().decode("unicode_escape").encode() for x in args.inject]
        if sum(map(len, inputs)) > args.max_bytes:
            raise ValueError("input limit")
        header = {"version": 2, "width": args.cols, "height": args.rows,
                  "timestamp": int(time.time()), "title": args.title}
        write_all(files["cast"], (json.dumps(header) + "\n").encode())
        pid, fd = pty.fork()
        if pid == 0:
            try:
                for sig in handlers:
                    signal.signal(sig, signal.SIG_DFL)
                fcntl.ioctl(1, termios.TIOCSWINSZ, struct.pack("HHHH", args.rows, args.cols, 0, 0))
                env = dict(os.environ, TERM="xterm-256color", COLORTERM="truecolor",
                           COLUMNS=str(args.cols), LINES=str(args.rows))
                os.execvpe(args.command[1], args.command[1:], env)
            except BaseException:
                os._exit(127)
        result.update(child_started=True, child_pid=pid)
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", args.rows, args.cols, 0, 0))
        os.set_blocking(fd, False)
        index = 0
        injected_at = observed_exit = None
        last_output = started
        while True:
            now = time.monotonic()
            if interrupted:
                reason = "INTERRUPTED"
                break
            if now - started >= args.max_seconds:
                reason = "TIMEOUT"
                break
            if observed_exit is None and exited(pid):
                observed_exit = now
            if observed_exit is not None and now - observed_exit >= 0.3:
                reason = "POST_EXIT_DRAIN_TIMEOUT"
                break
            if index < len(inputs) and now - started >= args.inject_delay + index * args.inject_gap:
                payload = inputs[index]
                if os.write(fd, payload) != len(payload):
                    raise OSError("partial terminal input")
                event("i", payload.decode("utf-8", "replace"))
                index += 1
                if index == len(inputs):
                    injected_at = now
            ready = ((injected_at is not None and now - injected_at >= args.min_after_inject)
                     or (args.min_runtime > 0 and now - started >= args.min_runtime))
            if args.idle_quit > 0 and total and ready and now - last_output >= args.idle_quit:
                reason = "IDLE_CAPTURE_STOPPED"
                break
            if not select.select([fd], [], [], 0.05)[0]:
                continue
            try:
                chunk = os.read(fd, min(65536, args.max_bytes - total + 1))
            except OSError as error:
                if error.errno == errno.EIO:
                    break
                if error.errno in (errno.EAGAIN, errno.EWOULDBLOCK):
                    continue
                raise
            if not chunk:
                break
            kept = chunk[:args.max_bytes - total]
            write_all(files["raw"], kept)
            total += len(kept)
            event("o", decoder.decode(kept))
            last_output = time.monotonic()
            if len(chunk) > len(kept):
                reason = "OUTPUT_LIMIT"
                break
        trailing = decoder.decode(b"", final=True)
        if trailing:
            event("o", trailing)
    except Exception as error:
        reason = "CAPTURE_ERROR"
        result["error_class"] = type(error).__name__
    finally:
        if pid is not None and pid > 0:
            if reason == "CHILD_EXIT":
                grace = time.monotonic() + 0.1
                while time.monotonic() < grace and not exited(pid):
                    time.sleep(0.005)
                if not exited(pid):
                    reason = "PTY_CLOSED_BEFORE_EXIT"
            status, gone = finish_group(pid)
            result["owned_group_gone"] = gone
            if not gone:
                reason = "GROUP_TEARDOWN_UNVERIFIED"
        if fd is not None:
            os.close(fd)
        for sig, handler in handlers.items():
            signal.signal(sig, handler)
    if status is not None:
        if os.WIFEXITED(status):
            result["child_exit"] = os.WEXITSTATUS(status)
        elif os.WIFSIGNALED(status):
            result["child_signal"] = os.WTERMSIG(status)
    code = result["child_exit"]
    if result["child_signal"] is not None:
        code = 128 + result["child_signal"]
    if reason == "TIMEOUT":
        code = 124
    elif reason == "INTERRUPTED":
        code = 128 + interrupted[0]
    elif reason != "CHILD_EXIT" or not total or code is None:
        code = code if code else 1
    result.update(stop_reason=reason, recorder_exit=code, bytes=total, events=events,
                  elapsed_seconds=time.monotonic() - started)
    return result


def read_retained(handle) -> bytes:
    os.fsync(handle.fileno())
    handle.seek(0)
    return handle.read()


def main() -> int:
    args = parse_args()
    try:
        files = reserve_files(args)
    except (OSError, ValueError):
        print("demo-gif-pty-record: output reservation refused", file=sys.stderr)
        return 2
    try:
        result = capture(args, files)
        raw = read_retained(files["raw"])
        text = redact(ANSI.sub("", raw.decode("utf-8", "replace")))
        write_all(files["transcript"], (text.rstrip() + "\n").encode())
        result["artifacts"] = {}
        for name in ("raw", "cast", "transcript"):
            data = read_retained(files[name])
            result["artifacts"][name] = {"sha256": hashlib.sha256(data).hexdigest(), "bytes": len(data)}
        write_all(files["result"], (json.dumps(result, indent=2) + "\n").encode())
        os.fsync(files["result"].fileno())
        return result["recorder_exit"]
    except (OSError, ValueError):
        print("demo-gif-pty-record: incomplete capture retained", file=sys.stderr)
        return 1
    finally:
        for handle in files.values():
            handle.close()


if __name__ == "__main__":
    raise SystemExit(main())
