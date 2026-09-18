#!/usr/bin/env bash
set -euo pipefail

script_path="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
script_dir="$(dirname "$script_path")"
lib_path="$script_dir/lib.sh"
repo_root="$(cd "$script_dir/../.." && pwd)"

if [[ "${BULLET_NO_CARGO_SOURCE_CHILD:-}" == 1 ]]; then
  # shellcheck source=ops/ci/lib.sh
  source "$lib_path"
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 0 ]] || {
    echo "RUST_TOOLCHAIN_LAZY_INIT_FAILED: source initialized Rust tools" >&2
    exit 1
  }
  builtin type -P cargo >/dev/null 2>&1 && {
    echo "NO_CARGO_SOURCE_CANARY_INVALID: Cargo unexpectedly resolved" >&2
    exit 1
  }
  exit 0
fi

if [[ "${BULLET_NO_CARGO_RUST_CHILD:-}" == 1 ]]; then
  # shellcheck source=ops/ci/lib.sh
  source "$lib_path"
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 0 ]] || exit 1
  if cargo --version >"${BULLET_NO_CARGO_LOG:?}" 2>&1; then
    echo "NO_CARGO_RUST_CANARY_FAILED: wrapped Cargo unexpectedly passed" >&2
    exit 1
  fi
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 0 ]] || {
    echo "NO_CARGO_RUST_CANARY_FAILED: partial initialization was retained" >&2
    exit 1
  }
  grep -Fxq '[ci] TOOL_MISSING: cargo' "$BULLET_NO_CARGO_LOG" || {
    echo "NO_CARGO_RUST_CANARY_FAILED: exact refusal missing" >&2
    exit 1
  }
  exit 0
fi

if [[ "${BULLET_COMPILER_BOUNDARY_SPOOF_CHILD:-}" == 1 ]]; then
  # shellcheck source=ops/ci/lib.sh
  source "$lib_path"
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 0 ]] || exit 1
  cargo --version >/dev/null
  [[ "$RUST_TOOLCHAIN_TOOLS_INITIALIZED" -eq 1 ]] || {
    echo "RUST_TOOLCHAIN_LAZY_INIT_FAILED: wrapped Cargo did not initialize" >&2
    exit 1
  }
  sha256_lf_text_file "$script_path" >/dev/null
  exit 0
fi

test_root="$(mktemp -d "${TMPDIR:-/tmp}/bullet-rust-boundary.XXXXXX")"
cleanup() { rm -rf -- "$test_root"; }
trap cleanup EXIT HUP INT TERM

python_cache_fixture="$test_root/python-cache-fixture"
mkdir "$python_cache_fixture"
cat >"$python_cache_fixture/helper.py" <<'PY'
VALUE = "no-bytecode"
PY
cat >"$python_cache_fixture/main.py" <<'PY'
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from helper import VALUE

print(VALUE)
PY
# shellcheck source=ops/ci/lib.sh
source "$lib_path"
python_cache_output="$(run_python_312 "$python_cache_fixture/main.py")"
[[ "$python_cache_output" == no-bytecode ]] || {
  echo "PYTHON_BYTECODE_SUPPRESSION_INVALID: imported fixture output drifted" >&2
  exit 1
}
if find "$python_cache_fixture" \
  \( -type d -name __pycache__ -o -type f -name '*.pyc' \) -print -quit \
  | grep -q .; then
  echo "PYTHON_BYTECODE_SUPPRESSION_FAILED: run_python_312 created a cache" >&2
  exit 1
fi

