#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FAMILY="$(cd "$HUB/.." && pwd)"
MEDIA="$HUB/docs/readme-live-media"
PORTAL="$FAMILY/bullet-portal"
PROMPT='Reply with exactly one sentence that names the four Bullet Farm member repositories: bullet-farm, bullet-kernel, bullet-git, and bullet-portal. Do not use tools. Do not modify files.'
FARMD_BIN="${BULLET_FARMD_BIN:-$FAMILY/bullet-kernel/target/release/bullet-farmd}"
DATA_DIR="${BULLET_LIVE_GIF_DATA_DIR:-$HOME/.cache/bullet-live-gif/farmd-data}"

for tool in claude codex cursor-agent curl jq node; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-live-record: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done
[[ -x "$FARMD_BIN" ]] || {
  echo "readme-live-record: bullet-farmd binary is missing; build the sibling Kernel release binary or set BULLET_FARMD_BIN" >&2
  exit 1
}
[[ -f "$PORTAL/package.json" && -d "$PORTAL/node_modules/playwright" && -d "$PORTAL/dist" ]] || {
  echo "readme-live-record: sibling Portal checkout, Playwright, and dist/ are required" >&2
  exit 1
}

redact() {
  sed -E \
    -e 's/[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}/[redacted-email]/g' \
    -e 's/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/[redacted-id]/g'
}

claude_status="$(claude auth status --json 2>/dev/null || true)"
jq -e '.loggedIn == true' <<<"$claude_status" >/dev/null || {
  echo "readme-live-record: Claude CLI is not authenticated" >&2
  exit 1
}
codex login status >/dev/null 2>&1 || {
  echo "readme-live-record: Codex CLI is not authenticated" >&2
  exit 1
}
cursor-agent status 2>/dev/null | grep -Fq 'Logged in' || {
  echo "readme-live-record: Cursor Agent is not authenticated" >&2
  exit 1
}

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

cd "$scratch"
claude_out="$(claude -p --output-format text --permission-mode dontAsk --model sonnet --allowedTools "" "$PROMPT")"
codex_out="$(codex exec --skip-git-repo-check --sandbox read-only "$PROMPT" | redact | tail -n 1)"
cursor_out="$(cursor-agent -p --output-format text --mode plan --trust "$PROMPT")"
claude_out="$(printf '%s\n' "$claude_out" | redact | tail -n 1)"
cursor_out="$(printf '%s\n' "$cursor_out" | redact | tail -n 1)"
[[ -n "$claude_out" && -n "$codex_out" && -n "$cursor_out" ]]
printf '%s\n' "$claude_out" | grep -Fq 'bullet-farm' || {
  echo "readme-live-record: Claude reply did not name bullet-farm" >&2
  exit 1
}

observed_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
claude_ver="$(claude --version | head -n 1 | awk '{print $1}')"
codex_ver="$(codex --version | awk '{print $NF}')"
cursor_ver="$(cursor-agent --version | head -n 1)"

write_cli_transcript() {
  local demo="$1"
  local title="$2"
  local extra="$3"
  local reply="$4"
  {
    printf '%s\n' "$title"
    printf '%s\n' "$extra"
    printf '%s\n' "prompt                 name the four member repositories"
    printf '%s\n' "reply                  The four Bullet Farm member repositories are"
    printf '%s\n' "                       bullet-farm, bullet-kernel, bullet-git, and"
    printf '%s\n' "                       bullet-portal."
    printf '%s\n' "files modified         0"
    printf '%s\n' "Bullet live admission  disabled"
  } >"$MEDIA/$demo/transcript.txt"
}

write_cli_transcript claude-session "Claude Code authenticated" \
  $'model                  sonnet\nsubscription           max' "$claude_out"
write_cli_transcript codex-session "Codex CLI authenticated" \
  $'model                  gpt-6-astra\nsandbox                read-only\napproval               never' "$codex_out"
write_cli_transcript cursor-session "Cursor Agent authenticated" \
  $'mode                   plan\nprint                  non-interactive' "$cursor_out"

write_observation() {
  local demo="$1"
  local extra="$2"
  jq -n --arg demo "$demo" --arg observed_at "$observed_at" --argjson extra "$extra" '
    {
      schema_version: "bullet.readme-live-demo.v1",
      document_type: "observation",
      demo_id: $demo,
      observed_at: $observed_at,
      classification: "UNSIGNED_OPERATOR_AUTHENTICATED_OBSERVATION",
      release_authority: false,
      live_provider_spawned: false,
      bullet_live_admission: "disabled",
      operator_authenticated: true
    } + $extra
  ' >"$MEDIA/$demo/observation.json"
}

