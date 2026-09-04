#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd -P)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd -P)"
# shellcheck source=ops/ci/lib.sh
source "$REPO_ROOT/ops/ci/lib.sh"

requested_mode="${1:-}"
case "$requested_mode" in
  write|check) mode="$requested_mode" ;;
  --self-test) mode="self-test" ;;
  *)
    printf 'usage: %s {write|check|--self-test}\n' "$0" >&2
    exit 2
    ;;
esac
[[ "$#" -eq 1 ]] || {
  printf 'usage: %s {write|check|--self-test}\n' "$0" >&2
  exit 2
}

initialize_rust_toolchain_tools
selected_binary="${BULLET_FAMILY_BIN:-}"
if [[ -z "$selected_binary" ]]; then
  "$CARGO_EXECUTABLE" build --locked --quiet --bin bullet-family
  selected_binary="$REPO_ROOT/target/debug/bullet-family"
fi
realpath_executable="$(resolved_executable realpath)"
selected_binary="$("$realpath_executable" -- "$selected_binary")"
[[ "$selected_binary" == /* && -f "$selected_binary" && -x "$selected_binary" && ! -L "$selected_binary" ]] || {
  refuse ASSURANCE_INVENTORY_BINARY_INVALID "$selected_binary"
  exit 1
}

run_python_312 "$SCRIPT_DIR/orphan_inventory/main.py" "$mode" \
  --root "$REPO_ROOT" --bin "$selected_binary"
