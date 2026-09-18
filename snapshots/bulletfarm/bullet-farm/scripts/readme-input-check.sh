#!/usr/bin/env bash
set -euo pipefail

MEDIA="${1:-}"
[[ "$#" -eq 1 && "$MEDIA" == /* && -d "$MEDIA" && ! -L "$MEDIA" ]] || {
  echo "readme-input-check: expected one absolute ordinary media directory" >&2
  exit 2
}
for tool in cmp mktemp; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-input-check: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

tmp="$(mktemp -d)"
trap 'rm -rf -- "$tmp"' EXIT

compare_expected() {
  local expected="$1" actual="$2" label="$3"
  [[ -f "$actual" && ! -L "$actual" ]] || {
    printf 'readme-input-check: missing or unsafe %s\n' "$label" >&2
    return 1
  }
  cmp -s "$expected" "$actual" || {
    printf 'readme-input-check: %s escaped its exact admitted bytes\n' "$label" >&2
    return 1
  }
}

write_tape() {
  local demo="$1" output="$2"
  printf '%s\n' \
    "Output docs/readme-media/$demo/$demo.gif" \
    'Set Shell "bash"' \
    'Set FontFamily "JetBrains Mono"' \
    'Set FontSize 22' \
    'Set Width 1200' \
    'Set Height 675' \
    'Set Framerate 12' \
    'Set TypingSpeed 0ms' \
    'Set PlaybackSpeed 1.0' \
    'Set Padding 28' \
    'Set Margin 0' \
    'Set CursorBlink false' \
    'Set Theme { "name": "Bullet", "black": "#111827", "red": "#ef4444", "green": "#22c55e", "yellow": "#f59e0b", "blue": "#60a5fa", "magenta": "#c084fc", "cyan": "#22d3ee", "white": "#e5e7eb", "brightBlack": "#6b7280", "brightRed": "#f87171", "brightGreen": "#4ade80", "brightYellow": "#fbbf24", "brightBlue": "#93c5fd", "brightMagenta": "#d8b4fe", "brightCyan": "#67e8f9", "brightWhite": "#f9fafb", "background": "#0b1020", "foreground": "#e5e7eb", "selection": "#334155", "cursor": "#0b1020" }' \
    '' \
    'Hide' \
    "Type \"export PS1='bullet\$ ' LANG=C.UTF-8 LC_ALL=C.UTF-8 TZ=UTC\"" \
    'Enter' \
    'Type "clear"' \
    'Enter' \
    'Show' \
    "Type \"bash docs/readme-media/playback.sh $demo\"" \
    'Enter' \
    'Sleep 8s' >"$output"
}

# shellcheck disable=SC2016 # This command writes literal shell source for exact-byte admission.
printf '%s\n' \
  '#!/usr/bin/env bash' \
  'set -euo pipefail' \
  '' \
  'ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"' \
  'case "${1:-}" in' \
  '  component-preview) transcript="$ROOT/component-preview/transcript.txt" ;;' \
  '  provider-safety) transcript="$ROOT/provider-safety/transcript.txt" ;;' \
  '  *)' \
  '    echo "usage: $0 {component-preview|provider-safety}" >&2' \
  '    exit 2' \
  '    ;;' \
  'esac' \
  '' \
  'sed -n '\''1,80p'\'' "$transcript"' >"$tmp/playback.sh"
compare_expected "$tmp/playback.sh" "$MEDIA/playback.sh" playback

for demo in component-preview provider-safety; do
  write_tape "$demo" "$tmp/$demo.tape"
  compare_expected "$tmp/$demo.tape" "$MEDIA/$demo/$demo.tape" "$demo tape"
done

printf '%s\n' \
  'Bullet Farm component preview' \
  'doctor                 BLOCKED (exit 3)' \
  'hub component checks   PASS' \
  'materialize replay     IDEMPOTENT' \
  'fence                  1 -> 2' \
  'stale authority        REFUSED' \
  'lost response effect   UNKNOWN' >"$tmp/component.txt"
compare_expected "$tmp/component.txt" "$MEDIA/component-preview/transcript.txt" 'component transcript'

printf '%s\n' \
  'Bullet Farm provider safety' \
  'Claude offline contract       PASS' \
  'Codex offline contract        PASS' \
  'Cursor offline contract       PASS' \
  'Antigravity offline contract  PASS' \
  'Claude live admission         POLICY_LIVE_ADMISSION_DISABLED' \
  'Codex live admission          POLICY_LIVE_ADMISSION_DISABLED' \
  'Cursor live admission         POLICY_LIVE_ADMISSION_DISABLED' \
  'Antigravity live admission    POLICY_LIVE_ADMISSION_DISABLED' \
  'provider processes spawned    0' \
  'live provider proof           ABSENT' >"$tmp/provider.txt"
compare_expected "$tmp/provider.txt" "$MEDIA/provider-safety/transcript.txt" 'provider transcript'

echo "readme-input-check: PASS (closed pre-execution inputs)"
