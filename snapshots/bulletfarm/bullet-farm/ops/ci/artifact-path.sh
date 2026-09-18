#!/usr/bin/env bash
# Shared lexical and symlink guard for sanitized CI artifacts. Callers own the
# typed refusal so this helper can be sourced by both producer and validator.

ci_path_within() {
  local root="$1" path="$2"
  case "$path" in
    "$root"|"$root"/*) return 0 ;;
    *) return 1 ;;
  esac
}

ci_canonical_directory() {
  local directory="$1"
  [[ -d "$directory" && ! -L "$directory" ]] || return 1
  (cd -P "$directory" && pwd)
}

prepare_ci_directory() {
  local root="$1" relative="$2" canonical_root cursor resolved segment
  local -a segments
  canonical_root="$(ci_canonical_directory "$root")" || return 1
  [[ -d "$canonical_root" && ! -L "$root" ]] || return 1
  case "$relative" in
    .ci-artifacts|.ci-artifacts/*|.ci-upload|.ci-upload/*) ;;
    *) return 1 ;;
  esac
  IFS=/ read -r -a segments <<<"$relative"
  cursor="$canonical_root"
  for segment in "${segments[@]}"; do
    [[ "$segment" =~ ^[A-Za-z0-9._-]+$ && "$segment" != . && "$segment" != .. ]] || return 1
    cursor="$cursor/$segment"
    [[ ! -L "$cursor" ]] || return 1
    if [[ -e "$cursor" ]]; then
      [[ -d "$cursor" ]] || return 1
    else
      mkdir -- "$cursor" || return 1
    fi
    resolved="$(ci_canonical_directory "$cursor")" || return 1
    ci_path_within "$canonical_root" "$resolved" || return 1
  done
}

validate_ci_artifact_path() {
  local path="$1" remainder cursor segment
  case "$path" in
    ''|/*|.ci-artifacts|.ci-artifacts/|*//*|*/.|*/./*|*/..|*/../*) return 1 ;;
  esac
  [[ "$path" == .ci-artifacts/* && -d .ci-artifacts && ! -L .ci-artifacts ]] || return 1
  remainder="${path#.ci-artifacts/}"
  cursor=.ci-artifacts
  while [[ -n "$remainder" ]]; do
    if [[ "$remainder" == */* ]]; then
      segment="${remainder%%/*}"
      remainder="${remainder#*/}"
    else
      segment="$remainder"
      remainder=""
    fi
    [[ -n "$segment" && "$segment" != . && "$segment" != .. ]] || return 1
    cursor="$cursor/$segment"
    [[ ! -L "$cursor" ]] || return 1
  done
  [[ "$cursor" == "$path" && -f "$cursor" ]]
}