# shellcheck disable=SC2317 # executed after export by the isolated child shell
python3() { : >"${BULLET_PYTHON_SPOOF_MARKER:?}"; printf 'Python 3.12.99 hostile-function\n'; }
# shellcheck disable=SC2317 # executed after export by the isolated child shell
cargo() { : >"${BULLET_CARGO_SPOOF_MARKER:?}"; printf 'cargo 1.97.1 hostile-function\n'; }
# shellcheck disable=SC2317 # executed after export by the isolated child shell
env() { : >"${BULLET_ENV_SPOOF_MARKER:?}"; command env "$@"; }
# shellcheck disable=SC2317 # executed after export by the isolated child shell
find() { : >"${BULLET_FIND_SPOOF_MARKER:?}"; command find "$@"; }
# shellcheck disable=SC2317 # executed after export by the isolated child shell
shopt() { : >"${BULLET_SHOPT_SPOOF_MARKER:?}"; builtin shopt "$@"; }
# shellcheck disable=SC2317 # executed after export by the isolated child shell
type() { : >"${BULLET_TYPE_SPOOF_MARKER:?}"; builtin type "$@"; }
export -f python3 cargo env find shopt type
if ! BULLET_COMPILER_BOUNDARY_SPOOF_CHILD=1 \
  BULLET_PYTHON_SPOOF_MARKER="$test_root/python-ran" \
  BULLET_CARGO_SPOOF_MARKER="$test_root/cargo-ran" \
  BULLET_ENV_SPOOF_MARKER="$test_root/env-ran" \
  BULLET_FIND_SPOOF_MARKER="$test_root/find-ran" \
  BULLET_SHOPT_SPOOF_MARKER="$test_root/shopt-ran" \
  BULLET_TYPE_SPOOF_MARKER="$test_root/type-ran" \
  "$BASH" "$script_path" >"$test_root/spoof.log" 2>&1; then
  echo "RUST_COMPILER_BOUNDARY_CANARY_INVALID: isolated spoof child failed" >&2
  sed -n '1,120p' "$test_root/spoof.log" >&2
  exit 1
fi
unset -f python3 cargo env find shopt type
for marker in python cargo env find shopt type; do
  [[ ! -e "$test_root/$marker-ran" ]] || {
    echo "RUST_COMPILER_BOUNDARY_CANARY_FAILED: ambient $marker function executed" >&2
    exit 1
  }
done

no_cargo_root="$test_root/no-cargo"
mkdir "$no_cargo_root"
link_tool() {
  local name="$1" path
  path="$(builtin type -P "$name")" || {
    echo "NO_CARGO_FIXTURE_TOOL_MISSING: $name" >&2
    exit 1
  }
  ln -s "$path" "$no_cargo_root/$name"
}
for tool in awk bash basename cmp cp dirname env find git grep jq ln mkdir mktemp mv \
  python3 realpath rm sed sha256sum sort tr wc; do
  link_tool "$tool"
done

if ! BULLET_NO_CARGO_SOURCE_CHILD=1 PATH="$no_cargo_root" \
  "$BASH" "$script_path" >"$test_root/source.log" 2>&1; then
  echo "NO_CARGO_SOURCE_CANARY_FAILED: shared library source failed" >&2
  sed -n '1,120p' "$test_root/source.log" >&2
  exit 1
fi
if ! BULLET_NO_CARGO_RUST_CHILD=1 BULLET_NO_CARGO_LOG="$test_root/no-cargo-rust.log" \
  PATH="$no_cargo_root" "$BASH" "$script_path"; then
  echo "NO_CARGO_RUST_CANARY_INVALID: typed refusal child failed" >&2
  exit 1
fi

cat >"$no_cargo_root/gitleaks" <<'SH'
#!/bin/bash
case "${1:-}" in
  version) printf '%s\n' 8.21.2 ;;
  detect|git) : >"${BULLET_NO_CARGO_GITLEAKS_MARKER:?}" ;;
  *) exit 2 ;;
esac
SH
cat >"$no_cargo_root/rg" <<'SH'
#!/bin/bash
[[ " $* " == *' --files '* ]] || exit 2
SH
cat >"$no_cargo_root/lychee" <<'SH'
#!/bin/bash
: >"${BULLET_NO_CARGO_LYCHEE_MARKER:?}"
SH
chmod +x "$no_cargo_root/gitleaks" "$no_cargo_root/rg" "$no_cargo_root/lychee"

