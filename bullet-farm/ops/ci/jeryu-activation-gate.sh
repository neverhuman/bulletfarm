#!/usr/bin/env bash
# Jeryu CI remains prepared but deliberately inactive until forge topology,
# immutable subjects, runners, and the protected context are ratified and read
# back. Every prepared job invokes this gate first so direct dispatch also
# refuses instead of producing a misleading green observation.
set -euo pipefail
printf '%s\n' \
  '{"schema_version":"bullet.ci-activation-refusal.v1","code":"JERYU_CI_NOT_RATIFIED","status":"BLOCKED","exit_code":78,"release_authority":false}' >&2
exit 78
