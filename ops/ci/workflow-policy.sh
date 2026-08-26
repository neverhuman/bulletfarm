#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"

validate_workflow_inventory() {
  local workflow_root="$1" entry required found
  local actual=() expected=("$workflow_root/ci.yml" "$workflow_root/scheduled.yml")
  [[ -d "$workflow_root" && ! -L "$workflow_root" ]] \
    || { refuse WORKFLOW_DIRECTORY_INVALID "$workflow_root"; return 1; }
  while IFS= read -r -d '' entry; do
    actual+=("$entry")
  done < <(find "$workflow_root" -mindepth 1 -maxdepth 1 -print0)
  [[ "${#actual[@]}" -eq "${#expected[@]}" ]] \
    || { refuse WORKFLOW_INVENTORY_DRIFT "${actual[*]}"; return 1; }
  for required in "${expected[@]}"; do
    found=false
    for entry in "${actual[@]}"; do
      [[ "$entry" == "$required" ]] && found=true
    done
    [[ "$found" == true ]] \
      || { refuse WORKFLOW_INVENTORY_DRIFT "${actual[*]}"; return 1; }
    [[ -f "$required" && ! -L "$required" ]] \
      || { refuse WORKFLOW_ENTRY_NOT_REGULAR "$required"; return 1; }
  done
}

validate_workflow_source() {
  local workflow="$1" expected_digest="$2" reason="$3" actual
  actual="$(sha256_file "$workflow")" || return 1
  [[ "$actual" == "$expected_digest" ]] \
    || { refuse "$reason" "$workflow sha256=$actual"; return 1; }
}

validate_workflow_inventory .github/workflows || exit 1
validate_workflow_source .github/workflows/ci.yml \
  34d0ca2ca91bfc59a9ce7ff6952b8b55ce45827ef02423dc65acdc0ce4c88b74 \
  HOSTED_REQUIRED_SOURCE_DRIFT || exit 1
validate_workflow_source .github/workflows/scheduled.yml \
  8930fbb2cfa34090be4f392a4d888169497872b2726a2f998c9f13cdce98248b \
  HOSTED_SCHEDULED_SOURCE_DRIFT || exit 1
validate_workflow_source ops/ci/platform-refusal.sh \
  b84718b1399d8b86e51c96dd6acc030c45d03c61d0718355b78b2b154d6f5361 \
  PLATFORM_LANE_SOURCE_DRIFT || exit 1
workflows=(.github/workflows/ci.yml .github/workflows/scheduled.yml)

# shellcheck source=ops/ci/workflow-policy-test.sh
source "$(dirname "${BASH_SOURCE[0]}")/workflow-policy-test.sh"

if rg -n 'pull_request_target|paths-ignore:|^[[:space:]]+paths:|ubuntu-latest|persist-credentials:[[:space:]]*true|actions/cache|Swatinem/rust-cache|continue-on-error:[[:space:]]*true|write-all|^[[:space:]]+[a-z-]+:[[:space:]]*write|merge-multiple:[[:space:]]*true|path:[[:space:]]*\.ci-artifacts/?$' "${workflows[@]}"; then
  refuse WORKFLOW_POLICY_VIOLATION "forbidden trigger/filter/runner/permission/cache/artifact merge"
  exit 1
fi
while IFS= read -r use_line; do
  reference="${use_line#*uses: }"
  reference="${reference%% *}"
  [[ "$reference" =~ @[0-9a-f]{40}$ ]] \
    || { refuse ACTION_REF_NOT_IMMUTABLE "$use_line"; exit 1; }
done < <(rg '^[[:space:]]*-[[:space:]]+uses:[[:space:]]+' "${workflows[@]}")

yaml_job_block() {
  local file="$1" job="$2"
  awk -v header="  $job:" '
    $0 == header { active=1 }
    active && $0 ~ /^  [A-Za-z0-9_-]+:/ && $0 != header { exit }
    active { print }
  ' "$file"
}

require_text() {
  local text="$1" needle="$2" code="$3"
  [[ "$text" == *"$needle"* ]] || { refuse "$code" "$needle"; return 1; }
}

ci=.github/workflows/ci.yml
for needle in 'name: CI' '  pull_request:' '  push:' '  merge_group:' '    types: [checks_requested]' \
  '  contents: read' "  cancel-in-progress: \${{ github.event_name == 'pull_request' }}"; do
  grep -Fqx "$needle" "$ci" || { refuse REQUIRED_WORKFLOW_CONTROL_MISSING "$needle"; exit 1; }
