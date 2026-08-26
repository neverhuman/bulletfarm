#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/toolchain-pins.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/toolchain-pins.sh"
lane="${1:-all}"
baseline=(awk bash dirname env git head jq mkdir mv realpath rm sed sort tr)
case "$lane" in
  source-scan) tools=("${baseline[@]}" cat gitleaks xargs) ;;
  fast) tools=("${baseline[@]}" cargo cargo-nextest chmod grep mktemp rmdir rustc) ;;
  lint) tools=("${baseline[@]}" actionlint b3sum cargo cargo-clippy cargo-nextest cmp comm cp find jsonschema ln rg rustc rustfmt shellcheck wc) ;;
  contract) tools=("${baseline[@]}" cargo cargo-nextest curl find grep java mktemp rustc sha1sum tee) ;;
  security) tools=("${baseline[@]}" cargo cargo-deny date gitleaks mktemp rustc zizmor) ;;
  docs) tools=("${baseline[@]}" cargo cmp cp docker file find grep id mktemp realpath rg rustc stat) ;;
  required) tools=("${baseline[@]}" actionlint b3sum cargo cargo-clippy cargo-deny cargo-nextest chmod cmp comm cp curl date docker file find gitleaks grep id java jsonschema ln mktemp realpath rg rmdir rustc rustfmt sha1sum shellcheck stat tee wc zizmor) ;;
  family|family-contract) tools=("${baseline[@]}" actionlint b3sum cargo cargo-clippy cargo-deny cargo-nextest chmod cmp comm cp curl date docker file find gitleaks grep id java jsonschema ln mktemp node npm rg rmdir rustc rustfmt rustup sha1sum shellcheck stat tee uname wc zizmor) ;;
  history) tools=("${baseline[@]}" cat gitleaks xargs) ;;
  links) tools=("${baseline[@]}" lychee rg) ;;
  advisory) tools=("${baseline[@]}" cargo cargo-deny date rustc) ;;
  coverage) tools=("${baseline[@]}" cargo cargo-llvm-cov cargo-nextest cmp comm grep ln mktemp rustc wc) ;;
  platform) tools=("${baseline[@]}" cargo cargo-clippy grep rustc uname) ;;
  audit) tools=("${baseline[@]}" jankurai) ;;
  toolchain-pinned) tools=("${baseline[@]}" b3sum cargo date grep rustc rustup tee wc) ;;
  all) tools=("${baseline[@]}" actionlint b3sum cargo cargo-clippy cargo-deny cargo-llvm-cov cargo-nextest cat chmod cmp comm cp curl date docker file find gitleaks grep id jankurai java jsonschema ln lychee mktemp node npm realpath rg rmdir rustc rustfmt rustup sha1sum shellcheck stat tee uname wc xargs zizmor) ;;
  *)
    echo "ci-doctor: expected source-scan|fast|lint|contract|security|docs|required|family|family-contract|history|links|advisory|coverage|platform|audit|toolchain-pinned|all" >&2
    exit 2
    ;;
esac

python_bin=
if command -v python3 >/dev/null 2>&1; then
  python_bin=python3
elif command -v python >/dev/null 2>&1; then
  python_bin=python
else
  printf 'ci-doctor: missing python3 or python for %s\n' "$lane" >&2
fi

missing=0
for tool in "${tools[@]}"; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    printf 'ci-doctor: missing %s for %s\n' "$tool" "$lane" >&2
    missing=1
  fi
done
[[ -n "$python_bin" ]] || missing=1
if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
  printf 'ci-doctor: missing sha256sum or shasum for %s\n' "$lane" >&2
  missing=1
fi
[[ "$missing" -eq 0 ]] || exit 1

python_version="$("$python_bin" --version 2>&1)"
[[ "$python_version" == "Python 3.12."* ]] || {
  printf 'ci-doctor: expected Python 3.12, found %s\n' "$python_version" >&2
  exit 1
}

if [[ "$lane" =~ ^(fast|lint|contract|security|docs|required|family|family-contract|advisory|coverage|platform|toolchain-pinned|all)$ ]]; then
  rust_version="$(rustc --version)"
  [[ "$rust_version" == "rustc 1.95.0 "* ]] || {
    printf 'ci-doctor: expected rustc 1.95.0, found %s\n' "$rust_version" >&2
    exit 1
  }
fi
if [[ "$lane" =~ ^(fast|lint|contract|required|family|family-contract|coverage|all)$ ]]; then
  nextest_version="$(cargo-nextest --version | head -n 1)"
  [[ "$nextest_version" == "cargo-nextest 0.9.137 "* ]] || {
    printf 'ci-doctor: expected cargo-nextest 0.9.137, found %s\n' "$nextest_version" >&2
    exit 1
  }
