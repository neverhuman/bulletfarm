#!/usr/bin/env bash
set -euo pipefail

HUB="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MEDIA="$HUB/docs/readme-media"
staged=false
if [[ "$#" -gt 0 ]]; then
  if [[ "$#" -ne 2 || "$1" != "--staged-root" || "$2" != /* ]]; then
    echo "usage: $0 [--staged-root ABSOLUTE_DIRECTORY]" >&2
    exit 2
  fi
  MEDIA="$2"
  staged=true
fi
MAX_GIF_BYTES=3145728
VHS_IMAGE='ghcr.io/charmbracelet/vhs@sha256:9d5fc3dc0c160b0fb1d2212baff07e6bdf3fa9438c504a3237484567302fcf93'
SOURCE_EPOCH=1787616000
VHS_VERSION_OUTPUT='vhs version v0.11.0 (c6af91a)'

for tool in cmp cp diff docker file find jq sha256sum stat; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'readme-check: missing required tool %s\n' "$tool" >&2
    exit 1
  }
done

tmp="$(mktemp -d)"
cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

# All validation reads a private no-dereference snapshot. A second snapshot at
# completion proves the caller-visible tree was not substituted mid-check.
source_media="$MEDIA"
stable_media="$tmp/media-snapshot"
if ! cp -a --no-dereference -- "$source_media" "$stable_media"; then
  echo "readme-check: could not take a no-dereference media snapshot" >&2
  exit 1
fi
if [[ ! -d "$stable_media" || -L "$stable_media" ]]; then
  echo "readme-check: media root must remain an ordinary directory" >&2
  exit 1
fi
MEDIA="$stable_media"

expected_media_inventory="$tmp/expected-media-inventory"
actual_media_inventory="$tmp/actual-media-inventory"
printf '%s\n' \
  README.md \
  playback.sh \
  snapshot.json \
  component-preview/component-preview.gif \
  component-preview/component-preview.tape \
  component-preview/fallback.png \
  component-preview/frames.framemd5 \
  component-preview/manifest.json \
  component-preview/observation.json \
  component-preview/transcript.txt \
  provider-safety/fallback.png \
  provider-safety/frames.framemd5 \
  provider-safety/manifest.json \
  provider-safety/observation.json \
  provider-safety/provider-safety.gif \
  provider-safety/provider-safety.tape \
  provider-safety/transcript.txt | LC_ALL=C sort >"$expected_media_inventory"
find "$MEDIA" -type f -print | sed "s#^$MEDIA/##" | LC_ALL=C sort >"$actual_media_inventory"
cmp "$expected_media_inventory" "$actual_media_inventory" || {
  echo "readme-check: stage-one media inventory drift; install/live media needs receipt-gated validation" >&2
  exit 1
}
if find "$MEDIA" -type l -print -quit | grep -q .; then
  echo "readme-check: media symlinks are forbidden" >&2
  exit 1
fi
bash "$HUB/scripts/readme-input-check.sh" "$MEDIA" >/dev/null

hostile_media="$tmp/hostile-media"
mkdir -p "$hostile_media"
cp -a "$MEDIA/." "$hostile_media/"
printf '%s\n' 'Type "true"' Enter >>"$hostile_media/component-preview/component-preview.tape"
if bash "$HUB/scripts/readme-input-check.sh" "$hostile_media" >/dev/null 2>&1; then
  echo "readme-check: pre-execution input admission accepted an extra tape command" >&2
  exit 1
fi
cp "$MEDIA/component-preview/component-preview.tape" \
  "$hostile_media/component-preview/component-preview.tape"
printf '%s\n' 'exec /forbidden-provider' >>"$hostile_media/playback.sh"
fake_bin="$tmp/fake-bin"
docker_marker="$tmp/docker-invoked"
mkdir -p "$fake_bin"
printf '%s\n' '#!/bin/sh' ": >\"\$README_DOCKER_MARKER\"" 'exit 99' >"$fake_bin/docker"
chmod +x "$fake_bin/docker"
set +e
README_DOCKER_MARKER="$docker_marker" README_MEDIA_ROOT="$hostile_media" \
  PATH="$fake_bin:$PATH" bash "$HUB/scripts/readme-render.sh" \
  --gif-root "$tmp/hostile-render" >/dev/null 2>&1
hostile_render_code=$?
set -e
[[ "$hostile_render_code" -ne 0 && ! -e "$docker_marker" ]] || {
  echo "readme-check: direct render reached Docker before refusing a hostile executable input" >&2
  exit 1
}

run_pinned_media_tool() {
  local entrypoint="$1"
  shift
  docker run --rm \
    --network none \
    --pull never \
    --user "$(id -u):$(id -g)" \
    --env HOME=/tmp \
    --env LANG=C.UTF-8 \
    --env LC_ALL=C.UTF-8 \
    --env TZ=UTC \
    --env SOURCE_DATE_EPOCH="$SOURCE_EPOCH" \
    --volume "$MEDIA:/media:ro" \
    --volume "$tmp:/out:rw" \
    --workdir /media \
    --entrypoint "$entrypoint" \
    "$VHS_IMAGE" "$@"
}

bash "$HUB/scripts/readme-schema-check.sh" "$MEDIA" >/dev/null

vhs_version="$(docker run --rm --network none --pull never "$VHS_IMAGE" --version)"
[[ "$vhs_version" == "$VHS_VERSION_OUTPUT" ]]
ffmpeg_version="$(docker run --rm --network none --pull never --entrypoint /usr/bin/ffmpeg "$VHS_IMAGE" -version | head -n 1)"
[[ "$ffmpeg_version" == "ffmpeg version 7.1.3-0+deb13u1 "* ]]

check_artifact_hashes() {
  local demo="$1"
  local directory="$MEDIA/$demo"
  local entry relative expected_sha expected_bytes path actual_sha actual_bytes
  while IFS= read -r entry; do
    relative="$(jq -r '.path' <<<"$entry")"
    [[ "$relative" != /* && "$relative" != *..* && "$relative" != */* ]] || {
      printf 'readme-check: unsafe artifact path %s\n' "$relative" >&2
      exit 1
    }
    path="$directory/$relative"
    [[ -f "$path" ]] || {
      printf 'readme-check: missing artifact %s\n' "$path" >&2
      exit 1
    }
    expected_sha="$(jq -r '.sha256' <<<"$entry")"
    expected_bytes="$(jq -r '.bytes' <<<"$entry")"
    actual_sha="$(sha256sum "$path" | awk '{print $1}')"
    actual_bytes="$(stat -c '%s' "$path")"
    [[ "$actual_sha" == "$expected_sha" && "$actual_bytes" == "$expected_bytes" ]] || {
      printf 'readme-check: artifact hash/size drift for %s\n' "$path" >&2
      exit 1
    }
  done < <(jq -c '.artifact_hashes[]' "$directory/manifest.json")
}