done

main_jobs=(source_scan fast lint contract security docs)
declare -A main_lanes
main_lanes[source_scan]=source-scan
main_lanes[fast]=fast
main_lanes[lint]=lint
main_lanes[contract]=contract
main_lanes[security]=security
main_lanes[docs]=docs
for job in "${main_jobs[@]}"; do
  lane="${main_lanes[$job]}"
  block="$(yaml_job_block "$ci" "$job")"
  [[ -n "$block" ]] || { refuse HOSTED_JOB_MISSING "$job"; exit 1; }
  require_text "$block" 'runs-on: ubuntu-24.04' HOSTED_RUNNER_DRIFT
  require_text "$block" "run: bash scripts/ci-local.sh $lane" HOSTED_LANE_EXECUTION_DRIFT
  require_text "$block" "run: bash ops/ci/artifact-check.sh $lane \"\$GITHUB_SHA\"" HOSTED_ARTIFACT_VALIDATOR_MISSING
  require_text "$block" "run: bash ops/ci/stage-artifacts.sh $lane \"\$GITHUB_SHA\"" HOSTED_ARTIFACT_STAGE_MISSING
  require_text "$block" 'id: stage' HOSTED_ARTIFACT_STAGE_ID_MISSING
  require_text "$block" "if: \${{ always() && steps.stage.outcome == 'success' }}" HOSTED_UNVALIDATED_UPLOAD_GUARD_MISSING
  require_text "$block" "name: hub-$lane-\${{ github.run_id }}-\${{ github.run_attempt }}" HOSTED_ARTIFACT_NAME_DRIFT
  require_text "$block" "path: .ci-upload/$lane/" HOSTED_ARTIFACT_PATH_DRIFT
  if [[ "$job" != source_scan ]]; then
    require_text "$block" 'needs: source_scan' HOSTED_SOURCE_SCAN_DEPENDENCY_MISSING
  fi
done
docs_block="$(yaml_job_block "$ci" docs)"
require_text "$docs_block" 'bash ops/ci/install-readme-jsonschema.sh' HOSTED_JSONSCHEMA_INSTALL_MISSING
require_text "$docs_block" '.ci-tools/readme-jsonschema/bin' HOSTED_JSONSCHEMA_PATH_MISSING

required_block="$(yaml_job_block "$ci" required)"
dollar='$'
require_text "$required_block" 'name: required' REQUIRED_WORKFLOW_CONTROL_MISSING
require_text "$required_block" "if: \${{ always() }}" REQUIRED_WORKFLOW_CONTROL_MISSING
require_text "$required_block" 'needs: [source_scan, fast, lint, contract, security, docs]' REQUIRED_WORKFLOW_CONTROL_MISSING
require_text "$required_block" "path: ${dollar}{{ runner.temp }}/bullet-atomic" REQUIRED_DOWNLOAD_LAYOUT_DRIFT
require_text "$required_block" 'merge-multiple: false' REQUIRED_DOWNLOAD_LAYOUT_DRIFT
checkout_subject_command="bash ops/ci/checkout-subject.sh \"${dollar}GITHUB_WORKSPACE\" \"${dollar}EXPECTED_COMMIT\""
require_text "$required_block" "$checkout_subject_command" REQUIRED_AGGREGATOR_SUBJECT_BINDING_MISSING
[[ "$(grep -Fc "$checkout_subject_command" <<<"$required_block")" -eq 2 ]] \
  || { refuse REQUIRED_AGGREGATOR_SUBJECT_BINDING_MISSING before-and-after; exit 1; }
require_text "$required_block" "bash ops/ci/aggregate.sh \"${dollar}RUNNER_TEMP/bullet-atomic\" \"${dollar}EXPECTED_COMMIT\"" REQUIRED_AGGREGATOR_MISSING
require_text "$required_block" "\"\${{ github.run_id }}\" \"\${{ github.run_attempt }}\"" REQUIRED_RUN_ID_BINDING_MISSING

checkout_count="$(rg -c 'uses: actions/checkout@' "${workflows[@]}" | awk -F: '{sum += $NF} END {print sum+0}')"
credential_count="$(rg -c 'persist-credentials: false' "${workflows[@]}" | awk -F: '{sum += $NF} END {print sum+0}')"
[[ "$checkout_count" -eq "$credential_count" ]] \
  || { refuse CHECKOUT_CREDENTIAL_POLICY "every checkout must disable credentials"; exit 1; }
