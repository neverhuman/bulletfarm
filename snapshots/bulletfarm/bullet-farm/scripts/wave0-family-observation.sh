#!/usr/bin/env bash
# Wave 0: unsigned dirty-tree / cleanliness observation.
# Never claims BaselineReceiptV1, never git add/commit, never invents four clean heads.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"

mode="${1:-}"
[[ "$#" -eq 1 && ( "$mode" == check || "$mode" == --self-test ) ]] \
  || { printf 'usage: %s {check|--self-test}\n' "$0" >&2; exit 2; }

family="$(cd "$REPO_ROOT/.." && pwd -P)"
repos=(bullet-farm bullet-kernel bullet-git bullet-portal)

if [[ "$mode" == --self-test ]]; then
  for repo in "${repos[@]}"; do
    [[ -d "$family/$repo/.git" ]] || { refuse WAVE0_REPO_MISSING "$repo"; exit 1; }
  done
  log "Wave 0 self-test: four canonical checkouts exist; cleanliness is observed, not claimed"
  exit 0
fi

dirty=0
for repo in "${repos[@]}"; do
  root="$family/$repo"
  head="$(git -C "$root" rev-parse HEAD)"
  tree="$(git -C "$root" rev-parse 'HEAD^{tree}')"
  porcelain="$(git -C "$root" status --porcelain)"
  count="$(printf '%s\n' "$porcelain" | grep -c . || true)"
  printf 'repo=%s commit=%s tree=%s dirty=%s\n' "$repo" "$head" "$tree" "$count"
  if [[ "$count" -ne 0 ]]; then
    dirty=1
  fi
done

if [[ "$dirty" -ne 0 ]]; then
  refuse WAVE0_DIRTY_SUBJECTS \
    "four clean heads are absent; orchestrator-only commits remain required; this observation is unsigned COMPONENT"
  exit 1
fi

log "Wave 0 check: four clean heads observed (unsigned; not BaselineReceiptV1)"