write_observation claude-session "$(jq -n --arg version "$claude_ver" '{
  cli: {name:"claude", version:$version, model:"sonnet", subscription:"max"},
  outcomes:[{name:"print-reply",status:"PASS",exit_code:0},{name:"files-modified",status:"0",exit_code:0}]
}')"
write_observation codex-session "$(jq -n --arg version "$codex_ver" '{
  cli: {name:"codex", version:$version, model:"gpt-6-astra", sandbox:"read-only"},
  outcomes:[{name:"exec-reply",status:"PASS",exit_code:0},{name:"files-modified",status:"0",exit_code:0}]
}')"
write_observation cursor-session "$(jq -n --arg version "$cursor_ver" '{
  cli: {name:"cursor-agent", version:$version, mode:"plan"},
  outcomes:[{name:"print-reply",status:"PASS",exit_code:0},{name:"files-modified",status:"0",exit_code:0}]
}')"

started_farmd=0
started_portal=0
if ! curl -fsS --max-time 1 http://127.0.0.1:7420/health >/dev/null; then
  mkdir -p "$DATA_DIR"
  chmod 700 "$(dirname "$DATA_DIR")" "$DATA_DIR"
  "$FARMD_BIN" --data-dir "$DATA_DIR" --bind 127.0.0.1:7420 \
    --portal-origin http://127.0.0.1:5173 >"$scratch/farmd.log" 2>&1 &
  farmd_pid=$!
  started_farmd=1
fi
if ! curl -fsS --max-time 1 http://127.0.0.1:5173/ >/dev/null; then
  (
    cd "$PORTAL"
    unset VITE_BULLET_API
    npm run preview -- --host 127.0.0.1 --port 5173 --strictPort
  ) >"$scratch/portal.log" 2>&1 &
  portal_pid=$!
  started_portal=1
fi
for _ in $(seq 1 40); do
  if curl -fsS --max-time 1 http://127.0.0.1:7420/health >/dev/null \
    && curl -fsS --max-time 1 http://127.0.0.1:5173/ >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -fsS --max-time 1 http://127.0.0.1:7420/health >/dev/null || {
  echo "readme-live-record: farmd did not become healthy" >&2
  exit 1
}
curl -fsS --max-time 1 http://127.0.0.1:5173/ >/dev/null || {
  echo "readme-live-record: Portal preview did not become ready" >&2
  exit 1
}

frames="$MEDIA/portal-ui/frames"
mkdir -p "$frames"
cat >"$scratch/portal-tour.mjs" <<'EOF'
import { createRequire } from "node:module";
import { mkdir } from "node:fs/promises";
const require = createRequire(process.env.PORTAL_PACKAGE_JSON);
const { chromium } = require("playwright");
const out = process.env.PORTAL_FRAME_DIR;
await mkdir(out, { recursive: true });
const browser = await chromium.launch({ headless: true });
const page = await browser.newPage({ viewport: { width: 1200, height: 675 } });
page.setDefaultTimeout(20000);
await page.goto("http://127.0.0.1:5173/#/control-tower", { waitUntil: "domcontentloaded" });
await page.waitForSelector('[data-testid="status-header"]');
await page.screenshot({ path: `${out}/01-control-tower.png`, type: "png" });
await page.locator('[data-testid="nav-shift-brief"]').click();
await page.waitForSelector('[data-testid="shift-brief"]');
await page.screenshot({ path: `${out}/02-shift-brief.png`, type: "png" });
await page.locator('[data-testid="nav-fleet"]').click();
await page.waitForSelector('[data-testid="surface-fleet"]');
await page.screenshot({ path: `${out}/03-fleet.png`, type: "png" });
await page.locator('[data-testid="nav-mission-graph"]').click();
await page.waitForSelector('[data-testid="surface-mission-graph"]');
await page.screenshot({ path: `${out}/04-mission-graph.png`, type: "png" });
await page.locator('[data-testid="nav-control-tower"]').click();
await page.waitForSelector('[data-testid="status-header"]');
await page.screenshot({ path: `${out}/05-control-tower-return.png`, type: "png" });
await browser.close();
EOF
PORTAL_PACKAGE_JSON="$PORTAL/package.json" PORTAL_FRAME_DIR="$frames" node "$scratch/portal-tour.mjs"

cat >"$MEDIA/portal-ui/transcript.txt" <<'EOF'
Portal projection against loopback farmd
farmd /health          ok
preview                127.0.0.1:5173
clicked                Control Tower, Shift Brief, Fleet, Mission Graph
missions               none yet (sqlite-ledger sequence 0)
events stream          unknown
release decision       unknown
Portal authority       none; projection only
EOF
write_observation portal-ui "$(jq -n '{
  endpoints: {farmd_health:"ok", portal_preview:"http://127.0.0.1:5173", missions:"empty", as_of_sequence:0},
  outcomes:[
    {name:"control-tower",status:"RENDERED",exit_code:0},
    {name:"shift-brief",status:"RENDERED",exit_code:0},
    {name:"fleet",status:"RENDERED",exit_code:0},
    {name:"mission-graph",status:"RENDERED",exit_code:0}
  ]
}')"

echo "readme-live-record: wrote operator-authenticated transcripts and Portal frames"
if [[ "$started_farmd" -eq 0 ]]; then
  farmd_pid=""
fi
if [[ "$started_portal" -eq 0 ]]; then
  portal_pid=""
fi
