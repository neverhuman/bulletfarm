#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'setup: %s\n' "$1" >&2
  exit 4
}

script_dir="${BASH_SOURCE[0]%/*}"
if [[ "$script_dir" == "${BASH_SOURCE[0]}" ]]; then
  script_dir=.
fi
cd -P -- "$script_dir/.." || fail "cannot resolve the Hub checkout"
HUB="$PWD"
cd -P -- "$HUB/.." || fail "cannot resolve the family root"
FAMILY="$PWD"
cd -- "$HUB"

setup_bin="${BULLET_SETUP_ADMITTED_BIN:-}"
[[ -n "$setup_bin" ]] || fail \
  "operator-pre-admitted bootstrap unavailable; BULLET_SETUP_ADMITTED_BIN must name an external bullet-family selected by the operator"
[[ "$setup_bin" = /* ]] || fail "BULLET_SETUP_ADMITTED_BIN must be an absolute path"
setup_resolved="$(/usr/bin/readlink -f -- "$setup_bin")" || fail \
  "BULLET_SETUP_ADMITTED_BIN cannot be resolved"
[[ "$setup_resolved" == "$setup_bin" ]] || fail \
  "BULLET_SETUP_ADMITTED_BIN must use its canonical non-symlink path"
if [[ "$setup_resolved" == "$FAMILY" || "$setup_resolved" == "$FAMILY/"* ]]; then
  fail "BULLET_SETUP_ADMITTED_BIN must resolve outside the source family"
fi
[[ -f "$setup_resolved" && -x "$setup_resolved" && ! -L "$setup_resolved" ]] || fail \
  "BULLET_SETUP_ADMITTED_BIN must be a regular executable, not a symlink"

cargo_bin="${BULLET_SETUP_CARGO_BIN:-}"
node_bin="${BULLET_SETUP_NODE_BIN:-}"
npm_cli="${BULLET_SETUP_NPM_CLI:-}"
for subject in \
  "BULLET_SETUP_CARGO_BIN:$cargo_bin" \
  "BULLET_SETUP_NODE_BIN:$node_bin" \
  "BULLET_SETUP_NPM_CLI:$npm_cli"
do
  name="${subject%%:*}"
  value="${subject#*:}"
  [[ "$value" = /* ]] || fail "$name must name an explicit absolute path"
done

exec /usr/bin/env -i HOME=/ PATH=/usr/bin:/bin LC_ALL=C \
  "$setup_resolved" setup --root "$FAMILY" --source jeryu \
  --cargo-bin "$cargo_bin" --node-bin "$node_bin" --npm-cli "$npm_cli" "$@"
