#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
export REPO_ROOT
export GIT_TERMINAL_PROMPT=0
export LC_ALL=C
export TZ=UTC

HUB_FILTER='package(bullet-family)'
WIRE_FILTER='package(bullet-wire)'
HUB_EXPECTED_TESTS=218
WIRE_EXPECTED_TESTS=92
TOTAL_EXPECTED_TESTS=310
export HUB_FILTER WIRE_FILTER HUB_EXPECTED_TESTS WIRE_EXPECTED_TESTS TOTAL_EXPECTED_TESTS
# shellcheck source=ops/ci/artifact-path.sh
source "$REPO_ROOT/ops/ci/artifact-path.sh"

log() { printf '[ci] %s\n' "$*"; }
refuse() { printf '[ci] %s: %s\n' "$1" "$2" >&2; return 1; }
require_file() { [[ -f "$REPO_ROOT/$1" ]] || refuse REQUIRED_FILE_MISSING "$1"; }
require_tool() { command -v "$1" >/dev/null 2>&1 || refuse TOOL_MISSING "$1"; }

resolved_executable() {
  local candidate
  candidate="$(builtin type -P "$1")" || {
    refuse TOOL_MISSING "$1"
    return 1
  }
  [[ "$candidate" == /* && -f "$candidate" && -x "$candidate" ]] || {
    refuse TOOL_IDENTITY_INVALID "$1 did not resolve to an absolute executable file"
    return 1
  }
  printf '%s\n' "$candidate"
}

sha256_file() {
  local tool digest remainder
  if tool="$(builtin type -P sha256sum)"; then
    IFS=' ' read -r digest remainder < <("$tool" "$1") || return 1
  elif tool="$(builtin type -P shasum)"; then
    IFS=' ' read -r digest remainder < <("$tool" -a 256 "$1") || return 1
  else
    refuse SHA256_TOOL_MISSING "install sha256sum or shasum"
    return 1
  fi
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || refuse SHA256_OUTPUT_INVALID "$1"
  printf '%s\n' "$digest"
}

verify_resolved_tool() {
  local path="$1" expected="$2" label="$3" actual
  [[ -f "$path" && -x "$path" ]] || {
    refuse TOOL_IDENTITY_CHANGED "$label executable is missing"
    return 1
  }
  actual="$(sha256_file "$path")" || return 1
  [[ "$actual" == "$expected" ]] || {
    refuse TOOL_IDENTITY_CHANGED "$label executable bytes changed"
    return 1
  }
}

# shellcheck source=ops/ci/rust-toolchain-boundary.sh
source "$REPO_ROOT/ops/ci/rust-toolchain-boundary.sh"

validate_cargo_manifest() {
  local manifest="$1" result
  if ! result="$(run_python_312 - "$manifest" <<'PY'
import sys
import tomllib

try:
    with open(sys.argv[1], "rb") as handle:
        document = tomllib.load(handle)
except (OSError, tomllib.TOMLDecodeError):
    raise SystemExit(2)
package = document.get("package")
if isinstance(package, dict) and "build" in package:
    raise SystemExit(3)
print("OK")
PY
)"; then
    refuse CARGO_MANIFEST_BUILD_SCRIPT_FORBIDDEN \
      "manifest has invalid TOML or an unadmitted package.build key"
    return 1
  fi
  result="${result%$'\r'}"
  [[ "$result" == OK ]] || {
    refuse CARGO_MANIFEST_BUILD_SCRIPT_FORBIDDEN "manifest validator output is invalid"
    return 1
  }
}

sha256_lf_text_file() {
  local path="$1" digest
  if ! digest="$(run_python_312 - "$path" <<'PY'
import hashlib
import pathlib
import sys

data = pathlib.Path(sys.argv[1]).read_bytes().replace(b"\r\n", b"\n")
if b"\r" in data:
    raise SystemExit(2)
print(hashlib.sha256(data).hexdigest())
PY
)"; then
    refuse RUST_BUILD_SUBJECT_TEXT_INVALID \
      "$path contains an unreadable or non-CRLF carriage return"
    return 1
  fi
  digest="${digest%$'\r'}"
  [[ "$digest" =~ ^[0-9a-f]{64}$ ]] || {
    refuse RUST_BUILD_SUBJECT_TEXT_INVALID "$path produced an invalid SHA-256 digest"
    return 1
  }
  printf '%s\n' "$digest"
}

validate_cargo_config() {
  local config="$1" controls
  [[ -f "$config" && ! -L "$config" ]] || {
    refuse CARGO_CONFIG_INVALID "$config is missing, not regular, or a symlink"
    return 1
  }
  if ! controls="$(run_python_312 - "$config" <<'PY'
import sys
import tomllib

path = sys.argv[1]
try:
    with open(path, "rb") as handle:
        document = tomllib.load(handle)
except (OSError, tomllib.TOMLDecodeError) as error:
    print(error)
    raise SystemExit(2)

allowed = {
    ("build", "jobs"): lambda value: type(value) is int and value > 0,
    ("net", "retry"): lambda value: type(value) is int and value >= 0,
    ("cache", "auto-clean-frequency"): lambda value: isinstance(value, (int, str)),
}
rejected = []

def walk(value, prefix=()):
    if isinstance(value, dict):
        for key, child in value.items():
            walk(child, prefix + (str(key).replace("_", "-").lower(),))
        return
    rule = allowed.get(prefix)
    if rule is None or not rule(value):
        rejected.append(".".join(prefix) or "<root>")

walk(document)
print(",".join(sorted(set(rejected))))
PY
)"; then
    refuse CARGO_CONFIG_INVALID "$config could not be parsed: $controls"
    return 1
  fi
  controls="${controls%$'\r'}"
  [[ "$controls" != *$'\r'* ]] || {
    refuse CARGO_CONFIG_INVALID "$config produced invalid carriage-return output"
    return 1
  }
  [[ -z "$controls" ]] || {
    refuse CARGO_CONFIG_CONTROL_FORBIDDEN "$config contains unadmitted keys: $controls"
    return 1
  }
}

validate_resolved_cargo_configs() {
  local subject_root="$1" directory parent config cargo_home default_cargo_home
  local resolved_cargo_home resolved_default_cargo_home
  [[ -n "${HOME:-}" ]] || {
    refuse CARGO_HOME_UNRESOLVED "HOME is unset"
    return 1
  }
  directory="$(cd "$subject_root" && pwd -P)" || return 1
  while :; do
    for config in "$directory/.cargo/config" "$directory/.cargo/config.toml"; do
      [[ ! -e "$config" ]] || validate_cargo_config "$config" || return 1
    done
    [[ "$directory" == / ]] && break
    parent="${directory%/*}"
    [[ -n "$parent" ]] || parent=/
    [[ "$parent" != "$directory" ]] || break
    directory="$parent"
  done
  default_cargo_home="${HOME%/}/.cargo"
  cargo_home="${CARGO_HOME:-$default_cargo_home}"
  [[ -d "$default_cargo_home" && ! -L "$default_cargo_home" ]] || {
    refuse CARGO_HOME_INVALID "$default_cargo_home is missing, not a directory, or a symlink"
    return 1
  }
  [[ -d "$cargo_home" && ! -L "$cargo_home" ]] || {
    refuse CARGO_HOME_INVALID "$cargo_home is missing, not a directory, or a symlink"
    return 1
  }
  resolved_default_cargo_home="$(cd "$default_cargo_home" && pwd -P)" || return 1
  resolved_cargo_home="$(cd "$cargo_home" && pwd -P)" || return 1
  [[ "$resolved_cargo_home" == "$resolved_default_cargo_home" ]] || {
    refuse CARGO_HOME_FORBIDDEN \
      "CARGO_HOME must resolve to the default $resolved_default_cargo_home"
    return 1
  }
  for config in "$cargo_home/config" "$cargo_home/config.toml"; do
    [[ ! -e "$config" ]] || validate_cargo_config "$config" || return 1
  done
}

enforce_rust_compiler_boundary() {
  local subject_root="${1:-$REPO_ROOT}" name manifest first_forbidden_path
  local restore_nocasematch=0
  local -a forbidden_environment=() forbidden_paths=()
  initialize_rust_toolchain_tools || return 1
  verify_resolved_tool "$ENV_EXECUTABLE" "$ENV_EXECUTABLE_SHA256" env || return 1
  verify_resolved_tool "$FIND_EXECUTABLE" "$FIND_EXECUTABLE_SHA256" find || return 1
  verify_resolved_tool "$SORT_EXECUTABLE" "$SORT_EXECUTABLE_SHA256" sort || return 1
  verify_resolved_tool "$SED_EXECUTABLE" "$SED_EXECUTABLE_SHA256" sed || return 1
  if ! builtin shopt -q nocasematch; then
    builtin shopt -s nocasematch
    restore_nocasematch=1
  fi
  while IFS='=' read -r name _; do
    case "$name" in
      CARGO_ALIAS_*|CARGO_BUILD_TARGET|CARGO_PROFILE_*)
        forbidden_environment+=("$name")
        ;;
      RUSTC|RUSTC_WRAPPER|RUSTC_WORKSPACE_WRAPPER|RUSTC_BOOTSTRAP|RUSTFLAGS)
        forbidden_environment+=("$name")
        ;;
      RUSTDOC|RUSTDOC_WRAPPER|RUSTDOCFLAGS|RUSTUP_TOOLCHAIN)
        forbidden_environment+=("$name")
        ;;
      RUSTFMT|RUSTFMT_ARGS)
        forbidden_environment+=("$name")
        ;;
      CARGO_ENCODED_RUSTFLAGS|CARGO_ENCODED_RUSTDOCFLAGS|CARGO_BUILD_RUSTC)
        forbidden_environment+=("$name")
        ;;
      CARGO_BUILD_RUSTC_WRAPPER|CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER|CARGO_BUILD_RUSTFLAGS)
        forbidden_environment+=("$name")
        ;;
      CARGO_BUILD_RUSTDOCFLAGS|CARGO_TARGET_*_RUSTFLAGS|CARGO_TARGET_*_RUSTDOCFLAGS)
        forbidden_environment+=("$name")
        ;;
      CARGO_TARGET_*_AR|CARGO_TARGET_*_LINKER|CARGO_TARGET_*_RUNNER)
        forbidden_environment+=("$name")
        ;;
      CLIPPY_ARGS|CLIPPY_CONF_DIR|CLIPPY_DRIVER_PATH)
        forbidden_environment+=("$name")
        ;;
    esac
  done < <("$ENV_EXECUTABLE")
  if [[ "$restore_nocasematch" -eq 1 ]]; then
    builtin shopt -u nocasematch
  fi
  [[ "${#forbidden_environment[@]}" -eq 0 ]] || {
    refuse RUST_COMPILER_CONTROL_ENV "forbidden environment: ${forbidden_environment[*]}"
    return 1
  }

  for manifest in "$subject_root/.cargo/config" "$subject_root/.cargo/config.toml"; do
    [[ ! -e "$manifest" ]] || forbidden_paths+=("${manifest#"$subject_root"/}")
  done
  [[ "${#forbidden_paths[@]}" -eq 0 ]] || {
    refuse CARGO_CONFIG_FORBIDDEN "workspace Cargo configuration: ${forbidden_paths[*]}"
    return 1
  }
  validate_resolved_cargo_configs "$subject_root" || return 1

  first_forbidden_path="$(
    "$FIND_EXECUTABLE" "$subject_root" \
      \( -path "$subject_root/.git" -o -path "$subject_root/target" \
         -o -path "$subject_root/.ci-artifacts" -o -path "$subject_root/.fusion" \
         -o -path "$subject_root/.jankurai" \) -prune -o \
      -iname build.rs -print | LC_ALL=C "$SORT_EXECUTABLE" | "$SED_EXECUTABLE" -n '1p'
  )"
  [[ -z "$first_forbidden_path" ]] || {
    refuse CARGO_BUILD_SCRIPT_FORBIDDEN "workspace build script: ${first_forbidden_path#"$subject_root"/}"
    return 1
  }

  while IFS= read -r manifest; do
    [[ -n "$manifest" ]] || continue
    validate_cargo_manifest "$manifest" || return 1
  done < <(
    "$FIND_EXECUTABLE" "$subject_root" \
      \( -path "$subject_root/.git" -o -path "$subject_root/target" \
         -o -path "$subject_root/.ci-artifacts" -o -path "$subject_root/.fusion" \
         -o -path "$subject_root/.jankurai" \) -prune -o \
      -type f -name Cargo.toml -print
  )
}

enforce_rust_build_subject() {
  local subject_root="${1:-$REPO_ROOT}" relative actual expected
  local expected_manifests manifests

  initialize_rust_toolchain_tools || return 1
  verify_resolved_tool "$FIND_EXECUTABLE" "$FIND_EXECUTABLE_SHA256" find || return 1
  verify_resolved_tool "$SORT_EXECUTABLE" "$SORT_EXECUTABLE_SHA256" sort || return 1
  expected_manifests="$(printf '%s\n' ./Cargo.toml ./crates/bullet-wire/Cargo.toml)"
  manifests="$(
    cd "$subject_root"
    "$FIND_EXECUTABLE" . \
      \( -path './.git' -o -path './target' -o -path './.ci-artifacts' \
         -o -path './.fusion' -o -path './.jankurai' \) -prune -o \
      -type f -name Cargo.toml -print | LC_ALL=C "$SORT_EXECUTABLE"
  )"
  [[ "$manifests" == "$expected_manifests" ]] || {
    refuse CARGO_MANIFEST_INVENTORY_DRIFT \
      "expected '$expected_manifests', found '$manifests'"
    return 1
  }

  for relative in Cargo.toml crates/bullet-wire/Cargo.toml Cargo.lock rust-toolchain.toml; do
    [[ -f "$subject_root/$relative" && ! -L "$subject_root/$relative" ]] || {
      refuse RUST_BUILD_SUBJECT_INVALID "$relative is missing, not regular, or a symlink"
      return 1
    }
    case "$relative" in
      Cargo.toml)
        expected=de114a4096cd51a8e33287b0e1b8d96c8a06c271466b123b1341b912a0eb8855
        ;;
      crates/bullet-wire/Cargo.toml)
        expected=1f7442c92c1e155a79237c55406cae300cb494d87a4b5dfa2595939a63fc9047
        ;;
      Cargo.lock)
        expected=e283e3210b0d02b18b67462dcb697eeb2e5a5bd11baddd7c426162d268cb1b18
        ;;
      rust-toolchain.toml)
        expected=e3a213e0d222e94d213cafbc20932eb3f76c643b4dd63756acf95192df2aa310
        ;;
    esac
    actual="$(sha256_lf_text_file "$subject_root/$relative")" || return 1
    [[ "$actual" == "$expected" ]] || {
      refuse RUST_BUILD_SUBJECT_DRIFT \
        "$relative expected $expected, found $actual"
      return 1
    }
  done
}

cargo() {
  local status
  initialize_rust_toolchain_tools || return 1
  verify_resolved_tool "$CARGO_EXECUTABLE" "$CARGO_EXECUTABLE_SHA256" Cargo || return 1
  enforce_rust_compiler_boundary "$REPO_ROOT" || return 1
  enforce_rust_build_subject "$REPO_ROOT" || return 1
  if "$CARGO_EXECUTABLE" "$@"; then
    status=0
  else
    status=$?
  fi
  verify_resolved_tool "$CARGO_EXECUTABLE" "$CARGO_EXECUTABLE_SHA256" Cargo || return 1
  enforce_rust_compiler_boundary "$REPO_ROOT" || return 1
  enforce_rust_build_subject "$REPO_ROOT" || return 1
  return "$status"
}

require_exact_output() {
  local expected="$1"
  shift
  local actual
  actual="$("$@")" || return 1
  actual="${actual%%$'\n'*}"
  [[ "$actual" == "$expected" ]] \
    || refuse TOOL_VERSION_MISMATCH "expected '$expected', found '$actual'"
}

partition_count() {
  cargo nextest list --locked --workspace --message-format json -E "$1" \
    | jq -er '."test-count"'
}

write_junit_summary() {
  local lane="$1" tests="$2" failures="$3" errors="$4" skipped="$5"
  prepare_ci_directory "$REPO_ROOT" .ci-artifacts/junit \
    || { refuse CI_ARTIFACT_ROOT_INVALID .ci-artifacts/junit; return 1; }
  printf '%s\n' \
    '<?xml version="1.0" encoding="UTF-8"?>' \
    "<testsuites tests=\"$tests\" failures=\"$failures\" errors=\"$errors\" skipped=\"$skipped\">" \
    "  <testsuite name=\"bullet-farm-$lane\" tests=\"$tests\" failures=\"$failures\" errors=\"$errors\" skipped=\"$skipped\"/>" \
    '</testsuites>' >"$REPO_ROOT/.ci-artifacts/junit/$lane.xml"
}

xml_integer_attribute() {
  local line="$1" attribute="$2" value
  value="$(sed -nE "s/.*[[:space:]]${attribute}=\"([0-9]+)\".*/\\1/p" <<<"$line")"
  [[ "$value" =~ ^[0-9]+$ ]] \
    || { refuse JUNIT_SOURCE_INVALID "missing integer attribute $attribute"; return 1; }
  printf '%s\n' "$value"
}

run_partition() {
  local lane="$1" profile="$2" filter="$3" expected="$4"
  local selected source_report status root_line actual_tests failures errors skipped
  require_tool cargo-nextest || return 1
  require_tool jq || return 1
  selected="$(partition_count "$filter")" || return 1
  [[ "$selected" -eq "$expected" && "$selected" -gt 0 ]] \
    || { refuse TEST_PARTITION_DRIFT "$lane selected $selected; expected $expected"; return 1; }
  source_report="$REPO_ROOT/target/nextest/$profile/junit.xml"
  prepare_ci_directory "$REPO_ROOT" .ci-artifacts/junit \
    || { refuse CI_ARTIFACT_ROOT_INVALID .ci-artifacts/junit; return 1; }
  rm -f "$source_report" "$REPO_ROOT/.ci-artifacts/junit/$lane.xml"
  log "$lane partition: $selected tests"
  set +e
  cargo nextest run --locked --workspace --profile "$profile" -E "$filter"
  status=$?
  set -e
  [[ -s "$source_report" ]] \
    || { refuse JUNIT_SOURCE_MISSING "$source_report"; return 1; }
  root_line="$(grep -m1 '<testsuites ' "$source_report")" \
    || { refuse JUNIT_SOURCE_INVALID "$source_report"; return 1; }
  actual_tests="$(xml_integer_attribute "$root_line" tests)" || return 1
  failures="$(xml_integer_attribute "$root_line" failures)" || return 1
  errors="$(xml_integer_attribute "$root_line" errors)" || return 1
  skipped="$(grep '<testsuite ' "$source_report" \
    | sed -nE 's/.*[[:space:]]disabled="([0-9]+)".*/\1/p' \
    | awk '{sum += $1} END {print sum + 0}')"
  [[ "$actual_tests" -eq "$selected" ]] \
    || { refuse JUNIT_TEST_COUNT_DRIFT "$lane report=$actual_tests selected=$selected"; return 1; }
  write_junit_summary "$lane" "$actual_tests" "$failures" "$errors" "$skipped"
  [[ "$skipped" -eq 0 ]] \
    || { refuse SKIPPED_TESTS_FORBIDDEN "$lane skipped=$skipped"; return 1; }
  if (( status == 0 && (failures != 0 || errors != 0) )); then
    refuse JUNIT_OUTCOME_CONTRADICTION "$lane exited zero with failures=$failures errors=$errors"
    return 1
  fi
  return "$status"
}
