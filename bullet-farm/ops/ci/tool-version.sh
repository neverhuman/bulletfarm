#!/usr/bin/env bash
# Shared closed vocabulary and lexical grammar for sanitized CI tool versions.

CI_TOOL_KEYS='actionlint b3sum cargo cargo_deny cargo_nextest cargo_llvm_cov cargo_pinned docker file git gitleaks jankurai java jsonschema lychee node npm python rustc rustc_pinned rustup shellcheck zizmor'

ci_tool_key_is_known() {
  local known
  for known in $CI_TOOL_KEYS; do
    [[ "$1" == "$known" ]] && return 0
  done
  return 1
}

ci_tool_version_shape_is_valid() {
  local key="$1" value="$2" pattern
  ci_tool_key_is_known "$key" || return 1
  [[ ${#value} -le 160 ]] || return 1
  pattern='^[[:print:]]{1,160}$'
  [[ "$value" =~ $pattern ]] || return 1
  case "$key" in
    git) pattern='^git version [0-9]+\.[0-9]+(\.[0-9]+)?(\.windows\.[0-9]+)?$' ;;
    rustc|rustc_pinned) pattern='^rustc [0-9]+\.[0-9]+\.[0-9]+ \([0-9a-f]{9,40} [0-9]{4}-[0-9]{2}-[0-9]{2}\)$' ;;
    cargo|cargo_pinned) pattern='^cargo [0-9]+\.[0-9]+\.[0-9]+ \([0-9a-f]{9,40} [0-9]{4}-[0-9]{2}-[0-9]{2}\)$' ;;
    cargo_nextest) pattern='^cargo-nextest [0-9]+\.[0-9]+\.[0-9]+ \([0-9a-f]{9,40} [0-9]{4}-[0-9]{2}-[0-9]{2}\)$' ;;
    actionlint|gitleaks|jsonschema|npm|shellcheck) pattern='^[0-9]+\.[0-9]+\.[0-9]+$' ;;
    b3sum) pattern='^b3sum [0-9]+\.[0-9]+\.[0-9]+$' ;;
    cargo_deny) pattern='^cargo-deny [0-9]+\.[0-9]+\.[0-9]+$' ;;
    cargo_llvm_cov) pattern='^cargo-llvm-cov [0-9]+\.[0-9]+\.[0-9]+$' ;;
    docker) pattern='^Docker version [0-9]+\.[0-9]+\.[0-9]+, build [0-9a-f]{7,40}$' ;;
    file) pattern='^file[- ][0-9]+\.[0-9]+$' ;;
    jankurai) pattern='^jankurai [0-9]+\.[0-9]+\.[0-9]+$' ;;
    java) pattern='^(openjdk|java) version "21\.[0-9]+(\.[0-9]+)?"( [0-9]{4}-[0-9]{2}-[0-9]{2})?( LTS)?$' ;;
    lychee) pattern='^lychee [0-9]+\.[0-9]+\.[0-9]+$' ;;
    node) pattern='^v[0-9]+\.[0-9]+\.[0-9]+$' ;;
    python) pattern='^Python [0-9]+\.[0-9]+\.[0-9]+$' ;;
    rustup) pattern='^rustup [0-9]+\.[0-9]+\.[0-9]+ \([0-9a-f]{9,40} [0-9]{4}-[0-9]{2}-[0-9]{2}\)$' ;;
    zizmor) pattern='^zizmor [0-9]+\.[0-9]+\.[0-9]+$' ;;
    *) return 1 ;;
  esac
  [[ "$value" =~ $pattern ]]
}
