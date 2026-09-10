#!/usr/bin/env python3
"""Copy a live bullet tui onto this PTY and send the HOLD-honest key story.

Does not read, print, or export credentials. The operator store is the default
authenticated session already present on the host.
"""
from __future__ import annotations

import os
import pty
import select
import sys
import time


def main() -> int:
    bullet = os.environ["BULLET_RECORD_BULLET_BIN"]
    pid, master = pty.fork()
    if pid == 0:
        os.execv(bullet, [bullet, "tui"])
    script = (
        (1.2, b""),
        (1.6, b"\x0b"),  # Ctrl+K palette
        (0.8, b"j"),
        (0.6, b"j"),
        (0.6, b"j"),
        (0.6, b"j"),
        (0.6, b"j"),
        (0.8, b"j"),  # first unknown surface
        (0.8, b"\r"),
        (0.8, b"\x0b"),
        (0.5, b"j"),
        (0.6, b"\r"),  # Tasks
        (1.0, b"?"),
        (1.4, b"\x1b"),
        (0.6, b"r"),
        (1.2, b"\x03"),  # detach
    )
    deadline = time.monotonic() + 70
    for hold, keys in script:
        end = time.monotonic() + hold
        while time.monotonic() < end and time.monotonic() < deadline:
            ready, _, _ = select.select([master], [], [], 0.05)
            if ready:
                try:
                    chunk = os.read(master, 8192)
                except OSError:
                    chunk = b""
                if not chunk:
                    os.waitpid(pid, 0)
                    return 0
                sys.stdout.buffer.write(chunk)
                sys.stdout.buffer.flush()
        if keys:
            os.write(master, keys)
    while time.monotonic() < deadline:
        ready, _, _ = select.select([master], [], [], 0.1)
        if not ready:
            continue
        try:
            chunk = os.read(master, 8192)
        except OSError:
            break
        if not chunk:
            break
        sys.stdout.buffer.write(chunk)
        sys.stdout.buffer.flush()
    os.waitpid(pid, 0)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
