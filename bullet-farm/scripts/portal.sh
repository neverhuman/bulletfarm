#!/usr/bin/env bash
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
cd "$FAMILY/bullet-portal"
# Development uses Vite's same-origin proxy. Do not let ambient host state
# turn browser requests into unsupported cross-origin farmd calls.
unset VITE_BULLET_API
exec npm run dev -- --host 127.0.0.1 --port 5173 --strictPort
