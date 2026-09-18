#!/usr/bin/env bash
# Source-only disk-floor admission for CI proofs. Read-only by construction:
# this file measures free space and refuses. It never creates, moves, changes
# the mode of, or deletes anything, and no future edit should give it that
# power -- "same UID" and "not currently held open" are not ownership or
# deletion authority, so a reclaimer belongs in its own lane with exact
# name/device/inode identity and an empty-only removal, never here.
#
# The claim this gate rests on is the bounded general one: a run that begins
# with little free space can exhaust the filesystem mid-write and leave a
# partial artifact behind -- a truncated observation, an incomplete lock record,
# a half-written ledger entry -- and this guard prevents a proof from starting
# below a configured floor. It asserts nothing about the cause of the
# 2026-08-26 torn coordination-ledger record: the admitted evidence there
# establishes a torn coordinator write and an owner-lifetime failure, not
# ENOSPC. The gate fails closed in both directions: an unmeasurable filesystem
# refuses, and an unusable floor override refuses rather than quietly falling
# back to the default.
#
# Deliberately still open, and not closed by this file: disk pressure itself
# (nothing here reclaims anything); whether the floor is sufficient for a cold
# build (it is not, by construction, see below); custody of the ambient df and
# awk this file shells out to, which are unpinned unlike the toolchains the
# repository pins elsewhere; and the four mode-000
# bullet-ci-target-quarantine.* directories under the kernel, which only an
# operator or a later, separately reviewed lane may clear.

# Floor policy. CI_SCRATCH_FLOOR_MIB is the free space, in MiB, required on
# every filesystem a lane will write to before the lane may acquire a lock or
# build anything. It is deliberately a torn-write guard, not a capacity planner:
# 8 GiB is 66x the largest single regular file any lane writes into a target/
# tree (123.3 MiB, measured across the four repositories) and >8000x the
# .ci-artifacts tree that carries the JUnit, formal and observation records
# (~1 MiB per repository) -- the surfaces that actually tore. It is not, and
# cannot be, large enough to guarantee a cold rebuild completes; a floor of one
# cold debug/deps tree (14.0 GiB for the hub, 128 GiB for the kernel) would
# refuse every proof on a host that is merely busy, which is a denial of service
# dressed up as a safety property. The floor is necessary, not sufficient: it
# stops a proof from starting into a nearly full disk, and it reclaims nothing,
# so disk pressure stays open until an operator clears the scratch and
# quarantine trees by hand.
CI_SCRATCH_FLOOR_MIB=8192

# The override is bounded in both directions so that it can tune the policy but
# never disable it. The minimum is still 16x the largest single build artifact,
# so a link cannot exhaust the volume even at the lowest admissible setting. The
# maximum catches a fat-fingered unit (bytes pasted where MiB were meant), which
# would otherwise refuse every proof forever without ever saying why.
CI_SCRATCH_FLOOR_MIN_MIB=2048
CI_SCRATCH_FLOOR_MAX_MIB=262144

# Observation only, for callers and tests; never an input to the decision.
CI_SCRATCH_FREE_KIB=
CI_SCRATCH_FREE_SUBJECT=
CI_SCRATCH_EFFECTIVE_FLOOR_MIB=

ci_scratch_refusal() {
  local code="$1" detail="$2"
  printf '%s\n' \
    "ci-local: $code: $detail" \
    "ci-local: nothing was locked and nothing was built; free space explicitly, then re-run" >&2
  return 69
}

# The effective floor in MiB, or a non-zero status if the override is present
# and unusable. An unusable override is never silently replaced by the default:
# a caller who asked for a specific policy and got a different one is exactly
# the fail-open case this file exists to prevent. The pattern is anchored to
# plain digits before any arithmetic comparison sees the value, so the bounds
# test can never evaluate an expression a caller smuggled in.
ci_scratch_floor_mib() {
  local raw
  if [[ ${CI_SCRATCH_FLOOR_MIB_OVERRIDE+x} ]]; then
    raw="$CI_SCRATCH_FLOOR_MIB_OVERRIDE"
    [[ "$raw" =~ ^[1-9][0-9]{0,6}$ ]] || return 1
    [[ "$raw" -ge "$CI_SCRATCH_FLOOR_MIN_MIB" && "$raw" -le "$CI_SCRATCH_FLOOR_MAX_MIB" ]] || return 1
    printf '%s\n' "$raw"
    return 0
  fi
  printf '%s\n' "$CI_SCRATCH_FLOOR_MIB"
}

