#!/usr/bin/env bash
set -euo pipefail
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
# shellcheck source=ops/ci/family-custody.sh
source "$REPO_ROOT/ops/ci/family-custody.sh"
fixture="$(mktemp -d)" outside="$(mktemp -d)"
cleanup() { rm -rf -- "$fixture" "$outside"; }
trap cleanup EXIT
for repository in bullet-farm bullet-git bullet-kernel bullet-portal; do
  mkdir -p "$fixture/$repository/.git"
  printf '%s\n' 'ref: refs/heads/main' >"$fixture/$repository/.git/HEAD"
done
expect_status_75() {
  local status
  set +e
  "$@" >"$fixture/refusal.out" 2>&1
  status=$?
  set -e
  [[ "$status" -eq 75 && "$(<"$fixture/refusal.out")" == *CI_PROOF_LOCKED_OR_STALE* ]]
}
farm="$fixture/bullet-farm" standalone_record=''
(
  umask 000
  ci_proof_acquire "$farm" bullet-farm standalone fast standalone_record
  [[ "$CI_PROOF_RECORD_PID" == "$$" && "$CI_PROOF_RECORD_LANE" == fast ]]
  [[ "$(find "$farm/.git/bullet-ci.lock.d" -maxdepth 0 -type d -uid "$(id -u)" -perm 0700 -print)" \
      == "$farm/.git/bullet-ci.lock.d" ]]
  [[ "$(find "$farm/.git/bullet-ci.lock.d/owner" -maxdepth 0 -type f -uid "$(id -u)" -perm 0600 -print)" \
      == "$farm/.git/bullet-ci.lock.d/owner" ]]
  ci_proof_release "$farm" bullet-farm "$standalone_record" standalone
)
[[ ! -e "$farm/.git/bullet-ci.lock.d" ]]
ci_proof_acquire "$farm" bullet-farm standalone fast standalone_record
printf preserve >"$fixture/protected-report"
expect_status_75 ci_proof_acquire "$farm" bullet-farm standalone fast replay_record
[[ "$(<"$fixture/protected-report")" == preserve ]]
ci_proof_release "$farm" bullet-farm "$standalone_record" standalone
expect_status_75 ci_proof_verify "$farm" bullet-farm "$standalone_record" standalone
ci_proof_acquire "$farm" bullet-farm standalone fast standalone_record
printf '\n' >>"$farm/.git/bullet-ci.lock.d/owner"
expect_status_75 ci_proof_verify "$farm" bullet-farm "$standalone_record" standalone
printf '%s\n' "$standalone_record" >"$farm/.git/bullet-ci.lock.d/owner"
printf '\0' >>"$farm/.git/bullet-ci.lock.d/owner"
expect_status_75 ci_proof_verify "$farm" bullet-farm "$standalone_record" standalone
printf '%s\n' "$standalone_record" >"$farm/.git/bullet-ci.lock.d/owner"
ci_proof_release "$farm" bullet-farm "$standalone_record" standalone
mkdir "$fixture/hostile-git"
printf outside >"$outside/sentinel"
ln -s "$outside" "$fixture/hostile-git/.git"
expect_status_75 ci_proof_acquire "$fixture/hostile-git" hostile-git standalone fast hostile_record
[[ "$(<"$outside/sentinel")" == outside ]]
family_custody_initialize "$fixture" family
kernel_record=''
ci_proof_acquire "$fixture/bullet-kernel" bullet-kernel standalone fast kernel_record
printf preserve >"$fixture/member-report"
expect_status_75 family_custody_acquire_all
[[ ! -e "$fixture/bullet-git/.git/bullet-ci.lock.d" \
  && -d "$fixture/bullet-kernel/.git/bullet-ci.lock.d" \
  && ! -e "$fixture/bullet-portal/.git/bullet-ci.lock.d" \
  && "$(<"$fixture/member-report")" == preserve ]]
