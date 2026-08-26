#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
test_root="$(mktemp -d)"
lock_parent=.bullet-family/locks
lock_dir="$lock_parent/observation-test"
mkdir -p "$lock_parent"
lock_acquired=false
_attempt=0
while (( _attempt < 300 )); do
  if mkdir "$lock_dir" 2>/dev/null; then
    lock_acquired=true
    break
  fi
  sleep 0.1
  _attempt=$((_attempt + 1))
done
[[ "$lock_acquired" == true ]] \
  || { refuse OBSERVATION_TEST_LOCK_TIMEOUT "$lock_dir"; exit 1; }
cleanup() {
  rm -rf -- "$test_root"
  rmdir "$lock_dir" 2>/dev/null || true
}
trap cleanup EXIT HUP INT TERM
if grep -Eq 'declare[[:space:]]+-A|(^|[[:space:]])(mapfile|readarray)([[:space:]]|$)' \
  ops/ci/artifact-check.sh; then
  refuse OBSERVATION_CHECKER_BASH3_INCOMPATIBLE "associative array or mapfile/readarray"; exit 1
fi
if grep -Eq 'realpath[[:space:]]+-[em]([[:space:]]|$)' \
  ops/ci/artifact-path.sh ops/ci/stage-artifacts.sh; then
  refuse OBSERVATION_CHECKER_REALPATH_INCOMPATIBLE "GNU-only realpath option"; exit 1
fi
mkdir -p .ci-artifacts/test
printf 'sanitized\n' >.ci-artifacts/test/artifact.txt
CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 0 \
  'bash ops/ci/observation-test.sh' .ci-artifacts/test/artifact.txt >/dev/null
observation=.ci-artifacts/observations/observation-test.json
jq -e '
  .schema_version == "bullet.ci-observation.v1" and .repository == "bullet-farm" and
  (.commit_oid | test("^[0-9a-f]{40}$")) and (.tree_oid | test("^[0-9a-f]{40}$")) and
  (.clean | type == "boolean") and .commands == ["bash ops/ci/observation-test.sh"] and
  .outcomes == [{"lane":"observation-test","status":"PASS","exit_code":0}] and
  (.artifact_hashes | length == 1) and
  .artifact_hashes[0].path == ".ci-artifacts/test/artifact.txt" and
  (.artifact_hashes[0].sha256 | test("^[0-9a-f]{64}$")) and
  .signed == false and .evidence_class == "DIAGNOSTIC_ONLY"
' "$observation" >/dev/null
bash ops/ci/artifact-check.sh observation-test >/dev/null
valid_observation="$test_root/valid-observation.json"
cp "$observation" "$valid_observation"
runtime_tool_keys="$(
  # shellcheck source=ops/ci/tool-version.sh
  source ops/ci/tool-version.sh
  for key in $CI_TOOL_KEYS; do printf '%s\n' "$key"; done | LC_ALL=C sort
)"
schema_tool_keys="$(jq -r '.properties.tool_versions.properties | keys[]' \
  docs/schemas/bullet.ci-observation.v1.schema.json | LC_ALL=C sort)"
[[ "$runtime_tool_keys" == "$schema_tool_keys" ]] \
  || { refuse OBSERVATION_TOOL_VOCABULARY_DRIFT "runtime/schema mismatch"; exit 1; }
jsonschema -i "$valid_observation" docs/schemas/bullet.ci-observation.v1.schema.json >/dev/null 2>&1 \
  || { refuse OBSERVATION_SCHEMA_REJECTED_PRODUCER_OUTPUT "$valid_observation"; exit 1; }
mkdir "$test_root/python-without-jsonschema"
printf '%s\n' '#!/usr/bin/env sh' \
  "if [ \"\${1:-}\" = \"--version\" ]; then printf \"%s\\n\" \"Python 3.12.3\"; exit 0; fi" \
  'exit 1' >"$test_root/python-without-jsonschema/python3"
chmod +x "$test_root/python-without-jsonschema/python3"
PATH="$test_root/python-without-jsonschema:$PATH" CI_COMMAND_COUNT=1 \
  bash scripts/ci-observation.sh observation-test 0 \
    'bash ops/ci/observation-test.sh' .ci-artifacts/test/artifact.txt >/dev/null
