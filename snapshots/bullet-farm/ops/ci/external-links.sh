#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
# shellcheck source=ops/ci/markdown-inputs.sh
source "$(dirname "${BASH_SOURCE[0]}")/markdown-inputs.sh"
cd "$REPO_ROOT"
collect_markdown_inputs "$REPO_ROOT" member
inputs=("${MARKDOWN_INPUTS[@]}")
collect_markdown_inputs "$REPO_ROOT" templates
templates=("${MARKDOWN_INPUTS[@]}")
options=(--no-progress --max-retries 2 --timeout 20 --hidden --no-ignore
  --exclude 'https?://(127\.0\.0\.1|localhost)([:/]|$)')
# Feed complete Markdown to Lychee: extracting only inline/autolinks loses
# reference definitions. Preserve existing member file-link checking as well.
lychee "${options[@]}" -- "${inputs[@]}"
if [[ "${#templates[@]}" -gt 0 ]]; then
  # Relative template destinations exist only in the generated aggregate.
  lychee "${options[@]}" --scheme http --scheme https -- "${templates[@]}"
  log "published template external links passed; generated-aggregate relative links require separate validation"
fi
log "scheduled external-link check passed (${#inputs[@]} member, ${#templates[@]} template Markdown files)"
