#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
case "${1:-}" in
  component-preview) transcript="$ROOT/component-preview/transcript.txt" ;;
  provider-safety) transcript="$ROOT/provider-safety/transcript.txt" ;;
  *)
    echo "usage: $0 {component-preview|provider-safety}" >&2
    exit 2
    ;;
esac

sed -n '1,80p' "$transcript"