if rg -n 'scripts/ci-local.sh (required|all)|ops/ci/required.sh' .github/workflows; then
  refuse HOSTED_DUPLICATE_REQUIRED "hosted jobs must invoke atomic lanes only"
  exit 1
fi

scheduled=.github/workflows/scheduled.yml
scheduled_jobs=(source_scan history links advisory coverage macos windows)
declare -A scheduled_lanes scheduled_runners scheduled_artifacts
scheduled_lanes[source_scan]=source-scan
scheduled_lanes[history]=history
scheduled_lanes[links]=links
scheduled_lanes[advisory]=advisory
scheduled_lanes[coverage]=coverage
scheduled_lanes[macos]=platform
scheduled_lanes[windows]=platform
scheduled_runners[source_scan]=ubuntu-24.04
scheduled_runners[history]=ubuntu-24.04
scheduled_runners[links]=ubuntu-24.04
scheduled_runners[advisory]=ubuntu-24.04
scheduled_runners[coverage]=ubuntu-24.04
scheduled_runners[macos]=macos-15
scheduled_runners[windows]=windows-2025
scheduled_artifacts[source_scan]=hub-scheduled-source-scan
scheduled_artifacts[history]=hub-history
scheduled_artifacts[links]=hub-links
scheduled_artifacts[advisory]=hub-advisory
scheduled_artifacts[coverage]=hub-coverage
scheduled_artifacts[macos]=hub-macos-refusal
scheduled_artifacts[windows]=hub-windows-refusal
for job in "${scheduled_jobs[@]}"; do
  lane="${scheduled_lanes[$job]}"
  block="$(yaml_job_block "$scheduled" "$job")"
  [[ -n "$block" ]] || { refuse SCHEDULED_JOB_MISSING "$job"; exit 1; }
  require_text "$block" "runs-on: ${scheduled_runners[$job]}" SCHEDULED_RUNNER_DRIFT
  require_text "$block" "run: bash scripts/ci-local.sh $lane" SCHEDULED_LANE_EXECUTION_DRIFT
  require_text "$block" "run: bash ops/ci/artifact-check.sh $lane \"\$GITHUB_SHA\"" SCHEDULED_ARTIFACT_VALIDATOR_MISSING
  require_text "$block" "run: bash ops/ci/stage-artifacts.sh $lane \"\$GITHUB_SHA\"" SCHEDULED_ARTIFACT_STAGE_MISSING
  require_text "$block" 'id: stage' SCHEDULED_ARTIFACT_STAGE_ID_MISSING
  require_text "$block" "if: \${{ always() && steps.stage.outcome == 'success' }}" SCHEDULED_UNVALIDATED_UPLOAD_GUARD_MISSING
  require_text "$block" "name: ${scheduled_artifacts[$job]}-\${{ github.run_id }}-\${{ github.run_attempt }}" SCHEDULED_ARTIFACT_NAME_DRIFT
  require_text "$block" "path: .ci-upload/$lane/" SCHEDULED_ARTIFACT_PATH_DRIFT
  if [[ "$job" != source_scan ]]; then
    require_text "$block" 'needs: source_scan' SCHEDULED_PREFLIGHT_GUARD_MISSING
    require_text "$block" "if: \${{ always() }}" SCHEDULED_SKIP_GUARD_MISSING
    require_text "$block" "SOURCE_SCAN_RESULT: \${{ needs.source_scan.result }}" SCHEDULED_PREFLIGHT_RESULT_DRIFT
  fi
done
for job in macos windows; do
  block="$(yaml_job_block "$scheduled" "$job")"
  require_text "$block" \
    'uses: actions/setup-python@a309ff8b426b58ec0e2a45f0f869d46889d02405 # v6.2.0' \
    SCHEDULED_PYTHON_ACTION_DRIFT
  require_text "$block" 'python-version: "3.12.12"' SCHEDULED_PYTHON_VERSION_DRIFT
  require_text "$block" 'components: clippy' SCHEDULED_CLIPPY_COMPONENT_MISSING
done
platform_source="$(<ops/ci/platform-refusal.sh)"
for needle in \
  'cargo test --locked -p bullet-wire --test canonical_hostile' \
  'cargo clippy --locked -p bullet-family --lib --bins --no-deps --' \
  '-D warnings -F clippy::disallowed_methods' \
  'cargo clippy --locked -p bullet-wire --lib --bins --no-deps --' \
  '-D warnings -D clippy::disallowed_methods'; do
  require_text "$platform_source" "$needle" PLATFORM_DECODER_POLICY_MISSING