ci_proof_release "$fixture/bullet-kernel" bullet-kernel "$kernel_record" standalone
family_custody_initialize "$fixture" family
family_custody_acquire_all
family_custody_verify_all
for repository in bullet-git bullet-kernel bullet-portal; do
  record="$(family_custody_record "$repository")"
  [[ "$record" =~ ^schema=2\ repository=$repository\ scope=family\ pid=$$\ lane=family\ nonce=[0-9]+-[0-9]+-[0-9]+-[0-9]+$ ]]
  expect_status_75 ci_proof_acquire "$fixture/$repository" "$repository" standalone fast replay_record
done
portal_record="$(family_custody_record bullet-portal)"
printf '%s\n' 'schema=2 repository=bullet-portal scope=family pid=1 lane=family nonce=1-2-3-4' \
  >"$fixture/bullet-portal/.git/bullet-ci.lock.d/owner"
expect_status_75 family_custody_verify_all
printf '%s\n' "$portal_record" >"$fixture/bullet-portal/.git/bullet-ci.lock.d/owner"
family_custody_release_all
for repository in bullet-git bullet-kernel bullet-portal; do
  [[ ! -e "$fixture/$repository/.git/bullet-ci.lock.d" ]]
done
if rg -n 'BULLET_CI_INHERITED_OWNER' \
  "$REPO_ROOT/scripts/ci-local.sh" "$REPO_ROOT/ops/ci/family.sh" \
  "$REPO_ROOT/ops/ci/family-custody.sh"; then
  echo '[ci] CI_PROOF_CUSTODY_LEGACY_VARIABLE_PRESENT' >&2
  exit 1
fi
family_source="$REPO_ROOT/ops/ci/family.sh"
acquire_line="$(grep -nF 'family_custody_acquire_all || exit $?' "$family_source" | head -n 1 | cut -d: -f1)"
# shellcheck disable=SC2016
reset_line="$(grep -nF 'rm -f -- "$(member_root "$report_member")/$relative"' \
  "$family_source" | head -n 1 | cut -d: -f1)"
publication_line="$(grep -nF 'assert_family_subjects after-observation-publication' \
  "$family_source" | tail -n 1 | cut -d: -f1)"
verify_line="$(grep -nF 'family_custody_verify_all || exit $?' \
  "$family_source" | tail -n 1 | cut -d: -f1)"
[[ "$acquire_line" =~ ^[0-9]+$ && "$reset_line" =~ ^[0-9]+$ \
  && "$publication_line" =~ ^[0-9]+$ && "$verify_line" =~ ^[0-9]+$ \
  && "$acquire_line" -lt "$reset_line" && "$publication_line" -lt "$verify_line" ]]
expected_reports=(
  'bullet-git|.ci-artifacts/reports/fast.junit.xml' 'bullet-git|.ci-artifacts/reports/contract.junit.xml'
  'bullet-kernel|.ci-artifacts/junit/fast.xml' 'bullet-kernel|.ci-artifacts/junit/contract.xml'
  'bullet-kernel|.ci-artifacts/junit/family.xml' 'bullet-portal|.ci-artifacts/reports/vitest.json'
  'bullet-portal|.ci-artifacts/reports/playwright.xml' 'bullet-portal|.ci-artifacts/reports/real-farmd.xml'
  'bullet-farm|.ci-artifacts/junit/contract.xml' 'bullet-farm|.ci-artifacts/formal/contract.json'
  'bullet-farm|.ci-artifacts/formal/contract.log'
)
report_specs_source="$(sed -n '/^report_specs=(/,/^)/p' "$family_source")"
for report in "${expected_reports[@]}"; do
  grep -Fq "$report|" <<<"$report_specs_source"
