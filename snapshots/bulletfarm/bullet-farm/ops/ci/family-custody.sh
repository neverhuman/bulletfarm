#!/usr/bin/env bash
# Source-only exact custody for fixed CI artifact namespaces.

CI_PROOF_RECORD_REPOSITORY=
CI_PROOF_RECORD_SCOPE=
CI_PROOF_RECORD_PID=
CI_PROOF_RECORD_LANE=

# Per-repo proof-lock custody for a process that is dying. A lock is "owned"
# only by the exact Bash process whose ci_proof_acquire created it. Shell state
# is copied by a fork, so BASHPID (not $$) fences a subshell; it is not exported
# through the separately executed lane Bash. This shell protocol assumes
# cooperative same-UID peers keep the admitted paths stable between verify and
# unlink; it does not claim descriptor-atomic deletion against path replacement.
# Opt-in, because it is deliberately narrower than family custody: ci-local sets
# CI_PROOF_OWN_ON_ACQUIRE=1 for its own per-repo lock, while ops/ci/family.sh
# leaves it 0 so that an interrupted family-wide custody window is never
# released by a signal handler and stays reserved for explicit reconciliation.
CI_PROOF_OWN_ON_ACQUIRE=0
CI_PROOF_OWNED_BASHPID=
CI_PROOF_OWNED_ROOT=
CI_PROOF_OWNED_REPOSITORY=
CI_PROOF_OWNED_SCOPE=
CI_PROOF_OWNED_RECORD=
CI_PROOF_SIGNAL_CRITICAL_BASHPID=
CI_PROOF_SIGNAL_PENDING=0

ci_proof_own() {
  CI_PROOF_OWNED_BASHPID="$BASHPID"
  CI_PROOF_OWNED_ROOT="$1"
  CI_PROOF_OWNED_REPOSITORY="$2"
  CI_PROOF_OWNED_SCOPE="$3"
  CI_PROOF_OWNED_RECORD="$4"
}

ci_proof_disown() {
  CI_PROOF_OWNED_BASHPID=
  CI_PROOF_OWNED_ROOT=
  CI_PROOF_OWNED_REPOSITORY=
  CI_PROOF_OWNED_SCOPE=
  CI_PROOF_OWNED_RECORD=
}

ci_proof_signal_defer() {
  local signal_status="$1"
  if [[ "$CI_PROOF_SIGNAL_CRITICAL_BASHPID" == "$BASHPID" ]]; then
    [[ "$CI_PROOF_SIGNAL_PENDING" -ne 0 ]] || CI_PROOF_SIGNAL_PENDING="$signal_status"
    return 0
  fi
  # critical_end clears the marker before it restores the ordinary handlers.
  # A signal in that transition must replay an already-recorded first signal,
  # rather than replacing it with the later signal that happened to arrive.
  if [[ "$CI_PROOF_SIGNAL_PENDING" =~ ^(129|130|143)$ ]]; then
    ci_proof_custody_signal "$CI_PROOF_SIGNAL_PENDING"
  fi
  ci_proof_custody_signal "$signal_status"
}

# Bash dispatches a pending trap after a foreground command returns and before
# the next statement. Defer catchable termination across mkdir, owner write and
# exact verification. If external mkdir creates the directory but reports
# failure, creation is ambiguous and the empty path is deliberately preserved
# as a typed stale lock; this cooperative shell protocol cannot adopt it safely.
ci_proof_signal_critical_begin() {
  CI_PROOF_SIGNAL_CRITICAL_BASHPID="$BASHPID"
  CI_PROOF_SIGNAL_PENDING=0
  trap ci_proof_custody_exit EXIT
  trap 'ci_proof_signal_defer 129' HUP
  trap 'ci_proof_signal_defer 130' INT
  trap 'ci_proof_signal_defer 143' TERM
}