done
grep -Fq 'tool: cargo-nextest@0.9.137,cargo-llvm-cov@0.8.7' "$scheduled" \
  || { refuse COVERAGE_TOOL_PIN_MISSING "coverage requires pinned nextest and llvm-cov"; exit 1; }

toml_job_block() {
  local job="$1"
  awk -v target="id = \"$job\"" '
    $0 == "[[job]]" {
      if (found) { exit }
      block=$0 ORS
      next
    }
    { block=block $0 ORS; if ($0 == target) { found=1 } }
    END { if (found) { printf "%s", block } }
  ' ci.toml
}

[[ "$(rg -c '^\[\[job\]\]$' ci.toml)" -eq 8 ]] \
  || { refuse JERYU_JOB_INVENTORY_DRIFT "ci.toml must declare eight prepared jobs"; exit 1; }
[[ "$(rg -c '^cache_mounts = \[\]$' ci.toml)" -eq 8 ]] \
  || { refuse JERYU_CACHE_POLICY_DRIFT "all prepared jobs must be cache-free"; exit 1; }
declare -A jeryu_artifacts
jeryu_artifacts[activation]='artifact_paths = []'
jeryu_artifacts[source_scan]='artifact_paths = [".ci-artifacts/observations/source-scan.json"]'
jeryu_artifacts[fast]='artifact_paths = [".ci-artifacts/observations/fast.json", ".ci-artifacts/junit/fast.xml"]'
jeryu_artifacts[lint]='artifact_paths = [".ci-artifacts/observations/lint.json"]'
jeryu_artifacts[contract]='artifact_paths = [".ci-artifacts/observations/contract.json", ".ci-artifacts/junit/contract.xml", ".ci-artifacts/formal/contract.json", ".ci-artifacts/formal/contract.log", ".ci-artifacts/contracts/bundle-manifest.json"]'
jeryu_artifacts[security]='artifact_paths = [".ci-artifacts/observations/security.json"]'
jeryu_artifacts[docs]='artifact_paths = [".ci-artifacts/observations/docs.json"]'
jeryu_artifacts[required]='artifact_paths = []'
for job in activation source_scan fast lint contract security docs required; do
  block="$(toml_job_block "$job")"
  [[ -n "$block" ]] || { refuse JERYU_JOB_MISSING "$job"; exit 1; }
  require_text "$block" 'run = ["bash ops/ci/jeryu-activation-gate.sh"' JERYU_ACTIVATION_GATE_MISSING
  require_text "$block" "${jeryu_artifacts[$job]}" JERYU_ARTIFACT_INVENTORY_DRIFT
  [[ "$(grep -c '^artifact_paths = ' <<<"$block")" -eq 1 ]] \
    || { refuse JERYU_ARTIFACT_INVENTORY_DRIFT "$job duplicate/missing declaration"; exit 1; }
done
source_block="$(toml_job_block source_scan)"
require_text "$source_block" 'needs = ["activation"]' JERYU_DEPENDENCY_DRIFT
for lane in fast lint contract security docs; do
  block="$(toml_job_block "$lane")"
  require_text "$block" 'needs = ["source_scan"]' JERYU_DEPENDENCY_DRIFT
  if [[ "$lane" == docs ]]; then
    require_text "$block" 'bash scripts/ci-local.sh docs' JERYU_COMMAND_DRIFT
  else
    require_text "$block" "\"bash scripts/ci-local.sh $lane\"" JERYU_COMMAND_DRIFT
  fi
done
docs_jeryu="$(toml_job_block docs)"
require_text "$docs_jeryu" '"bash ops/ci/install-readme-jsonschema.sh"' JERYU_JSONSCHEMA_INSTALL_MISSING
require_text "$docs_jeryu" '.ci-tools/readme-jsonschema/bin' JERYU_JSONSCHEMA_PATH_MISSING
required_jeryu="$(toml_job_block required)"
require_text "$required_jeryu" 'needs = ["source_scan", "fast", "lint", "contract", "security", "docs"]' JERYU_DEPENDENCY_DRIFT
if rg -n 'path = "\.\./|artifact_paths = \["\.ci-artifacts"\]|artifact_paths = \[[^]]*"/|artifact_paths = \[[^]]*"[^" ]*/\.\.?(/|\")' ci.toml; then
  refuse JERYU_POLICY_VIOLATION "sibling, absolute, traversal, or broad artifact path"
  exit 1
fi

log "workflow topology, isolated artifacts, and inactive forge-neutral graph passed"