done
[[ "$(grep -cE "^[[:space:]]+[\"']bullet-(farm|git|kernel|portal)\\|" \
  <<<"$report_specs_source")" \
  -eq "${#expected_reports[@]}" ]]
grep -Fxq 'exec bash ops/ci/family.sh' "$REPO_ROOT/ops/ci/family-contract.sh"
grep -Fq "trap 'family_custody_uncertain=1; exit 129' HUP" "$family_source"
grep -Fq "trap 'family_custody_uncertain=1; exit 130' INT" "$family_source"
grep -Fq "trap 'family_custody_uncertain=1; exit 143' TERM" "$family_source"
custody_source="$REPO_ROOT/ops/ci/family-custody.sh" hub_local="$REPO_ROOT/scripts/ci-local.sh"
family_custody_initialize "$fixture" family
for repository in bullet-git bullet-kernel bullet-portal; do
  [[ "${FAMILY_CUSTODY_ROOTS[$repository]}" == "$fixture/$repository" ]]
done
fixture_lock_dirs() {
  find "$fixture" -type d -name bullet-ci.lock.d -print | sort
}
fixture_lock_list() {
  local root
  for root in "$@"; do
    printf '%s\n' "$fixture/$root/.git/bullet-ci.lock.d"
  done | sort
}
cat >"$fixture/holder.sh" <<'HOLDER'
#!/usr/bin/env bash
set -euo pipefail
custody_source="$1" root="$2" mode="$3" ready="$4"
# shellcheck source=/dev/null
source "$custody_source"
CI_PROOF_OWN_ON_ACQUIRE=1
ci_proof_custody_trap
if [[ "$mode" == first-signal ]]; then
  CI_PROOF_SIGNAL_CRITICAL_BASHPID="$BASHPID"
  CI_PROOF_SIGNAL_PENDING=129
  trap_injected=0
  trap() {
    builtin trap "$@"
    if [[ "$#" -eq 4 && -z "$1" && "$2" == HUP && "$3" == INT && "$4" == TERM && "$trap_injected" -eq 0 ]]; then
      trap_injected=1
      builtin printf disarmed >"$ready"
      kill -TERM "$$"
    fi
  }
  ci_proof_signal_critical_end
fi
partial_signal="${mode##*-}"
case "$mode" in
  partial-mkdir-*)
    mkdir() {
      command mkdir "$@"
      builtin printf checkpoint >"$ready"
      kill "-$partial_signal" "$$"
    }
    ;;
  partial-owner-*)
    printf() {
      builtin printf "$@"
      if [[ "$BASHPID" != "$$" ]]; then
        builtin printf checkpoint >"$ready"
        kill "-$partial_signal" "$$"
      fi
    }
    ;;
  mkdir-created-failure)
    mkdir() {
      command mkdir "$@"
      return 143
    }
    ;;
esac
holder_record=''
ci_proof_acquire "$root" bullet-farm standalone fast holder_record || exit $?
printf '%s\n' "$holder_record" >"$ready"
if [[ "$mode" =~ ^signal-(before-rm|after-rm|after-rmdir)-(HUP|INT|TERM)$ ]]; then
  release_stage="${BASH_REMATCH[1]}"
  release_signal="${BASH_REMATCH[2]}"
  case "$release_stage" in
    before-rm) rm() { kill "-$release_signal" "$$"; command rm "$@"; } ;;
    after-rm) rm() { command rm "$@"; kill "-$release_signal" "$$"; } ;;
    after-rmdir) rmdir() { command rmdir "$@"; kill "-$release_signal" "$$"; } ;;
  esac
fi
case "$mode" in
  fail) exit 3 ;;
  fail-refusal)
    printf '%s\n' 'schema=2 repository=bullet-farm scope=standalone pid=1 lane=fast nonce=1-2-3-4' \
      >"$root/.git/bullet-ci.lock.d/owner"
    exit 3
    ;;
  subshell)
    (ci_proof_release_owned)
    [[ -d "$root/.git/bullet-ci.lock.d" ]]
    ci_proof_release_owned
    exit 0
    ;;
  foreground)
    bash "${ready%/*}/foreground-child.sh" "${ready%/*}/foreground.pid"
    exit $?
    ;;
  partial-*) exit 71 ;;
  release)
    ci_proof_release_owned || exit $?
    ci_proof_release_owned || exit $?
    exit 0
    ;;
  signal-*) ci_proof_release_owned; exit $? ;;
  *) ;;
