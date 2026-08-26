#!/usr/bin/env bash
# Emit one unsigned diagnostic bound to the current repository subject. It is
# never Bullet Evidence or a release receipt.
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"
# shellcheck source=ops/ci/artifact-path.sh
source "$repo_root/ops/ci/artifact-path.sh"
# shellcheck source=ops/ci/tool-version.sh
export LC_ALL=C
source "$repo_root/ops/ci/tool-version.sh"

lane="${1:-}"
exit_code="${2:-}"
shift "$(( $# >= 2 ? 2 : $# ))"
refuse() { printf '[ci-observation] %s: %s\n' "$1" "$2" >&2; exit 1; }
[[ "$lane" =~ ^[a-z][a-z0-9-]*$ ]] || refuse LANE_INVALID "$lane"
[[ "$exit_code" =~ ^[0-9]+$ ]] || refuse EXIT_CODE_INVALID "$exit_code"
(( exit_code <= 255 )) || refuse EXIT_CODE_INVALID "$exit_code exceeds 255"
command -v git >/dev/null 2>&1 || refuse TOOL_MISSING git
command -v jq >/dev/null 2>&1 || refuse TOOL_MISSING jq

command_count="${CI_COMMAND_COUNT:-0}"
[[ "$command_count" =~ ^[0-9]+$ && "$command_count" -le "$#" ]] \
  || refuse COMMAND_COUNT_INVALID "$command_count"
commands='[]'
for ((index = 0; index < command_count; index++)); do
  command_text="$1"
  shift
  [[ -n "$command_text" ]] || refuse COMMAND_INVALID "empty command"
  commands="$(jq -c --arg value "$command_text" '. + [$value]' <<<"$commands")"
done

sha256_file() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{ print $1 }'
  else
    refuse TOOL_MISSING "sha256sum or shasum"
  fi
}

commit_oid="$(git rev-parse HEAD)"
tree_oid="$(git rev-parse 'HEAD^{tree}')"
[[ "$commit_oid" =~ ^[0-9a-f]{40}$ ]] || refuse COMMIT_OID_INVALID "$commit_oid"
[[ "$tree_oid" =~ ^[0-9a-f]{40}$ ]] || refuse TREE_OID_INVALID "$tree_oid"
clean=true
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || clean=false
status=PASS
(( exit_code == 0 )) || status=FAIL

tool_versions='{}'
record_tool() {
  local key="$1"
  shift
  local value
  ci_tool_key_is_known "$key" || refuse TOOL_VERSION_KEY_UNKNOWN "$key"
  command -v "$1" >/dev/null 2>&1 || return 0
  if ! value="$("$@" 2>&1 | head -n 1)"; then
    return 0
  fi
  if [[ -n "$value" ]]; then
    ci_tool_version_shape_is_valid "$key" "$value" \
      || refuse TOOL_VERSION_INVALID "$key"
    tool_versions="$(jq -c --arg key "$key" --arg value "$value" \
      '. + {($key):$value}' <<<"$tool_versions")"
  fi
}
record_tool git git --version
record_tool rustc rustc --version
record_tool cargo cargo --version
record_tool cargo_nextest cargo-nextest --version
record_tool actionlint actionlint -version
if command -v shellcheck >/dev/null 2>&1; then
  shellcheck_version="$(shellcheck --version | awk '$1 == "version:" { print $2 }')"
  ci_tool_version_shape_is_valid shellcheck "$shellcheck_version" \
    || refuse TOOL_VERSION_INVALID shellcheck
  tool_versions="$(jq -c --arg value "$shellcheck_version" '. + {shellcheck:$value}' <<<"$tool_versions")"
fi
record_tool gitleaks gitleaks version
record_tool cargo_deny cargo-deny --version
record_tool zizmor zizmor --version
record_tool lychee lychee --version
record_tool jankurai jankurai --version
record_tool rustup rustup --version
record_tool b3sum b3sum --version
record_tool java java -version
record_tool docker docker --version
record_tool file file --version
record_tool node node --version
record_tool npm npm --version
python_bin=
if command -v python3 >/dev/null 2>&1; then
  python_bin=python3
elif command -v python >/dev/null 2>&1; then
  python_bin=python
fi
if [[ -n "$python_bin" ]]; then
  record_tool python "$python_bin" --version
  record_tool jsonschema "$python_bin" -c 'from importlib.metadata import version; print(version("jsonschema"))'
fi
if command -v cargo-llvm-cov >/dev/null 2>&1; then
  record_tool cargo_llvm_cov cargo llvm-cov --version
fi
if command -v rustup >/dev/null 2>&1 \
  && rustup toolchain list 2>/dev/null | grep -q '^1\.97\.1-'; then
  record_tool rustc_pinned rustup run 1.97.1 rustc --version
  record_tool cargo_pinned rustup run 1.97.1 cargo --version
fi

artifact_hashes='[]'
for artifact in "$@"; do
  validate_ci_artifact_path "$artifact" || refuse ARTIFACT_PATH_INVALID "$artifact"
  digest="$(sha256_file "$artifact")"
  artifact_hashes="$(jq -c --arg path "$artifact" --arg sha256 "$digest" \
    '. + [{path:$path,sha256:$sha256}]' <<<"$artifact_hashes")"
done
unique_count="$(jq -r '[.[].path] | unique | length' <<<"$artifact_hashes")"
[[ "$unique_count" -eq "$#" ]] || refuse ARTIFACT_DUPLICATE "artifact paths must be unique"

prepare_ci_directory "$repo_root" .ci-artifacts/observations \
  || refuse ARTIFACT_ROOT_INVALID .ci-artifacts/observations
output=".ci-artifacts/observations/$lane.json"
temporary="$output.tmp.$$"
jq -n \
  --arg repository bullet-farm --arg commit_oid "$commit_oid" --arg tree_oid "$tree_oid" \
  --arg lane "$lane" --arg status "$status" --argjson exit_code "$exit_code" \
  --argjson clean "$clean" --argjson commands "$commands" \
  --argjson tool_versions "$tool_versions" --argjson artifact_hashes "$artifact_hashes" '
  {schema_version:"bullet.ci-observation.v1",repository:$repository,
   commit_oid:$commit_oid,tree_oid:$tree_oid,clean:$clean,commands:$commands,
   tool_versions:$tool_versions,outcomes:[{lane:$lane,status:$status,exit_code:$exit_code}],
   artifact_hashes:$artifact_hashes,signed:false,evidence_class:"DIAGNOSTIC_ONLY"}' >"$temporary"
mv "$temporary" "$output"
printf 'ci-observation: wrote %s (%s, unsigned diagnostic only)\n' "$output" "$status"
