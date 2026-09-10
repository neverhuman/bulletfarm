#!/usr/bin/env bash
# xbabe2-only Head confirmation. Hosted CI must never invoke this recipe.
# Off-node and CI/GITHUB_ACTIONS exit 78. This is not a provider replace.
set -euo pipefail

# shellcheck source=ops/ci/lib.sh
source "$(dirname "${BASH_SOURCE[0]}")/../ops/ci/lib.sh"

PINNED_JANKURAI_OID="${JANKURAI_TUIWRIGHT_OID:-b88562fdb124aa86dedd70ab972e7d0d87e58be1}"
STATE_DIR="${BULLET_XBABE2_HEAD_STATE:-${XDG_STATE_HOME:-$HOME/.local/state}/bullet-xbabe2-head}"
FAMILY_ROOT="$(cd "$REPO_ROOT/.." && pwd)"
KERNEL="$FAMILY_ROOT/bullet-kernel"
PORTAL="$FAMILY_ROOT/bullet-portal"
SCENARIO="${XBABE2_HEAD_SCENARIO:-head-doors}"
PROVIDER_SET="${XBABE2_HEAD_PROVIDER_SET:-none}"

usage() {
  printf 'usage: %s [--slack --slack-token-file /abs/0600/xapp-file]\n' "$0" >&2
  exit 2
}

slack_token_file=
want_slack=0
want_telegram=0
while [[ $# -gt 0 ]]; do
  case "$1" in
    --slack) want_slack=1; shift ;;
    --slack-token-file)
      [[ $# -ge 2 ]] || usage
      slack_token_file="$2"
      want_slack=1
      shift 2
      ;;
    --telegram) want_telegram=1; shift ;;
    -h|--help) usage ;;
    *) usage ;;
  esac
done

refuse78() {
  printf '%s: %s\n' "$1" "$2" >&2
  exit 78
}

host="$(hostname -s 2>/dev/null || hostname)"
if [[ -n "${CI:-}" || -n "${GITHUB_ACTIONS:-}" ]]; then
  refuse78 XBABE2_PROVIDER_PROOF_UNAVAILABLE \
    "just xbabe2-head refuses CI and GITHUB_ACTIONS; hosted workflows must not invoke it"
fi
if [[ "$host" != "xbabe2" ]]; then
  refuse78 XBABE2_PROVIDER_PROOF_UNAVAILABLE \
    "just xbabe2-head runs only on the Linux development host xbabe2 (observed ${host})"
fi

if [[ -d "$REPO_ROOT/.github/workflows" ]]; then
  if rg -q 'xbabe2-head|tuiwright|TUIWright' "$REPO_ROOT/.github/workflows"; then
    printf 'HOSTED_HEAD_PROOF_FORBIDDEN: Hub workflows must not invoke xbabe2-head or Tuiwright\n' >&2
    exit 1
  fi
fi

if [[ "$want_telegram" -eq 1 ]]; then
  refuse78 TELEGRAM_UNAVAILABLE \
    "no durable Telegram bind; Slack Socket Mode is the inbound door"
fi

if [[ "$want_slack" -eq 1 ]]; then
  if [[ -z "$slack_token_file" || "$slack_token_file" != /* || ! -f "$slack_token_file" || -L "$slack_token_file" ]]; then
    printf 'SLACK_TOKEN_FILE_INVALID: require an absolute regular file, never argv token bytes\n' >&2
    exit 2
  fi
  mode="$(stat -c '%a' "$slack_token_file")"
  if [[ "$mode" != "600" ]]; then
    printf 'SLACK_TOKEN_FILE_INVALID: token file mode must be 0600\n' >&2
    exit 2
  fi
  prefix="$(head -c 5 "$slack_token_file" || true)"
  if [[ "$prefix" != "xapp-" ]]; then
    printf 'SLACK_TOKEN_FILE_INVALID: token file must start with xapp-\n' >&2
    exit 2
  fi
fi

schema="27"
if [[ -f "$KERNEL/crates/adapters/src/sqlite/migrations/0028_slack_binds.sql" ]]; then
  schema="28"
fi
if [[ "$want_slack" -eq 1 && "$schema" != "28" ]]; then
  refuse78 SLACK_BIND_UNAVAILABLE \
    "durable Slack team/channel bind waits for an append-only schema row"
fi

kernel_tree="missing"
portal_tree="missing"
if [[ -d "$KERNEL/.git" ]]; then
  kernel_tree="$(git -C "$KERNEL" rev-parse 'HEAD^{tree}')"
fi
if [[ -d "$PORTAL/.git" ]]; then
  portal_tree="$(git -C "$PORTAL" rev-parse 'HEAD^{tree}')"
fi

key_material="${kernel_tree}"$'\n'"${portal_tree}"$'\n'"${SCENARIO}"$'\n'"${schema}"$'\n'"${PROVIDER_SET}"
key="$(printf '%s' "$key_material" | sha256sum | awk '{print $1}')"
receipt="$STATE_DIR/receipts/${key}"

if [[ -f "$receipt" ]]; then
  mode="$(stat -c '%a' "$receipt")"
  if [[ "$mode" == "600" ]]; then
    log "xbabe2-head skip registry hit ${key}"
    exit 0
  fi
  printf 'XBABE2_HEAD_RECEIPT_INVALID: skip receipt exists but is not mode 0600\n' >&2
  exit 1
fi

jankurai="${JANKURAI_CHECKOUT:-/home/ubuntu/jankurai}"
if [[ -d "$jankurai/.git" ]]; then
  observed="$(git -C "$jankurai" rev-parse HEAD)"
  if [[ "$observed" != "$PINNED_JANKURAI_OID" ]]; then
    log "Tuiwright pin ${PINNED_JANKURAI_OID}; checkout HEAD ${observed} is not the pin (consume-only, no path= dep)"
  else
    log "Tuiwright pin ${PINNED_JANKURAI_OID} matches ${jankurai}"
  fi
else
  log "Tuiwright pin ${PINNED_JANKURAI_OID} from https://github.com/neverhuman/jankurai; no local checkout required to refuse"
fi

blockers=()
if [[ ! -f "$KERNEL/apps/bullet/src/talk.rs" ]]; then
  blockers+=("TALK_CLI_UNAVAILABLE: conversational CLI waits for root talk.rs")
fi
if [[ ! -f "$PORTAL/playwright.head.config.ts" ]]; then
  blockers+=("HEAD_PLAYWRIGHT_UNAVAILABLE: authenticated Head Playwright is xbabe2-only and not hosted")
fi
if [[ "$schema" != "28" ]]; then
  blockers+=("SLACK_BIND_UNAVAILABLE: durable Slack team/channel bind waits for an append-only schema row")
fi
blockers+=("TELEGRAM_UNAVAILABLE: no durable Telegram bind; Slack Socket Mode is the inbound door")
blockers+=("HEAD_RUNTIME_BINDING_REQUIRED: native Head outcome port is not bound")

printf 'XBABE2_HEAD_PROOF_INCOMPLETE:\n' >&2
for blocker in "${blockers[@]}"; do
  printf '  %s\n' "$blocker" >&2
done
exit 78