check_media_properties() {
  local demo="$1"
  local directory="$MEDIA/$demo"
  local gif="$directory/$demo.gif"
  local fallback="$directory/fallback.png"
  local width height rate duration bytes frame_count last_frame frame_stats y_min y_max
  [[ "$(file --brief --mime-type "$gif")" == "image/gif" ]]
  [[ "$(file --brief --mime-type "$fallback")" == "image/png" ]]
  width="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=width -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  height="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=height -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  rate="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -show_entries stream=r_frame_rate -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  duration="$(run_pinned_media_tool /usr/bin/ffprobe -v error -show_entries format=duration -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  bytes="$(stat -c '%s' "$gif")"
  [[ "$width" == 1200 && "$height" == 675 && "$rate" == "12/1" ]]
  awk -v duration="$duration" 'BEGIN { exit !(duration > 0 && duration <= 30) }'
  (( bytes <= MAX_GIF_BYTES ))

  run_pinned_media_tool /usr/bin/ffmpeg -v error -i "/media/$demo/$demo.gif" \
    -f framemd5 - | awk '!/^#/' >"$tmp/$demo.framemd5"
  cmp "$tmp/$demo.framemd5" "$directory/frames.framemd5"
  frame_count="$(run_pinned_media_tool /usr/bin/ffprobe -v error -select_streams v:0 -count_frames -show_entries stream=nb_read_frames -of default=nokey=1:noprint_wrappers=1 "/media/$demo/$demo.gif")"
  last_frame=$((frame_count - 1))
  run_pinned_media_tool /usr/bin/ffmpeg -v error -y -i "/media/$demo/$demo.gif" \
    -vf "select=eq(n\\,$last_frame)" -frames:v 1 "/out/$demo.png"
  cmp "$tmp/$demo.png" "$fallback"
  frame_stats="$(run_pinned_media_tool /usr/bin/ffmpeg -v error -i "/media/$demo/$demo.gif" \
    -vf "select=eq(n\\,$last_frame),signalstats,metadata=print:file=-" \
    -frames:v 1 -f null -)"
  y_min="$(awk -F= '$1 == "lavfi.signalstats.YMIN" { print $2 }' <<<"$frame_stats")"
  y_max="$(awk -F= '$1 == "lavfi.signalstats.YMAX" { print $2 }' <<<"$frame_stats")"
  [[ "$y_min" =~ ^[0-9]+([.][0-9]+)?$ && "$y_max" =~ ^[0-9]+([.][0-9]+)?$ ]] || {
    printf 'readme-check: missing final-frame luma statistics for %s\n' "$demo" >&2
    exit 1
  }
  awk -v y_min="$y_min" -v y_max="$y_max" 'BEGIN { exit !((y_max - y_min) >= 64) }' || {
    printf 'readme-check: final frame for %s has no visible foreground text (YMIN=%s YMAX=%s)\n' \
      "$demo" "$y_min" "$y_max" >&2
    exit 1
  }
}

for demo in component-preview provider-safety; do
  check_artifact_hashes "$demo"
  check_media_properties "$demo"
done

grep -Fq '](docs/readme-media/component-preview/component-preview.gif)' "$HUB/README.md"
grep -Fq '](docs/readme-media/provider-safety/provider-safety.gif)' "$HUB/README.md"
component_alt="$(grep -F '](docs/readme-media/component-preview/component-preview.gif)' "$HUB/README.md")"
provider_alt="$(grep -F '](docs/readme-media/provider-safety/provider-safety.gif)' "$HUB/README.md")"
[[ "$component_alt" == *'doctor BLOCKED'* && "$component_alt" == *'UNKNOWN'* ]]
[[ "$provider_alt" == *'four offline provider protocol suites passing'* && "$provider_alt" == *'four POLICY_LIVE_ADMISSION_DISABLED outcomes'* && "$provider_alt" == *'zero provider spawns'* ]]

printf '%s\n' \
  docs/readme-media/component-preview/component-preview.gif \
  docs/readme-media/provider-safety/provider-safety.gif >"$tmp/expected-readme-gifs"
rg -o 'docs/readme-media/[^)[:space:]]+\.gif' "$HUB/README.md" | LC_ALL=C sort >"$tmp/actual-readme-gifs"
cmp "$tmp/expected-readme-gifs" "$tmp/actual-readme-gifs" || {
  echo "readme-check: README GIF inventory drift; stage-two recordings require signed receipts" >&2
  exit 1
}
relative_links=(
  docs/readme-media/component-preview/fallback.png
  docs/readme-media/component-preview/transcript.txt
  docs/readme-media/component-preview/manifest.json
  docs/readme-media/provider-safety/fallback.png
  docs/readme-media/provider-safety/transcript.txt
  docs/readme-media/provider-safety/manifest.json
)
for relative in "${relative_links[@]}"; do
  grep -Fq "($relative)" "$HUB/README.md"
  [[ -f "$HUB/$relative" ]]
done

required_claims=(
  '**Many minds. One verified line to main.**'
  'Bullet Farm is building the transaction boundary for coding agents: fenced authority, one repository writer, exact Candidates, independent Evidence, durable effect reconciliation, and protected integration.'
  '**Current alpha:** the boundaries are component-proved; public installation, live providers, and the connected transaction remain blocked.'
  'Public installation is not available.'
  "The offline local bridge is component evidence only; \`TRANSACTION_PROOF\`, transaction-ready, and production-ready remain false."
  'https://github.com/gastownhall/gastown/releases/tag/v1.2.1'
  'https://github.com/gastownhall/gascity/releases/tag/v1.4.1'
  'https://github.com/deepseek-ai/DeepSeek-Harness/releases/tag/dsh-v0.1.1-rc.2'
  'https://github.com/omnigent-ai/omnigent/releases/tag/v0.10.0'
)
for claim in "${required_claims[@]}"; do
  grep -Fq "$claim" "$HUB/README.md" || {
    printf 'readme-check: required public claim missing: %s\n' "$claim" >&2
    exit 1
  }
done
grep -Fxq 'doctor                 BLOCKED (exit 3)' \
  "$MEDIA/component-preview/transcript.txt" || {
  echo "readme-check: component transcript lost the exact BLOCKED doctor status" >&2
  exit 1
}
grep -Fxq 'live provider proof           ABSENT' \
  "$MEDIA/provider-safety/transcript.txt" || {
  echo "readme-check: provider transcript lost the exact ABSENT live-proof status" >&2
  exit 1
}
grep -Fq 'Install media stays absent until two clean signed schema-3 installations exist.' \
  "$MEDIA/README.md" || {
  echo "readme-check: receipt-gated install-media sentence is absent" >&2
  exit 1
}
grep -Fq "Live-agent task media stays absent until exact runtime probing, provider onboarding, sealed live receipts, and a connected \`TRANSACTION_PROOF\` exist." \
  "$MEDIA/README.md" || {
  echo "readme-check: receipt-gated live-agent-media sentence is absent" >&2
  exit 1
}
[[ "$(grep -Fc 'contract-tested / live blocked' "$HUB/README.md")" -eq 4 ]] || {
  echo "readme-check: provider table must contain exactly four contract-tested / live blocked rows" >&2
  exit 1
}
provider_section="$(sed -n '/^## Provider boundary status$/,/^## Seven functions, five transaction authorities$/p' "$HUB/README.md")"
if grep -Eiq '\b(supported|ready)\b' <<<"$provider_section"; then
  echo "readme-check: provider table uses unsupported readiness vocabulary" >&2
  exit 1
fi
awk -F'|' '
  /^## Pinned public comparison$/ { comparison = 1; next }
  comparison && /^## / { comparison = 0 }
  comparison && /^\|/ && $2 !~ /Pinned subject|---/ {
    rows++
    for (column = 3; column <= 8; column++) {
      value = $column
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", value)
      if (value != "Documented" && value != "Partial/configuration-dependent" &&
          value != "Not documented" && value != "Unknown" && value != "N/A") exit 1
    }
  }
  END { if (rows != 5) exit 1 }
' "$HUB/README.md" || {
  echo "readme-check: comparison table escaped its five-value vocabulary or five-row inventory" >&2
  exit 1
}
bash "$HUB/ops/ci/check-links.sh"

