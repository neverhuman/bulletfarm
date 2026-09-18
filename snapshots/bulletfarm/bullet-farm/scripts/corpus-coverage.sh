#!/usr/bin/env bash
# Render or check the generated zone docs/assurance/corpus-coverage.generated.md
# from policy/corpus-coverage-v1.json. The renderer is the hub library; until
# the CLI gains a `check corpus-coverage` verb this script routes through the
# ignored regeneration test so no extra binary enters the release archive.
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-check}"

case "$MODE" in
  check)
    (cd "$HUB" && cargo test --locked --quiet --test corpus_coverage)
    ;;
  write)
    (cd "$HUB" && BULLET_CORPUS_COVERAGE_WRITE=1 cargo test --locked --quiet \
      --test corpus_coverage regenerate_page -- --exact)
    (cd "$HUB" && cargo test --locked --quiet --test corpus_coverage)
    ;;
  *)
    echo "usage: corpus-coverage.sh [check|write]" >&2
    exit 2
    ;;
esac
