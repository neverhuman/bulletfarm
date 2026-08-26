#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
REPO_ROOT="$PWD"
# shellcheck source=ops/ci/artifact-path.sh
source "$REPO_ROOT/ops/ci/artifact-path.sh"

run_observed() {
  local lane="$1" script="$2" status artifact command_count
  shift 2
  local -a produced=() commands=("bash scripts/ci-doctor.sh $lane")
  command -v realpath >/dev/null 2>&1 || {
    printf 'ci-local: missing required tool realpath before %s doctor\n' "$lane" >&2
    return 1
  }
  prepare_ci_directory "$REPO_ROOT" .ci-artifacts || {
    printf 'ci-local: unsafe .ci-artifacts root\n' >&2
    return 1
  }
  for artifact in "$@"; do
    [[ "$artifact" == .ci-artifacts/* ]] || {
      printf 'ci-local: unsafe artifact path %s\n' "$artifact" >&2
      return 1
    }
    prepare_ci_directory "$REPO_ROOT" "${artifact%/*}" || {
      printf 'ci-local: unsafe artifact parent %s\n' "${artifact%/*}" >&2
      return 1
    }
    rm -f -- "$artifact"
  done
  set +e
  bash scripts/ci-doctor.sh "$lane"
  status=$?
  if [[ "$status" -eq 0 ]]; then
    commands+=("bash $script")
    bash "$script"
    status=$?
  fi
  set -e
  if [[ "$status" -eq 0 ]]; then
    for artifact in "$@"; do
      [[ -f "$artifact" ]] && produced+=("$artifact")
    done
  fi
  command_count="${#commands[@]}"
  CI_COMMAND_COUNT="$command_count" bash scripts/ci-observation.sh "$lane" "$status" \
    "${commands[@]}" "${produced[@]}"
  return "$status"
}

lane="${1:-required}"
case "$lane" in
  source-scan) run_observed source-scan ops/ci/source-scan.sh ;;
  fast) run_observed fast ops/ci/fast.sh .ci-artifacts/junit/fast.xml ;;
  lint) run_observed lint ops/ci/lint.sh ;;
  contract) run_observed contract ops/ci/contract.sh \
    .ci-artifacts/junit/contract.xml .ci-artifacts/formal/contract.json \
    .ci-artifacts/formal/contract.log .ci-artifacts/contracts/bundle-manifest.json ;;
  security) run_observed security ops/ci/security.sh ;;
  docs) run_observed docs ops/ci/docs.sh ;;
  required) run_observed required ops/ci/required.sh \
    .ci-artifacts/junit/fast.xml .ci-artifacts/junit/contract.xml \
    .ci-artifacts/formal/contract.json .ci-artifacts/formal/contract.log \
    .ci-artifacts/contracts/bundle-manifest.json ;;
  family) run_observed family ops/ci/family.sh .ci-artifacts/family/subjects.json ;;
  family-contract) run_observed family-contract ops/ci/family-contract.sh .ci-artifacts/family/subjects.json ;;
  history) run_observed history ops/ci/history.sh ;;
  links) run_observed links ops/ci/external-links.sh ;;
  advisory) run_observed advisory ops/ci/advisory.sh ;;
  coverage) run_observed coverage ops/ci/coverage.sh .ci-artifacts/coverage/cobertura.xml ;;
  platform) run_observed platform ops/ci/platform-refusal.sh ;;
  audit) run_observed audit ops/ci/audit.sh ;;
  toolchain-pinned) run_observed toolchain-pinned ops/ci/toolchain-pinned.sh ;;
  all) run_observed required ops/ci/required.sh \
    .ci-artifacts/junit/fast.xml .ci-artifacts/junit/contract.xml \
    .ci-artifacts/formal/contract.json .ci-artifacts/formal/contract.log \
    .ci-artifacts/contracts/bundle-manifest.json ;;
  *)
    echo "usage: $0 {source-scan|fast|lint|contract|security|docs|required|family|family-contract|history|links|advisory|coverage|platform|audit|toolchain-pinned|all}" >&2
    exit 2
    ;;
esac
