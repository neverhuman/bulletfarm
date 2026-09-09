#!/usr/bin/env bash
# Record high-resolution operator-authenticated TUI and Portal demos.
# Requires already-signed-in Claude, Codex, and Cursor sessions.
# Never writes provider keys, enrollments, or bootstrap tokens into the tree.
set -euo pipefail
export PATH="$HOME/.local/bin:$PATH"

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
FAMILY="$(cd "$HUB/.." && pwd -P)"
MEDIA="${DEMO_GIF_ROOT:-$HUB/docs/demo-gif}"
PORTAL="$FAMILY/bullet-portal"
FARMD_BIN="${BULLET_FARMD_BIN:-$FAMILY/bullet-kernel/target/release/bullet-farmd}"
DATA_ROOT="${BULLET_DEMO_GIF_CACHE:-$HOME/.cache/bullet-demo-gif}"
DATA_DIR="$DATA_ROOT/farmd-data"
PTY_RECORD="$HUB/scripts/lib/demo-gif-pty-record.py"
PORTAL_TOUR="$HUB/scripts/lib/demo-gif-portal-tour.mjs"
PROMPT='Explain Bullet Farm in six short lines: name the four member repos (bullet-farm, bullet-kernel, bullet-git, bullet-portal), the fenced Attempt, exact Candidate identity, independent Evidence, and why UNKNOWN is a first-class outcome rather than a green lie. Do not use tools. Do not modify files.'

for tool in claude codex cursor-agent curl jq node python3 ffmpeg; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'demo-gif-record: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
[[ -x "$FARMD_BIN" ]] || {
  echo "demo-gif-record: bullet-farmd binary is missing; set BULLET_FARMD_BIN" >&2
  exit 1
}
[[ -f "$PORTAL/package.json" && -d "$PORTAL/node_modules/playwright" && -d "$PORTAL/dist" ]] || {
  echo "demo-gif-record: sibling Portal checkout, Playwright, and dist/ are required" >&2
  exit 1
}

claude_status="$(claude auth status --json 2>/dev/null || true)"
jq -e '.loggedIn == true' <<<"$claude_status" >/dev/null || {
  echo "demo-gif-record: Claude CLI is not authenticated" >&2
  exit 1
}
codex login status >/dev/null 2>&1 || {
  echo "demo-gif-record: Codex CLI is not authenticated" >&2
  exit 1
}
cursor-agent status 2>/dev/null | grep -Fq 'Logged in' || {
  echo "demo-gif-record: Cursor Agent is not authenticated" >&2
  exit 1
}

mkdir -p "$DATA_ROOT" "$DATA_DIR"
chmod 700 "$DATA_ROOT" "$DATA_DIR"

scratch="$(mktemp -d)"
farmd_pid=""
portal_pid=""
cleanup() {
  if [[ -n "$farmd_pid" ]] && kill -0 "$farmd_pid" 2>/dev/null; then
    kill "$farmd_pid" 2>/dev/null || true
    wait "$farmd_pid" 2>/dev/null || true
  fi
  if [[ -n "$portal_pid" ]] && kill -0 "$portal_pid" 2>/dev/null; then
    kill "$portal_pid" 2>/dev/null || true
    wait "$portal_pid" 2>/dev/null || true
  fi
  rm -rf "$scratch"
}
trap cleanup EXIT

record_cli() {
  local name="$1"
  shift
  local dest="$MEDIA/$name"
  mkdir -p "$dest"
  python3 "$PTY_RECORD" \
    --cast "$dest/session.cast" \
    --transcript "$dest/transcript.txt" \
    --cols 140 \
    --rows 38 \
    --title "$name" \
    "$@"
}

cd "$FAMILY"

echo "demo-gif-record: recording authenticated Claude TUI"
record_cli claude-tui \
  --idle-quit 14.0 \
  --min-runtime 55 \
  --max-seconds 180 \
  -- claude --model sonnet --permission-mode dontAsk "$PROMPT"

echo "demo-gif-record: recording authenticated Codex TUI"
record_cli codex-tui \
  --inject-delay 2.6 \
  --inject "1\\r" \
  --idle-quit 14.0 \
  --min-runtime 55 \
  --max-seconds 180 \
  -- codex --sandbox read-only "$PROMPT"

echo "demo-gif-record: recording authenticated Cursor TUI"
record_cli cursor-tui \
  --idle-quit 14.0 \
  --min-runtime 55 \
  --max-seconds 180 \
  -- cursor-agent --mode plan --trust "$PROMPT"

