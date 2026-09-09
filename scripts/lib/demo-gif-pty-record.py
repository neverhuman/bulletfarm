#!/usr/bin/env python3
"""Record a real PTY session as an asciinema v2 cast.

This captures the live TUI or ANSI stream. It does not synthesize frames.
"""

from __future__ import annotations

import argparse
import errno
import fcntl
import json
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


EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
UUID = re.compile(
    r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    re.IGNORECASE,
)
BOOT = re.compile(r"boot_[0-9a-f]{32,}")
WRK = re.compile(r"wrk_[0-9a-f]{32,}")
ANSI = re.compile(r"\x1b(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~]|\][^\x07]*(?:\x07|\x1b\\))")


def redact(text: str) -> str:
    text = EMAIL.sub("[redacted-email]", text)
    text = UUID.sub("[redacted-id]", text)
    text = BOOT.sub("boot_[redacted]", text)
    text = WRK.sub("wrk_[redacted]", text)
    return text


def strip_ansi(text: str) -> str:
    return ANSI.sub("", text)


def set_winsize(fd: int, rows: int, cols: int) -> None:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))


def decode_inject(raw: str) -> bytes:
    return raw.encode("utf-8").decode("unicode_escape").encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cast", required=True)
    parser.add_argument("--transcript", required=True)
    parser.add_argument("--cols", type=int, default=140)
    parser.add_argument("--rows", type=int, default=38)
    parser.add_argument("--inject-delay", type=float, default=0.0)
    parser.add_argument("--inject", action="append", default=[])
    parser.add_argument("--inject-gap", type=float, default=0.12)
    parser.add_argument("--idle-quit", type=float, default=2.4)
    parser.add_argument("--min-after-inject", type=float, default=0.0)
    parser.add_argument("--min-runtime", type=float, default=0.0)
    parser.add_argument("--max-seconds", type=float, default=120.0)
    parser.add_argument("--title", default="")
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if not args.command or args.command[0] != "--":
        print("demo-gif-pty-record: expected -- COMMAND...", file=sys.stderr)
        return 2
    command = args.command[1:]
    if not command:
        print("demo-gif-pty-record: missing command", file=sys.stderr)
        return 2

    env = os.environ.copy()
    env["TERM"] = "xterm-256color"
    env["COLORTERM"] = "truecolor"
    env["COLUMNS"] = str(args.cols)
    env["LINES"] = str(args.rows)
    for key in ("CI", "GITHUB_ACTIONS", "GITHUB_WORKFLOW", "CURSOR_AGENT"):
        env.pop(key, None)

    started = time.monotonic()
    pid, fd = pty.fork()
    if pid == 0:
        set_winsize(sys.stdout.fileno(), args.rows, args.cols)
        os.execvpe(command[0], command, env)

    set_winsize(fd, args.rows, args.cols)
    flags = fcntl.fcntl(fd, fcntl.F_GETFL)
    fcntl.fcntl(fd, fcntl.F_SETFL, flags | os.O_NONBLOCK)

    header = {
        "version": 2,
        "width": args.cols,
        "height": args.rows,
        "timestamp": int(time.time()),
        "title": args.title,
        "env": {"SHELL": env.get("SHELL", "/bin/bash"), "TERM": "xterm-256color"},
    }
    events: list[tuple[float, str, str]] = []
    raw_chunks: list[str] = []
    injects = [decode_inject(item) for item in args.inject]
    inject_index = 0
    last_output = started
    injected_at: float | None = None
    exit_code = 1

    try:
        while True:
            now = time.monotonic()
            elapsed = now - started
            if elapsed >= args.max_seconds:
                os.kill(pid, signal.SIGTERM)
                break
            if (
                inject_index < len(injects)
                and elapsed >= args.inject_delay + inject_index * args.inject_gap
            ):
                payload = injects[inject_index]
                os.write(fd, payload)
                events.append((round(elapsed, 6), "i", payload.decode("utf-8", "replace")))
                inject_index += 1
                if inject_index == len(injects):
                    injected_at = now
            ready_from_inject = injected_at is not None and now - injected_at >= max(
                args.idle_quit, args.min_after_inject
            )
            ready_from_runtime = args.min_runtime > 0 and elapsed >= args.min_runtime
            if (
                args.idle_quit > 0
                and raw_chunks
                and now - last_output >= args.idle_quit
                and (ready_from_inject or ready_from_runtime)
            ):
                os.kill(pid, signal.SIGHUP)
                break
            try:
                ready, _, _ = select.select([fd], [], [], 0.05)
            except InterruptedError:
                continue
            if not ready:
                continue
            try:
                chunk = os.read(fd, 65536)
            except OSError as err:
                if err.errno in (errno.EAGAIN, errno.EWOULDBLOCK):
                    continue
                break
            if not chunk:
                break
            last_output = time.monotonic()
            text = chunk.decode("utf-8", "replace")
            raw_chunks.append(text)
            events.append((round(last_output - started, 6), "o", text))
    finally:
        try:
            _pid, status = os.waitpid(pid, 0)
            if os.WIFEXITED(status):
                exit_code = os.WEXITSTATUS(status)
            elif os.WIFSIGNALED(status):
                exit_code = 128 + os.WTERMSIG(status)
        except ChildProcessError:
            pass
        try:
            os.close(fd)
        except OSError:
            pass

    cast_path = Path(args.cast)
    transcript_path = Path(args.transcript)
    cast_path.parent.mkdir(parents=True, exist_ok=True)
    transcript_path.parent.mkdir(parents=True, exist_ok=True)
    with cast_path.open("w", encoding="utf-8") as handle:
        handle.write(json.dumps(header, separators=(",", ":")) + "\n")
        for when, kind, data in events:
            handle.write(json.dumps([when, kind, redact(data)], separators=(",", ":")) + "\n")
    plain = redact(strip_ansi("".join(raw_chunks)))
    transcript_path.write_text(plain.rstrip() + "\n", encoding="utf-8")
    if not raw_chunks:
        print("demo-gif-pty-record: captured no terminal output", file=sys.stderr)
        return 1
    return 0 if exit_code in (0, 129, 130, 143, 1) else exit_code


if __name__ == "__main__":
    raise SystemExit(main())
