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
  "$build_subject_root/crates/bullet-wire/fuzz" \
  "$build_subject_root/devnode/tui" \
  "$build_subject_root/devnode/record"
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
# The proof-only console harness. It is staged here for the same reason it is
# pinned in lib.sh: a harness that no lane can see is a harness nobody reviews.
cp "$repo_root/devnode/tui/Cargo.toml" "$build_subject_root/devnode/tui/Cargo.toml"
cp "$repo_root/devnode/tui/Cargo.lock" "$build_subject_root/devnode/tui/Cargo.lock"
cp "$repo_root/devnode/record/Cargo.toml" "$build_subject_root/devnode/record/Cargo.toml"
cp "$repo_root/devnode/record/Cargo.lock" "$build_subject_root/devnode/record/Cargo.lock"

enforce_rust_build_subject "$build_subject_root"
for subject in \
  Cargo.toml \
  crates/bullet-linux-lease/Cargo.toml \
  crates/bullet-wire/Cargo.toml \
  crates/bullet-wire/fuzz/Cargo.toml \
  Cargo.lock \
  crates/bullet-wire/fuzz/Cargo.lock \
  rust-toolchain.toml \
  devnode/tui/Cargo.toml \
  devnode/tui/Cargo.lock \
  devnode/record/Cargo.toml \
  devnode/record/Cargo.lock
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

# Proof-only class regressions. Each of these would have passed silently had the
# guard pruned ./devnode from the search instead of pinning what lives there.

# A second harness beside the pinned one is inventory drift, not a free pass.
mkdir -p "$build_subject_root/devnode/second"
printf '%s\n' \
  '[package]' \
  'name = "unreviewed-proof-harness"' \
  'version = "0.0.0"' \
  >"$build_subject_root/devnode/second/Cargo.toml"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/extra-proof-manifest.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: a second devnode manifest was admitted" >&2
  exit 1
fi
grep -Fq CARGO_MANIFEST_INVENTORY_DRIFT \
  "$fixture_root/extra-proof-manifest.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: extra proof manifest failed for an unrelated reason" >&2
  exit 1
}
rm -f -- "$build_subject_root/devnode/second/Cargo.toml"
rmdir -- "$build_subject_root/devnode/second"

# Removing the pinned harness is drift in the other direction.
mv "$build_subject_root/devnode/tui/Cargo.toml" "$fixture_root/proof-manifest.hold"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/missing-proof-manifest.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: a missing proof manifest was admitted" >&2
  exit 1
fi
grep -Fq CARGO_MANIFEST_INVENTORY_DRIFT \
  "$fixture_root/missing-proof-manifest.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: missing proof manifest failed for an unrelated reason" >&2
  exit 1
}
mv "$fixture_root/proof-manifest.hold" "$build_subject_root/devnode/tui/Cargo.toml"
enforce_rust_build_subject "$build_subject_root"

# The proof harness is digest-pinned exactly as a product manifest is, so a
# changed harness dependency is refused rather than absorbed.
printf '%s\n' '' '[dev-dependencies.rustversion]' 'version = "=1.0.23"' \
  >>"$build_subject_root/devnode/tui/Cargo.toml"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/proof-manifest-drift.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: a changed proof manifest was admitted" >&2
  exit 1
fi
grep -Fq RUST_BUILD_SUBJECT_DRIFT "$fixture_root/proof-manifest-drift.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: proof manifest drift failed for an unrelated reason" >&2
  exit 1
}
cp "$repo_root/devnode/tui/Cargo.toml" "$build_subject_root/devnode/tui/Cargo.toml"

# And so is its lock, which is what makes --locked mean anything in that lane.
printf '%s\n' '' '# unreviewed edit' >>"$build_subject_root/devnode/tui/Cargo.lock"
if enforce_rust_build_subject "$build_subject_root" \
  >"$fixture_root/proof-lock-drift.log" 2>&1; then
  echo "RUST_BUILD_SUBJECT_CANARY_FAILED: a changed proof lock was admitted" >&2
  exit 1
fi
grep -Fq RUST_BUILD_SUBJECT_DRIFT "$fixture_root/proof-lock-drift.log" || {
  echo "RUST_BUILD_SUBJECT_CANARY_INVALID: proof lock drift failed for an unrelated reason" >&2
  exit 1
}
cp "$repo_root/devnode/tui/Cargo.lock" "$build_subject_root/devnode/tui/Cargo.lock"
cp "$repo_root/devnode/record/Cargo.toml" "$build_subject_root/devnode/record/Cargo.toml"
cp "$repo_root/devnode/record/Cargo.lock" "$build_subject_root/devnode/record/Cargo.lock"
enforce_rust_build_subject "$build_subject_root"

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
