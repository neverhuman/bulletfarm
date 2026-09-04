#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

declare -A MISSING_TOOLS=()

lane_tools() {
  local lane="$1"
  case "$lane" in
    all)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc \
        actionlint b3sum cargo cargo-clippy cargo-deny cargo-llvm-cov cargo-nextest cat chmod \
        cmp comm cp curl date docker file gitleaks grep jankurai java ln lychee mktemp node \
        jsonschema npm rmdir rg rustc rustfmt rustup sha1sum shellcheck stat tee uname xargs zizmor
      ;;
    fast)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc \
        cargo cargo-nextest chmod grep mktemp rmdir rustc
      ;;
    lint)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc actionlint b3sum cargo cargo-clippy \
        cargo-deny cargo-nextest cmp comm cp jsonschema ln rg rustc rustfmt shellcheck
      ;;
    required)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc actionlint b3sum cargo cargo-clippy \
        cargo-deny cargo-nextest chmod cmp comm cp curl date docker file gitleaks grep java \
        jsonschema ln mktemp rg rmdir rustc rustfmt sha1sum shellcheck stat tee zizmor
      ;;
    coverage)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc cargo cargo-llvm-cov \
        cargo-nextest cmp comm grep ln mktemp rustc
      ;;
    platform)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc cargo cargo-clippy grep rustc uname
      ;;
    toolchain-pinned)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc b3sum cargo date grep rustc rustup tee
      ;;
    family|family-contract)
      printf '%s\n' \
        awk bash dirname env find git head id jq mkdir mv realpath rm sed sort tr wc actionlint b3sum cargo cargo-clippy cargo-deny \
        cargo-nextest chmod cmp comm cp curl date docker file gitleaks grep java ln mktemp \
        jsonschema node npm rg rmdir rustc rustfmt rustup sha1sum shellcheck stat tee uname zizmor
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
    [[ -e "$fixture/$tool" ]] || ln -s "$src" "$fixture/$tool"
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

