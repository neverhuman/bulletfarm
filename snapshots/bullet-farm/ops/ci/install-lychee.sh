#!/usr/bin/env bash
# Checksum-pinned lychee for the scheduled external-link lane.
#
# The lane used to ask taiki-e/install-action for lychee@0.24.0. That action
# does not carry lychee at all at the pinned revision, so the step failed with
# "install-action does not support lychee@0.24.0" and the lane had never once
# installed its own tool. Nobody saw it because the scheduled workflow had never
# been dispatched. This follows the same shape as ops/ci/install-gitleaks.sh:
# fetch one named asset over TLS, check it against a digest recorded here, and
# refuse rather than continue if it does not match.
#
# The digest below is the one upstream publishes beside the asset as
# lychee-lychee-v0.24.0-x86_64-unknown-linux-gnu.tar.gz.sha256.
set -euo pipefail
version=0.24.0
tag="lychee-v${version}"
stem="lychee-${tag}-x86_64-unknown-linux-gnu"
archive="${RUNNER_TEMP:?RUNNER_TEMP is required}/${stem}.tar.gz"
tools_dir="$RUNNER_TEMP/bullet-tools"
expected='fc63c7f503dc7a910348a6cd7af27656d8278d4df9cdc5699d0a0e6fd11fed91'
curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error \
  "https://github.com/lycheeverse/lychee/releases/download/${tag}/${stem}.tar.gz" \
  --output "$archive"
printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --status
mkdir -p "$tools_dir"
tar -xzf "$archive" -C "$tools_dir" --strip-components=1 "${stem}/lychee"
[[ "$("$tools_dir/lychee" --version)" == "lychee ${version}" ]]
printf '%s\n' "$tools_dir" >>"${GITHUB_PATH:?GITHUB_PATH is required}"
