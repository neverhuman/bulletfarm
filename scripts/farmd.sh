#!/usr/bin/env bash
set -euo pipefail
HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
DATA="${BULLET_DATA_DIR:-$FAMILY/bullet-kernel/target/demo}"
mkdir -p "$DATA"
cd "$FAMILY/bullet-kernel"
exec cargo run --locked -p bullet-farmd -- --data-dir "$DATA" --bind 127.0.0.1:7420
