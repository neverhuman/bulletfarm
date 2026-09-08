"""Strict family-manifest and cross-repository subject validation."""

from __future__ import annotations

import tomllib
from pathlib import Path
from typing import Any

from strict_io import (
    InventoryError,
    exact_keys,
    expect_list,
    expect_object,
    read_relative,
    text,
    unique_objects,
)

REPOSITORIES = ["bullet-farm", "bullet-kernel", "bullet-git", "bullet-portal"]
MANIFEST_FIELDS = {
    "schema_version", "workers", "split_root", "family", "repo_family",
    "umbrella_repo", "onboarding_gates", "required_repos", "repo",
}
REPO_FIELDS = {
    "path", "name", "slug", "jeryu_slug", "role", "profile", "branch",
    "default_branch", "has_jankurai_std", "note",
}


def load_repositories(family_root: Path, expected_schema: str) -> tuple[dict[str, Path], bytes]:
    raw = read_relative(family_root, "repos.manifest.toml")
    try:
        manifest = tomllib.loads(raw.decode("utf-8"))
    except (UnicodeDecodeError, tomllib.TOMLDecodeError) as error:
        raise InventoryError("family manifest is not strict UTF-8 TOML") from error
    manifest = expect_object(manifest, "family manifest")
    exact_keys(manifest, MANIFEST_FIELDS, set(), "family manifest")
    if manifest["schema_version"] != expected_schema:
        raise InventoryError("family manifest schema drifted")
    if (
        manifest["split_root"] != str(family_root)
        or manifest["family"] != "bullet-farm"
        or manifest["repo_family"] != "bullet-farm"
        or manifest["umbrella_repo"] != "bullet-farm"
    ):
        raise InventoryError("family manifest identity drifted")
    required = expect_list(manifest["required_repos"], "family manifest.required_repos")
    if required != REPOSITORIES:
        raise InventoryError("family manifest required repository order drifted")
    rows, indexed = unique_objects(manifest["repo"], "name", "family manifest.repo")
    if list(indexed) != REPOSITORIES:
        raise InventoryError("family manifest repository inventory drifted")
    repositories: dict[str, Path] = {}
    for row in rows:
        exact_keys(row, REPO_FIELDS, set(), f"family repository {row['name']}")
        expected = family_root / row["name"]
        if row["path"] != str(expected):
            raise InventoryError(f"family repository {row['name']}: path drifted")
        if expected.is_symlink() or not expected.is_dir():
            raise InventoryError(f"family repository {row['name']}: checkout is absent or a symlink")
        repositories[row["name"]] = expected
    return repositories, raw


def validate_corpus_source(family_root: Path, path: str, label: str) -> None:
    relative = text(path, f"{label}.path")
    read_relative(family_root, relative)


def validate_adr(hub_root: Path, value: Any, label: str) -> None:
    name = text(value, f"{label}.value")
    if "/" in name or name in {".", ".."}:
        raise InventoryError(f"{label}: unsafe ADR name")
    read_relative(hub_root, f"docs/decisions/{name}")


def validate_subject(
    subject: dict[str, Any],
    repositories: dict[str, Path],
    label: str,
    allowed_kinds: set[str],
) -> None:
    exact_keys(subject, {"kind", "repo", "path", "symbol"}, set(), label)
    if subject["kind"] not in allowed_kinds:
        raise InventoryError(f"{label}: unknown subject kind")
    repository = subject["repo"]
    if repository not in repositories:
        raise InventoryError(f"{label}: unknown repository")
    relative = text(subject["path"], f"{label}.path")
    symbol = text(subject["symbol"], f"{label}.symbol")
    data = read_relative(repositories[repository], relative)
    try:
        source = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise InventoryError(f"{label}: source is not UTF-8") from error
    if symbol not in source:
        raise InventoryError(f"{label}: absent symbol {symbol}")