esac
for ((waited = 0; waited < 600; waited++)); do
  sleep 0.05
done
exit 70
HOLDER
cat >"$fixture/foreground-child.sh" <<'FOREGROUND_CHILD'
#!/usr/bin/env bash
printf '%s\n' "$$" >"$1"
exec sleep 30
FOREGROUND_CHILD
cat >"$fixture/family-window.sh" <<'FAMILY_WINDOW'
#!/usr/bin/env bash
set -euo pipefail
custody_source="$1" family_root="$2" pid_file="$3"
# shellcheck source=/dev/null
source "$custody_source"
[[ ${BULLET_CI_PROOF_CUSTODY+x} ]] || { ci_proof_refusal "$family_root/bullet-farm"; exit 75; }
hub_custody="$BULLET_CI_PROOF_CUSTODY"
unset BULLET_CI_PROOF_CUSTODY
ci_proof_verify "$family_root/bullet-farm" bullet-farm "$hub_custody" family || exit $?
family_custody_initialize "$family_root" family || exit $?
family_custody_verify_hub "$family_root/bullet-farm" "$hub_custody" family || exit $?
family_custody_uncertain=0
release_family_custody() {
  local original_status=$? release_status=0
  trap - EXIT HUP INT TERM
  if [[ "$family_custody_uncertain" -eq 1 ]]; then
    printf '%s\n' \
      '[ci] CI_PROOF_LOCKED_OR_STALE: interrupted family custody remains reserved for explicit reconciliation' >&2
    exit "$original_status"
  fi
  set +e
  [[ "$FAMILY_CUSTODY_ACTIVE" -eq 0 ]] || family_custody_release_all || release_status=75
  if [[ "$release_status" -ne 0 ]]; then
    exit "$release_status"
  fi
  exit "$original_status"
}
trap release_family_custody EXIT
trap 'family_custody_uncertain=1; exit 129' HUP
trap 'family_custody_uncertain=1; exit 130' INT
trap 'family_custody_uncertain=1; exit 143' TERM
family_custody_acquire_all || exit $?
family_custody_verify_all || exit $?
printf '%s\n' "$$" >"$pid_file"
for ((waited = 0; waited < 600; waited++)); do
  sleep 0.05
done
exit 70
FAMILY_WINDOW
cat >"$fixture/family-hub.sh" <<'FAMILY_HUB'
#!/usr/bin/env bash
set -euo pipefail
custody_source="$1" family_root="$2" window="$3" ready="$4" pid_file="$5"
# shellcheck source=/dev/null
source "$custody_source"
CI_PROOF_OWN_ON_ACQUIRE=1
ci_proof_custody_trap
hub_record=''
ci_proof_acquire "$family_root/bullet-farm" bullet-farm family family hub_record || exit $?
printf '%s\n' "$hub_record" >"$ready"
set +e
BULLET_CI_PROOF_CUSTODY="$hub_record" bash "$window" "$custody_source" "$family_root" "$pid_file"
window_status=$?
set -e
exit "$window_status"
FAMILY_HUB
holder_pid=''
holder_start() {
  local root="$1" mode="$2" waited
  : >"$fixture/holder.ready"
  : >"$fixture/holder.err"
  bash "$fixture/holder.sh" "$custody_source" "$root" "$mode" "$fixture/holder.ready" \
    >"$fixture/holder.out" 2>"$fixture/holder.err" &
  holder_pid=$!
  for ((waited = 0; waited < 600; waited++)); do
    if [[ -s "$fixture/holder.ready" ]]; then
      return 0
    fi
    sleep 0.05
  done
  return 1
}
REAPED_STATUS=0
reap_status() {
  set +e
  wait "$1"
  REAPED_STATUS=$?
  set -e
}
for checkpoint in mkdir owner; do
  for signal_case in HUP:129 INT:130 TERM:143; do
    signal_name="${signal_case%%:*}"
    expected_status="${signal_case#*:}"
    : >"$fixture/holder.ready"
    set +e
    env --default-signal="$signal_name" bash "$fixture/holder.sh" "$custody_source" \
      "$farm" "partial-$checkpoint-$signal_name" "$fixture/holder.ready" >/dev/null 2>"$fixture/holder.err"
    partial_status=$?
    set -e
    [[ "$partial_status" -eq "$expected_status" && ! -e "$farm/.git/bullet-ci.lock.d" ]]
  done