for demo in claude-tui codex-tui cursor-tui; do
  grep -Fq 'bullet-farm' "$MEDIA/$demo/transcript.txt" || {
    printf 'demo-gif-record: %s transcript did not name bullet-farm; trying print-mode PTY\n' "$demo" >&2
    case "$demo" in
      claude-tui)
        record_cli claude-tui \
          --idle-quit 2.0 --max-seconds 120 -- \
          claude -p "$PROMPT" --output-format text --permission-mode dontAsk --model sonnet
        ;;
      codex-tui)
        record_cli codex-tui \
          --idle-quit 2.0 --max-seconds 120 -- \
          codex exec --skip-git-repo-check --sandbox read-only "$PROMPT"
        ;;
      cursor-tui)
        record_cli cursor-tui \
          --idle-quit 2.0 --max-seconds 120 -- \
          cursor-agent -p --output-format text --mode plan --trust "$PROMPT"
        ;;
    esac
    grep -Fq 'bullet-farm' "$MEDIA/$demo/transcript.txt" || {
      printf 'demo-gif-record: %s still missing bullet-farm after print fallback\n' "$demo" >&2
      exit 1
    }
  }
done

FARMD_BIND="${BULLET_DEMO_FARMD_BIND:-127.0.0.1:7421}"
PORTAL_BIND="${BULLET_DEMO_PORTAL_BIND:-127.0.0.1:5174}"
PORTAL_ORIGIN="http://$PORTAL_BIND"
rm -f "$DATA_DIR/ledger.sqlite" "$DATA_DIR/ledger.sqlite-wal" "$DATA_DIR/ledger.sqlite-shm"
"$FARMD_BIN" --data-dir "$DATA_DIR" --bind "$FARMD_BIND" \
  --portal-origin "$PORTAL_ORIGIN" >"$scratch/farmd.log" 2>&1 &
farmd_pid=$!
(
  cd "$PORTAL"
  unset VITE_BULLET_API
  export BULLET_FARMD_TEST_PROXY="http://$FARMD_BIND"
  npm run preview -- --host 127.0.0.1 --port "${PORTAL_BIND##*:}" --strictPort
) >"$scratch/portal.log" 2>&1 &
portal_pid=$!

for _ in $(seq 1 80); do
  if curl -fsS --max-time 1 "http://$FARMD_BIND/health" >/dev/null \
    && curl -fsS --max-time 1 "$PORTAL_ORIGIN/" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS --max-time 1 "http://$FARMD_BIND/health" >/dev/null || {
  echo "demo-gif-record: farmd did not become healthy" >&2
  tail -n 40 "$scratch/farmd.log" >&2 || true
  exit 1
}
curl -fsS --max-time 1 "$PORTAL_ORIGIN/" >/dev/null || {
  echo "demo-gif-record: Portal preview did not become ready" >&2
  tail -n 40 "$scratch/portal.log" >&2 || true
  exit 1
}

bootstrap="$(awk '/Bullet Farm one-time bootstrap: /{print $NF; exit}' "$scratch/farmd.log")"
[[ "$bootstrap" == boot_* ]] || {
  echo "demo-gif-record: farmd did not print a one-time bootstrap token" >&2
  exit 1
}

mkdir -p "$MEDIA/portal-form"
PORTAL_PACKAGE_JSON="$PORTAL/package.json" \
  PORTAL_ORIGIN="$PORTAL_ORIGIN" \
  PORTAL_CAPTURE_DIR="$scratch/portal-capture" \
  BULLET_BOOTSTRAP_TOKEN="$bootstrap" \
  node "$PORTAL_TOUR"

mkdir -p "$MEDIA/portal-form/frames"
cp -a "$scratch/portal-capture/frames/." "$MEDIA/portal-form/frames/"
video_src="$(tr -d '\n' <"$scratch/portal-capture/video-path.txt")"
cp "$video_src" "$MEDIA/portal-form/portal-form.webm"
ffmpeg -y -hide_banner -loglevel error -i "$video_src" \
  -vf "fps=12,scale=1920:1080:flags=lanczos" -an \
  "$MEDIA/portal-form/portal-form.mp4"

cat >"$MEDIA/portal-form/transcript.txt" <<'EOF'
Portal Control Tower against loopback farmd
form                   One-time bootstrap token + Authenticate local session
session                local browser material present after exchange
command                Submit durable demo command (projection; UNKNOWN stays honest)
clicked                Control Tower, Shift Brief, Fleet, Mission Graph
Portal authority       none; projection only
Bullet live admission  disabled
EOF

observed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
jq -n --arg observed_at "$observed_at" '
{
  schema_version: "bullet.demo-gif.v1",
  classification: "UNSIGNED_OPERATOR_AUTHENTICATED_OBSERVATION",
  release_authority: false,
  live_provider_spawned: false,
  bullet_live_admission: "disabled",
  operator_authenticated: true,
  observed_at: $observed_at,
  geometry: {cli_cols: 140, cli_rows: 38, portal_width: 1920, portal_height: 1080}
}' >"$MEDIA/capture.json"

echo "demo-gif-record: wrote casts, transcripts, and Portal capture under $MEDIA"
echo "demo-gif-record: next: just demo-gif-render && just demo-gif-check"
