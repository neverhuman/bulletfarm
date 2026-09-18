#!/usr/bin/env bash
# Shared Markdown source inventory. Templates are rendered at the aggregate root,
# so their relative links need a separate check against that generated subject.

markdown_inventory_paths() (
  local root="$1" scope="$2" directory
  cd "$root" || return 1
  case "$scope" in
    member)
      [[ -f README.md && -d docs && ! -L docs ]] || {
        printf '[ci] MARKDOWN_INVENTORY_INVALID: README.md/docs required\n' >&2
        return 1
      }
      find . -maxdepth 1 \( -type f -o -type l \) -name '*.md' -print0 || return 1
      for directory in docs .github; do
        [[ -e "$directory" || -L "$directory" ]] || continue
        [[ -d "$directory" && ! -L "$directory" ]] || return 1
        find "$directory" \( \( -type f -name '*.md' \) -o -type l \) -print0 || return 1
      done
      ;;
    templates)
      directory=publication/root
      [[ -e "$directory" || -L "$directory" ]] || return 0
      [[ -d "$directory" && ! -L "$directory" ]] || return 1
      case "$(realpath -e -- "$directory")" in
        "$root"/*) ;;
        *) printf '[ci] MARKDOWN_TEMPLATE_ROOT_ESCAPES: %s\n' "$directory" >&2; return 1 ;;
      esac
      find "$directory" \( \( -type f -name '*.md' \) -o -type l \) -print0 || return 1
      ;;
    *) printf '[ci] MARKDOWN_INVENTORY_SCOPE_INVALID: %s\n' "$scope" >&2; return 1 ;;
  esac
)

# Assign an array only after successful enumeration, with no lost process-
# substitution status and no newline or whitespace splitting of filenames.
collect_markdown_inputs() {
  local root="$1" scope="$2" inventory source resolved canonical_root tool
  local -a candidates=() selected=()
  MARKDOWN_INPUTS=()
  for tool in find mktemp realpath rm sort; do
    command -v "$tool" >/dev/null 2>&1 || {
      printf '[ci] TOOL_MISSING: %s\n' "$tool" >&2
      return 1
    }
  done
  canonical_root="$(realpath -e -- "$root")" || return 1
  inventory="$(mktemp)" || return 1
  if ! markdown_inventory_paths "$canonical_root" "$scope" | LC_ALL=C sort -z >"$inventory"; then
    rm -f -- "$inventory"
    printf '[ci] MARKDOWN_INVENTORY_FAILED: %s\n' "$scope" >&2
    return 1
  fi
  mapfile -d '' -t candidates <"$inventory"
  rm -f -- "$inventory"
  for source in "${candidates[@]}"; do
    # find does not traverse directory symlinks. Refuse them instead of silently
    # omitting Markdown below a linked community/template directory.
    if [[ -L "$canonical_root/$source" && -d "$canonical_root/$source" ]]; then
      printf '[ci] MARKDOWN_DIRECTORY_SYMLINK: %s\n' "$source" >&2
      return 1
    fi
    [[ "$source" == *.md ]] || continue
    resolved="$(realpath -e -- "$canonical_root/$source")" || return 1
    case "$resolved" in
      "$canonical_root"/*) ;;
      *) printf '[ci] MARKDOWN_INPUT_ESCAPES_ROOT: %s\n' "$source" >&2; return 1 ;;
    esac
    [[ -f "$resolved" ]] || {
      printf '[ci] MARKDOWN_INPUT_INVALID: %s\n' "$source" >&2
      return 1
    }
    selected+=("$source")
  done
  # shellcheck disable=SC2034 # public array consumed by both link checkers
  MARKDOWN_INPUTS=("${selected[@]}")
}
