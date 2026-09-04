#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

README_JSONSCHEMA_WHEELHOUSE="$REPO_ROOT/.config/readme-jsonschema-wheelhouse"
README_JSONSCHEMA_REQUIREMENTS="$REPO_ROOT/.config/readme-jsonschema-requirements.txt"
README_JSONSCHEMA_PYTHON=
README_JSONSCHEMA_PYTHON_SHA256=

readonly -a README_JSONSCHEMA_WHEEL_NAMES=(
  attrs-25.4.0-py3-none-any.whl
  jsonschema-4.26.0-py3-none-any.whl
  jsonschema_specifications-2025.9.1-py3-none-any.whl
  pip-26.2.1-py3-none-any.whl
  referencing-0.37.0-py3-none-any.whl
  rpds_py-0.30.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
  typing_extensions-4.15.0-py3-none-any.whl
)
readonly -a README_JSONSCHEMA_WHEEL_SIZES=(
  67615 90630 18437 1816632 26766 394080 44614
)
readonly -a README_JSONSCHEMA_WHEEL_SHA256=(
  adcf7e2a1fb3b36ac48d97835bb6d8ade15b8dcce26aba8bf1d14847b57a3373
  d489f15263b8d200f8387e64b4c3a75f06629559fb73deb8fdfb525f2dab50ce
  98802fee3a11ee76ecaca44429fda8a41bff98b00a0f2838151b113f210cc6fe
  71138adf1f4ca900cdb7d289c21b7494329f2332b6d85f0e1c42108c0384ed3e
  381329a9f99628c9069361716891d34ad94af76e461dcb0335825aecc7692231
  47f236970bccb2233267d89173d3ad2703cd36a0e2a6e92d0560d333871a3d23
  f0fa19c6845758ab08074a0cfa8b7aecb71c999ca73d62883bc25cc018c4e548
)

readme_jsonschema_sha256() {
  local digest remainder
  IFS=' ' read -r digest remainder < <(/usr/bin/sha256sum -- "$1") || return 1
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || {
    refuse JSONSCHEMA_SHA256_INVALID "$1"
    return 1
  }
  printf '%s\n' "$digest"
}