redaction_pattern='(/home/|/Users/|[A-Za-z]:\\|/tmp/|Authorization:|Bearer[[:space:]]|BEGIN [A-Z ]*PRIVATE KEY|sk-[A-Za-z0-9_-]{16,}|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}|(glpat|gldt)-[A-Za-z0-9_-]{16,}|(AKIA|ASIA)[0-9A-Z]{16}|xox[baprs]-[A-Za-z0-9-]{10,}|xapp-[A-Za-z0-9-]{10,}|https://hooks[.]slack[.]com/services/[A-Za-z0-9/_-]{10,}|api[_-]?key|oauth[_-]?token)'
text_media=(
  "$HUB/README.md"
  "$MEDIA/README.md"
  "$MEDIA/playback.sh"
  "$MEDIA/snapshot.json"
  "$MEDIA/component-preview/component-preview.tape"
  "$MEDIA/component-preview/frames.framemd5"
  "$MEDIA/component-preview/manifest.json"
  "$MEDIA/component-preview/observation.json"
  "$MEDIA/component-preview/transcript.txt"
  "$MEDIA/provider-safety/frames.framemd5"
  "$MEDIA/provider-safety/manifest.json"
  "$MEDIA/provider-safety/observation.json"
  "$MEDIA/provider-safety/provider-safety.tape"
  "$MEDIA/provider-safety/transcript.txt"
)
for canary in '/home/operator/private' 'Authorization: Bearer test-credential' \
  'authorization: bearer test-credential' 'Api_Key=test-credential' \
  'ghp_''12345678901234567890' 'gho_''12345678901234567890' \
  'ghu_''12345678901234567890' 'ghs_''12345678901234567890' \
  'ghr_''12345678901234567890' 'github_pat_''12345678901234567890' \
  'glpat-''1234567890123456' 'gldt-''1234567890123456' \
  'AKIA''1234567890123456' 'ASIA''1234567890123456' \
  'xoxb-''1234567890-token' 'xapp-''1234567890-token' \
  'https://hooks.slack.com/''services/T00000000/B00000000/XXXXXXXXXXXXXXXXXXXXXXXX'; do
  printf '%s\n' "$canary" | LC_ALL=C grep -aEiq "$redaction_pattern" || {
    echo "readme-check: redaction canary escaped the forbidden-value pattern" >&2
    exit 1
  }
