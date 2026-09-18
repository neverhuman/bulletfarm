#!/usr/bin/env bash
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
cd "$FAMILY/bullet-portal"
# Development uses Vite's same-origin proxy. Do not let ambient host state
# turn browser requests into unsupported cross-origin farmd calls.
unset VITE_BULLET_API
portal_port="${BULLET_PORTAL_PORT:-5173}"
[[ "$portal_port" =~ ^[0-9]{1,5}$ && $((10#$portal_port)) -ge 1 && $((10#$portal_port)) -le 65535 ]] || {
  printf 'PORTAL_PORT_INVALID: %s\n' "$portal_port" >&2
  exit 1
}
exec npm run dev -- --host 127.0.0.1 --port "$portal_port" --strictPort
