#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
# No authentication/configuration lookup or provider execution is permitted in
# this data-only stage. The former recorder is retained in its Git source object.
if [[ "$#" == 0 ]]; then
  echo 'readme-live-record: LIVE_CAPTURE_UNQUALIFIED (supervised producer and custody required)' >&2
  exit 78
fi
[[ "$#" == 4 && "$1" == --from-capture && "$3" == --staged-root ]] || {
  echo 'usage: readme-live-record.sh --from-capture ABSOLUTE_DIRECTORY --staged-root NEW_ABSOLUTE_DIRECTORY' >&2
  exit 2
}
# shellcheck source=scripts/readme-live-check.sh
source "$HUB/scripts/readme-live-check.sh"
live_start
stage=''
cleanup() {
  local code=$?
  rm -rf -- "$LIVE_TMP"
  if [[ -n "$stage" && -d "$stage" ]]; then
    printf 'readme-live-record: retained incomplete staging at %s\n' "$stage" >&2
  fi
  return "$code"
}
trap cleanup EXIT
capture="$2"
destination="$4"
[[ "$destination" == /* && "$destination" != "$LIVE_HUB" &&
  "$destination" != "$LIVE_HUB/"* && "$destination" != "$capture" &&
  "$destination" != "$capture/"* && ! -e "$destination" && ! -L "$destination" ]] || live_die UNSAFE_OR_EXISTING_DESTINATION
parent="${destination%/*}"
name="${destination##*/}"
[[ -n "$name" && "$name" != . && "$name" != .. ]] || live_die UNSAFE_DESTINATION
parent_subject="$(live_root_subject "$parent")"
sources="$(live_sources)"
capture_subject="$(live_root_subject "$capture")"
live_profile "$capture"
live_snapshot "$capture" "$LIVE_TMP/capture" "${LIVE_CAPTURE_FILES[@]}"
live_capture "$LIVE_TMP/capture"

# Work is retained privately on failure. No existing generation is overwritten.
stage="$(mktemp -d -- "$parent/.readme-live-normalize.XXXXXXXXXX")"
for file in "${LIVE_CAPTURE_FILES[@]}"; do
  cp -a --no-dereference -- "$LIVE_TMP/capture/$file" "$stage/$file"
done
live_expected "$stage" "$stage"
live_manifest "$stage" "$stage/manifest.json" "$sources"
live_stage "$stage" "$sources"
live_recheck "$capture" "$LIVE_TMP/capture" "$capture_subject" "$sources" "${LIVE_CAPTURE_FILES[@]}"
[[ "$(live_root_subject "$parent")" == "$parent_subject" &&
  ! -e "$destination" && ! -L "$destination" ]] || live_die DESTINATION_CHANGED
stage_subject="$(live_root_subject "$stage")"
mv -T --no-clobber -- "$stage" "$destination" || live_die PUBLICATION_INCOMPLETE
[[ ! -e "$stage" && ! -L "$stage" &&
  "$(live_root_subject "$destination")" == "$stage_subject" ]] || live_die PUBLICATION_INCOMPLETE
stage=''
echo 'readme-live-record: NORMALIZED_CAPTURE_ONLY (operator assertions retained; no media or provider proof)'
