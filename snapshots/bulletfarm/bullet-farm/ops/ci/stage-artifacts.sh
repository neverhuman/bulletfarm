#!/usr/bin/env bash
# Copy only one validated lane's observation and named sanitized artifacts into
# a fresh upload root. Hosted workflows upload this root, never ambient lane
# output, so unrelated logs cannot hitchhike into diagnostics.
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/artifact-path.sh
source "$(dirname "${BASH_SOURCE[0]}")/artifact-path.sh"
cd "$REPO_ROOT"

lane="${1:?lane is required}"
expected_commit="${2:-}"
[[ "$lane" =~ ^[a-z][a-z0-9-]*$ ]] \
  || { refuse CI_LANE_INVALID "$lane"; exit 1; }
bash ops/ci/artifact-check.sh "$lane" "$expected_commit" "$REPO_ROOT" local

stage_root="$REPO_ROOT/.ci-upload/$lane"
prepare_ci_directory "$REPO_ROOT" .ci-upload \
  || { refuse CI_UPLOAD_ROOT_INVALID .ci-upload; exit 1; }
upload_root="$(ci_canonical_directory "$REPO_ROOT/.ci-upload")" \
  || { refuse CI_UPLOAD_ROOT_INVALID .ci-upload; exit 1; }
if [[ -e "$stage_root" || -L "$stage_root" ]]; then
  [[ -d "$stage_root" && ! -L "$stage_root" ]] \
    || { refuse CI_UPLOAD_ROOT_INVALID ".ci-upload/$lane"; exit 1; }
  resolved_stage="$(ci_canonical_directory "$stage_root")"
  ci_path_within "$upload_root" "$resolved_stage" \
    || { refuse CI_UPLOAD_ROOT_INVALID ".ci-upload/$lane"; exit 1; }
fi
rm -rf -- "$stage_root"
prepare_ci_directory "$REPO_ROOT" ".ci-upload/$lane" \
  || { refuse CI_UPLOAD_ROOT_INVALID ".ci-upload/$lane"; exit 1; }
copy_artifact() {
  local relative="$1" destination destination_parent relative_parent
  validate_ci_artifact_path "$relative" \
    || { refuse CI_ARTIFACT_PATH_INVALID "$relative"; return 1; }
  destination="$stage_root/$relative"
  destination_parent="${destination%/*}"
  relative_parent="${destination_parent#"$REPO_ROOT/"}"
  prepare_ci_directory "$REPO_ROOT" "$relative_parent" \
    || { refuse CI_UPLOAD_ROOT_INVALID "$relative_parent"; return 1; }
  cp -- "$relative" "$destination"
}

copy_artifact ".ci-artifacts/observations/$lane.json"
while IFS= read -r relative; do
  [[ -n "$relative" ]] || continue
  copy_artifact "$relative"
done < <(jq -r '.artifact_hashes[].path' ".ci-artifacts/observations/$lane.json")

bash ops/ci/artifact-check.sh "$lane" "$expected_commit" "$stage_root" atomic
log "staged exact sanitized upload: .ci-upload/$lane"