done
set +e
env --default-signal=HUP --default-signal=TERM bash "$fixture/holder.sh" "$custody_source" \
  "$farm" first-signal "$fixture/holder.ready" >/dev/null 2>"$fixture/holder.err"
first_signal_status=$?
set -e
[[ "$first_signal_status" -eq 129 && "$(<"$fixture/holder.ready")" == disarmed && ! -e "$farm/.git/bullet-ci.lock.d" ]]
for checkpoint in before-rm after-rm after-rmdir; do
  for signal_case in HUP:129 INT:130 TERM:143; do
    signal_name="${signal_case%%:*}"
    expected_status="${signal_case#*:}"
    set +e
    env --default-signal="$signal_name" bash "$fixture/holder.sh" "$custody_source" \
      "$farm" "signal-$checkpoint-$signal_name" "$fixture/holder.ready" \
      >/dev/null 2>"$fixture/holder.err"
    release_signal_status=$?
    set -e
    [[ "$release_signal_status" -eq "$expected_status" \
      && ! -e "$farm/.git/bullet-ci.lock.d" ]]
  done
done
set +e
bash "$fixture/holder.sh" "$custody_source" "$farm" mkdir-created-failure \
  "$fixture/holder.ready" >/dev/null 2>"$fixture/holder.err"
mkdir_failure_status=$?
set -e
[[ "$mkdir_failure_status" -eq 75 && -d "$farm/.git/bullet-ci.lock.d" \
  && ! -e "$farm/.git/bullet-ci.lock.d/owner" \
  && "$(<"$fixture/holder.err")" == *CI_PROOF_LOCKED_OR_STALE* ]]
rmdir -- "$farm/.git/bullet-ci.lock.d"
holder_start "$farm" subshell
reap_status "$holder_pid"
[[ "$REAPED_STATUS" -eq 0 && ! -e "$farm/.git/bullet-ci.lock.d" ]]
: >"$fixture/foreground.pid"
holder_start "$farm" foreground
for ((waited = 0; waited < 600; waited++)); do
  [[ ! -s "$fixture/foreground.pid" ]] || break
  sleep 0.01
done
[[ -s "$fixture/foreground.pid" ]]
foreground_pid="$(<"$fixture/foreground.pid")"
kill -TERM "$holder_pid"
sleep 0.1
kill -0 "$holder_pid"
kill -0 "$foreground_pid"
[[ -d "$farm/.git/bullet-ci.lock.d" ]]
kill -TERM "$foreground_pid"
reap_status "$holder_pid"
[[ "$REAPED_STATUS" -eq 143 && ! -e "$farm/.git/bullet-ci.lock.d" ]]
[[ -z "$(fixture_lock_dirs)" ]]
holder_start "$farm" hold
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" ]]
kill -TERM "$holder_pid"
reap_status "$holder_pid"
term_status="$REAPED_STATUS"
[[ "$term_status" -eq 143 ]]
[[ -z "$(fixture_lock_dirs)" ]]
holder_start "$farm" hold
killed_record="$(<"$fixture/holder.ready")"
kill -KILL "$holder_pid"
{ reap_status "$holder_pid"; } 2>/dev/null
kill_status="$REAPED_STATUS"
[[ "$kill_status" -eq 137 ]]
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" \
  && "$(<"$farm/.git/bullet-ci.lock.d/owner")" == "$killed_record" ]]