done
if LC_ALL=C grep -anEi "$redaction_pattern" "${text_media[@]}"; then
  echo "readme-check: media contains a forbidden path or credential-shaped value" >&2
  exit 1
fi
promotion_pattern="(TRANSACTION[_ \`-]*PROOF.{0,48}(EXISTS?|PRESENT|PASS(ED)?|VERIFIED|AVAILABLE|READY|COMPLETE(D)?|SUCCEEDED?|SCORECARD-ADMITTED)|(EXISTS?|PRESENT|PASS(ED)?|VERIFIED|AVAILABLE|READY|COMPLETE(D)?|SUCCEEDED?).{0,24}TRANSACTION[_ \`-]*PROOF|PUBLIC[ -]+INSTALLATION.{0,48}(AVAILABLE|READY|SUPPORTED|COMPLETE(D)?|PRESENT|ENABLED|WORKS)|LIVE[ -]+PROVIDERS?.{0,48}(SUPPORTED|READY|AVAILABLE|ENABLED|EXECUTED|RUNNING|COMPLETE(D)?|PASS(ED)?|VERIFIED|PRESENT|SUCCEEDED?)|LIVE[ -]+PROVIDER[ -]+(EXECUTION|READINESS|SUPPORT|PROOF).{0,32}(SUPPORTED|READY|AVAILABLE|ENABLED|COMPLETE(D)?|PASS(ED)?|VERIFIED|PRESENT|SUCCEEDED?)|CONNECTED[ -]+TRANSACTION.{0,48}(EXISTS?|PRESENT|PASS(ED)?|VERIFIED|AVAILABLE|READY|COMPLETE(D)?|SUCCEEDED?)|(TRANSACTION|PRODUCTION|RELEASE)[ -]+(IS[ -]+)?(READY|READINESS|AVAILABLE|SUPPORTED|COMPLETE(D)?))"
promotion_matches() {
  {
    LC_ALL=C grep -aRhEi "$promotion_pattern" "$@" || true
  } | sed -E \
    -e "s/TRANSACTION[_ \`-]*PROOF\`?[[:space:]]+IS[[:space:]]+ABSENT//Ig" \
    -e 's/PUBLIC[ -]+INSTALLATION[[:space:]]+IS[[:space:]]+NOT[[:space:]]+AVAILABLE//Ig' \
    -e 's/LIVE[ -]+PROVIDERS?[[:space:]]+REMAINS?[[:space:]]+BLOCKED//Ig' \
    -e 's/CONNECTED[ -]+TRANSACTION[[:space:]]+IS[[:space:]]+UNAVAILABLE//Ig' \
    -e 's/TRANSACTION-READY,?[[:space:]]+AND[[:space:]]+PRODUCTION-READY[[:space:]]+REMAINS?[[:space:]]+FALSE//Ig' \
    -e 's/(TRANSACTION|PRODUCTION|RELEASE)[ -]+READY[[:space:]]+REMAINS?[[:space:]]+FALSE//Ig' \
    -e "s/LIVE-AGENT TASK[[:space:]]+MEDIA[[:space:]]+STAYS[[:space:]]+ABSENT[[:space:]]+UNTIL[^.]*TRANSACTION[_ \`-]*PROOF\`?[[:space:]]+EXISTS?//Ig" \
    -e "s/INSTALL[[:space:]]+MEDIA[[:space:]]+STAYS[[:space:]]+ABSENT[[:space:]]+UNTIL[^.]*TRANSACTION[_ \`-]*PROOF\`?[[:space:]]+EXISTS?//Ig" \
    -e 's/PRODUCTION[ -]+READINESS[[:space:]]+BEFORE.{0,96}RECEIPTS[[:space:]]+EXIST//Ig' \
    | LC_ALL=C grep -aEi "$promotion_pattern"
}
promotion_is_absent() {
  ! promotion_matches "$@" >/dev/null
}
for canary in 'TRANSACTION_PROOF:VERIFIED' 'transaction_proof=pass' \
  'TRANSACTION_PROOF is VERIFIED' 'TRANSACTION_PROOF exists' \
  'transaction proof is complete' 'public installation is available' \
  'live providers are supported' 'live provider execution succeeded' \
  'connected transaction is complete' 'release-READY' 'release is READY' \
  'production is ready' 'live provider proof: PRESENT' \
  'TRANSACTION_PROOF for the offline local saga is scorecard-admitted' \
  'live provider proof is PRESENT'; do
  printf '%s\n' "$canary" | LC_ALL=C grep -Eiq "$promotion_pattern" || {
    echo "readme-check: promotion canary escaped the forbidden-claim pattern" >&2
    exit 1
  }
  printf '%s\n' "$canary" >"$tmp/promotion-canary"
  if promotion_is_absent "$tmp/promotion-canary"; then
    printf 'readme-check: positive claim escaped polarity admission: %s\n' "$canary" >&2
    exit 1
  fi