jq -e '.tool_versions.python == "Python 3.12.3" and
  (.tool_versions | has("jsonschema") | not)' "$observation" >/dev/null \
  || { refuse OBSERVATION_OPTIONAL_TOOL_PROBE_FAILED jsonschema; exit 1; }
cp "$valid_observation" "$observation"
python_path="$(command -v python3 || command -v python)"
mkdir "$test_root/python-only"
ln -s "$python_path" "$test_root/python-only/python"
PATH="$test_root/python-only" /bin/bash ops/ci/strict-json.sh "$valid_observation" >/dev/null
assert_strict_json_refuses() {
  local label="$1" path="$2" output
  if output="$(bash ops/ci/strict-json.sh "$path" 2>&1)"; then
    refuse STRICT_JSON_HOSTILE_ADMITTED "$label"; exit 1
  fi
  if [[ "$output" != "[ci] STRICT_JSON_INVALID: $path" || "$output" == *Traceback* ]]; then
    refuse STRICT_JSON_DIAGNOSTIC_LEAK "$label: $output"; exit 1
  fi
}
deep_json="$test_root/deep.json"
"$python_path" -I -S -c 'print("[" * 2000 + "0" + "]" * 2000)' >"$deep_json"
assert_strict_json_refuses recursion "$deep_json"
underflow_json="$test_root/underflow.json"
printf '{"number":1e-9999}\n' >"$underflow_json"
assert_strict_json_refuses underflow "$underflow_json"
precision_loss_json="$test_root/precision-loss.json"
printf '{"number":0.10000000000000001}\n' >"$precision_loss_json"
assert_strict_json_refuses precision-loss "$precision_loss_json"
unsafe_integer_json="$test_root/unsafe-integer.json"
printf '{"number":9007199254740992}\n' >"$unsafe_integer_json"
assert_strict_json_refuses unsafe-integer "$unsafe_integer_json"
unsafe_decimal_json="$test_root/unsafe-integral-decimal.json"
printf '{"number":9007199254740992.0}\n' >"$unsafe_decimal_json"
assert_strict_json_refuses unsafe-integral-decimal "$unsafe_decimal_json"
oversize_json="$test_root/oversize.json"
"$python_path" -I -S -c 'import sys; sys.stdout.write("0" * (4 * 1024 * 1024 + 1))' \
  >"$oversize_json"
assert_strict_json_refuses oversize "$oversize_json"
if ! "$python_path" -I -S - ops/ci/strict-json.sh scripts/readme-schema-check.sh <<'PY'
import ast
import sys

CONSTANTS = {"MAX_DOCUMENT_BYTES", "MAX_SAFE_INTEGER"}
FUNCTIONS = {"parse_strict_integer", "parse_strict_float"}


def numeric_contract(path):
    text = open(path, encoding="utf-8").read()
    start = text.index("<<'PY'\n") + len("<<'PY'\n")
    end = text.index("\nPY\n", start)
    tree = ast.parse(text[start:end])
    selected = []
    for node in tree.body:
        if isinstance(node, ast.Assign):
            names = {target.id for target in node.targets if isinstance(target, ast.Name)}
            if names & CONSTANTS:
                selected.append(node)
        elif isinstance(node, ast.FunctionDef) and node.name in FUNCTIONS:
            selected.append(node)
    return ast.dump(ast.Module(body=selected, type_ignores=[]), include_attributes=False)


if numeric_contract(sys.argv[1]) != numeric_contract(sys.argv[2]):
    raise SystemExit(1)
PY
then
  refuse STRICT_JSON_COPY_DRIFT "numeric strict-parser contracts differ"; exit 1
fi
{
  printf '%s\n' '{' '  "repository": "hostile-duplicate",'
  sed '1d' "$valid_observation"
} >"$observation"
if output="$(CI_STRICT_JSON_PYTHON=/bin/true bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_JSON_STRICT_INVALID* ]]; then
  refuse OBSERVATION_DUPLICATE_JSON_GUARD_FAILED "$output"; exit 1
fi
{
  printf '%s\n' '{' '  "non_finite": NaN,'
  sed '1d' "$valid_observation"
} >"$observation"
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_JSON_STRICT_INVALID* ]]; then
  refuse OBSERVATION_NONFINITE_JSON_GUARD_FAILED "$output"; exit 1