fi
if [[ "$lane" =~ ^(source-scan|security|required|family|family-contract|history|all)$ ]]; then
  [[ "$(gitleaks version)" == 8.21.2 ]] || { echo "ci-doctor: expected gitleaks 8.21.2" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(security|required|family|family-contract|advisory|all)$ ]]; then
  [[ "$(cargo-deny --version)" == "cargo-deny 0.19.8" ]] || { echo "ci-doctor: expected cargo-deny 0.19.8" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(lint|required|family|family-contract|all)$ ]]; then
  [[ "$(actionlint -version | head -n 1)" == 1.7.8 ]] || { echo "ci-doctor: expected actionlint 1.7.8" >&2; exit 1; }
  [[ "$(shellcheck --version | awk '/^version:/{print $2}')" == 0.10.0 ]] || { echo "ci-doctor: expected ShellCheck 0.10.0" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(security|required|family|family-contract|all)$ ]]; then
  [[ "$(zizmor --version)" == "zizmor 1.25.2" ]] || { echo "ci-doctor: expected zizmor 1.25.2" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(contract|required|family|family-contract|all)$ ]]; then
  java_version="$(java -version 2>&1 | head -n 1)"
  [[ "$java_version" == *'"21.'* ]] || { printf 'ci-doctor: expected Java 21, found %s\n' "$java_version" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(lint|docs|required|family|family-contract|all)$ ]]; then
  jsonschema_version="$("$python_bin" -c 'from importlib.metadata import version; print(version("jsonschema"))' 2>/dev/null || true)"
  [[ "$jsonschema_version" == 4.26.0 ]] \
    || { printf 'ci-doctor: expected jsonschema 4.26.0, found %s\n' "${jsonschema_version:-missing}" >&2; exit 1; }
fi
if [[ "$lane" =~ ^(lint|required|family|family-contract|all)$ ]]; then
  [[ "$(jsonschema --version)" == 4.26.0 ]] \
    || { echo "ci-doctor: expected jsonschema CLI 4.26.0" >&2; exit 1; }
  [[ "$(b3sum --version)" == "b3sum 1.8.2" ]] \
    || { echo "ci-doctor: expected b3sum 1.8.2 for $lane" >&2; exit 1; }
fi
if [[ "$lane" == links || "$lane" == all ]]; then
  [[ "$(lychee --version)" == "lychee 0.24.0" ]] || { echo "ci-doctor: expected lychee 0.24.0" >&2; exit 1; }
fi
if [[ "$lane" == coverage || "$lane" == all ]]; then
  [[ "$(cargo llvm-cov --version)" == "cargo-llvm-cov 0.8.7" ]] || { echo "ci-doctor: expected cargo-llvm-cov 0.8.7" >&2; exit 1; }
fi
if [[ "$lane" == family || "$lane" == family-contract || "$lane" == all ]]; then
  [[ "$(node --version)" == "v$PINNED_NODE_VERSION" ]] \
    || { printf 'ci-doctor: expected Node v%s, found %s\n' "$PINNED_NODE_VERSION" "$(node --version)" >&2; exit 1; }
  [[ "$(npm --version)" == "$PINNED_NPM_VERSION" ]] \
    || { printf 'ci-doctor: expected npm %s, found %s\n' "$PINNED_NPM_VERSION" "$(npm --version)" >&2; exit 1; }
  [[ "$(rustup --version 2>/dev/null | head -n 1)" == "rustup 1.29.0 "* ]] \
    || { echo "ci-doctor: expected rustup 1.29.0 for $lane" >&2; exit 1; }
  rustup toolchain list | grep -q '^1\.97\.1-' \
    || { echo "ci-doctor: Rust 1.97.1 toolchain is missing for $lane" >&2; exit 1; }
  [[ "$(rustup run 1.97.1 rustc --version)" == "rustc 1.97.1 "* ]] \
    || { echo "ci-doctor: invalid Rust 1.97.1 toolchain for $lane" >&2; exit 1; }
  [[ "$(rustup run 1.97.1 cargo --version)" == "cargo 1.97.1 "* ]] \
    || { echo "ci-doctor: invalid Cargo 1.97.1 toolchain for $lane" >&2; exit 1; }
fi
if [[ "$lane" == audit || "$lane" == all ]]; then
  [[ "$(jankurai --version)" == "jankurai 1.6.11" ]] || { echo "ci-doctor: expected jankurai 1.6.11" >&2; exit 1; }
fi
if [[ "$lane" == toolchain-pinned || "$lane" == all ]]; then
  export RUSTUP_AUTO_INSTALL=0
  [[ "$(rustup --version 2>/dev/null | head -n 1)" == "rustup 1.29.0 "* ]] \
    || { echo "ci-doctor: expected rustup 1.29.0" >&2; exit 1; }
  rustup toolchain list | grep -q '^1\.97\.1-' || { echo "ci-doctor: Rust 1.97.1 toolchain is missing" >&2; exit 1; }
  [[ "$(rustup run 1.97.1 rustc --version)" == "rustc 1.97.1 "* ]] || { echo "ci-doctor: invalid Rust 1.97.1 toolchain" >&2; exit 1; }
  [[ "$(rustup run 1.97.1 cargo --version)" == "cargo 1.97.1 "* ]] || { echo "ci-doctor: invalid Cargo 1.97.1 toolchain" >&2; exit 1; }
  [[ "$(b3sum --version)" == "b3sum 1.8.2" ]] || { echo "ci-doctor: expected b3sum 1.8.2" >&2; exit 1; }
fi
printf 'ci-doctor: %s lane tools present and pinned\n' "$lane"
