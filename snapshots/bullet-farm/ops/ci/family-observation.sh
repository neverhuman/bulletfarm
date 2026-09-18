#!/usr/bin/env bash
# Write or validate the deterministic unsigned family component observation.
# The identity excludes its own field and binds canonical semantic outcomes,
# never raw test timings, logs, or rebuilt binary bytes.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

mode="${1:-}"
input="${2:-}"
output="${3:-}"
schema=docs/schemas/bullet.family-ci-observation.v1.schema.json
identity_domain='bullet.family-ci-observation.v1'

[[ "$mode" == write || "$mode" == check ]] \
  || { refuse FAMILY_OBSERVATION_USAGE 'write CANDIDATE OUTPUT | check OBSERVATION'; exit 2; }
[[ -n "$input" && -f "$input" && ! -L "$input" && -s "$input" ]] \
  || { refuse FAMILY_OBSERVATION_MISSING "${input:-missing}"; exit 1; }
[[ -f "$schema" && ! -L "$schema" ]] \
  || { refuse FAMILY_OBSERVATION_SCHEMA_MISSING "$schema"; exit 1; }
[[ "$(b3sum --version)" == 'b3sum 1.8.2' ]] \
  || { refuse FAMILY_OBSERVATION_B3SUM_INVALID "$(b3sum --version 2>&1)"; exit 1; }

python_bin=
if command -v python3 >/dev/null 2>&1; then
  python_bin=python3
elif command -v python >/dev/null 2>&1; then
  python_bin=python
fi
[[ -n "$python_bin" && "$($python_bin --version 2>&1)" == 'Python 3.12.'* ]] \
  || { refuse FAMILY_OBSERVATION_PYTHON_INVALID "${python_bin:-missing}"; exit 1; }

work="$(mktemp -d)"
stage=
cleanup() {
  rm -rf -- "$work"
  [[ -z "$stage" ]] || rm -f -- "$stage"
}
trap cleanup EXIT HUP INT TERM

strict_json() {
  local byte_count
  byte_count="$(wc -c <"$1")"
  [[ "$byte_count" -le 1048576 ]] \
    || { refuse FAMILY_OBSERVATION_JSON_TOO_LARGE "$1"; return 1; }
  bash ops/ci/strict-json.sh "$1" >/dev/null 2>&1 \
    || { refuse FAMILY_OBSERVATION_JSON_INVALID "$1"; return 1; }
}

validate_shape() {
  local document="$1"
  strict_json "$document" || return 1
  jsonschema -i "$document" "$schema" >/dev/null 2>&1 \
    || { refuse FAMILY_OBSERVATION_SCHEMA_INVALID "$document"; return 1; }
  jq -e '
    [.reports[].id] == [
      "bullet-git-fast","bullet-git-contract",
      "bullet-kernel-fast","bullet-kernel-contract","bullet-kernel-family",
      "bullet-portal-vitest","bullet-portal-playwright","bullet-portal-real-farmd",
      "bullet-farm-contract","bullet-farm-formal","bullet-farm-formal-log"
    ] and
    ([.reports[].id] | unique | length) == 11 and
    [.reports[] | [.id,.repository,.summary.kind]] == [
      ["bullet-git-fast","bullet-git","junit"],
      ["bullet-git-contract","bullet-git","junit"],
      ["bullet-kernel-fast","bullet-kernel","junit"],
      ["bullet-kernel-contract","bullet-kernel","junit"],
      ["bullet-kernel-family","bullet-kernel","junit"],
      ["bullet-portal-vitest","bullet-portal","vitest"],
      ["bullet-portal-playwright","bullet-portal","junit"],
      ["bullet-portal-real-farmd","bullet-portal","junit"],
      ["bullet-farm-contract","bullet-farm","junit"],
      ["bullet-farm-formal","bullet-farm","formal"],
      ["bullet-farm-formal-log","bullet-farm","formal-log"]
    ] and
    .sole_writer_daemon.repository == "bullet-git" and
    .sole_writer_daemon.commit_oid == .subjects["bullet-git"].commit_oid and
    .sole_writer_daemon.build == {
      cargo_locked:true,incremental:false,fresh_target:true,offline:true,toolchain:"1.97.1",
      binary_hash_verified_during_run:true
    } and
    all(.reports[] | select(.summary.kind == "junit");
      .summary.executed == .summary.tests and .summary.skipped == 0) and
    all(.reports[] | select(.summary.kind == "vitest");
      .summary.passed == .summary.tests) and
    (.tool_versions.hub_rustc | startswith("rustc 1.95.0 ")) and
    (.tool_versions.hub_cargo | startswith("cargo 1.95.0 ")) and
    (.tool_versions.bullet_git_rustc | startswith("rustc 1.97.1 ")) and
    (.tool_versions.bullet_git_cargo | startswith("cargo 1.97.1 ")) and
    .tool_versions.node == "v22.23.2" and .tool_versions.npm == "10.9.8" and
    .tool_versions.b3sum == "b3sum 1.8.2" and
    .signed == false and .evidence_class == "DIAGNOSTIC_ONLY" and
    .release_authority == false
  ' "$document" >/dev/null \
    || { refuse FAMILY_OBSERVATION_SEMANTICS_INVALID "$document"; return 1; }
}

