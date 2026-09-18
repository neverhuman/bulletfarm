#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"

usage() {
  refuse COVERAGE_USAGE "coverage-sanitize.sh check REPORT | normalize INPUT OUTPUT"
  exit 2
}

check_report() {
  local report="$1" root_line lines_valid lines_covered branches_valid branches_covered
  local line_rate branch_rate source filename
  local -a sources filenames
  [[ -f "$report" && ! -L "$report" && -s "$report" ]] \
    || { refuse COVERAGE_REPORT_MISSING "$report"; exit 1; }
  token_count() {
    { grep -Eo "$1" "$report" || true; } | wc -l | tr -d ' '
  }
  [[ "$(token_count '<coverage[[:space:]]')" -eq 1 \
    && "$(token_count '</coverage>')" -eq 1 ]] \
    || { refuse COVERAGE_REPORT_INVALID "coverage root is missing or ambiguous"; exit 1; }
  if grep -Eiq '<![[:space:]]*(DOCTYPE|ENTITY)' "$report"; then
    refuse COVERAGE_REPORT_INVALID "DTD and entity declarations are forbidden"
    exit 1
  fi
  root_line="$(grep -m1 '<coverage[[:space:]]' "$report")"
  xml_attribute() {
    local name="$1" matches count value
    matches="$(grep -oE "[[:space:]]${name}=\"[^\"]+\"" <<<"$root_line" || true)"
    count="$(grep -c . <<<"$matches")"
    [[ "$count" -eq 1 ]] \
      || { refuse COVERAGE_REPORT_INVALID "expected one $name attribute"; return 1; }
    value="${matches#*=\"}"
    value="${value%\"}"
    printf '%s\n' "$value"
  }
  lines_valid="$(xml_attribute lines-valid)"
  lines_covered="$(xml_attribute lines-covered)"
  branches_valid="$(xml_attribute branches-valid)"
  branches_covered="$(xml_attribute branches-covered)"
  line_rate="$(xml_attribute line-rate)"
  branch_rate="$(xml_attribute branch-rate)"
  [[ "$lines_valid" =~ ^[1-9][0-9]*$ && "$lines_covered" =~ ^[0-9]+$ \
    && "$branches_valid" =~ ^[0-9]+$ && "$branches_covered" =~ ^[0-9]+$ ]] \
    || { refuse COVERAGE_REPORT_EMPTY \
      "lines=$lines_covered/$lines_valid branches=$branches_covered/$branches_valid"; exit 1; }
  awk -v valid="$lines_valid" -v covered="$lines_covered" -v line_rate="$line_rate" \
    -v branches_valid="$branches_valid" -v branches_covered="$branches_covered" \
    -v branch_rate="$branch_rate" 'BEGIN {
      expected_line_rate = covered / valid
      expected_branch_rate = branches_valid == 0 ? 0 : branches_covered / branches_valid
      line_delta = line_rate - expected_line_rate
      branch_delta = branch_rate - expected_branch_rate
      if (line_delta < 0) line_delta = -line_delta
      if (branch_delta < 0) branch_delta = -branch_delta
      if (covered > valid || line_rate !~ /^(0|1)(\.[0-9]+)?$/ ||
          branches_covered > branches_valid || branch_rate !~ /^(0|1)(\.[0-9]+)?$/ ||
          line_rate > 1 || branch_rate > 1 || line_delta > 0.000001 ||
          branch_delta > 0.000001) exit 1
    }' || { refuse COVERAGE_REPORT_INVALID "inconsistent coverage counters or rates"; exit 1; }
  mapfile -t sources < <(
    grep -Eo '<source>[^<]*</source>' "$report" | sed -E 's#^<source>(.*)</source>$#\1#'
  )
  mapfile -t filenames < <(
    grep -Eo 'filename="[^"]+"' "$report" | sed -E 's/^filename="(.*)"$/\1/'
  )
  [[ "$(token_count '<source>')" -eq "${#sources[@]}" \
    && "$(token_count 'filename=')" -eq "${#filenames[@]}" ]] \
    || { refuse COVERAGE_REPORT_INVALID "noncanonical source or filename field"; exit 1; }
  [[ "$(token_count '<package[[:space:]]')" -gt 0 && "${#filenames[@]}" -gt 0 ]] \
    || { refuse COVERAGE_REPORT_EMPTY "no package/class subjects"; exit 1; }
  validate_relative_path() {
    local kind="$1" value="$2" segment
    local -a segments
    if [[ "$kind" == source && "$value" == . ]]; then
      return 0
    fi
    case "$value" in
      ''|/*|\\*|[A-Za-z]:*|*\\*|*//*) return 1 ;;
    esac
    [[ "$value" != *'&'* ]] || return 1
    IFS=/ read -r -a segments <<<"$value"
    for segment in "${segments[@]}"; do
      [[ -n "$segment" && "$segment" != . && "$segment" != .. ]] || return 1
    done
  }
  for source in "${sources[@]}"; do
    validate_relative_path source "$source" \
      || { refuse COVERAGE_PATH_LEAK "source=$source"; exit 1; }
  done
  for filename in "${filenames[@]}"; do
    validate_relative_path filename "$filename" \
      || { refuse COVERAGE_PATH_LEAK "filename=$filename"; exit 1; }
  done
  if grep -Fq "$REPO_ROOT" "$report"; then
    refuse COVERAGE_PATH_LEAK "repository root"
    exit 1
  fi
  if grep -Eq '(<source>[[:space:]]*/|filename="/|filename="[A-Za-z]:[/\\]|/home/|/Users/|/tmp/|[A-Za-z]:\\)' "$report"; then
    refuse COVERAGE_PATH_LEAK "$report"
    exit 1
  fi
  log "coverage paths are repository-relative and sanitized"
}

