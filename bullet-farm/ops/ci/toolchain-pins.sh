#!/usr/bin/env bash
# Exact Node/npm subjects shared by Hub shell entrypoints. This file reads data;
# it never evaluates pin contents as shell source.
set -euo pipefail

toolchain_pin_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

read_toolchain_pin() {
  local path="$1" value bytes
  [[ -f "$path" && ! -L "$path" ]] || return 1
  IFS= read -r value <"$path" || return 1
  [[ "$value" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || return 1
  bytes="$(wc -c <"$path" | tr -d '[:space:]')"
  [[ "$bytes" == "$((${#value} + 1))" ]] || return 1
  printf '%s\n' "$value"
}

toolchain_pin_failure() {
  printf 'TOOLCHAIN_PIN_INVALID: %s\n' "$1" >&2
  return 1
}

PINNED_NODE_VERSION="$(read_toolchain_pin "$toolchain_pin_root/.node-version")" \
  || toolchain_pin_failure .node-version
PINNED_NPM_VERSION="$(read_toolchain_pin "$toolchain_pin_root/.npm-version")" \
  || toolchain_pin_failure .npm-version
readonly PINNED_NODE_VERSION PINNED_NPM_VERSION
export PINNED_NODE_VERSION PINNED_NPM_VERSION