canonical_without_identity() {
  # Shape validation admits ASCII strings, booleans, and safe integers only;
  # compact sorted jq output is therefore byte-identical to RFC 8785 here.
  jq -cS 'del(.observation_id)' "$1" | "$python_bin" -I -S -c \
    'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().rstrip(b"\n"))'
}

canonical_document() {
  jq -cS . "$1" | "$python_bin" -I -S -c \
    'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().rstrip(b"\n"))'
}

framed_digest() {
  local domain="$1" payload="$2" framed="$work/framed.bin" digest
  "$python_bin" -I -S - "$payload" "$framed" "$domain" <<'PY'
import pathlib
import struct
import sys

payload = pathlib.Path(sys.argv[1]).read_bytes()
domain = sys.argv[3].encode("ascii")
prefix = b"bullet-wire.v1\0"
frame = lambda value: struct.pack("<Q", len(value)) + value
pathlib.Path(sys.argv[2]).write_bytes(prefix + frame(domain) + frame(payload))
PY
  digest="$(b3sum --no-names "$framed")"
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] \
    || { refuse FAMILY_OBSERVATION_IDENTITY_INVALID b3sum; return 1; }
  printf '%s\n' "$digest"
}

identity_for() {
  local document="$1" canonical="$work/identity-subject.json"
  canonical_without_identity "$document" >"$canonical"
  printf 'blake3:%s\n' "$(framed_digest "$identity_domain" "$canonical")"
}

verify_wire_golden() {
  local fixture=fixtures/canonical/canonical-golden.json payload="$work/wire-golden.json"
  local domain expected actual
  strict_json "$fixture" || return 1
  domain="$(jq -er '.domain' "$fixture")" || return 1
  expected="$(jq -er '.framed_blake3' "$fixture")" || return 1
  jq -j '.canonical_json_utf8' "$fixture" >"$payload"
  actual="$(framed_digest "$domain" "$payload")" || return 1
  [[ "$actual" == "$expected" ]] \
    || { refuse FAMILY_OBSERVATION_WIRE_GOLDEN_DRIFT "$fixture"; return 1; }
}

verify_identity_golden() {
  local payload="$work/identity-golden.json"
  local expected='76d3cfd63c6167cf314b7888200073930e14b7581c9f8e35fe4a3d36e1189002'
  printf '%s' '{"a":1}' >"$payload"
  [[ "$(framed_digest "$identity_domain" "$payload")" == "$expected" ]] \
    || { refuse FAMILY_OBSERVATION_IDENTITY_GOLDEN_DRIFT "$identity_domain"; return 1; }
}

verify_wire_golden || exit 1
verify_identity_golden || exit 1

if [[ "$mode" == write ]]; then
  [[ -n "$output" ]] \
    || { refuse FAMILY_OBSERVATION_USAGE 'write requires OUTPUT'; exit 2; }
  strict_json "$input" || exit 1
  jq -e 'has("observation_id") | not' "$input" >/dev/null \
    || { refuse FAMILY_OBSERVATION_CANDIDATE_INVALID observation_id; exit 1; }
  [[ ! -L "$output" ]] \
    || { refuse FAMILY_OBSERVATION_OUTPUT_INVALID "$output"; exit 1; }
  parent="${output%/*}"
  [[ "$parent" != "$output" && -d "$parent" && ! -L "$parent" ]] \
    || { refuse FAMILY_OBSERVATION_OUTPUT_INVALID "$output"; exit 1; }
  placeholder="blake3:$(printf '0%.0s' {1..64})"
  jq -cS --arg identity "$placeholder" '. + {observation_id:$identity}' \
    "$input" >"$work/with-id.json"
  validate_shape "$work/with-id.json" || exit 1
  identity="$(identity_for "$work/with-id.json")" || exit 1
  jq -cS --arg identity "$identity" '.observation_id=$identity' \
    "$work/with-id.json" | "$python_bin" -I -S -c \
      'import sys; sys.stdout.buffer.write(sys.stdin.buffer.read().rstrip(b"\n"))' \
      >"$work/final.json"
  validate_shape "$work/final.json" || exit 1
  [[ "$(identity_for "$work/final.json")" == "$identity" ]] \
    || { refuse FAMILY_OBSERVATION_IDENTITY_INVALID writer; exit 1; }
  stage="$(mktemp "$parent/.family-observation.XXXXXX")" \
    || { refuse FAMILY_OBSERVATION_OUTPUT_INVALID "$output"; exit 1; }
  cp -- "$work/final.json" "$stage"
  mv -- "$stage" "$output"
  stage=
  printf '[ci] family observation wrote %s (%s; unsigned diagnostic only)\n' \
    "$output" "$identity"
  exit 0
fi

[[ -z "$output" ]] \
  || { refuse FAMILY_OBSERVATION_USAGE 'check accepts one OBSERVATION'; exit 2; }
validate_shape "$input" || exit 1
canonical_document "$input" >"$work/canonical-document.json"
cmp -s -- "$input" "$work/canonical-document.json" \
  || { refuse FAMILY_OBSERVATION_NONCANONICAL "$input"; exit 1; }
expected="$(jq -er '.observation_id' "$input")"
actual="$(identity_for "$input")" || exit 1
[[ "$actual" == "$expected" ]] \
  || { refuse FAMILY_OBSERVATION_IDENTITY_MISMATCH "$input"; exit 1; }
printf '[ci] family observation passed (%s; unsigned diagnostic only)\n' "$expected"
