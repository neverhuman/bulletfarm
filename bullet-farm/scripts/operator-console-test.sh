#!/usr/bin/env bash
# Admission refusals only. Does not start farmd, login, or print secrets.
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SCRIPT="$HUB/scripts/operator-console.sh"

expect() {
  local label="$1" want="$2"
  shift 2
  local out
  out="$(bash "$SCRIPT" "$@" 2>&1 || true)"
  printf '%s\n' "$out" | grep -q "$want" \
    || { printf 'operator-console-test: %s expected %s, got:\n%s\n' "$label" "$want" "$out" >&2; exit 1; }
}

expect missing-data-dir OPERATOR_CONSOLE_DATA_DIR_REQUIRED
expect relative-data-dir OPERATOR_CONSOLE_DATA_DIR_NOT_ABSOLUTE --data-dir relative/console
expect tmp-data-dir OPERATOR_CONSOLE_DATA_DIR_TMP --data-dir /tmp/bullet-operator-console
expect clone-data-dir OPERATOR_CONSOLE_DATA_DIR_INSIDE_CLONE --data-dir "$HUB/console-data"
echo 'operator-console-test: PASS (typed refusals only; no farmd)'