ci_proof_signal_critical_end() {
  [[ "$CI_PROOF_SIGNAL_CRITICAL_BASHPID" == "$BASHPID" ]] || return 75
  [[ "$CI_PROOF_SIGNAL_PENDING" =~ ^(0|129|130|143)$ ]] || return 75
  # Leave the deferring handlers installed while clearing the marker. A signal
  # delivered after this assignment either replays the first pending status or,
  # when there was none, exits with its own status. No signal can fall between a
  # pending-value read and three independent trap restorations.
  CI_PROOF_SIGNAL_CRITICAL_BASHPID=
  [[ "$CI_PROOF_SIGNAL_PENDING" -eq 0 ]] \
    || ci_proof_custody_signal "$CI_PROOF_SIGNAL_PENDING"
  trap 'ci_proof_custody_signal 129' HUP
  trap 'ci_proof_custody_signal 130' INT
  trap 'ci_proof_custody_signal 143' TERM
  CI_PROOF_SIGNAL_PENDING=0
}

ci_proof_refusal() {
  printf '%s\n' \
    "ci-local: CI_PROOF_LOCKED_OR_STALE: $1/.git/bullet-ci.lock.d is occupied or cannot be trusted" \
    "ci-local: verify the exact owner, then explicitly reconcile only $1/.git/bullet-ci.lock.d" >&2
  return 75
}

ci_proof_exact_directory() {
  local path="$1" mode="$2" uid
  uid="$(id -u)"
  [[ -d "$path" && ! -L "$path" \
    && "$(find "$path" -maxdepth 0 -type d -uid "$uid" -perm "$mode" -print)" == "$path" ]]
}

ci_proof_exact_file() {
  local path="$1" mode="$2" uid
  uid="$(id -u)"
  [[ -f "$path" && ! -L "$path" \
    && "$(find "$path" -maxdepth 0 -type f -uid "$uid" -perm "$mode" -print)" == "$path" ]]
}

ci_proof_parse() {
  local record="$1" repository="$2" scope="$3"
  [[ "$record" =~ ^schema=2\ repository=([a-z0-9-]+)\ scope=(standalone|family)\ pid=([1-9][0-9]*)\ lane=([a-z0-9-]+)\ nonce=([0-9]+-[0-9]+-[0-9]+-[0-9]+)$ ]] || return 1
  CI_PROOF_RECORD_REPOSITORY="${BASH_REMATCH[1]}"
  CI_PROOF_RECORD_SCOPE="${BASH_REMATCH[2]}"
  CI_PROOF_RECORD_PID="${BASH_REMATCH[3]}"
  CI_PROOF_RECORD_LANE="${BASH_REMATCH[4]}"
  [[ "$CI_PROOF_RECORD_REPOSITORY" == "$repository" \
    && "$CI_PROOF_RECORD_SCOPE" == "$scope" ]]
}