fi
cp "$valid_observation" "$observation"
for mutation in \
  '.outcomes[0].raw_detail="forbidden"' \
  '.artifact_hashes[0].raw_detail="forbidden"' \
  '.tool_versions.fixture={raw_detail:"forbidden"}'; do
  jq "$mutation" "$valid_observation" >"$observation"
  if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
    || [[ "$output" != *CI_OBSERVATION_INVALID* ]]; then
    refuse OBSERVATION_NESTED_SCHEMA_GUARD_FAILED "$mutation: $output"; exit 1
  fi
done
cp "$valid_observation" "$observation"
for mutation in \
  '.tool_versions.python="Python 3.12.3\ncredential_ghp_1234567890abcdef"' \
  '.tool_versions.python="Python 3.12.3\n"' \
  '.tool_versions.python="Python 3.12.3\t"' \
  '.tool_versions.python="Python 3.12.3\u007f"' \
  '.tool_versions.python=("Python 3.12.3" + ("x" * 161))'; do
  jq "$mutation" "$valid_observation" >"$observation"
  if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
    || [[ "$output" != *CI_OBSERVATION_INVALID* ]]; then
    refuse OBSERVATION_TOOL_METADATA_SCHEMA_GUARD_FAILED "$mutation: $output"; exit 1
  fi
done
for mutation in \
  '.tool_versions.exfiltration="credential_ghp_1234567890abcdef"' \
  '.tool_versions.python_token="Python 3.12.3"' \
  '.tool_versions["credential_ghp_1234567890abcdef"]="Python 3.12.3"'; do
  jq "$mutation" "$valid_observation" >"$observation"
  if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
    || [[ "$output" != *CI_OBSERVATION_INVALID* ]] \
    || [[ "$output" == *credential_ghp_1234567890abcdef* ]]; then
    refuse OBSERVATION_TOOL_KEY_GUARD_FAILED "$mutation: $output"; exit 1
  fi
done
jq '.tool_versions.python="Python 3.12.3 credential_ghp_1234567890abcdef"' \
  "$valid_observation" >"$observation"
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_TOOL_VERSION_INVALID* ]] \
  || [[ "$output" == *credential_ghp_1234567890abcdef* ]]; then
  refuse OBSERVATION_TOOL_METADATA_GRAMMAR_GUARD_FAILED "$output"; exit 1
fi
cp "$valid_observation" "$observation"
schema_pattern="$(jq -r '.properties.artifact_hashes.items.properties.path.pattern' \
  docs/schemas/bullet.ci-observation.v1.schema.json)"
jq -ne --arg pattern "$schema_pattern" --arg path '.ci-artifacts/report.xml' \
  '$path | test($pattern)' >/dev/null
for invalid in report.xml .ci-artifacts/../escape .ci-artifacts/a//b '.ci-artifacts/a\b'; do
  if jq -ne --arg pattern "$schema_pattern" --arg path "$invalid" '$path | test($pattern)' >/dev/null; then
    refuse OBSERVATION_SCHEMA_PATH_GUARD_FAILED "$invalid"; exit 1
  fi
done
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 0 valid ../escape >/dev/null 2>&1; then
  refuse OBSERVATION_PATH_GUARD_FAILED "parent traversal accepted"; exit 1
fi
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 0 valid .ci-artifacts/../escape >/dev/null 2>&1; then
  refuse OBSERVATION_PATH_GUARD_FAILED "root traversal accepted"; exit 1
fi
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 0 '' >/dev/null 2>&1; then
  refuse OBSERVATION_COMMAND_GUARD_FAILED "empty command accepted"; exit 1
fi
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 256 valid >/dev/null 2>&1; then
  refuse OBSERVATION_EXIT_GUARD_FAILED "exit 256 accepted"; exit 1
fi
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh ../escape 1 valid >/dev/null 2>&1; then
  refuse OBSERVATION_LANE_GUARD_FAILED "path-shaped lane accepted"; exit 1
fi

printf 'secret-shaped raw diagnostic\n' >.ci-artifacts/test/raw.log
CI_COMMAND_COUNT=2 bash scripts/ci-observation.sh lint 1 \
  'bash scripts/ci-doctor.sh lint' 'bash ops/ci/lint.sh' .ci-artifacts/test/raw.log >/dev/null
