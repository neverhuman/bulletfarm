#!/usr/bin/env python3
"""Bounded compatibility launcher for the Rust diagnostic dogfood board."""

from __future__ import annotations

import os
import signal
import subprocess
import sys
import time
from pathlib import Path

TIMEOUT_SECONDS = 120
TERMINATION_GRACE_SECONDS = 0.25
USAGE = (
    "usage: dogfood-board.py [--json] [--self-check]\n"
    "compatibility launcher for: bullet-family check dogfood --json\n"
)


def _hub_dir() -> Path:
    return Path(__file__).resolve().parent.parent


def _family_command() -> list[str]:
    selected = os.environ.get("BULLET_FAMILY_BIN")
    if selected:
        return [selected]
    local = _hub_dir() / "target/debug/bullet-family"
    if local.is_file() and os.access(local, os.X_OK):
        return [str(local)]
    return [
        "cargo",
        "run",
        "--locked",
        "--quiet",
        "--bin",
        "bullet-family",
        "--",
    ]


def _accept_compatibility_args(args: list[str]) -> bool:
    if args == ["--help"]:
        sys.stdout.write(USAGE)
        return False
    allowed = {"--json", "--self-check"}
    if len(args) > len(allowed) or len(args) != len(set(args)):
        sys.stderr.write(USAGE)
        raise SystemExit(2)
    if any(argument not in allowed for argument in args):
        sys.stderr.write(USAGE)
        raise SystemExit(2)
    return True


def _terminate_process_group(process: subprocess.Popen[bytes]) -> None:
    signal_error: OSError | None = None
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    except OSError as error:
        signal_error = error

    try:
        time.sleep(TERMINATION_GRACE_SECONDS)
    finally:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        except OSError as error:
            signal_error = signal_error or error

        try:
            process.wait(timeout=TERMINATION_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait()

    if signal_error is not None:
        raise signal_error


def main() -> int:
    if not _accept_compatibility_args(sys.argv[1:]):
        return 0

    environment = os.environ.copy()
    environment.pop("HOME", None)
    if os.name != "posix":
        sys.stderr.write("DOGFOOD_BOARD_LAUNCH_FAILED\n")
        return 127
    try:
        process = subprocess.Popen(
            [*_family_command(), "check", "dogfood", "--json"],
            cwd=_hub_dir(),
            env=environment,
            start_new_session=True,
        )
    except OSError:
        sys.stderr.write("DOGFOOD_BOARD_LAUNCH_FAILED\n")
        return 127

    try:
        return process.wait(timeout=TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        try:
            _terminate_process_group(process)
        except OSError:
            sys.stderr.write("DOGFOOD_BOARD_LAUNCH_FAILED\n")
            return 127
        sys.stderr.write("DOGFOOD_BOARD_TIMEOUT\n")
        return 124
    except OSError:
        try:
            _terminate_process_group(process)
        except OSError:
            pass
        sys.stderr.write("DOGFOOD_BOARD_LAUNCH_FAILED\n")
        return 127
    except BaseException:
        _terminate_process_group(process)
        raise


if __name__ == "__main__":
    raise SystemExit(main())
