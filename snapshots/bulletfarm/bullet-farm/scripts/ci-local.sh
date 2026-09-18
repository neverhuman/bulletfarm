#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
REPO_ROOT="$PWD"
# shellcheck source=ops/ci/artifact-path.sh
source "$REPO_ROOT/ops/ci/artifact-path.sh"
# shellcheck source=ops/ci/family-custody.sh
source "$REPO_ROOT/ops/ci/family-custody.sh"
# shellcheck source=ops/ci/scratch-floor.sh
source "$REPO_ROOT/ops/ci/scratch-floor.sh"

CI_PROOF_LOCK_RECORD=""
CI_PROOF_LOCK_SCOPE=""
CI_PROOF_LOCK_LANE=""
CI_PROOF_LOCK_OWNS=false

# Proof custody must survive an interrupted or killed session. Without these
# handlers the exclusive lock taken below is released only on the normal exit
# path, so a Ctrl-C, a SIGTERM or a killed session leaks
# .git/bullet-ci.lock.d and blocks every later hub proof until a human
# reconciles it. The handlers release ONLY the per-repo lock this process
# itself created, never another agent's; an interrupted family-wide custody
# window is a different object and stays reserved for explicit reconciliation
# in ops/ci/family.sh.
CI_PROOF_OWN_ON_ACQUIRE=1
trap ci_proof_custody_exit EXIT
trap 'ci_proof_custody_signal 129' HUP
trap 'ci_proof_custody_signal 130' INT
trap 'ci_proof_custody_signal 143' TERM

if [[ ${BULLET_CI_PROOF_CUSTODY+x} ]]; then
  unset BULLET_CI_PROOF_CUSTODY
  ci_proof_refusal "$REPO_ROOT"
  exit 75
fi

verify_proof_lock() {
  ci_proof_verify "$REPO_ROOT" bullet-farm "$CI_PROOF_LOCK_RECORD" \
    "$CI_PROOF_LOCK_SCOPE" || return $?
  [[ "$CI_PROOF_LOCK_OWNS" == true \
    && "$CI_PROOF_RECORD_PID" == "$$" \
    && "$CI_PROOF_RECORD_LANE" == "$CI_PROOF_LOCK_LANE" ]] || {
    ci_proof_refusal "$REPO_ROOT"
    return 75
  }
}

acquire_proof_lock() {
  local lane="$1"
  CI_PROOF_LOCK_LANE="$lane"
  if [[ "$lane" == family || "$lane" == family-contract ]]; then
    CI_PROOF_LOCK_SCOPE=family
  else
    CI_PROOF_LOCK_SCOPE=standalone
  fi
  CI_PROOF_LOCK_OWNS=true
  ci_proof_acquire "$REPO_ROOT" bullet-farm "$CI_PROOF_LOCK_SCOPE" "$lane" \
    CI_PROOF_LOCK_RECORD || return $?
  verify_proof_lock || return $?
}

release_proof_lock() {
  verify_proof_lock || return $?
  ci_proof_release_owned || {
    ci_proof_refusal "$REPO_ROOT"
    return 75
  }
}

run_observed_locked() {
  local lane="$1" script="$2" status artifact command_count observation_status
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
    if [[ -L "$artifact" || (-e "$artifact" && ! -f "$artifact") ]]; then
      printf 'ci-local: unsafe artifact subject %s\n' "$artifact" >&2
      return 1
    fi
    rm -f -- "$artifact" || {
      printf 'ci-local: cannot reset artifact %s\n' "$artifact" >&2
      return 1
    }
  done
  set +e
  bash scripts/ci-doctor.sh "$lane"
  status=$?
  if [[ "$status" -eq 0 ]]; then
    commands+=("bash $script")
    if [[ "$lane" == family || "$lane" == family-contract ]]; then
      BULLET_CI_PROOF_CUSTODY="$CI_PROOF_LOCK_RECORD" bash "$script"
    else
      bash "$script"
    fi
    status=$?
  fi
  set -e
  if [[ "$status" -eq 0 ]]; then
    for artifact in "$@"; do
      [[ -f "$artifact" ]] && produced+=("$artifact")
    done
  fi
  command_count="${#commands[@]}"
  verify_proof_lock || return $?
  set +e
  CI_COMMAND_COUNT="$command_count" bash scripts/ci-observation.sh "$lane" "$status" \
    "${commands[@]}" "${produced[@]}"
  observation_status=$?
  set -e
  [[ "$observation_status" -eq 0 ]] || return "$observation_status"
  return "$status"
}

run_observed() {
  local lane="$1" status
  # A lane that begins with little free space can exhaust the filesystem
  # mid-write and leave a partial artifact behind: a truncated observation or an
  # incomplete lock record. The floor is therefore checked before this lane may
  # acquire anything or run the doctor, so a refusal costs nothing and leaves
  # nothing to reconcile. This asserts no cause for any past incident, and it
  # reclaims no space. A non-zero status here is returned unchanged.
  ci_scratch_floor_check "$REPO_ROOT/target" "${TMPDIR:-/tmp}" || return $?
  acquire_proof_lock "$lane" || return $?
  if run_observed_locked "$@"; then
    status=0
  else
    status=$?
  fi
  release_proof_lock || return $?
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
