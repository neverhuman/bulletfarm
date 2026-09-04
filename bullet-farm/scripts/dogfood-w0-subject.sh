#!/usr/bin/env bash
# Create-once four-repository W0 subject. Unsigned COMPONENT observation.
# Never git add/commit. Never claims four clean heads. Fresh Genesis later
# consumes this file and refuses drift.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"

usage="usage: $0 --out ABSOLUTE_PATH"
[[ "$#" -eq 2 && "$1" == --out ]] || { printf '%s\n' "$usage" >&2; exit 2; }
out="$2"
[[ "$out" == /* ]] || { printf 'out path must be absolute\n' >&2; exit 2; }
[[ ! -e "$out" ]] || { refuse W0_SUBJECT_EXISTS "$out"; exit 1; }

family="$(cd "$REPO_ROOT/.." && pwd -P)"
repos=(bullet-farm bullet-kernel bullet-git bullet-portal)
tmp="$(mktemp)"
trap 'rm -f -- "$tmp"' EXIT

{
  printf '{\n  "kind": "WAVE0_SUBJECT",\n  "schema_version": "v0",\n  "authoritative": false,\n  "repos": [\n'
  first=1
  dirty=0
  for repo in "${repos[@]}"; do
    root="$family/$repo"
    [[ -d "$root/.git" ]] || { refuse WAVE0_REPO_MISSING "$repo"; exit 1; }
    head="$(git -C "$root" rev-parse HEAD)"
    tree="$(git -C "$root" rev-parse 'HEAD^{tree}')"
    porcelain="$(git -C "$root" status --porcelain)"
    count="$(printf '%s\n' "$porcelain" | grep -c . || true)"
    [[ "$count" -eq 0 ]] || dirty=1
    [[ "$first" -eq 1 ]] || printf ',\n'
    first=0
    printf '    {"repo":"%s","commit":"%s","tree":"%s","dirty":%s}' \
      "$repo" "$head" "$tree" "$count"
  done
  printf '\n  ],\n  "clean": %s\n}\n' "$([[ "$dirty" -eq 0 ]] && echo true || echo false)"
} >"$tmp"

umask 077
install -m 0600 "$tmp" "$out"
log "wrote create-once W0 subject to $out"
[[ "$dirty" -eq 0 ]] || { refuse WAVE0_DIRTY_SUBJECTS "subject recorded dirty trees; Genesis must refuse"; exit 1; }
