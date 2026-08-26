#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
PORTAL="$FAMILY/bullet-portal"
# shellcheck source=ops/ci/toolchain-pins.sh
source "$HUB/ops/ci/toolchain-pins.sh"

for tool in cargo curl node npm setsid; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'dev: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
[[ "$(node --version)" == "v$PINNED_NODE_VERSION" ]] || {
  printf 'dev: expected Node v%s, found %s\n' "$PINNED_NODE_VERSION" "$(node --version)" >&2
  exit 1
}
[[ "$(npm --version)" == "$PINNED_NPM_VERSION" ]] || {
  printf 'dev: expected npm %s, found %s\n' "$PINNED_NPM_VERSION" "$(npm --version)" >&2
  exit 1
}
[[ -f "$PORTAL/package-lock.json" ]] || {
  echo "dev: bullet-portal sibling checkout is missing" >&2
  exit 1
}

runtime_root="$(mktemp -d)"
farmd_pid=""
portal_pid=""
cleanup() {
  local alive
  trap - EXIT INT TERM
  for pid in "$portal_pid" "$farmd_pid"; do
    [[ -n "$pid" ]] || continue
    if kill -0 -- "-$pid" 2>/dev/null; then
      kill -TERM -- "-$pid" 2>/dev/null || true
    fi
  done
  for _ in {1..50}; do
    alive=0
    for pid in "$portal_pid" "$farmd_pid"; do
      [[ -n "$pid" ]] || continue
      if kill -0 -- "-$pid" 2>/dev/null; then
        alive=1
      fi
    done
    [[ "$alive" -eq 0 ]] && break
    sleep 0.1
  done
  for pid in "$portal_pid" "$farmd_pid"; do
    [[ -n "$pid" ]] || continue
    if kill -0 -- "-$pid" 2>/dev/null; then
      printf 'dev: process group %s exceeded shutdown deadline; sending KILL\n' "$pid" >&2
      kill -KILL -- "-$pid" 2>/dev/null || true
    fi
  done
  for pid in "$portal_pid" "$farmd_pid"; do
    [[ -n "$pid" ]] || continue
    wait "$pid" 2>/dev/null || true
  done
  rm -rf "$runtime_root"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "== install locked Portal dependencies (lifecycle scripts disabled) =="
(
  cd "$PORTAL"
  node ops/ci/preinstall-scan.mjs
  npm ci --ignore-scripts --no-audit --no-fund
)

echo "== start farmd and Portal under one supervisor =="
setsid env BULLET_DATA_DIR="$runtime_root/data" bash "$HUB/scripts/farmd.sh" >"$runtime_root/farmd.log" 2>&1 &
farmd_pid=$!
setsid bash "$HUB/scripts/portal.sh" >"$runtime_root/portal.log" 2>&1 &
portal_pid=$!

farmd_ready=0
portal_ready=0
for _ in {1..120}; do
  if ! kill -0 "$farmd_pid" 2>/dev/null; then
    echo "dev: farmd exited before becoming healthy" >&2
    exit 1
  fi
  if ! kill -0 "$portal_pid" 2>/dev/null; then
    echo "dev: Portal exited before both services became ready" >&2
    exit 1
  fi
  if curl --fail --silent --show-error --max-time 1 http://127.0.0.1:7420/health >/dev/null 2>&1; then
    farmd_ready=1
  fi
  if curl --fail --silent --show-error --max-time 1 http://127.0.0.1:5173/ >/dev/null 2>&1; then
    portal_ready=1
  fi
  if [[ "$farmd_ready" -eq 1 && "$portal_ready" -eq 1 ]]; then
    break
  fi
  sleep 0.25
done
[[ "$farmd_ready" -eq 1 ]] || {
  echo "dev: farmd health deadline exceeded" >&2
  exit 1
}
[[ "$portal_ready" -eq 1 ]] || {
  echo "dev: Portal readiness deadline exceeded" >&2
  exit 1
}

echo "dev: farmd healthy at http://127.0.0.1:7420"
echo "dev: Portal ready at http://127.0.0.1:5173"
echo "dev: Ctrl-C stops and waits for both process groups"

set +e
wait -n "$farmd_pid" "$portal_pid"
child_exit=$?
set -e
printf 'dev: a supervised child exited with status %s; stopping the family\n' "$child_exit" >&2
cleanup
if [[ "$child_exit" -eq 0 ]]; then
  echo "dev: unexpected supervised child exit is a failure" >&2
  exit 1
fi
exit "$child_exit"
