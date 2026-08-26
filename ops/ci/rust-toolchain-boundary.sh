#!/usr/bin/env bash
# Lazy, identity-pinned tools for Rust-dependent CI helpers. Sourcing this file
# performs no discovery or execution; the first Rust helper initializes it.
# shellcheck disable=SC2034 # globals are consumed by lib.sh after this source

resolve_python_312() {
  local env_executable="$1" candidate version
  for candidate in python3 python; do
    candidate="$(builtin type -P "$candidate")" || continue
    [[ "$candidate" == /* && -f "$candidate" && -x "$candidate" ]] || continue
    version="$("$env_executable" -i HOME="${HOME:-/}" PATH="${candidate%/*}:/usr/bin:/bin" \
      LC_ALL=C TZ=UTC "$candidate" -I -S --version 2>&1)" || continue
    version="${version%$'\r'}"
    if [[ "$version" =~ ^Python\ 3\.12\.[0-9]+$ ]]; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done
  refuse TOOL_VERSION_MISMATCH "Python 3.12 is required before Cargo execution"
}

RUST_TOOLCHAIN_TOOLS_INITIALIZED=0
ENV_EXECUTABLE=
ENV_EXECUTABLE_SHA256=
FIND_EXECUTABLE=
FIND_EXECUTABLE_SHA256=
SORT_EXECUTABLE=
SORT_EXECUTABLE_SHA256=
SED_EXECUTABLE=
SED_EXECUTABLE_SHA256=
PYTHON_312_EXECUTABLE=
PYTHON_312_EXECUTABLE_SHA256=
CARGO_EXECUTABLE=
CARGO_EXECUTABLE_SHA256=

initialize_rust_toolchain_tools() {
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 0 ]] || return 0

  local env_executable env_sha256 find_executable find_sha256
  local sort_executable sort_sha256 sed_executable sed_sha256
  local python_executable python_sha256 cargo_executable cargo_sha256 cargo_version
  env_executable="$(resolved_executable env)" || return 1
  env_sha256="$(sha256_file "$env_executable")" || return 1
  find_executable="$(resolved_executable find)" || return 1
  find_sha256="$(sha256_file "$find_executable")" || return 1
  sort_executable="$(resolved_executable sort)" || return 1
  sort_sha256="$(sha256_file "$sort_executable")" || return 1
  sed_executable="$(resolved_executable sed)" || return 1
  sed_sha256="$(sha256_file "$sed_executable")" || return 1
  python_executable="$(resolve_python_312 "$env_executable")" || return 1
  python_sha256="$(sha256_file "$python_executable")" || return 1
  cargo_executable="$(resolved_executable cargo)" || return 1
  cargo_sha256="$(sha256_file "$cargo_executable")" || return 1
  if ! cargo_version="$("$env_executable" -i HOME="${HOME:-/}" \
    PATH="${cargo_executable%/*}:/usr/bin:/bin" LC_ALL=C TZ=UTC \
    "$cargo_executable" --version 2>&1)"; then
    refuse TOOL_VERSION_MISMATCH "Cargo 1.95.0 is required before Cargo execution"
    return 1
  fi
  cargo_version="${cargo_version%$'\r'}"
  if [[ ! "$cargo_version" =~ ^cargo\ 1\.95\.0\ \([0-9a-f]{9,40}\ [0-9]{4}-[0-9]{2}-[0-9]{2}\)$ ]]; then
    refuse TOOL_VERSION_MISMATCH \
      "expected an exact Cargo 1.95.0 version token, found '$cargo_version'"
    return 1
  fi

  ENV_EXECUTABLE="$env_executable"
  ENV_EXECUTABLE_SHA256="$env_sha256"
  FIND_EXECUTABLE="$find_executable"
  FIND_EXECUTABLE_SHA256="$find_sha256"
  SORT_EXECUTABLE="$sort_executable"
  SORT_EXECUTABLE_SHA256="$sort_sha256"
  SED_EXECUTABLE="$sed_executable"
  SED_EXECUTABLE_SHA256="$sed_sha256"
  PYTHON_312_EXECUTABLE="$python_executable"
  PYTHON_312_EXECUTABLE_SHA256="$python_sha256"
  CARGO_EXECUTABLE="$cargo_executable"
  CARGO_EXECUTABLE_SHA256="$cargo_sha256"
  RUST_TOOLCHAIN_TOOLS_INITIALIZED=1
  readonly RUST_TOOLCHAIN_TOOLS_INITIALIZED
  readonly PYTHON_312_EXECUTABLE PYTHON_312_EXECUTABLE_SHA256
  readonly CARGO_EXECUTABLE CARGO_EXECUTABLE_SHA256
  readonly ENV_EXECUTABLE ENV_EXECUTABLE_SHA256 FIND_EXECUTABLE FIND_EXECUTABLE_SHA256
  readonly SORT_EXECUTABLE SORT_EXECUTABLE_SHA256 SED_EXECUTABLE SED_EXECUTABLE_SHA256
}

run_python_312() {
  initialize_rust_toolchain_tools || return 1
  verify_resolved_tool \
    "$PYTHON_312_EXECUTABLE" "$PYTHON_312_EXECUTABLE_SHA256" Python || return 1
  local status
  if "$ENV_EXECUTABLE" -i HOME="${HOME:-/}" \
    PATH="${PYTHON_312_EXECUTABLE%/*}:/usr/bin:/bin" \
    LC_ALL=C TZ=UTC PYTHONIOENCODING=utf-8 \
    "$PYTHON_312_EXECUTABLE" -I -S "$@"; then
    status=0
  else
    status=$?
  fi
  verify_resolved_tool \
    "$PYTHON_312_EXECUTABLE" "$PYTHON_312_EXECUTABLE_SHA256" Python || return 1
  return "$status"
}