if output="$(bash ops/ci/artifact-check.sh lint 2>&1)" \
  || [[ "$output" != *CI_ARTIFACT_INVENTORY_INVALID* ]]; then
  refuse OBSERVATION_FAIL_ARTIFACT_GUARD_FAILED "FAIL observation accepted raw artifact: $output"; exit 1
fi
rm -f .ci-artifacts/test/raw.log .ci-artifacts/observations/lint.json

CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 1 \
  'bash ops/ci/observation-test.sh' .ci-artifacts/test/artifact.txt >/dev/null
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_ARTIFACT_INVENTORY_INVALID* ]]; then
  refuse OBSERVATION_FAIL_ALLOWLIST_GUARD_FAILED \
    "FAIL observation accepted an allow-listed but unvalidated artifact: $output"; exit 1
fi
CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-test 0 \
  'bash ops/ci/observation-test.sh' .ci-artifacts/test/artifact.txt >/dev/null

CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh history 0 \
  'bash ops/ci/not-history.sh' >/dev/null
if output="$(bash ops/ci/artifact-check.sh history 2>&1)" \
  || [[ "$output" != *CI_OBSERVATION_COMMAND_INVALID* ]]; then
  refuse OBSERVATION_SCHEDULED_COMMAND_GUARD_FAILED "$output"; exit 1
fi
CI_COMMAND_COUNT=2 bash scripts/ci-observation.sh history 0 \
  'bash scripts/ci-doctor.sh history' 'bash ops/ci/history.sh' >/dev/null
jq 'del(.tool_versions.gitleaks)' .ci-artifacts/observations/history.json >"$test_root/history.json"
mv "$test_root/history.json" .ci-artifacts/observations/history.json
if output="$(bash ops/ci/artifact-check.sh history 2>&1)" \
  || [[ "$output" != *CI_TOOL_VERSION_MISSING* ]]; then
  refuse OBSERVATION_SCHEDULED_TOOL_GUARD_FAILED "$output"; exit 1
fi
rm -f .ci-artifacts/observations/history.json

jq '.commands=["bash scripts/ci-doctor.sh audit","bash ops/ci/audit.sh"] |
  .outcomes=[{lane:"audit",status:"PASS",exit_code:0}] | .artifact_hashes=[] |
  .tool_versions.jankurai="jankurai 1.6.11"' "$valid_observation" \
  >.ci-artifacts/observations/audit.json
bash ops/ci/artifact-check.sh audit >/dev/null
for mutation in \
  'del(.tool_versions.jankurai)' \
  '.tool_versions.jankurai="jankurai 0.0.0"'; do
  jq "$mutation" .ci-artifacts/observations/audit.json >"$test_root/audit-hostile.json"
  mv "$test_root/audit-hostile.json" .ci-artifacts/observations/audit-hostile.json
  mv .ci-artifacts/observations/audit.json "$test_root/audit-valid.json"
  mv .ci-artifacts/observations/audit-hostile.json .ci-artifacts/observations/audit.json
  if output="$(bash ops/ci/artifact-check.sh audit 2>&1)" \
    || [[ "$output" != *CI_TOOL_VERSION_* ]]; then
    refuse OBSERVATION_AUDIT_TOOL_GUARD_FAILED "$mutation: $output"; exit 1
  fi
  mv "$test_root/audit-valid.json" .ci-artifacts/observations/audit.json
done
rm -f .ci-artifacts/observations/audit.json

jq '.commands=["bash scripts/ci-doctor.sh toolchain-pinned","bash ops/ci/toolchain-pinned.sh"] |
  .outcomes=[{lane:"toolchain-pinned",status:"PASS",exit_code:0}] | .artifact_hashes=[] |
  .tool_versions += {
    rustup:"rustup 1.29.0 (123456789 2026-01-01)",b3sum:"b3sum 1.8.2",
    rustc_pinned:"rustc 1.97.1 (123456789 2026-01-01)",
    cargo_pinned:"cargo 1.97.1 (123456789 2026-01-01)"
  }' "$valid_observation" >.ci-artifacts/observations/toolchain-pinned.json
