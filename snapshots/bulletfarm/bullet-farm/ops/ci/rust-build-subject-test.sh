#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
lib_path="$script_dir/lib.sh"
# shellcheck source=ops/ci/lib.sh
source "$lib_path"
repo_root="$REPO_ROOT"
fixture_root="$(mktemp -d "${TMPDIR:-/tmp}/bullet-rust-build-subject.XXXXXX")"
trap 'rm -rf -- "$fixture_root"' EXIT

build_subject_root="$fixture_root/build-subject"
mkdir -p \
  "$build_subject_root/crates/bullet-linux-lease" \
  "$build_subject_root/crates/bullet-wire/fuzz"
cp "$repo_root/Cargo.toml" "$build_subject_root/Cargo.toml"
cp "$repo_root/crates/bullet-linux-lease/Cargo.toml" \
  "$build_subject_root/crates/bullet-linux-lease/Cargo.toml"
cp "$repo_root/crates/bullet-wire/Cargo.toml" \
  "$build_subject_root/crates/bullet-wire/Cargo.toml"
cp "$repo_root/crates/bullet-wire/fuzz/Cargo.toml" \
  "$build_subject_root/crates/bullet-wire/fuzz/Cargo.toml"
cp "$repo_root/Cargo.lock" "$build_subject_root/Cargo.lock"
cp "$repo_root/crates/bullet-wire/fuzz/Cargo.lock" \
  "$build_subject_root/crates/bullet-wire/fuzz/Cargo.lock"
cp "$repo_root/rust-toolchain.toml" "$build_subject_root/rust-toolchain.toml"

enforce_rust_build_subject "$build_subject_root"
for subject in \
  Cargo.toml \
  crates/bullet-linux-lease/Cargo.toml \
  crates/bullet-wire/Cargo.toml \
  crates/bullet-wire/fuzz/Cargo.toml \
  Cargo.lock \
  crates/bullet-wire/fuzz/Cargo.lock \
  rust-toolchain.toml
do
  lf_digest="$(sha256_lf_text_file "$build_subject_root/$subject")"
  crlf_lines=0
  while IFS= read -r line || [[ -n "$line" ]]; do
    printf '%s\r\n' "$line"
    crlf_lines=$((crlf_lines + 1))
  done <"$build_subject_root/$subject" >"$build_subject_root/$subject.crlf"
  [[ "$crlf_lines" -gt 0 ]] || {
    echo "RUST_BUILD_SUBJECT_CANARY_INVALID: empty CRLF fixture $subject" >&2
    exit 1
  }
  observed_lines=0
  while IFS= read -r line; do
    [[ "$line" == *$'\r' ]] || {
      echo "RUST_BUILD_SUBJECT_CANARY_INVALID: non-CRLF line in $subject" >&2
      exit 1
    }
    observed_lines=$((observed_lines + 1))
  done <"$build_subject_root/$subject.crlf"
  [[ "$observed_lines" -eq "$crlf_lines" ]] || {
    echo "RUST_BUILD_SUBJECT_CANARY_INVALID: CRLF line count drift in $subject" >&2
    exit 1
  }
  crlf_digest="$(sha256_lf_text_file "$build_subject_root/$subject.crlf")"
  [[ "$crlf_digest" == "$lf_digest" ]] || {
    echo "RUST_BUILD_SUBJECT_CANARY_INVALID: CRLF normalization digest drift in $subject" >&2
    exit 1
  }
  mv "$build_subject_root/$subject.crlf" "$build_subject_root/$subject"
done
enforce_rust_build_subject "$build_subject_root"

printf 'line-one\rline-two\n' >"$build_subject_root/lone-cr.txt"
if sha256_lf_text_file "$build_subject_root/lone-cr.txt" \
  >"$fixture_root/lone-cr.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: lone carriage return was admitted" >&2
  exit 1
fi
grep -Fq RUST_BUILD_SUBJECT_TEXT_INVALID "$fixture_root/lone-cr.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: lone carriage return failed for an unrelated reason" >&2
  exit 1
}

mkdir -p "$build_subject_root/unexpected"
printf '%s\n' \
  '[package]' \
  'name = "unexpected-build-subject"' \
  'version = "0.0.0"' \
  >"$build_subject_root/unexpected/Cargo.toml"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/unexpected-manifest.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: unexpected Cargo.toml was admitted" >&2
  exit 1
fi
grep -Fq CARGO_MANIFEST_INVENTORY_DRIFT \
  "$fixture_root/unexpected-manifest.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: unexpected manifest failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$build_subject_root/unexpected/Cargo.toml"
rmdir -- "$build_subject_root/unexpected"

printf '%s\n' '' '[dependencies.rustversion]' 'version = "=1.0.23"' \
  >>"$build_subject_root/crates/bullet-wire/Cargo.toml"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/build-subject.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: changed direct dependency was admitted" >&2
  exit 1
fi
grep -Fq RUST_BUILD_SUBJECT_DRIFT "$fixture_root/build-subject.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: dependency change failed for an unrelated reason" >&2
  exit 1
}

echo "Rust build-subject canary: PASS"