expect_status_75 ci_proof_acquire "$farm" bullet-farm standalone fast stolen_record
[[ "$(<"$farm/.git/bullet-ci.lock.d/owner")" == "$killed_record" ]]
set +e
bash "$fixture/holder.sh" "$custody_source" "$farm" hold "$fixture/holder.ready" \
  >"$fixture/holder.out" 2>"$fixture/holder.err"
loser_status=$?
set -e
[[ "$loser_status" -eq 75 && "$(<"$fixture/holder.err")" == *CI_PROOF_LOCKED_OR_STALE* ]]
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" \
  && "$(<"$farm/.git/bullet-ci.lock.d/owner")" == "$killed_record" ]]
rm -- "$farm/.git/bullet-ci.lock.d/owner"
set +e
bash "$fixture/holder.sh" "$custody_source" "$farm" hold "$fixture/holder.ready" \
  >"$fixture/holder.out" 2>"$fixture/holder.err"
empty_loser_status=$?
set -e
[[ "$empty_loser_status" -eq 75 ]]
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" ]]
rmdir -- "$farm/.git/bullet-ci.lock.d"
[[ -z "$(fixture_lock_dirs)" ]]
holder_start "$farm" fail
reap_status "$holder_pid"
fail_status="$REAPED_STATUS"
[[ "$fail_status" -eq 3 ]]
[[ -z "$(fixture_lock_dirs)" ]]
holder_start "$farm" fail-refusal
reap_status "$holder_pid"
[[ "$REAPED_STATUS" -eq 3 && "$(<"$fixture/holder.err")" == *CI_PROOF_LOCKED_OR_STALE* ]]
[[ -d "$farm/.git/bullet-ci.lock.d" ]]
rm -- "$farm/.git/bullet-ci.lock.d/owner"
rmdir -- "$farm/.git/bullet-ci.lock.d"
holder_start "$farm" release
reap_status "$holder_pid"
release_status="$REAPED_STATUS"
[[ "$release_status" -eq 0 ]]
[[ -z "$(fixture_lock_dirs)" ]]
victim_record=''
ci_proof_acquire "$farm" bullet-farm standalone fast victim_record
ci_proof_own "$farm" bullet-farm standalone \
  'schema=2 repository=bullet-farm scope=standalone pid=1 lane=fast nonce=1-2-3-4'
expect_status_75 ci_proof_release_owned
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" \
  && "$(<"$farm/.git/bullet-ci.lock.d/owner")" == "$victim_record" ]]
ci_proof_own "$farm" bullet-farm standalone "$victim_record"
CI_PROOF_OWNED_BASHPID=1
ci_proof_release_owned
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" ]]
ci_proof_release "$farm" bullet-farm "$victim_record" standalone
[[ -z "$(fixture_lock_dirs)" ]]
foreign_record='schema=2 repository=bullet-farm scope=standalone pid=1 lane=fast nonce=1-2-3-4'
(umask 077; mkdir -- "$farm/.git/bullet-ci.lock.d")
(umask 077; printf '%s\n' "$foreign_record" >"$farm/.git/bullet-ci.lock.d/owner")
ci_proof_own "$farm" bullet-farm standalone "$foreign_record"
expect_status_75 ci_proof_release_owned
[[ "$(fixture_lock_dirs)" == "$(fixture_lock_list bullet-farm)" \
  && "$(<"$farm/.git/bullet-ci.lock.d/owner")" == "$foreign_record" ]]
