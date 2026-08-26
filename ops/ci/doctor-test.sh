#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

declare -A MISSING_TOOLS=()

lane_tools() {
  local lane="$1"
  case "$lane" in
    all)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr \
        actionlint b3sum cargo cargo-clippy cargo-deny cargo-llvm-cov cargo-nextest cat chmod \
        cmp comm cp curl date docker file find gitleaks grep id jankurai java ln lychee mktemp node \
        jsonschema npm rmdir rg rustc rustfmt rustup sha1sum shellcheck stat tee uname wc xargs zizmor
      ;;
    lint)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr actionlint b3sum cargo cargo-clippy \
        cargo-deny cargo-nextest cmp comm cp find jsonschema ln rg rustc rustfmt shellcheck wc
      ;;
    required)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr actionlint b3sum cargo cargo-clippy \
        cargo-deny cargo-nextest chmod cmp comm cp curl date docker file find gitleaks grep id java \
        jsonschema ln mktemp rg rmdir rustc rustfmt sha1sum shellcheck stat tee wc zizmor
      ;;
    coverage)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr cargo cargo-llvm-cov \
        cargo-nextest cmp comm grep ln mktemp rustc wc
      ;;
    platform)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr cargo cargo-clippy grep rustc uname
      ;;
    toolchain-pinned)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr b3sum cargo date grep rustc rustup tee wc
      ;;
    family|family-contract)
      printf '%s\n' \
        awk bash dirname env git head jq mkdir mv realpath rm sed sort tr actionlint b3sum cargo cargo-clippy cargo-deny \
        cargo-nextest chmod cmp comm cp curl date docker file find gitleaks grep id java ln mktemp \
        jsonschema node npm rg rmdir rustc rustfmt rustup sha1sum shellcheck stat tee uname wc zizmor
      ;;
    *)
      return 1
      ;;
  esac
}

probe_with_missing_tools() {
  local lane="$1" missing_tool="$2" include_realpath_missing="$3"
  local fixture
  local -a tools
  local -a skipped=("$missing_tool")
  local output status tool

  fixture="$(mktemp -d)"
  trap 'rm -rf -- "$fixture"' RETURN
  if [[ "$include_realpath_missing" == 1 ]]; then
    skipped+=(realpath)
  fi

  if ! mapfile -t tools < <(lane_tools "$lane"); then
    printf '[ci] DOCTOR_TEST_UNKNOWN_LANE: %s\n' "$lane" >&2
    return 1
  fi
  for tool in "${tools[@]}"; do
    local omit=false
    local skip_tool
    for skip_tool in "${skipped[@]}"; do
      if [[ "$tool" == "$skip_tool" ]]; then
        omit=true
      fi
    done
    $omit && continue
    local src
    src="$(command -v "$tool" 2>/dev/null || true)"
    if [[ -z "$src" ]]; then
      printf '[ci] DOCTOR_TEST_FIXTURE_MISSING_TOOL: %s\n' "$tool" >&2
      return 1
    fi
    ln -s "$src" "$fixture/$tool"
  done

  # ci-doctor loads toolchain-pins early and requires these helpers.
  local bootstrap=()
  bootstrap=(wc tr)
  local bootstrap_tool
  for bootstrap_tool in "${bootstrap[@]}"; do
    local omitted=false
    local skip_tool
    for skip_tool in "${skipped[@]}"; do
      if [[ "$bootstrap_tool" == "$skip_tool" ]]; then
        omitted=true
      fi
    done
    $omitted && continue
    if [[ -e "$fixture/$bootstrap_tool" ]]; then
      continue
    fi
    src="$(command -v "$bootstrap_tool" 2>/dev/null || true)"
    if [[ -z "$src" ]]; then
      printf '[ci] DOCTOR_TEST_FIXTURE_MISSING_TOOL: %s\n' "$bootstrap_tool" >&2
      return 1
    fi
    ln -s "$src" "$fixture/$bootstrap_tool"
  done

  set +e
  output="$(PATH="$fixture" /bin/bash "$REPO_ROOT/scripts/ci-doctor.sh" "$lane" 2>&1)"
  status=$?
  set -e

  MISSING_TOOLS["output"]="$output"
  MISSING_TOOLS["status"]="$status"
}

for tool in b3sum curl env node npm realpath rustup jsonschema lychee cargo-llvm-cov; do
  probe_with_missing_tools all "$tool" 0
  output="${MISSING_TOOLS["output"]}"
  status="${MISSING_TOOLS["status"]}"
  [[ "$status" -ne 0 && "$output" == *"missing $tool for all"* ]] || {
    printf '[ci] DOCTOR_ALL_INVENTORY_MISSING: %s status=%s\n' "$tool" "$status" >&2
    exit 1
  }
done
for lane_tool in \
  'lint cp' 'lint ln' 'lint b3sum' 'lint env' 'lint jsonschema' \
  'required b3sum' 'required env' 'required jsonschema' \
  'coverage cmp' 'coverage comm' 'coverage ln' \
  'platform grep' \
  'toolchain-pinned date' 'toolchain-pinned grep' 'toolchain-pinned tee' \
  'family-contract node' 'family b3sum' 'family env' 'family rustup' 'family jsonschema'; do
  read -r lane tool <<<"$lane_tool"
  probe_with_missing_tools "$lane" "$tool" 1
  output="${MISSING_TOOLS["output"]}"
  status="${MISSING_TOOLS["status"]}"
  [[ "$status" -ne 0 && "$output" == *"missing $tool for $lane"* \
    && "$output" == *"missing realpath for $lane"* ]] || {
    printf '[ci] DOCTOR_LANE_INVENTORY_MISSING: %s/%s status=%s\n' \
      "$lane" "$tool" "$status" >&2
    exit 1
  }
done
source_text="$(<"$REPO_ROOT/scripts/ci-doctor.sh")"
local_source="$(<"$REPO_ROOT/scripts/ci-local.sh")"
dollar='$'
[[ "$source_text" == *"\"${dollar}lane\" == links || \"${dollar}lane\" == all"* \
  && "$source_text" == *"\"${dollar}lane\" == coverage || \"${dollar}lane\" == all"* \
  && "$source_text" == *"\"${dollar}lane\" == family || \"${dollar}lane\" == family-contract || \"${dollar}lane\" == all"* \
  && "$source_text" == *"\"${dollar}lane\" == toolchain-pinned || \"${dollar}lane\" == all"* ]] || {
  echo '[ci] DOCTOR_ALL_VERSION_UNION_MISSING' >&2
  exit 1
}
[[ "$local_source" == *'family-contract) run_observed family-contract ops/ci/family-contract.sh'* ]] || {
  echo '[ci] FAMILY_CONTRACT_OBSERVATION_IDENTITY_DRIFT' >&2
  exit 1
}
printf '[ci] ci-doctor all-lane union guards passed\n'
