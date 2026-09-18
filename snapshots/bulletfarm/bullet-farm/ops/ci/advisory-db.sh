#!/usr/bin/env bash
# Shared fail-closed RustSec database freshness proof.

refresh_advisory_database() {
  local maximum_age_days=14 advisory_db="" candidate commit_time now age_days
  cargo deny fetch db
  for candidate in "$REPO_ROOT"/target/advisory-db/advisory-db-*; do
    [[ -d "$candidate/.git" ]] || continue
    advisory_db="$candidate"
    break
  done
  [[ -n "$advisory_db" ]] \
    || { refuse ADVISORY_DB_ABSENT "no database under target/advisory-db"; return 1; }
  commit_time="$(git -C "$advisory_db" log -1 --format=%ct)" \
    || { refuse ADVISORY_DB_UNREADABLE "$advisory_db"; return 1; }
  [[ "$commit_time" =~ ^[0-9]+$ ]] \
    || { refuse ADVISORY_DB_UNREADABLE "$advisory_db has an invalid commit time"; return 1; }
  now="$(date -u +%s)"
  (( now >= commit_time )) \
    || { refuse ADVISORY_DB_TIME_INVALID "$commit_time is in the future"; return 1; }
  age_days=$(( (now - commit_time) / 86400 ))
  (( age_days <= maximum_age_days )) \
    || { refuse ADVISORY_DB_STALE "$age_days days old; maximum is $maximum_age_days"; return 1; }
  log "advisory database is ${age_days}d old (maximum ${maximum_age_days}d)"
}
