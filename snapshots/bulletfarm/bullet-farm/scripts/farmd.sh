#!/usr/bin/env bash
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
DATA="${BULLET_DATA_DIR:-$FAMILY/bullet-kernel/target/demo}"
# The browser sits at the Vite dev server and reaches farmd through its
# same-origin proxy, so the Origin header farmd sees is the Portal's, not its
# own. require_origin compares that header for exact equality against
# --portal-origin, which otherwise defaults to the bind address and refuses
# every bootstrap with ORIGIN_DENIED.
PORTAL_ORIGIN="${BULLET_PORTAL_ORIGIN:-http://127.0.0.1:5173}"
mkdir -p "$DATA"
cd "$FAMILY/bullet-kernel"
exec cargo run --locked -p bullet-farmd -- \
  --data-dir "$DATA" \
  --bind 127.0.0.1:7420 \
  --portal-origin "$PORTAL_ORIGIN"