for lane in source-scan history; do
  marker="$test_root/$lane-ran"
  if ! BULLET_NO_CARGO_GITLEAKS_MARKER="$marker" PATH="$no_cargo_root" \
    "$BASH" "$repo_root/ops/ci/$lane.sh" >"$test_root/$lane.log" 2>&1; then
    echo "NO_CARGO_PURE_LANE_FAILED: $lane" >&2
    sed -n '1,120p' "$test_root/$lane.log" >&2
    exit 1
  fi
  [[ -e "$marker" ]] || { echo "NO_CARGO_PURE_LANE_INVALID: $lane" >&2; exit 1; }
done
if ! BULLET_NO_CARGO_LYCHEE_MARKER="$test_root/links-ran" PATH="$no_cargo_root" \
  "$BASH" "$repo_root/ops/ci/external-links.sh" >"$test_root/links.log" 2>&1; then
  echo "NO_CARGO_PURE_LANE_FAILED: links" >&2
  sed -n '1,120p' "$test_root/links.log" >&2
  exit 1
fi
[[ -e "$test_root/links-ran" ]] || { echo "NO_CARGO_PURE_LANE_INVALID: links" >&2; exit 1; }

if ! PATH="$no_cargo_root" "$BASH" "$repo_root/ops/ci/aggregate-test.sh" \
  >"$test_root/aggregate.log" 2>&1; then
  echo "NO_CARGO_PURE_LANE_FAILED: positive aggregation/artifact validation" >&2
  sed -n '1,160p' "$test_root/aggregate.log" >&2
  exit 1
fi

junit="$test_root/junit.xml"
printf '%s\n' '<testsuites tests="1" failures="0" errors="0" skipped="0">' \
  '  <testsuite name="no-cargo" tests="1" failures="0" errors="0" disabled="0"/>' \
  '</testsuites>' >"$junit"
PATH="$no_cargo_root" "$BASH" "$repo_root/ops/ci/family-report-check.sh" \
  junit "$junit" 1 0 >/dev/null

fixture_repo="$test_root/fixture-repo"
mkdir -p "$fixture_repo/ops/ci" "$fixture_repo/.ci-artifacts/observations"
for relative in artifact-check.sh artifact-path.sh lib.sh rust-toolchain-boundary.sh \
  stage-artifacts.sh strict-json.sh tool-version.sh toolchain-pins.sh; do
  cp "$repo_root/ops/ci/$relative" "$fixture_repo/ops/ci/$relative"
done
cp "$repo_root/.node-version" "$repo_root/.npm-version" "$fixture_repo/"
printf '%s\n' \
  '{"schema_version":"bullet.ci-observation.v1","repository":"bullet-farm",' \
  '"commit_oid":"0000000000000000000000000000000000000000",' \
  '"tree_oid":"0000000000000000000000000000000000000000","clean":false,' \
  '"commands":["bash scripts/ci-doctor.sh history"],' \
  '"tool_versions":{"python":"Python 3.12.3"},' \
  '"outcomes":[{"lane":"history","status":"FAIL","exit_code":1}],' \
  '"artifact_hashes":[],"signed":false,"evidence_class":"DIAGNOSTIC_ONLY"}' \
  | tr -d '\n' >"$fixture_repo/.ci-artifacts/observations/history.json"
printf '\n' >>"$fixture_repo/.ci-artifacts/observations/history.json"
PATH="$no_cargo_root" "$BASH" "$fixture_repo/ops/ci/stage-artifacts.sh" history
[[ -s "$fixture_repo/.ci-upload/history/.ci-artifacts/observations/history.json" ]] || {
  echo "NO_CARGO_STAGING_INVALID: validated observation was not staged" >&2
  exit 1
}

echo "Rust toolchain lazy-boundary canary: PASS"
