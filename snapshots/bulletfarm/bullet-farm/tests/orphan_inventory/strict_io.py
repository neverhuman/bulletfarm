"""Bounded, duplicate-rejecting JSON and file helpers for the assurance inventory."""

from __future__ import annotations

import hashlib
import json
import os
import stat
from pathlib import Path
from typing import Any

MAX_JSON_BYTES = 1024 * 1024


class InventoryError(ValueError):
    """A typed, user-facing inventory refusal."""


def _object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise InventoryError(f"duplicate JSON member: {key}")
        result[key] = value
    return result


def loads_strict(data: bytes, label: str) -> Any:
    if not data or len(data) > MAX_JSON_BYTES:
        raise InventoryError(f"{label}: JSON size is outside 1..={MAX_JSON_BYTES}")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise InventoryError(f"{label}: JSON is not UTF-8") from error
    try:
        return json.loads(
            text,
            object_pairs_hook=_object,
            parse_constant=lambda token: (_ for _ in ()).throw(
                InventoryError(f"{label}: non-finite JSON number {token}")
            ),
        )
    except (json.JSONDecodeError, RecursionError) as error:
        raise InventoryError(f"{label}: invalid JSON") from error


def _relative_parts(relative: str) -> tuple[str, ...]:
    path = Path(relative)
    if path.is_absolute() or not path.parts or any(part in ("", ".", "..") for part in path.parts):
        raise InventoryError(f"unsafe relative source path: {relative!r}")
    return path.parts


def read_relative(root: Path, relative: str) -> bytes:
    parts = _relative_parts(relative)
    flags = os.O_RDONLY | os.O_CLOEXEC | getattr(os, "O_NOFOLLOW", 0)
    directory_flags = flags | getattr(os, "O_DIRECTORY", 0)
    descriptors: list[int] = []
    try:
        current = os.open(root, directory_flags)
        descriptors.append(current)
        for part in parts[:-1]:
            current = os.open(part, directory_flags, dir_fd=current)
            descriptors.append(current)
        subject = os.open(parts[-1], flags, dir_fd=current)
        descriptors.append(subject)
        before = os.fstat(subject)
        if not stat.S_ISREG(before.st_mode) or before.st_nlink != 1:
            raise InventoryError(f"{relative}: source must be a single-link regular file")
        if before.st_size < 1 or before.st_size > MAX_JSON_BYTES:
            raise InventoryError(f"{relative}: source size is outside 1..={MAX_JSON_BYTES}")
        chunks: list[bytes] = []
        remaining = before.st_size
        while remaining:
            chunk = os.read(subject, min(remaining, 65536))
            if not chunk:
                raise InventoryError(f"{relative}: source shortened during read")
            chunks.append(chunk)
            remaining -= len(chunk)
        if os.read(subject, 1):
            raise InventoryError(f"{relative}: source grew during read")
        after = os.fstat(subject)
        identity = lambda value: (
            value.st_dev,
            value.st_ino,
            value.st_mode,
            value.st_nlink,
            value.st_uid,
            value.st_size,
            value.st_mtime_ns,
            value.st_ctime_ns,
        )
        if identity(before) != identity(after):
            raise InventoryError(f"{relative}: source identity changed during read")
        return b"".join(chunks)
    except OSError as error:
        raise InventoryError(f"{relative}: source admission failed: {error.strerror}") from error
    finally:
        for descriptor in reversed(descriptors):
            os.close(descriptor)


def load_relative(root: Path, relative: str) -> tuple[Any, bytes]:
    data = read_relative(root, relative)
    return loads_strict(data, relative), data


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def expect_object(value: Any, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise InventoryError(f"{label}: expected object")
    return value


def expect_list(value: Any, label: str) -> list[Any]:
    if not isinstance(value, list):
        raise InventoryError(f"{label}: expected array")
    return value


def exact_keys(value: dict[str, Any], required: set[str], optional: set[str], label: str) -> None:
    missing = required - value.keys()
    extra = value.keys() - required - optional
    if missing:
        raise InventoryError(f"{label}: missing fields {sorted(missing)}")
    if extra:
        raise InventoryError(f"{label}: unknown fields {sorted(extra)}")


def text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 512:
        raise InventoryError(f"{label}: expected non-empty bounded string")
    return value


def unique_objects(values: Any, key: str, label: str) -> tuple[list[dict[str, Any]], dict[str, dict[str, Any]]]:
    items = expect_list(values, label)
    objects: list[dict[str, Any]] = []
    indexed: dict[str, dict[str, Any]] = {}
    for index, raw in enumerate(items):
        item = expect_object(raw, f"{label}[{index}]")
        identity = text(item.get(key), f"{label}[{index}].{key}")
        if identity in indexed:
            raise InventoryError(f"{label}: duplicate {key} {identity}")
        indexed[identity] = item
        objects.append(item)
    if not objects:
        raise InventoryError(f"{label}: zero-item partition")
    return objects, indexed