bash ops/ci/artifact-check.sh toolchain-pinned >/dev/null
for key in rustup b3sum rustc_pinned cargo_pinned; do
  jq --arg key "$key" 'del(.tool_versions[$key])' \
    .ci-artifacts/observations/toolchain-pinned.json >"$test_root/toolchain-hostile.json"
  mv .ci-artifacts/observations/toolchain-pinned.json "$test_root/toolchain-valid.json"
  mv "$test_root/toolchain-hostile.json" .ci-artifacts/observations/toolchain-pinned.json
  if output="$(bash ops/ci/artifact-check.sh toolchain-pinned 2>&1)" \
    || [[ "$output" != *CI_TOOL_VERSION_MISSING* ]]; then
    refuse OBSERVATION_TOOLCHAIN_TOOL_GUARD_FAILED "$key: $output"; exit 1
  fi
  mv "$test_root/toolchain-valid.json" .ci-artifacts/observations/toolchain-pinned.json
done
jq '.tool_versions.rustc_pinned="rustc 1.95.0 hostile"' \
  .ci-artifacts/observations/toolchain-pinned.json >"$test_root/toolchain-hostile.json"
mv .ci-artifacts/observations/toolchain-pinned.json "$test_root/toolchain-valid.json"
mv "$test_root/toolchain-hostile.json" .ci-artifacts/observations/toolchain-pinned.json
if output="$(bash ops/ci/artifact-check.sh toolchain-pinned 2>&1)" \
  || [[ "$output" != *CI_TOOL_VERSION_INVALID* ]]; then
  refuse OBSERVATION_TOOLCHAIN_TOOL_GUARD_FAILED "wrong pinned rustc: $output"; exit 1
fi
rm -f .ci-artifacts/observations/toolchain-pinned.json "$test_root/toolchain-valid.json"

guard_root="$test_root/artifact-root"
guard_outside="$test_root/outside"
mkdir -p "$guard_root" "$guard_outside"
ln -s "$guard_outside" "$guard_root/.ci-artifacts"
if prepare_ci_directory "$guard_root" .ci-artifacts; then
  refuse OBSERVATION_ROOT_SYMLINK_GUARD_FAILED "root symlink accepted"; exit 1
fi
rm "$guard_root/.ci-artifacts"
mkdir "$guard_root/.ci-artifacts"
ln -s "$guard_outside" "$guard_root/.ci-artifacts/junit"
if prepare_ci_directory "$guard_root" .ci-artifacts/junit; then
  refuse OBSERVATION_NESTED_SYMLINK_GUARD_FAILED "nested symlink accepted"; exit 1
fi

saved_observation="$observation.saved"
cp "$observation" "$saved_observation"
jq '.artifact_hashes[0].path=".ci-artifacts/../escape"' "$saved_observation" >"$observation"
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_OBSERVATION_INVALID* ]]; then
  refuse OBSERVATION_VALIDATOR_GUARD_FAILED "validator accepted traversal: $output"; exit 1
fi
secret_path=".ci-artifacts/test/credential_ghp_1234567890abcdef"
jq --arg path "$secret_path" '.artifact_hashes[0].path=$path' \
  "$saved_observation" >"$observation"
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_ARTIFACT_PATH_INVALID* ]] \
  || [[ "$output" == *credential_ghp_1234567890abcdef* ]]; then
  refuse OBSERVATION_PATH_REDACTION_GUARD_FAILED "$output"; exit 1
fi
mv "$saved_observation" "$observation"

outside="$REPO_ROOT/.ci-artifact-outside.$$"
mv .ci-artifacts/test/artifact.txt "$outside"
ln -s "../../${outside##*/}" .ci-artifacts/test/artifact.txt
if output="$(bash ops/ci/artifact-check.sh observation-test 2>&1)" \
  || [[ "$output" != *CI_ARTIFACT_PATH_INVALID* ]]; then
  refuse OBSERVATION_SYMLINK_GUARD_FAILED "validator accepted symlink: $output"; exit 1
fi
if CI_COMMAND_COUNT=1 bash scripts/ci-observation.sh observation-symlink-test 0 valid \
  .ci-artifacts/test/artifact.txt >/dev/null 2>&1; then
  refuse OBSERVATION_SYMLINK_GUARD_FAILED "producer accepted symlink"; exit 1
fi
rm .ci-artifacts/test/artifact.txt
mv "$outside" .ci-artifacts/test/artifact.txt
rm -rf .ci-artifacts/test
rm -f "$observation"
log "CI observation guards passed"