normalize_report() {
  local input="$1" output="$2" normalized_tmp canonical_doctype expected_source
  [[ -f "$input" && ! -L "$input" && -s "$input" ]] \
    || { refuse COVERAGE_REPORT_MISSING "$input"; exit 1; }
  if [[ "$input" -ef "$output" ]] \
    || [[ "$(realpath -m -- "$input")" == "$(realpath -m -- "$output")" ]]; then
    refuse COVERAGE_OUTPUT_ALIASES_INPUT "$output"
    exit 1
  fi
  [[ ! -e "$output" && ! -L "$output" ]] \
    || { refuse COVERAGE_OUTPUT_INVALID "$output"; exit 1; }
  canonical_doctype='<!DOCTYPE coverage SYSTEM "https://cobertura.sourceforge.net/xml/coverage-04.dtd">'
  expected_source="<source>$REPO_ROOT</source>"
  [[ "$({ grep -Eo '<![[:space:]]*(DOCTYPE|ENTITY)' "$input" || true; } | wc -l | tr -d ' ')" -eq 1 \
    && "$({ grep -Fxo "$canonical_doctype" "$input" || true; } | wc -l | tr -d ' ')" -eq 1 ]] \
    || { refuse COVERAGE_DOCTYPE_INVALID "$input"; exit 1; }
  [[ "$({ grep -Foh '<source>' "$input" || true; } | wc -l | tr -d ' ')" -eq 1 \
    && "$({ grep -Fo "$expected_source" "$input" || true; } | wc -l | tr -d ' ')" -eq 1 ]] \
    || { refuse COVERAGE_SOURCE_INVALID "$input"; exit 1; }
  normalized_tmp="$(mktemp "${output}.normalize.XXXXXX")"
  cleanup_normalized() { rm -f -- "$normalized_tmp"; }
  trap cleanup_normalized EXIT
  awk -v doctype="$canonical_doctype" -v source="$expected_source" '
    $0 == doctype { next }
    {
      start = index($0, source)
      if (start > 0) {
        $0 = substr($0, 1, start - 1) "<source>.</source>" substr($0, start + length(source))
      }
      print
    }
  ' "$input" >"$normalized_tmp"
  check_report "$normalized_tmp"
  ln -- "$normalized_tmp" "$output" \
    || { refuse COVERAGE_OUTPUT_RACE "$output"; exit 1; }
  rm -f -- "$normalized_tmp"
  trap - EXIT
  log "coverage report normalized"
}

case "${1:-}" in
  check)
    [[ "$#" -eq 2 ]] || usage
    check_report "$2"
    ;;
  normalize)
    [[ "$#" -eq 3 ]] || usage
    normalize_report "$2" "$3"
    ;;
  *) usage ;;
esac
