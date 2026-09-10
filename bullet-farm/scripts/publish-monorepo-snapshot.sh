#!/usr/bin/env bash
# Rebuild the public monorepo snapshot at github.com/neverhuman/bulletfarm
# from the four member repositories' committed trees.
#
# The monorepo is a snapshot for people who want one clone, not source
# authority. Each member repository remains the authority for its own history;
# this only mirrors the committed tree of each member's current branch and
# records which commit each tree came from.
#
# Only committed trees are copied. `git archive` cannot see uncommitted or
# untracked files, so a dirty member cannot leak into the published snapshot.
set -euo pipefail

FAMILY="${FAMILY_ROOT:-/home/ubuntu/bullet}"
MONO="${MONO_CHECKOUT:?MONO_CHECKOUT must name a clone of the monorepo}"
MEMBERS=(bullet-farm bullet-kernel bullet-git bullet-portal)

[[ -d "$MONO/.git" ]] || { echo "MONO_CHECKOUT is not a git checkout" >&2; exit 2; }

declare -A HEADS
for member in "${MEMBERS[@]}"; do
  src="$FAMILY/$member"
  [[ -d "$src/.git" ]] || { echo "missing member: $src" >&2; exit 2; }
  HEADS["$member"]="$(git -C "$src" rev-parse HEAD)"
done

for member in "${MEMBERS[@]}"; do
  rm -rf "${MONO:?}/$member"
  mkdir -p "$MONO/$member"
  git -C "$FAMILY/$member" archive "${HEADS[$member]}" | tar -x -C "$MONO/$member"
done

{
  printf '# Snapshot provenance\n\n'
  printf 'This tree is a snapshot. Each member repository is the authority for\n'
  printf 'its own history; the trees below are copies of one commit each.\n\n'
  printf '| Member | Commit | Source |\n| --- | --- | --- |\n'
  for member in "${MEMBERS[@]}"; do
    # The backticks are a Markdown code span in the emitted table, not a
    # command substitution, so the format string stays single-quoted.
    # shellcheck disable=SC2016
    printf '| %s | `%s` | https://github.com/neverhuman/%s |\n' \
      "$member" "${HEADS[$member]}" "$member"
  done
} >"$MONO/SNAPSHOT.md"

cd "$MONO"
git add -A
if git diff --cached --quiet; then
  echo "snapshot already current"
  exit 0
fi
git commit -q -F - <<EOF
Snapshot the four member trees

$(for member in "${MEMBERS[@]}"; do printf '%-14s %s\n' "$member" "${HEADS[$member]}"; done)

Each member repository remains the authority for its own history. This tree
is a copy of one commit from each, for people who want a single clone.
EOF
echo "snapshot committed: $(git rev-parse --short HEAD)"
