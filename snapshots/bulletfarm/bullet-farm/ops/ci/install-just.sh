#!/usr/bin/env bash
# The fast lane proves `just setup` keeps its typed refusal without a PATH, so
# it needs `just` itself. The ubuntu-24.04 runner image does not carry it, and
# `command -v just` under `set -e` exited the lane with no output at all.
# Installed the same way as gitleaks and ripgrep: pinned version, pinned
# digest, verified version string, no package manager.
set -euo pipefail
version=1.58.0
target=x86_64-unknown-linux-musl
archive="${RUNNER_TEMP:?RUNNER_TEMP is required}/just-${version}-${target}.tar.gz"
tools_dir="$RUNNER_TEMP/bullet-tools"
expected='4a5cc2f53e6f0f8c59092a6cc38291eb729d46a7dd95d3ae582008881b84931d'
curl --proto '=https' --tlsv1.2 --fail --location --silent --show-error \
  "https://github.com/casey/just/releases/download/${version}/just-${version}-${target}.tar.gz" \
  --output "$archive"
printf '%s  %s\n' "$expected" "$archive" | sha256sum --check --status
mkdir -p "$tools_dir"
tar -xzf "$archive" -C "$tools_dir" just
[[ "$("$tools_dir/just" --version)" == "just ${version}"* ]]
printf '%s\n' "$tools_dir" >>"${GITHUB_PATH:?GITHUB_PATH is required}"
