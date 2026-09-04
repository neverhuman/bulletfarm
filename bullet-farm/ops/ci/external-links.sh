#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
cd "$REPO_ROOT"
mapfile -t inputs < <(printf '%s\n' README.md; rg --files docs -g '*.md' | LC_ALL=C sort)
lychee --no-progress --max-retries 2 --timeout 20 \
  --exclude 'https?://(127\.0\.0\.1|localhost)([:/]|$)' "${inputs[@]}"
log "scheduled external-link check passed"