for tool in b3sum curl env node npm realpath rustup jsonschema lychee cargo-llvm-cov wc; do
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
for tool in find id wc; do
  probe_with_missing_tools fast "$tool" 0
  output="${MISSING_TOOLS["output"]}"
  status="${MISSING_TOOLS["status"]}"
  [[ "$status" -ne 0 && "$output" == *"missing $tool for fast"* ]] || {
    printf '[ci] DOCTOR_GLOBAL_CUSTODY_TOOL_MISSING: %s status=%s\n' "$tool" "$status" >&2
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

lock_fixture="$(mktemp -d)"
lock_outside="$(mktemp -d)"
lock_owner_pid=
lock_child_pid=
cleanup_lock_fixture() {
  [[ -z "$lock_owner_pid" ]] || kill -KILL "$lock_owner_pid" 2>/dev/null || true
  [[ -z "$lock_child_pid" ]] || kill -KILL "$lock_child_pid" 2>/dev/null || true
  rm -rf -- "$lock_fixture" "$lock_outside"
}
trap cleanup_lock_fixture EXIT
mkdir -p "$lock_fixture/.git" "$lock_fixture/scripts" "$lock_fixture/ops/ci"
printf '%s\n' 'ref: refs/heads/main' >"$lock_fixture/.git/HEAD"
cp "$REPO_ROOT/scripts/ci-local.sh" "$lock_fixture/scripts/ci-local.sh"
cp "$REPO_ROOT/ops/ci/artifact-path.sh" "$lock_fixture/ops/ci/artifact-path.sh"
cp "$REPO_ROOT/ops/ci/family-custody.sh" "$lock_fixture/ops/ci/family-custody.sh"
cp "$REPO_ROOT/ops/ci/family-contract.sh" "$lock_fixture/ops/ci/family-contract.sh"
cp "$REPO_ROOT/ops/ci/lib.sh" "$lock_fixture/ops/ci/lib.sh"
cp "$REPO_ROOT/ops/ci/scratch-floor.sh" "$lock_fixture/ops/ci/scratch-floor.sh"
cp "$REPO_ROOT/ops/ci/rust-toolchain-boundary.sh" \
  "$lock_fixture/ops/ci/rust-toolchain-boundary.sh"
# These are literal fixture program lines.
# shellcheck disable=SC2016
printf '%s\n' '#!/usr/bin/env bash' '[[ ! ${BULLET_CI_PROOF_CUSTODY+x} ]]' \
  'printf "doctor:%s\n" "$$" >> children' \
  >"$lock_fixture/scripts/ci-doctor.sh"
# shellcheck disable=SC2016
printf '%s\n' '#!/usr/bin/env bash' 'set -eu' \
  '[[ ! ${BULLET_CI_PROOF_CUSTODY+x} ]]' \
  'mkdir -p .ci-artifacts/observations' \
  'printf "observation:%s\n" "$$" >> observation-calls' \
  "printf observed > \".ci-artifacts/observations/${dollar}1.json\"" \
  >"$lock_fixture/scripts/ci-observation.sh"
cat >"$lock_fixture/ops/ci/family.sh" <<'FIXTURE'
#!/usr/bin/env bash
set -euo pipefail
[[ ${BULLET_CI_PROOF_CUSTODY+x} ]]
record="$BULLET_CI_PROOF_CUSTODY"
unset BULLET_CI_PROOF_CUSTODY
source ops/ci/family-custody.sh
ci_proof_verify "$PWD" bullet-farm "$record" family
[[ "$CI_PROOF_RECORD_PID" == "$PPID" \
  && ("$CI_PROOF_RECORD_LANE" == family || "$CI_PROOF_RECORD_LANE" == family-contract) \
  && "$(<.git/bullet-ci.lock.d/owner)" == "$record" ]]
printf '%s\n' "$record" >"family-child-$CI_PROOF_RECORD_LANE"
mkdir -p .ci-artifacts/family
printf observed >.ci-artifacts/family/subjects.json
FIXTURE
cat >"$lock_fixture/ops/ci/fast.sh" <<'FIXTURE'
#!/usr/bin/env bash
set -eu
printf 'fast:%s\n' "$$" >> children
: > child-started
if [[ "${CI_FIXTURE_NESTED:-0}" == 1 ]]; then
  set +e
  CI_FIXTURE_NESTED=0 bash scripts/ci-local.sh fast >nested-output 2>&1
  printf '%s\n' "$?" >nested-status
  set -e
fi
while [[ "${CI_FIXTURE_HOLD:-0}" == 1 && ! -e release-child ]]; do sleep 0.05; done
exit "${CI_FIXTURE_STATUS:-0}"
FIXTURE
chmod +x "$lock_fixture/scripts/"*.sh "$lock_fixture/ops/ci/"*.sh

wait_for_lock_fixture() {
  local path="$1"
  for _ in {1..200}; do
    [[ -e "$path" ]] && return 0
    sleep 0.05
  done
  printf '[ci] CI_PROOF_LOCK_TEST_TIMEOUT: %s\n' "$path" >&2
  return 1
}

start_lock_owner() {
  local requested_umask="${1:-}"
  rm -f -- "$lock_fixture/child-started" "$lock_fixture/release-child" \
    "$lock_fixture/children" "$lock_fixture/nested-output" "$lock_fixture/nested-status"
  (
    [[ -z "$requested_umask" ]] || umask "$requested_umask"
    cd "$lock_fixture"
    exec env CI_FIXTURE_HOLD=1 bash scripts/ci-local.sh fast
  ) >"$lock_fixture/owner-output" 2>&1 &
  lock_owner_pid=$!
  wait_for_lock_fixture "$lock_fixture/child-started"
}

for supplied_custody in '' hostile; do
  rm -f -- "$lock_fixture/children" "$lock_fixture/observation-calls"
  set +e
  lock_output="$(cd "$lock_fixture" \
    && env BULLET_CI_PROOF_CUSTODY="$supplied_custody" bash scripts/ci-local.sh fast 2>&1)"
  lock_status=$?
  set -e
  [[ "$lock_status" -eq 75 && "$lock_output" == *CI_PROOF_LOCKED_OR_STALE* \
    && ! -e "$lock_fixture/children" && ! -e "$lock_fixture/observation-calls" \
    && ! -e "$lock_fixture/.git/bullet-ci.lock.d" ]] \
    || { echo '[ci] CI_PROOF_CALLER_CUSTODY_REFUSAL_INVALID' >&2; exit 1; }
done

for family_lane in family family-contract; do
  rm -f -- "$lock_fixture/family-child-family" \
    "$lock_fixture/family-child-family-contract"
  (cd "$lock_fixture" && bash scripts/ci-local.sh "$family_lane" >/dev/null)
  family_record="$(<"$lock_fixture/family-child-$family_lane")"
  [[ "$family_record" =~ ^schema=2\ repository=bullet-farm\ scope=family\ pid=[1-9][0-9]*\ lane=$family_lane\ nonce=[0-9]+-[0-9]+-[0-9]+-[0-9]+$ \
    && ! -e "$lock_fixture/.git/bullet-ci.lock.d" ]]
done

start_lock_owner 000
[[ "$(find "$lock_fixture/.git/bullet-ci.lock.d" -maxdepth 0 -type d -perm 0700 -print)" \
    == "$lock_fixture/.git/bullet-ci.lock.d" \
  && "$(find "$lock_fixture/.git/bullet-ci.lock.d/owner" -maxdepth 0 -type f -perm 0600 -print)" \
    == "$lock_fixture/.git/bullet-ci.lock.d/owner" ]] \
  || { echo '[ci] CI_PROOF_LOCK_MODE_INVALID' >&2; exit 1; }
mkdir -p "$lock_fixture/.ci-artifacts/junit"
printf preserve >"$lock_fixture/.ci-artifacts/junit/fast.xml"
set +e
lock_output="$(cd "$lock_fixture" && bash scripts/ci-local.sh fast 2>&1)"
lock_status=$?
set -e
[[ "$lock_status" -eq 75 && "$lock_output" == *CI_PROOF_LOCKED_OR_STALE* \
  && "$(<"$lock_fixture/.ci-artifacts/junit/fast.xml")" == preserve \
  && "$(wc -l <"$lock_fixture/children")" -eq 2 \
  && ! -e "$lock_fixture/.ci-artifacts/observations/fast.json" ]] || {
  echo '[ci] CI_PROOF_OVERLAP_REFUSAL_INVALID' >&2
  exit 1
}
: >"$lock_fixture/release-child"
wait "$lock_owner_pid"
lock_owner_pid=
[[ ! -e "$lock_fixture/.git/bullet-ci.lock.d" ]] \
  || { echo '[ci] CI_PROOF_PASS_LOCK_RETAINED' >&2; exit 1; }

set +e
(cd "$lock_fixture" && CI_FIXTURE_STATUS=19 bash scripts/ci-local.sh fast >/dev/null 2>&1)
lock_status=$?
set -e
[[ "$lock_status" -eq 19 && ! -e "$lock_fixture/.git/bullet-ci.lock.d" ]] \
  || { echo '[ci] CI_PROOF_FAIL_LOCK_RETAINED' >&2; exit 1; }

rm -f "$lock_fixture/children"
mkdir -p "$lock_fixture/.ci-artifacts/junit" "$lock_fixture/.ci-artifacts/observations"
rm -f "$lock_fixture/.ci-artifacts/junit/fast.xml"
mkdir "$lock_fixture/.ci-artifacts/junit/fast.xml"
printf preserve >"$lock_fixture/.ci-artifacts/observations/fast.json"
set +e
(cd "$lock_fixture" && bash scripts/ci-local.sh fast >/dev/null 2>&1)
lock_status=$?
set -e
[[ "$lock_status" -eq 1 && ! -e "$lock_fixture/children" \
  && "$(<"$lock_fixture/.ci-artifacts/observations/fast.json")" == preserve \
  && ! -e "$lock_fixture/.git/bullet-ci.lock.d" ]] \
  || { echo '[ci] CI_PROOF_ARTIFACT_TYPE_REFUSAL_INVALID' >&2; exit 1; }
rmdir "$lock_fixture/.ci-artifacts/junit/fast.xml"

(cd "$lock_fixture" && CI_FIXTURE_NESTED=1 bash scripts/ci-local.sh fast >/dev/null 2>&1)
[[ "$(<"$lock_fixture/nested-status")" -eq 75 \
  && "$(<"$lock_fixture/nested-output")" == *CI_PROOF_LOCKED_OR_STALE* ]] \
  || { echo '[ci] CI_PROOF_NESTED_REFUSAL_INVALID' >&2; exit 1; }

start_lock_owner
while IFS=: read -r kind pid; do
  [[ "$kind" != fast ]] || lock_child_pid="$pid"
done <"$lock_fixture/children"
kill -KILL "$lock_owner_pid"
set +e
wait "$lock_owner_pid" 2>/dev/null
set -e
lock_owner_pid=
kill -TERM "$lock_child_pid" 2>/dev/null || true
for _ in {1..100}; do
  kill -0 "$lock_child_pid" 2>/dev/null || break
  sleep 0.05
done
if kill -0 "$lock_child_pid" 2>/dev/null; then
  kill -KILL "$lock_child_pid" 2>/dev/null || true
  for _ in {1..100}; do
    kill -0 "$lock_child_pid" 2>/dev/null || break
    sleep 0.05
  done
fi
kill -0 "$lock_child_pid" 2>/dev/null \
  && { echo '[ci] CI_PROOF_CRASH_CHILD_SURVIVED' >&2; exit 1; }
lock_child_pid=
set +e
lock_output="$(cd "$lock_fixture" && bash scripts/ci-local.sh fast 2>&1)"
lock_status=$?
set -e
[[ "$lock_status" -eq 75 && "$lock_output" == *CI_PROOF_LOCKED_OR_STALE* ]] \
  || { echo '[ci] CI_PROOF_CRASH_STALE_REFUSAL_INVALID' >&2; exit 1; }
rm -- "$lock_fixture/.git/bullet-ci.lock.d/owner"
rmdir -- "$lock_fixture/.git/bullet-ci.lock.d"

printf preserve >"$lock_outside/sentinel"
rm -- "$lock_fixture/.git/HEAD"
rmdir "$lock_fixture/.git"
ln -s "$lock_outside" "$lock_fixture/.git"
set +e
(cd "$lock_fixture" && bash scripts/ci-local.sh fast >/dev/null 2>&1)
lock_status=$?
set -e
[[ "$lock_status" -eq 75 && "$(<"$lock_outside/sentinel")" == preserve \
  && ! -e "$lock_outside/bullet-ci.lock.d" ]] \
  || { echo '[ci] CI_PROOF_GIT_SYMLINK_REFUSAL_INVALID' >&2; exit 1; }
rm "$lock_fixture/.git"
mkdir "$lock_fixture/.git"
printf '%s\n' 'ref: refs/heads/main' >"$lock_fixture/.git/HEAD"
ln -s "$lock_outside" "$lock_fixture/.git/bullet-ci.lock.d"
set +e
(cd "$lock_fixture" && bash scripts/ci-local.sh fast >/dev/null 2>&1)
lock_status=$?
set -e
[[ "$lock_status" -eq 75 && "$(<"$lock_outside/sentinel")" == preserve ]] \
  || { echo '[ci] CI_PROOF_LOCK_SYMLINK_REFUSAL_INVALID' >&2; exit 1; }
rm "$lock_fixture/.git/bullet-ci.lock.d"

start_lock_owner
observation_count_before="$(wc -l <"$lock_fixture/observation-calls")"
rm "$lock_fixture/.git/bullet-ci.lock.d/owner"
ln -s "$lock_outside/sentinel" "$lock_fixture/.git/bullet-ci.lock.d/owner"
: >"$lock_fixture/release-child"
set +e
wait "$lock_owner_pid"
lock_status=$?
set -e
lock_owner_pid=
[[ "$lock_status" -eq 75 && "$(<"$lock_outside/sentinel")" == preserve \
  && "$(wc -l <"$lock_fixture/observation-calls")" -eq "$observation_count_before" \
  && -L "$lock_fixture/.git/bullet-ci.lock.d/owner" ]] \
  || { echo '[ci] CI_PROOF_OWNER_SYMLINK_REFUSAL_INVALID' >&2; exit 1; }
rm "$lock_fixture/.git/bullet-ci.lock.d/owner"
rmdir "$lock_fixture/.git/bullet-ci.lock.d"

cleanup_lock_fixture
trap - EXIT
printf '[ci] ci-doctor all-lane union guards passed\n'
