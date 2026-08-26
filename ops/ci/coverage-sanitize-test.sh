#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
fixtures="$(mktemp -d)"
cleanup() { rm -rf -- "$fixtures"; }
trap cleanup EXIT
canonical_doctype='<!DOCTYPE coverage SYSTEM "https://cobertura.sourceforge.net/xml/coverage-04.dtd">'

write_fixture() {
  local source="$1" filename="$2" lines_valid="${3:-1}" lines_covered="${4:-1}"
  local line_rate="${5:-1}" branches_valid="${6:-0}" branches_covered="${7:-0}"
  local branch_rate="${8:-0}"
  printf '<coverage lines-valid="%s" lines-covered="%s" line-rate="%s" branches-valid="%s" branches-covered="%s" branch-rate="%s"><sources><source>%s</source></sources><packages><package name="fixture"><classes><class name="fixture" filename="%s"/></classes></package></packages></coverage>\n' \
    "$lines_valid" "$lines_covered" "$line_rate" "$branches_valid" "$branches_covered" \
    "$branch_rate" "$source" "$filename" >"$fixtures/report.xml"
}
expect_refusal() {
  local code="$1" output status
  set +e
  output="$(bash ops/ci/coverage-sanitize.sh check "$fixtures/report.xml" 2>&1)"
  status=$?
  set -e
  [[ "$status" -ne 0 && "$output" == *"$code"* ]] \
    || { refuse COVERAGE_NEGATIVE_FAILED "$code status=$status output=$output"; exit 1; }
}
write_raw_fixture() {
  local doctype="${1:-$canonical_doctype}" source="${2:-$REPO_ROOT}"
  printf '%s\n' '<?xml version="1.0" ?>' "$doctype" \
    '<coverage lines-valid="1" lines-covered="1" line-rate="1" branches-valid="0" branches-covered="0" branch-rate="0">' \
    '    <sources>' "        <source>$source</source>" '    </sources>' \
    '    <packages><package name="fixture"><classes><class name="fixture" filename="src/lib.rs"/></classes></package></packages>' \
    '</coverage>' >"$fixtures/raw.xml"
}
expect_normalize_refusal() {
  local code="$1" input="${2:-$fixtures/raw.xml}" output="${3:-$fixtures/normalize-negative.xml}"
  local message status
  set +e
  message="$(bash ops/ci/coverage-sanitize.sh normalize "$input" "$output" 2>&1)"
  status=$?
  set -e
  [[ "$status" -ne 0 && "$message" == *"$code"* ]] \
    || { refuse COVERAGE_NORMALIZE_NEGATIVE_FAILED "$code status=$status output=$message"; exit 1; }
}

write_raw_fixture
cp "$fixtures/raw.xml" "$fixtures/raw.before.xml"
bash ops/ci/coverage-sanitize.sh normalize "$fixtures/raw.xml" "$fixtures/normalized.xml" >/dev/null
cmp -s "$fixtures/raw.xml" "$fixtures/raw.before.xml" \
  || { refuse COVERAGE_NORMALIZE_MUTATED_INPUT "$fixtures/raw.xml"; exit 1; }
bash ops/ci/coverage-sanitize.sh check "$fixtures/normalized.xml" >/dev/null
[[ "$(grep -Foc '<source>.</source>' "$fixtures/normalized.xml")" -eq 1 ]] \
  || { refuse COVERAGE_NORMALIZE_SOURCE_INVALID "$fixtures/normalized.xml"; exit 1; }
if grep -Fq "$REPO_ROOT" "$fixtures/normalized.xml" \
  || grep -Eq '<![[:space:]]*(DOCTYPE|ENTITY)' "$fixtures/normalized.xml"; then
  refuse COVERAGE_NORMALIZE_LEAK "$fixtures/normalized.xml"; exit 1
fi
if output="$(bash ops/ci/coverage-sanitize.sh check "$fixtures/raw.xml" 2>&1)" \
  || [[ "$output" != *COVERAGE_REPORT_INVALID* ]]; then
  refuse COVERAGE_RAW_CHECK_GUARD_FAILED "$output"; exit 1
fi
expect_normalize_refusal COVERAGE_OUTPUT_ALIASES_INPUT "$fixtures/raw.xml" "$fixtures/raw.xml"
printf 'preserve me\n' >"$fixtures/existing.xml"
cp "$fixtures/existing.xml" "$fixtures/existing.before.xml"
expect_normalize_refusal COVERAGE_OUTPUT_INVALID "$fixtures/raw.xml" "$fixtures/existing.xml"
cmp -s "$fixtures/existing.xml" "$fixtures/existing.before.xml" \
  || { refuse COVERAGE_NORMALIZE_OVERWROTE_OUTPUT "$fixtures/existing.xml"; exit 1; }
write_raw_fixture '<!DOCTYPE coverage SYSTEM "https://invalid.example/coverage.dtd">'
expect_normalize_refusal COVERAGE_DOCTYPE_INVALID
write_raw_fixture "$canonical_doctype" /opt/private/build
expect_normalize_refusal COVERAGE_SOURCE_INVALID
write_raw_fixture
ln -s "$fixtures/raw.before.xml" "$fixtures/normalized-link.xml"
expect_normalize_refusal COVERAGE_OUTPUT_INVALID "$fixtures/raw.xml" "$fixtures/normalized-link.xml"

write_fixture . src/lib.rs
bash ops/ci/coverage-sanitize.sh check "$fixtures/report.xml" >/dev/null
write_fixture /home/runner/work /home/runner/work/src/lib.rs
expect_refusal COVERAGE_PATH_LEAK
write_fixture /Users/runner/work /Users/runner/work/src/lib.rs
expect_refusal COVERAGE_PATH_LEAK
write_fixture 'C:\runner\work' 'C:\runner\work\src\lib.rs'
expect_refusal COVERAGE_PATH_LEAK
write_fixture /tmp/build /tmp/build/src/lib.rs
expect_refusal COVERAGE_PATH_LEAK
write_fixture ../workspace src/lib.rs
expect_refusal COVERAGE_PATH_LEAK
write_fixture . src/../lib.rs
expect_refusal COVERAGE_PATH_LEAK
write_fixture . 'src&#47;lib.rs'
expect_refusal COVERAGE_PATH_LEAK

write_fixture . src/lib.rs 0 0 0
expect_refusal COVERAGE_REPORT_EMPTY
write_fixture . src/lib.rs
valid_report="$(<"$fixtures/report.xml")"
printf '%s%s\n' "$valid_report" "$valid_report" >"$fixtures/report.xml"
expect_refusal COVERAGE_REPORT_INVALID
printf '<coverage lines-valid="1" lines-covered="1" line-rate="1" branches-valid="0" branches-covered="0" branch-rate="0">\n' >"$fixtures/report.xml"
expect_refusal COVERAGE_REPORT_INVALID
write_fixture . src/lib.rs 1 2 1
expect_refusal COVERAGE_REPORT_INVALID
write_fixture . src/lib.rs 2 1 1
expect_refusal COVERAGE_REPORT_INVALID
write_fixture . src/lib.rs 1 1 1 2 1 1
expect_refusal COVERAGE_REPORT_INVALID
printf '<coverage lines-valid="1" lines-covered="1" line-rate="1" branches-valid="0" branches-covered="0" branch-rate="0"><packages/></coverage>\n' >"$fixtures/report.xml"
expect_refusal COVERAGE_REPORT_EMPTY
log "coverage sanitizer hostile path and semantic matrix passed"