rm -- "$farm/.git/bullet-ci.lock.d/owner"
rmdir -- "$farm/.git/bullet-ci.lock.d"
[[ -z "$(fixture_lock_dirs)" ]]
ci_proof_disown
trap cleanup EXIT
: >"$fixture/family.ready"
: >"$fixture/family.pid"
: >"$fixture/family.err"
bash "$fixture/family-hub.sh" "$custody_source" "$fixture" \
  "$fixture/family-window.sh" "$fixture/family.ready" "$fixture/family.pid" \
  >"$fixture/family.out" 2>"$fixture/family.err" &
family_hub_pid=$!
for ((family_waited = 0; family_waited < 600; family_waited++)); do
  if [[ -s "$fixture/family.pid" ]]; then
    break
  fi
  sleep 0.05
done
[[ -s "$fixture/family.pid" ]]
family_window_pid="$(<"$fixture/family.pid")"
[[ "$(fixture_lock_dirs)" \
  == "$(fixture_lock_list bullet-farm bullet-git bullet-kernel bullet-portal)" ]]
kill -TERM "$family_hub_pid"
kill -TERM "$family_window_pid"
reap_status "$family_hub_pid"
family_status="$REAPED_STATUS"
[[ "$family_status" -eq 143 ]]
[[ "$(<"$fixture/family.err")" \
  == *'interrupted family custody remains reserved for explicit reconciliation'* ]]
[[ "$(fixture_lock_dirs)" \
  == "$(fixture_lock_list bullet-git bullet-kernel bullet-portal)" ]]
family_custody_initialize "$fixture" family
expect_status_75 family_custody_acquire_all
[[ "$(fixture_lock_dirs)" \
  == "$(fixture_lock_list bullet-git bullet-kernel bullet-portal)" ]]
for repository in bullet-git bullet-kernel bullet-portal; do
  rm -- "$fixture/$repository/.git/bullet-ci.lock.d/owner"
  rmdir -- "$fixture/$repository/.git/bullet-ci.lock.d"
done
[[ -z "$(fixture_lock_dirs)" ]]
grep -Fq 'interrupted family custody remains reserved for explicit reconciliation' "$family_source"
grep -Fxq 'trap release_family_custody EXIT' "$family_source"
[[ "$(grep -cF 'CI_PROOF_OWN_ON_ACQUIRE' "$family_source")" -eq 0 ]]
grep -Fxq 'CI_PROOF_OWN_ON_ACQUIRE=1' "$hub_local"
grep -Fq 'ci_proof_release_owned' "$hub_local"
[[ "$(grep -cF 'ci_proof_release "' "$hub_local")" -eq 0 ]]
grep -Fxq 'trap ci_proof_custody_exit EXIT' "$hub_local"
grep -Fxq "trap 'ci_proof_custody_signal 129' HUP" "$hub_local"
grep -Fxq "trap 'ci_proof_custody_signal 130' INT" "$hub_local"
grep -Fxq "trap 'ci_proof_custody_signal 143' TERM" "$hub_local"
[[ "$(grep -cE '^trap ' "$hub_local")" -eq 4 ]]
hub_traps="$(grep -E '^trap ' "$hub_local")"
shared_traps="$(sed -n '/^ci_proof_custody_trap() {$/,/^}$/p' "$custody_source" \
  | sed -n 's/^  \(trap .*\)$/\1/p')"
[[ -n "$hub_traps" && "$hub_traps" == "$shared_traps" ]]
hub_trap_line="$(grep -nFx 'trap ci_proof_custody_exit EXIT' "$hub_local" \
  | head -n 1 | cut -d: -f1)"
# shellcheck disable=SC2016
hub_acquire_line="$(grep -nF 'acquire_proof_lock "$lane" || return $?' "$hub_local" \
  | head -n 1 | cut -d: -f1)"
[[ "$hub_trap_line" =~ ^[0-9]+$ && "$hub_acquire_line" =~ ^[0-9]+$ \
  && "$hub_trap_line" -lt "$hub_acquire_line" ]]
echo '[ci] family proof custody fixture passed'