done
for allowed in 'TRANSACTION_PROOF is absent' \
  "The offline local bridge is component evidence only; \`TRANSACTION_PROOF\`, transaction-ready, and production-ready remain false." \
  'public installation is not available' 'live providers remain blocked' \
  'connected transaction is unavailable' 'production-ready remains false' \
  'Install media stays absent until a connected TRANSACTION_PROOF exists'; do
  printf '%s\n' "$allowed" >"$tmp/promotion-canary"
  promotion_is_absent "$tmp/promotion-canary" || {
    printf 'readme-check: negative claim was misclassified as promotion: %s\n' "$allowed" >&2
    exit 1
  }
done
public_truth_roots=("$HUB/README.md" "$MEDIA")
if ! promotion_is_absent "${public_truth_roots[@]}"; then
  echo "readme-check: public README or media promotes an unavailable release/live claim" >&2
  exit 1
fi
# shellcheck disable=SC2016 # Backticks are literal Markdown delimiters in the hostile input.
sed 's/The offline local bridge is component evidence only; `TRANSACTION_PROOF`, transaction-ready, and production-ready remain false./The offline local saga has `TRANSACTION_PROOF` scorecard-admitted; transaction-ready and production-ready remain false./' \
  "$HUB/README.md" >"$tmp/hostile-readme.md"