readme_jsonschema_require_directory() {
  local path="$1" label="$2" mode uid links size
  [[ -d "$path" && ! -L "$path" ]] || {
    refuse JSONSCHEMA_DIRECTORY_INVALID "$label"
    return 1
  }
  IFS=: read -r mode uid links size < <(/usr/bin/stat -c '%a:%u:%h:%s' -- "$path")
  [[ "$uid" == "$EUID" && "$mode" =~ ^[0-7]{3,4}$ ]] || {
    refuse JSONSCHEMA_DIRECTORY_CUSTODY_INVALID "$label"
    return 1
  }
  (( (8#$mode & 002) == 0 )) || {
    refuse JSONSCHEMA_DIRECTORY_WORLD_WRITABLE "$label"
    return 1
  }
  [[ "$links" =~ ^[1-9][0-9]*$ && "$size" =~ ^[0-9]+$ ]] || {
    refuse JSONSCHEMA_DIRECTORY_CUSTODY_INVALID "$label"
    return 1
  }
}

readme_jsonschema_require_file() {
  local path="$1" expected_size="$2" expected_hash="$3" label="$4"
  local mode uid links size actual
  [[ -f "$path" && ! -L "$path" ]] || {
    refuse JSONSCHEMA_SUBJECT_INVALID "$label"
    return 1
  }
  IFS=: read -r mode uid links size < <(/usr/bin/stat -c '%a:%u:%h:%s' -- "$path")
  [[ ("$mode" == 644 || "$mode" == 664) && "$uid" == "$EUID" \
    && "$links" == 1 && "$size" == "$expected_size" ]] || {
    refuse JSONSCHEMA_SUBJECT_CUSTODY_INVALID "$label"
    return 1
  }
  actual="$(readme_jsonschema_sha256 "$path")" || return 1
  [[ "$actual" == "$expected_hash" ]] || {
    refuse JSONSCHEMA_SUBJECT_DIGEST_MISMATCH "$label"
    return 1
  }
}

readme_jsonschema_check_python_subject() {
  local path="$1"
  local version abi mode uid links
  [[ "$path" == /usr/bin/python3.12 && -f "$path" && ! -L "$path" \
    && -x "$path" ]] || {
    refuse JSONSCHEMA_PYTHON_INVALID "$path"
    return 1
  }
  IFS=: read -r mode uid links < <(/usr/bin/stat -c '%a:%u:%h' -- "$path")
  [[ "$mode" == 755 && "$uid" == 0 && "$links" == 1 ]] || {
    refuse JSONSCHEMA_PYTHON_CUSTODY_INVALID "$path"
    return 1
  }
  version="$(/usr/bin/env -i HOME=/ PATH=/usr/bin:/bin LC_ALL=C TZ=UTC \
    "$path" -I -B -S --version 2>&1)" || return 1
  [[ "$version" =~ ^Python\ 3\.12\.[0-9]+$ ]] || {
    refuse JSONSCHEMA_PYTHON_VERSION_INVALID "$version"
    return 1
  }
  abi="$(/usr/bin/env -i HOME=/ PATH=/usr/bin:/bin LC_ALL=C TZ=UTC \
    "$path" -I -B -S -c \
    'import platform,sys; print(f"{sys.platform}:{platform.machine()}:{sys.implementation.cache_tag}")')" \
    || return 1
  [[ "$abi" == linux:x86_64:cpython-312 ]] || {
    refuse JSONSCHEMA_PLATFORM_UNSUPPORTED "$abi"
    return 1
  }
}

readme_jsonschema_capture_python() {
  [[ -z "$README_JSONSCHEMA_PYTHON" && -z "$README_JSONSCHEMA_PYTHON_SHA256" ]] || {
    refuse JSONSCHEMA_PYTHON_ALREADY_CAPTURED verify-instead
    return 1
  }
  README_JSONSCHEMA_PYTHON=/usr/bin/python3.12
  readme_jsonschema_check_python_subject "$README_JSONSCHEMA_PYTHON" || return 1
  README_JSONSCHEMA_PYTHON_SHA256="$(readme_jsonschema_sha256 "$README_JSONSCHEMA_PYTHON")" \
    || return 1
}

readme_jsonschema_verify_python() {
  local actual
  [[ -n "$README_JSONSCHEMA_PYTHON" && -n "$README_JSONSCHEMA_PYTHON_SHA256" ]] || {
    refuse JSONSCHEMA_PYTHON_NOT_CAPTURED capture-first
    return 1
  }
  readme_jsonschema_check_python_subject "$README_JSONSCHEMA_PYTHON" || return 1
  actual="$(readme_jsonschema_sha256 "$README_JSONSCHEMA_PYTHON")" || return 1
  [[ "$actual" == "$README_JSONSCHEMA_PYTHON_SHA256" ]] || {
    refuse JSONSCHEMA_PYTHON_IDENTITY_CHANGED "$README_JSONSCHEMA_PYTHON"
    return 1
  }
}

readme_jsonschema_validate_wheelhouse() {
  local wheelhouse="$1" requirements="$2" index
  local -a expected actual
  if [[ -z "$README_JSONSCHEMA_PYTHON" && -z "$README_JSONSCHEMA_PYTHON_SHA256" ]]; then
    readme_jsonschema_capture_python || return 1
  else
    readme_jsonschema_verify_python || return 1
  fi
  [[ "$(/usr/bin/uname -s):$(/usr/bin/uname -m)" == Linux:x86_64 ]] || {
    refuse JSONSCHEMA_PLATFORM_UNSUPPORTED "$(/usr/bin/uname -s):$(/usr/bin/uname -m)"
    return 1
  }
  readme_jsonschema_require_directory "${wheelhouse%/*}" wheelhouse-parent || return 1
  readme_jsonschema_require_directory "$wheelhouse" wheelhouse || return 1
  readme_jsonschema_require_file "$requirements" 641 \
    cfbd0a8a619fec0cd97c30b175785f2d6eb2c60e91ea898f54e3cb15231d8c2a requirements || return 1
  readme_jsonschema_require_file "$wheelhouse/manifest-v1.json" 1297 \
    dd4385916aa47c9345bc2497c40e17a6c667b05a52298a03cf123782189a2c3b manifest || return 1

  expected=(
    attrs-25.4.0-py3-none-any.whl
    jsonschema-4.26.0-py3-none-any.whl
    jsonschema_specifications-2025.9.1-py3-none-any.whl
    manifest-v1.json
    pip-26.2.1-py3-none-any.whl
    referencing-0.37.0-py3-none-any.whl
    rpds_py-0.30.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl
    typing_extensions-4.15.0-py3-none-any.whl
  )
  mapfile -t actual < <(/usr/bin/find "$wheelhouse" -mindepth 1 -maxdepth 1 -printf '%f\n' \
    | /usr/bin/sort)
  [[ "${actual[*]}" == "${expected[*]}" ]] || {
    refuse JSONSCHEMA_WHEEL_INVENTORY_DRIFT "${actual[*]}"
    return 1
  }
  for index in "${!README_JSONSCHEMA_WHEEL_NAMES[@]}"; do
    readme_jsonschema_require_file \
      "$wheelhouse/${README_JSONSCHEMA_WHEEL_NAMES[$index]}" \
      "${README_JSONSCHEMA_WHEEL_SIZES[$index]}" \
      "${README_JSONSCHEMA_WHEEL_SHA256[$index]}" \
      "${README_JSONSCHEMA_WHEEL_NAMES[$index]}" || return 1
  done
  readme_jsonschema_verify_python
}

readme_jsonschema_run_pip() {
  local venv_python="$1" private_home="$2" pip_wheel="$3"
  shift 3
  /usr/bin/env -i HOME="$private_home" PATH=/usr/bin:/bin LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 PIP_CONFIG_FILE=/dev/null \
    "$venv_python" -I -B -c \
    'import runpy,sys; sys.path.insert(0,sys.argv.pop(1)); sys.argv[0]="pip"; runpy.run_module("pip",run_name="__main__")' \
    "$pip_wheel" "$@"
}

readme_jsonschema_validate_installed() {
  local venv="$1" report="$2" wheelhouse="$3" private_home="$4" resolved
  readme_jsonschema_require_directory "$venv" installed-venv || return 1
  resolved="$(/usr/bin/readlink -f -- "$venv/bin/python3")" || return 1
  [[ "$resolved" == "$README_JSONSCHEMA_PYTHON" ]] || {
    refuse JSONSCHEMA_VENV_PYTHON_MISMATCH "$resolved"
    return 1
  }
  [[ -f "$report" && ! -L "$report" ]] || {
    refuse JSONSCHEMA_INSTALL_REPORT_INVALID "$report"
    return 1
  }
  /usr/bin/env -i HOME="$private_home" PATH=/usr/bin:/bin LC_ALL=C TZ=UTC \
    PYTHONDONTWRITEBYTECODE=1 "$venv/bin/python3" -I -B - \
    "$venv" "$report" "$wheelhouse" "$README_JSONSCHEMA_PYTHON" <<'PY' || {
import base64
import hashlib
import importlib.metadata
import json
import os
import pathlib
import stat
import sys
import sysconfig

venv = pathlib.Path(sys.argv[1]).resolve(strict=True)
report_path = pathlib.Path(sys.argv[2])
wheelhouse = pathlib.Path(sys.argv[3]).resolve(strict=True)
captured_python = pathlib.Path(sys.argv[4]).resolve(strict=True)
expected = {
    "attrs": ("25.4.0", "attrs-25.4.0-py3-none-any.whl", "adcf7e2a1fb3b36ac48d97835bb6d8ade15b8dcce26aba8bf1d14847b57a3373"),
    "jsonschema": ("4.26.0", "jsonschema-4.26.0-py3-none-any.whl", "d489f15263b8d200f8387e64b4c3a75f06629559fb73deb8fdfb525f2dab50ce"),
    "jsonschema-specifications": ("2025.9.1", "jsonschema_specifications-2025.9.1-py3-none-any.whl", "98802fee3a11ee76ecaca44429fda8a41bff98b00a0f2838151b113f210cc6fe"),
    "referencing": ("0.37.0", "referencing-0.37.0-py3-none-any.whl", "381329a9f99628c9069361716891d34ad94af76e461dcb0335825aecc7692231"),
    "rpds-py": ("0.30.0", "rpds_py-0.30.0-cp312-cp312-manylinux_2_17_x86_64.manylinux2014_x86_64.whl", "47f236970bccb2233267d89173d3ad2703cd36a0e2a6e92d0560d333871a3d23"),
    "typing-extensions": ("4.15.0", "typing_extensions-4.15.0-py3-none-any.whl", "f0fa19c6845758ab08074a0cfa8b7aecb71c999ca73d62883bc25cc018c4e548"),
}

def normalized(name):
    return name.lower().replace("_", "-").replace(".", "-")

def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON member: {key}")
        result[key] = value
    return result

with report_path.open("r", encoding="utf-8") as handle:
    report = json.load(handle, object_pairs_hook=unique_object)
installs = report.get("install")
if not isinstance(installs, list) or len(installs) != len(expected):
    raise SystemExit("install report count mismatch")
seen = set()
for item in installs:
    metadata = item.get("metadata", {})
    name = normalized(metadata.get("name", ""))
    if name not in expected or name in seen:
        raise SystemExit(f"unexpected install report distribution: {name}")
    version, filename, digest = expected[name]
    download = item.get("download_info", {})
    archive = download.get("archive_info", {})
    if metadata.get("version") != version:
        raise SystemExit(f"install report version mismatch: {name}")
    if download.get("url") != (wheelhouse / filename).as_uri():
        raise SystemExit(f"install report URL mismatch: {name}")
    if archive.get("hashes") != {"sha256": digest}:
        raise SystemExit(f"install report digest mismatch: {name}")
    seen.add(name)
if seen != set(expected):
    raise SystemExit("install report inventory mismatch")

site = pathlib.Path(sysconfig.get_path("purelib")).resolve(strict=True)
if not site.is_relative_to(venv):
    raise SystemExit("site-packages escaped the venv")
distributions = list(importlib.metadata.distributions(path=[str(site)]))
actual = {}
recorded_site = set()
recorded_venv = set()
for distribution in distributions:
    name = normalized(distribution.metadata.get("Name", ""))
    if name in actual or name not in expected:
        raise SystemExit(f"unexpected installed distribution: {name}")
    if distribution.version != expected[name][0]:
        raise SystemExit(f"installed version mismatch: {name}")
    files = distribution.files
    if not files:
        raise SystemExit(f"missing RECORD inventory: {name}")
    for entry in files:
        target = pathlib.Path(distribution.locate_file(entry))
        if target.is_symlink():
            raise SystemExit(f"symlinked installed file: {entry}")
        resolved_target = target.resolve(strict=True)
        if not resolved_target.is_relative_to(venv):
            raise SystemExit(f"installed file escaped venv: {entry}")
        recorded_venv.add(resolved_target)
        if not stat.S_ISREG(resolved_target.stat().st_mode):
            raise SystemExit(f"installed entry is not regular: {entry}")
        if resolved_target.is_relative_to(site):
            recorded_site.add(resolved_target)
        if entry.hash is None or entry.size is None:
            if pathlib.PurePosixPath(str(entry)).name != "RECORD":
                raise SystemExit(f"unhashed installed entry: {entry}")
            continue
        if entry.hash.mode != "sha256" or resolved_target.stat().st_size != entry.size:
            raise SystemExit(f"installed metadata mismatch: {entry}")
        digest = hashlib.sha256(resolved_target.read_bytes()).digest()
        encoded = base64.urlsafe_b64encode(digest).rstrip(b"=").decode("ascii")
        if encoded != entry.hash.value:
            raise SystemExit(f"installed RECORD digest mismatch: {entry}")
    actual[name] = distribution.version
if actual != {name: values[0] for name, values in expected.items()}:
    raise SystemExit("installed distribution inventory mismatch")

actual_site = set()
for root, directories, files in os.walk(site, followlinks=False):
    root_path = pathlib.Path(root)
    for directory in directories:
        if (root_path / directory).is_symlink():
            raise SystemExit(f"symlinked installed directory: {directory}")
    for filename in files:
        path = root_path / filename
        if path.suffix in {".pth", ".egg-link"} or filename == "direct_url.json":
            raise SystemExit(f"forbidden installed metadata: {filename}")
        actual_site.add(path.resolve(strict=True))
if actual_site != recorded_site:
    raise SystemExit("installed site-packages contains an unrecorded file")

bin_dir = venv / "bin"
if bin_dir.is_symlink() or not bin_dir.is_dir():
    raise SystemExit("installed bin directory is invalid")
bin_stat = bin_dir.stat(follow_symlinks=False)
if (not stat.S_ISDIR(bin_stat.st_mode) or bin_stat.st_uid != os.geteuid()
        or stat.S_IMODE(bin_stat.st_mode) & 0o022):
    raise SystemExit("installed bin directory custody is unsafe")
expected_bin = {
    "Activate.ps1", "activate", "activate.csh", "activate.fish", "jsonschema",
    "python", "python3", "python3.12",
}
actual_bin = {entry.name for entry in bin_dir.iterdir()}
if actual_bin != expected_bin:
    raise SystemExit("installed bin inventory mismatch")
for name in ("Activate.ps1", "activate", "activate.csh", "activate.fish"):
    entry = bin_dir / name
    entry_stat = entry.stat(follow_symlinks=False)
    if (entry.is_symlink() or not stat.S_ISREG(entry_stat.st_mode)
            or entry_stat.st_uid != os.geteuid() or entry_stat.st_nlink != 1
            or stat.S_IMODE(entry_stat.st_mode) & 0o133):
        raise SystemExit(f"installed activation entry is unsafe: {name}")
jsonschema_cli = bin_dir / "jsonschema"
jsonschema_stat = jsonschema_cli.stat(follow_symlinks=False)
if (jsonschema_cli.is_symlink() or not stat.S_ISREG(jsonschema_stat.st_mode)
        or jsonschema_stat.st_uid != os.geteuid() or jsonschema_stat.st_nlink != 1
        or not jsonschema_stat.st_mode & 0o100
        or stat.S_IMODE(jsonschema_stat.st_mode) & 0o022
        or jsonschema_cli.resolve(strict=True) not in recorded_venv):
    raise SystemExit("installed jsonschema entry is unsafe or unrecorded")
python_links = {
    "python": "python3.12",
    "python3": "python3.12",
    "python3.12": "/usr/bin/python3.12",
}
for name, target in python_links.items():
    entry = bin_dir / name
    if not entry.is_symlink() or os.readlink(entry) != target:
        raise SystemExit(f"installed Python link mismatch: {name}")
    if entry.resolve(strict=True) != captured_python:
        raise SystemExit(f"installed Python subject mismatch: {name}")

from jsonschema import Draft202012Validator
Draft202012Validator.check_schema({"type": "object", "additionalProperties": False})
print("installed-ok")
PY
    refuse JSONSCHEMA_INSTALLED_CUSTODY_INVALID "$venv"
    return 1
  }
  readme_jsonschema_run_pip "$venv/bin/python3" "$private_home" \
    "$wheelhouse/pip-26.2.1-py3-none-any.whl" \
    --isolated --disable-pip-version-check --no-input check || {
    refuse JSONSCHEMA_DEPENDENCY_CHECK_FAILED "$venv"
    return 1
  }
  [[ -f "$venv/bin/jsonschema" && ! -L "$venv/bin/jsonschema" \
    && -x "$venv/bin/jsonschema" ]] || {
    refuse JSONSCHEMA_CLI_INVALID "$venv/bin/jsonschema"
    return 1
  }
  readme_jsonschema_verify_python
}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  [[ "$#" -eq 0 ]] || {
    refuse JSONSCHEMA_ARGUMENT_INVALID no-arguments
    exit 2
  }
  readme_jsonschema_validate_wheelhouse \
    "$README_JSONSCHEMA_WHEELHOUSE" "$README_JSONSCHEMA_REQUIREMENTS"
  log "README jsonschema wheelhouse custody passed"
fi