ci_proof_owner_matches() {
  local owner="$1" record="$2" first='' extra='' second_status descriptor byte_count
  read -r byte_count < <(LC_ALL=C wc -c <"$owner") || return 1
  [[ "$byte_count" =~ ^[0-9]+$ && "$byte_count" -eq $((${#record} + 1)) ]] || return 1
  exec {descriptor}<"$owner" || return 1
  IFS= read -r -u "$descriptor" first || {
    exec {descriptor}<&-
    return 1
  }
  if IFS= read -r -u "$descriptor" extra; then
    second_status=0
  else
    second_status=$?
  fi
  exec {descriptor}<&-
  [[ "$first" == "$record" && "$second_status" -ne 0 && -z "$extra" ]]
}

ci_proof_verify() {
  local root="$1" repository="$2" record="$3" scope="$4"
  local lock_dir owner
  lock_dir="$root/.git/bullet-ci.lock.d"
  owner="$lock_dir/owner"
  if [[ ! -d "$root/.git" || -L "$root/.git" \
    || ! -f "$root/.git/HEAD" || -L "$root/.git/HEAD" ]] \
    || ! ci_proof_exact_directory "$lock_dir" 0700 \
    || ! ci_proof_exact_file "$owner" 0600 \
    || ! ci_proof_parse "$record" "$repository" "$scope" \
    || ! ci_proof_owner_matches "$owner" "$record"; then
    ci_proof_refusal "$root"
    return 75
  fi
}

ci_proof_acquire() {
  local root="$1" repository="$2" scope="$3" lane="$4" output_name="$5"
  local lock_dir owner acquired_record status=0 end_status=0
  local signal_critical=0 acquired_here=0
  lock_dir="$root/.git/bullet-ci.lock.d"
  owner="$lock_dir/owner"
  [[ "$repository" =~ ^[a-z0-9-]+$ && "$scope" =~ ^(standalone|family)$ \
    && "$lane" =~ ^[a-z0-9-]+$ && -d "$root/.git" && ! -L "$root/.git" \
    && -f "$root/.git/HEAD" && ! -L "$root/.git/HEAD" ]] || {
    ci_proof_refusal "$root"
    return 75
  }
  if [[ "$CI_PROOF_OWN_ON_ACQUIRE" -eq 1 ]]; then
    ci_proof_signal_critical_begin
    signal_critical=1
  fi
  if (umask 077; mkdir -- "$lock_dir") 2>/dev/null; then
    if [[ "$CI_PROOF_OWN_ON_ACQUIRE" -eq 1 ]]; then
      ci_proof_own "$root" "$repository" "$scope" ''
      acquired_here=1
    fi
    acquired_record="schema=2 repository=$repository scope=$scope pid=$$ lane=$lane nonce=$$-${BASHPID:-$$}-$RANDOM-$RANDOM"
    if (umask 077; set -o noclobber; printf '%s\n' "$acquired_record" >"$owner") 2>/dev/null; then
      if [[ "$CI_PROOF_OWN_ON_ACQUIRE" -eq 1 ]]; then
        CI_PROOF_OWNED_RECORD="$acquired_record"
      fi
      if ci_proof_verify "$root" "$repository" "$acquired_record" "$scope"; then
        printf -v "$output_name" '%s' "$acquired_record"
      else
        status=$?
      fi
    else
      ci_proof_refusal "$root" || true
      status=75
    fi
  else
    ci_proof_refusal "$root" || true
    status=75
  fi
  if [[ "$status" -ne 0 && "$acquired_here" -eq 1 ]]; then
    ci_proof_release_owned || true
  fi
  if [[ "$signal_critical" -eq 1 ]]; then
    if ci_proof_signal_critical_end; then
      :
    else
      end_status=$?
      [[ "$status" -ne 0 ]] || status="$end_status"
    fi
  fi
  return "$status"
}

ci_proof_release() {
  local root="$1" repository="$2" record="$3" scope="$4"
  local lock_dir="$root/.git/bullet-ci.lock.d"
  ci_proof_verify "$root" "$repository" "$record" "$scope" || return $?
  rm -- "$lock_dir/owner" || {
    ci_proof_refusal "$root"
    return 75
  }
  rmdir -- "$lock_dir" || {
    ci_proof_refusal "$root"
    return 75
  }
}

# Settle the per-repo proof lock this very process created, and nothing else.
# Ownership remains live through verification, owner unlink and exact empty
# rmdir, and is consumed only after path absence is observed. The caller keeps
# catchable signals deferred (ordinary release) or ignored (EXIT settlement).
# A fork merely disowns its copied shell state. Path replacement after verify
# remains outside this cooperative shell boundary.
ci_proof_release_owned_settle() {
  local root="$CI_PROOF_OWNED_ROOT" repository="$CI_PROOF_OWNED_REPOSITORY"
  local scope="$CI_PROOF_OWNED_SCOPE" record="$CI_PROOF_OWNED_RECORD" lock_dir owner
  [[ -n "$CI_PROOF_OWNED_BASHPID" ]] || return 0
  if [[ "$CI_PROOF_OWNED_BASHPID" != "$BASHPID" ]]; then
    ci_proof_disown
    return 0
  fi
  lock_dir="$root/.git/bullet-ci.lock.d"
  owner="$lock_dir/owner"
  if [[ -z "$record" ]]; then
    # Our mkdir won but no owner record was ever written, so the directory we
    # created is still empty. rmdir refuses a populated directory, so a lock
    # another process has since written cannot be removed on this path.
    if [[ ! -e "$lock_dir" && ! -L "$lock_dir" ]]; then
      ci_proof_disown
      return 0
    fi
    ci_proof_exact_directory "$lock_dir" 0700 || {
      ci_proof_refusal "$root"
      return 75
    }
    rmdir -- "$lock_dir" 2>/dev/null || {
      if [[ ! -e "$lock_dir" && ! -L "$lock_dir" ]]; then
        ci_proof_disown
        return 0
      fi
      ci_proof_refusal "$root"
      return 75
    }
    [[ ! -e "$lock_dir" && ! -L "$lock_dir" ]] || {
      ci_proof_refusal "$root"
      return 75
    }
    ci_proof_disown
    return 0
  fi
  ci_proof_verify "$root" "$repository" "$record" "$scope" || return $?
  [[ "$CI_PROOF_RECORD_PID" == "$$" ]] || {
    ci_proof_refusal "$root"
    return 75
  }
  rm -- "$owner" || {
    [[ ! -e "$owner" && ! -L "$owner" ]] || {
      ci_proof_refusal "$root"
      return 75
    }
  }
  [[ ! -e "$owner" && ! -L "$owner" ]] || {
    ci_proof_refusal "$root"
    return 75
  }
  rmdir -- "$lock_dir" || {
    [[ ! -e "$lock_dir" && ! -L "$lock_dir" ]] || {
      ci_proof_refusal "$root"
      return 75
    }
  }
  [[ ! -e "$lock_dir" && ! -L "$lock_dir" ]] || {
    ci_proof_refusal "$root"
    return 75
  }
  ci_proof_disown
}

# Idempotent owned release. When called from the acquisition critical section it
# shares that section; otherwise it creates one spanning the complete settlement.
ci_proof_release_owned() {
  local status=0 end_status=0 started_critical=0
  [[ -n "$CI_PROOF_OWNED_BASHPID" ]] || return 0
  if [[ "$CI_PROOF_OWNED_BASHPID" != "$BASHPID" ]]; then
    ci_proof_disown
    return 0
  fi
  if [[ "$CI_PROOF_SIGNAL_CRITICAL_BASHPID" != "$BASHPID" ]]; then
    ci_proof_signal_critical_begin
    started_critical=1
  fi
  if ci_proof_release_owned_settle; then
    :
  else
    status=$?
  fi
  if [[ "$started_critical" -eq 1 ]]; then
    if ci_proof_signal_critical_end; then
      :
    else
      end_status=$?
      [[ "$status" -ne 0 ]] || status="$end_status"
    fi
  fi
  return "$status"
}

# EXIT handler. Preserves the status the script had already decided on: a
# non-zero proof status is never overwritten and never becomes success, and a
# refused release is reported only when the proof itself had succeeded.
ci_proof_custody_exit() {
  local original_status=$? release_status=0
  trap - EXIT
  # The first exit reason has already won. Ignore later catchable signals only
  # while completing the exact owned settlement; SIGKILL remains uncatchable.
  trap '' HUP INT TERM
  set +e
  ci_proof_release_owned_settle
  release_status=$?
  set -e
  [[ "$original_status" -eq 0 ]] || exit "$original_status"
  exit "$release_status"
}

# Signal handler. Ignores later catchable signals, then exits with the
# conventional 128+signal status; the EXIT handler performs the narrow release.
ci_proof_custody_signal() {
  local signal_status="$1"
  trap '' HUP INT TERM
  exit "$signal_status"
}

ci_proof_custody_trap() {
  trap ci_proof_custody_exit EXIT
  trap 'ci_proof_custody_signal 129' HUP
  trap 'ci_proof_custody_signal 130' INT
  trap 'ci_proof_custody_signal 143' TERM
}

FAMILY_CUSTODY_ACTIVE=0
FAMILY_CUSTODY_LANE=
declare -ag FAMILY_CUSTODY_ACQUIRED=()
declare -Ag FAMILY_CUSTODY_ROOTS=()
declare -Ag FAMILY_CUSTODY_RECORDS=()

family_custody_initialize() {
  local family_root="$1" lane="$2"
  [[ "$lane" =~ ^family(-contract)?$ ]] || return 75
  FAMILY_CUSTODY_ROOTS=(
    [bullet-git]="$family_root/bullet-git"
    [bullet-kernel]="$family_root/bullet-kernel"
    [bullet-portal]="$family_root/bullet-portal"
  )
  FAMILY_CUSTODY_RECORDS=()
  FAMILY_CUSTODY_ACQUIRED=()
  FAMILY_CUSTODY_ACTIVE=0
  FAMILY_CUSTODY_LANE="$lane"
}

family_custody_verify_hub() {
  local root="$1" record="$2" lane="$3"
  ci_proof_verify "$root" bullet-farm "$record" family || return $?
  [[ "$CI_PROOF_RECORD_PID" == "$PPID" \
    && "$CI_PROOF_RECORD_LANE" == "$lane" ]] || {
    ci_proof_refusal "$root"
    return 75
  }
}

family_custody_verify_member() {
  local member="$1"
  ci_proof_verify "${FAMILY_CUSTODY_ROOTS[$member]}" "$member" \
    "${FAMILY_CUSTODY_RECORDS[$member]}" family || return $?
  [[ "$CI_PROOF_RECORD_PID" == "$$" \
    && "$CI_PROOF_RECORD_LANE" == "$FAMILY_CUSTODY_LANE" ]] || {
    ci_proof_refusal "${FAMILY_CUSTODY_ROOTS[$member]}"
    return 75
  }
}

family_custody_release_member() {
  local member="$1"
  family_custody_verify_member "$member" || return $?
  ci_proof_release "${FAMILY_CUSTODY_ROOTS[$member]}" "$member" \
    "${FAMILY_CUSTODY_RECORDS[$member]}" family || return $?
  unset 'FAMILY_CUSTODY_RECORDS[$member]'
}

family_custody_release_all() {
  local index member status=0
  for ((index=${#FAMILY_CUSTODY_ACQUIRED[@]} - 1; index >= 0; index--)); do
    member="${FAMILY_CUSTODY_ACQUIRED[$index]}"
    [[ -v "FAMILY_CUSTODY_RECORDS[$member]" ]] || continue
    family_custody_release_member "$member" || status=$?
  done
  FAMILY_CUSTODY_ACQUIRED=()
  FAMILY_CUSTODY_ACTIVE=0
  return "$status"
}

family_custody_acquire_member() {
  local member="$1" record
  ci_proof_acquire "${FAMILY_CUSTODY_ROOTS[$member]}" "$member" family \
    "$FAMILY_CUSTODY_LANE" record \
    || return $?
  FAMILY_CUSTODY_RECORDS[$member]="$record"
  FAMILY_CUSTODY_ACQUIRED+=("$member")
  family_custody_verify_member "$member"
}

family_custody_acquire_all() {
  local member status
  for member in bullet-git bullet-kernel bullet-portal; do
    if family_custody_acquire_member "$member"; then
      continue
    else
      status=$?
      family_custody_release_all || true
      return "$status"
    fi
  done
  FAMILY_CUSTODY_ACTIVE=1
}

family_custody_verify_all() {
  local member
  [[ "$FAMILY_CUSTODY_ACTIVE" -eq 1 ]] || return 75
  for member in bullet-git bullet-kernel bullet-portal; do
    family_custody_verify_member "$member" || return $?
  done
}

family_custody_record() {
  local member="$1"
  family_custody_verify_all || return $?
  [[ -v "FAMILY_CUSTODY_RECORDS[$member]" ]] || return 75
  printf '%s\n' "${FAMILY_CUSTODY_RECORDS[$member]}"
}
