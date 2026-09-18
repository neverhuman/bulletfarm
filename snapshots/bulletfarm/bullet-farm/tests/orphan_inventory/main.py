#!/usr/bin/env python3
"""CLI for the typed assurance inventory generator."""

from __future__ import annotations

import argparse
import os
import stat
import sys
import tempfile
from pathlib import Path

PRIVATE_MODULE_ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(PRIVATE_MODULE_ROOT))

from model import load_and_validate
from render import render
from runtime import collect
from self_test import run as run_self_test
from strict_io import InventoryError, expect_list, expect_object, load_relative, read_relative, text

OUTPUT = "docs/assurance/orphan-inventory.generated.md"


def _arguments() -> argparse.Namespace:
    parser = argparse.ArgumentParser(add_help=False)
    parser.add_argument("mode", choices=("write", "check", "self-test"))
    parser.add_argument("--root", required=True)
    parser.add_argument("--bin", required=True, dest="binary")
    return parser.parse_args()


def _profiles(root: Path) -> list[str]:
    ledger, _ = load_relative(root, "policy/assurance-inventory-v1.json")
    ledger = expect_object(ledger, "inventory")
    profiles = expect_list(ledger.get("release_profiles"), "release_profiles")
    return [
        text(
            expect_object(item, f"release_profiles[{index}]").get("name"),
            f"release_profiles[{index}].name",
        )
        for index, item in enumerate(profiles)
    ]


def _write_atomic(path: Path, content: bytes) -> None:
    parent = path.parent
    if path.is_symlink():
        raise InventoryError(f"{OUTPUT}: output is a symlink")
    if path.exists():
        metadata = path.stat()
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise InventoryError(f"{OUTPUT}: output must be a single-link regular file")
    descriptor, temporary = tempfile.mkstemp(prefix=".orphan-inventory-", dir=parent)
    try:
        os.fchmod(descriptor, 0o644)
        offset = 0
        while offset < len(content):
            offset += os.write(descriptor, content[offset:])
        os.fsync(descriptor)
        os.close(descriptor)
        descriptor = -1
        os.replace(temporary, path)
        directory = os.open(parent, os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_DIRECTORY", 0))
        try:
            os.fsync(directory)
        finally:
            os.close(directory)
    finally:
        if descriptor >= 0:
            os.close(descriptor)
        try:
            os.unlink(temporary)
        except FileNotFoundError:
            pass


def main() -> int:
    arguments = _arguments()
    root = Path(arguments.root)
    binary = Path(arguments.binary)
    if not root.is_absolute() or root.is_symlink() or not root.is_dir():
        raise InventoryError("--root must be an absolute non-symlink directory")
    reports = collect(binary, root, _profiles(root))
    inventory = load_and_validate(root, reports)
    first = render(inventory).encode("utf-8")
    second = render(inventory).encode("utf-8")
    if first != second:
        raise InventoryError("renderer is nondeterministic")
    if arguments.mode == "self-test":
        run_self_test(inventory)
        return 0
    if arguments.mode == "write":
        _write_atomic(root / OUTPUT, first)
        print(f"wrote {OUTPUT}")
        return 0
    current = read_relative(root, OUTPUT)
    if current != first:
        raise InventoryError(f"{OUTPUT} is stale; run scripts/orphan-inventory.sh write")
    print(f"check: {OUTPUT} is current; typed graph and double rendering passed")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except InventoryError as error:
        print(f"orphan-inventory: {error}", file=sys.stderr)
        raise SystemExit(1)
