#!/usr/bin/env bash
# Check local Markdown links and fragments without making network requests.
set -euo pipefail

default_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
root="$default_root"
if [[ "${1:-}" == "--root" ]]; then
  [[ "$#" -ge 2 ]] || { echo "usage: check-links.sh [--root ROOT] [MARKDOWN ...]" >&2; exit 2; }
  root="$2"
  shift 2
fi

for tool in awk basename dirname realpath rg sed sort tr; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf '[ci] TOOL_MISSING: %s\n' "$tool" >&2
    exit 1
  }
done

canonical_root="$(realpath -e -- "$root")"
[[ -d "$canonical_root" ]] || { printf '[ci] MARKDOWN_ROOT_INVALID: %s\n' "$root" >&2; exit 1; }

declare -a markdown_files=()
scan_default_inventory=false
if [[ "$#" -gt 0 ]]; then
  markdown_files=("$@")
else
  scan_default_inventory=true
  cd "$canonical_root"
  mapfile -t markdown_files < <(printf '%s\n' README.md; rg --files docs -g '*.md' | LC_ALL=C sort)
fi
[[ "${#markdown_files[@]}" -gt 0 ]] || { printf '[ci] MARKDOWN_INVENTORY_EMPTY: no Markdown files\n' >&2; exit 1; }

within_root() {
  case "$1" in
    "$canonical_root"|"$canonical_root"/*) return 0 ;;
    *) return 1 ;;
  esac
}

github_slug() {
  sed -E \
    -e 's/<[^>]*>//g' \
    -e 's/`//g' \
    -e 's/[^[:alnum:] _-]//g' \
    -e 's/[[:space:]]+/-/g' \
    -e 's/^-+//' \
    -e 's/-+$//' \
    | tr '[:upper:]' '[:lower:]'
}

has_markdown_anchor() {
  local file="$1" wanted="$2" heading base slug count
  declare -A occurrences=()
  while IFS= read -r heading; do
    heading="${heading#* }"
    heading="$(sed -E 's/[[:space:]]+#+[[:space:]]*$//' <<<"$heading")"
    base="$(github_slug <<<"$heading")"
    [[ -n "$base" ]] || continue
    count="${occurrences[$base]:-0}"
    slug="$base"
    (( count == 0 )) || slug="$base-$count"
    occurrences[$base]=$((count + 1))
    [[ "$slug" == "$wanted" ]] && return 0
  done < <(rg '^#{1,6}[[:space:]]+' "$file" --no-filename || true)
  return 1
}

failures=0
check_target() {
  local source_file="$1" line_number="$2" raw="$3"
  local target path_part fragment='' source_dir candidate resolved target_for_error
  raw="$(sed -E 's/^[[:space:]]+//; s/[[:space:]]+$//' <<<"$raw")"
  if [[ "$raw" == \<* ]]; then
    target="${raw#<}"
    target="${target%%>*}"
  else
    target="${raw%%[[:space:]]*}"
  fi
  target_for_error="$target"
  case "$target" in
    ''|http://*|https://*|mailto:*|data:*|app://*) return 0 ;;
  esac
  if [[ "$target" == /* ]]; then
    printf '[ci] ABSOLUTE_DOC_LINK: %s:%s -> %s\n' "$source_file" "$line_number" "$target" >&2
    failures=1
    return 0
  fi
  if [[ "$target" == *'#'* ]]; then
    fragment="${target#*#}"
    fragment="${fragment%%\?*}"
  fi
  path_part="${target%%#*}"
  path_part="${path_part%%\?*}"
  source_dir="$(dirname "$canonical_root/$source_file")"
  candidate="$source_dir/${path_part:-$(basename "$source_file")}" 
  if ! resolved="$(realpath -e -- "$candidate" 2>/dev/null)"; then
    printf '[ci] BROKEN_RELATIVE_LINK: %s:%s -> %s\n' "$source_file" "$line_number" "$target_for_error" >&2
    failures=1
    return 0
  fi
  if ! within_root "$resolved"; then
    printf '[ci] DOC_LINK_ESCAPES_ROOT: %s:%s -> %s\n' "$source_file" "$line_number" "$target_for_error" >&2
    failures=1
    return 0
  fi
  if [[ -n "$fragment" && "$resolved" == *.md ]] && ! has_markdown_anchor "$resolved" "$fragment"; then
    printf '[ci] BROKEN_MARKDOWN_FRAGMENT: %s:%s -> %s\n' "$source_file" "$line_number" "$target_for_error" >&2
    failures=1
  fi
}

for source_file in "${markdown_files[@]}"; do
  [[ "$source_file" != /* ]] || {
    printf '[ci] ABSOLUTE_MARKDOWN_INPUT: %s\n' "$source_file" >&2
    failures=1
    continue
  }
  if ! canonical_source="$(realpath -e -- "$canonical_root/$source_file" 2>/dev/null)"; then
    printf '[ci] MARKDOWN_INPUT_MISSING: %s\n' "$source_file" >&2
    failures=1
    continue
  fi
  if ! within_root "$canonical_source" || [[ ! -f "$canonical_source" ]]; then
    printf '[ci] MARKDOWN_INPUT_INVALID: %s\n' "$source_file" >&2
    failures=1
    continue
  fi

  while IFS=$'\t' read -r line_number raw; do
    [[ -n "$line_number" ]] || continue
    raw="${raw#']('}"
    raw="${raw%')'}"
    check_target "$source_file" "$line_number" "$raw"
  done < <(rg -n -o --no-heading '\]\([^)]*\)' "$canonical_source" \
    | awk -F: '{ line=$1; sub(/^[^:]*:/, ""); print line "\t" $0 }' || true)

  while IFS=$'\t' read -r line_number raw; do
    [[ -n "$line_number" ]] || continue
    raw="$(sed -E 's/^[[:space:]]{0,3}\[[^]]+\]:[[:space:]]*//' <<<"$raw")"
    check_target "$source_file" "$line_number" "$raw"
  done < <(rg -n --no-heading '^[[:space:]]{0,3}\[[^]]+\]:[[:space:]]*<?[^[:space:]>]+>?' "$canonical_source" \
    | awk -F: '{ line=$1; sub(/^[^:]*:/, ""); print line "\t" $0 }' || true)
done

if [[ "$scan_default_inventory" == true ]]; then
  competitor_claims="$(rg -n --no-heading \
    'Gas[[:space:]]+(Town|City)|DeepSeek[[:space:]]+Harness|Omnigent' \
    docs/brand/mascots/[0-9][0-9]-*.md || true)"
  if [[ -n "$competitor_claims" ]]; then
    printf '[ci] UNPINNED_BRAND_COMPETITOR_CLAIM:\n%s\n' "$competitor_claims" >&2
    failures=1
  fi
fi

(( failures == 0 )) || exit 1
printf '[ci] relative Markdown links and fragments passed (%s files)\n' "${#markdown_files[@]}"
