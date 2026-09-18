#!/usr/bin/env python3
"""Run bullet tui on this PTY (the outer recorder) and inherit its winsize.

Does not read, print, or export credentials. The operator store is the default
authenticated session already present on the host.

An inner pty.fork() without TIOCSWINSZ produced the published white tape:
ratatui never painted, and agg collapsed three off-white frames. This process
replaces itself with `bullet tui` so the recorder's 225x54 slave is the only
terminal. Keystrokes are injected by record-tui.sh on the outer master.
"""
from __future__ import annotations

import os
import sys


def main() -> int:
    bullet = os.environ["BULLET_RECORD_BULLET_BIN"]
    os.execv(bullet, [bullet, "tui"])
    return 127


if __name__ == "__main__":
    raise SystemExit(main())