# The nearest existing directory at or above a subject path, so that a target
# directory a lane has not created yet is still measured on the filesystem that
# will hold it. Lexical only: it inspects nothing and changes nothing.
ci_scratch_existing_ancestor() {
  local path="$1"
  [[ "$path" == /* ]] || return 1
  while [[ ! -d "$path" ]]; do
    [[ "$path" != / ]] || return 1
    path="${path%/*}"
    [[ -n "$path" ]] || path=/
  done
  printf '%s\n' "$path"
}

# Free 1024-byte blocks on the filesystem holding a directory. Any answer that
# is not a plain integer is treated as no answer at all.
ci_scratch_free_kib() {
  local directory="$1" available
  available="$(LC_ALL=C df -P -k -- "$directory" 2>/dev/null | awk 'NR == 2 { print $4 }')" || return 1
  [[ "$available" =~ ^[0-9]+$ ]] || return 1
  printf '%s\n' "$available"
}

# The admission gate. Refuses before the caller may acquire anything or build
# anything when the scarcest filesystem behind any subject path is below the
# effective floor. Exactly at the floor is admitted: the floor is the minimum
# admissible state, not the first refused one. Every refusal is status 69 with a
# typed reason code, and the caller is expected to return that status unchanged.
ci_scratch_floor_check() {
  local subject ancestor free floor worst='' worst_subject=''
  CI_SCRATCH_FREE_KIB=
  CI_SCRATCH_FREE_SUBJECT=
  CI_SCRATCH_EFFECTIVE_FLOOR_MIB=
  floor="$(ci_scratch_floor_mib)" || {
    ci_scratch_refusal CI_PROOF_DISK_FLOOR_BOUND \
      "CI_SCRATCH_FLOOR_MIB_OVERRIDE must be an integer in [$CI_SCRATCH_FLOOR_MIN_MIB, $CI_SCRATCH_FLOOR_MAX_MIB] MiB"
    return $?
  }
  [[ "$#" -ge 1 ]] || {
    ci_scratch_refusal CI_PROOF_DISK_FLOOR_SUBJECT 'no filesystem subject was named'
    return $?
  }
  for subject in "$@"; do
    [[ "$subject" == /* ]] || {
      ci_scratch_refusal CI_PROOF_DISK_FLOOR_SUBJECT "subject $subject is not an absolute path"
      return $?
    }
    ancestor="$(ci_scratch_existing_ancestor "$subject")" || {
      ci_scratch_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED "no existing directory at or above $subject"
      return $?
    }
    free="$(ci_scratch_free_kib "$ancestor")" || {
      ci_scratch_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED \
        "free space on the filesystem holding $ancestor is unreadable"
      return $?
    }
    # Defence in depth: nothing but a plain integer may reach the arithmetic
    # below, whatever a replaced measurement or a surprising df hands back.
    [[ "$free" =~ ^[0-9]+$ ]] || {
      ci_scratch_refusal CI_PROOF_DISK_FLOOR_UNDETERMINED \
        "free space on the filesystem holding $ancestor is not a plain integer"
      return $?
    }
    if [[ -z "$worst" || "$free" -lt "$worst" ]]; then
      worst="$free"
      worst_subject="$ancestor"
    fi
  done
  CI_SCRATCH_FREE_KIB="$worst"
  CI_SCRATCH_FREE_SUBJECT="$worst_subject"
  CI_SCRATCH_EFFECTIVE_FLOOR_MIB="$floor"
  [[ "$CI_SCRATCH_FREE_KIB" -ge $((CI_SCRATCH_EFFECTIVE_FLOOR_MIB * 1024)) ]] || {
    ci_scratch_refusal CI_PROOF_DISK_FLOOR \
      "$CI_SCRATCH_FREE_SUBJECT has $((CI_SCRATCH_FREE_KIB / 1024)) MiB free, below the $CI_SCRATCH_EFFECTIVE_FLOOR_MIB MiB floor"
    return $?
  }
}
