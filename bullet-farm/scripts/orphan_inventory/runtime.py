"""Collect compiled release/profile truth without credentials or mutation."""

from __future__ import annotations

import hashlib
import os
import stat
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from strict_io import InventoryError, loads_strict


def _binary_identity(path: Path) -> tuple[int, int, int, int, str]:
    if not path.is_absolute() or path.is_symlink():
        raise InventoryError("runtime binary must be an absolute non-symlink path")
    try:
        metadata = path.stat()
    except OSError as error:
        raise InventoryError(f"runtime binary admission failed: {error}") from error
    if not stat.S_ISREG(metadata.st_mode) or not os.access(path, os.X_OK):
        raise InventoryError("runtime binary must be an executable regular file")
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(65536), b""):
            digest.update(chunk)
    return metadata.st_dev, metadata.st_ino, metadata.st_size, metadata.st_mtime_ns, digest.hexdigest()


def collect(binary: Path, root: Path, profiles: list[str]) -> list[dict[str, Any]]:
    before = _binary_identity(binary)
    reports: list[dict[str, Any]] = []
    with tempfile.TemporaryDirectory(prefix="bullet-assurance-inventory-") as temporary:
        base = Path(temporary)
        absent_registry = base / "registry-must-remain-absent"
        isolated_home = base / "empty-home"
        isolated_home.mkdir(mode=0o700)
        environment = {
            "HOME": str(isolated_home), "PATH": "/usr/bin:/bin", "LC_ALL": "C",
            "TZ": "UTC", "GIT_TERMINAL_PROMPT": "0",
        }
        for profile in profiles:
            command = [
                str(binary), "--root", str(root), "check", "release", "--profile", profile,
                "--receipts", str(absent_registry), "--json",
            ]
            try:
                result = subprocess.run(
                    command, cwd="/", env=environment, stdin=subprocess.DEVNULL,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=20,
                    check=False,
                )
            except (OSError, subprocess.TimeoutExpired) as error:
                raise InventoryError(f"profile {profile}: runtime report failed: {error}") from error
            if result.returncode != 3:
                raise InventoryError(
                    f"profile {profile}: expected BLOCKED exit 3, found {result.returncode}"
                )
            if result.stderr:
                raise InventoryError(f"profile {profile}: runtime report wrote stderr")
            report = loads_strict(result.stdout, f"profile {profile} report")
            if not isinstance(report, dict):
                raise InventoryError(f"profile {profile}: runtime report is not an object")
            reports.append(report)
        if absent_registry.exists() or absent_registry.is_symlink():
            raise InventoryError("release report collection created the absent registry")
    if before != _binary_identity(binary):
        raise InventoryError("runtime binary identity changed during report collection")
    return reports