cmp -s "$HUB/README.md" "$tmp/hostile-readme.md" && {
  echo "readme-check: exact public README hostile mutation did not apply" >&2
  exit 1
}
hostile_truth_roots=("$tmp/hostile-readme.md" "$MEDIA")
if promotion_is_absent "${hostile_truth_roots[@]}"; then
  echo "readme-check: hostile public README promotion escaped the exact refusal path" >&2
  exit 1
fi

if [[ "$staged" == false ]]; then
  mkdir -p "$tmp/render-a" "$tmp/render-b"
  bash "$HUB/scripts/readme-render.sh" --gif-root "$tmp/render-a" >/dev/null
  bash "$HUB/scripts/readme-render.sh" --gif-root "$tmp/render-b" >/dev/null
  for demo in component-preview provider-safety; do
    cmp "$tmp/render-a/$demo.gif" "$tmp/render-b/$demo.gif"
    cmp "$tmp/render-a/$demo.gif" "$MEDIA/$demo/$demo.gif"
  done
fi

source_recheck="$tmp/source-recheck"
if ! cp -a --no-dereference -- "$source_media" "$source_recheck"; then
  echo "readme-check: media source changed during validation" >&2
  exit 1
fi
if [[ ! -d "$source_recheck" || -L "$source_recheck" ]] ||
  find "$source_recheck" -type l -print -quit | grep -q .; then
  echo "readme-check: media source gained a symlink during validation" >&2
  exit 1
fi
if ! diff -qr --no-dereference -- "$MEDIA" "$source_recheck" >/dev/null; then
  echo "readme-check: media source changed during validation" >&2
  exit 1
fi

echo "readme-check: PASS (unsigned component media; no release authority)"
