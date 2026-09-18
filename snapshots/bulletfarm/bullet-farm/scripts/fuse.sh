#!/usr/bin/env bash
# Thin source launcher. Validation and atomic publication live in Rust.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")/.."
exec cargo run --quiet --locked --bin bullet-family -- fuse "$@"
