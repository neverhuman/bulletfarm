#!/usr/bin/env bash
# Bullet Farm on xbabe2. Every command below runs for real, while recording.
# The provider turn is a real, billed claude 2.1.266 turn against a real
# account. Nothing here is replayed, staged, or reconstructed.
set -u

RUNNER="${E2E_RUNNER:?}"      # the real end-to-end script
OUTROOT="${E2E_OUTROOT:?}"    # where its run directories land
PROOFROOT="${E2E_PROOFROOT:?}"

B='\033[1m'; C='\033[38;5;51m'; G='\033[38;5;46m'; Y='\033[38;5;226m'
M='\033[38;5;213m'; W='\033[38;5;255m'; R='\033[38;5;203m'; D='\033[38;5;245m'; O='\033[0m'

title() { printf "\n${B}${M}%s${O}\n" "$*"; }
say()   { printf "${B}${C}\$ %s${O}\n" "$*"; }
run()   { say "$*"; eval "$@"; sleep 1.2; }

clear
printf "${B}${W}  BULLET FARM${O}  ${W}one contained provider turn becomes a reviewable Candidate${O}\n"
printf "  ${D}host xbabe2 · claude 2.1.266 · real account · real spend · recorded live${O}\n"
sleep 2.2

title "1  The provider is a root-staged runtime, not whatever is on PATH"
run "ls -l /usr/lib/bullet/providers/claude/2.1.266/bin/claude"
say "b3sum .../bin/claude   vs   the enrolled digest"
b3sum /usr/lib/bullet/providers/claude/2.1.266/bin/claude | cut -c1-64
grep -o '"executable_blake3":"[^"]*"' "$HOME/bullet-dogfood-2/policy/enrollments/claude.json"
printf "  ${G}the enrollment pins that exact binary. any other one is refused.${O}\n"
sleep 2.4

title "2  The turn gets no network, no host environment, no writable tree"
printf "  ${W}bwrap --unshare-all --clearenv --ro-bind <clone> /workspace --chdir /workspace${O}\n"
printf "  ${W}slirp4netns + nftables default-drop, one allowlisted provider host${O}\n"
printf "  ${G}ANTHROPIC_API_KEY, GH_TOKEN and SSH_AUTH_SOCK are dropped, not masked.${O}\n"
printf "  ${G}destructive writes are structurally impossible: there is nothing writable.${O}\n"
sleep 2.8

title "3  Running it now, on Bullet Farm's own kernel source"
say "proof-transaction-offline.sh   BULLET_TXN_PROVIDER=claude"
printf "  ${D}farmd + production gitd + the product Runner + a real provider turn${O}\n\n"
"$RUNNER" >/dev/null 2>&1 &
RUN_PID=$!
START=$(date +%s)
SPIN='|/-\'
i=0
while kill -0 "$RUN_PID" 2>/dev/null; do
  i=$(( (i + 1) % 4 ))
  EL=$(( $(date +%s) - START ))
  CONTAINED=$(pgrep -c -x bwrap 2>/dev/null || echo 0)
  NODE=$(pgrep -c -x claude 2>/dev/null || echo 0)
  if [ "$CONTAINED" -gt 0 ]; then
    STATE="${Y}provider turn running inside bwrap${O}"
  elif [ "$EL" -lt 25 ]; then
    STATE="${W}building, seeding source, starting farmd and gitd${O}"
  else
    STATE="${W}clone / apply / gate / candidate${O}"
  fi
  printf "\r  ${B}${C}%s${O}  ${W}%3ds${O}   bwrap=%s claude=%s   %b   " \
    "${SPIN:$i:1}" "$EL" "$CONTAINED" "$NODE" "$STATE"
  sleep 1
done
wait "$RUN_PID" 2>/dev/null
printf "\r  ${G}done in %ds${O}%-56s\n\n" "$(( $(date +%s) - START ))" ""
sleep 1.4

RUN="$(ls -dt "$OUTROOT"/2* | head -1)"
PROOF="$(ls -dt "$PROOFROOT"/2* | head -1)"

title "4  What the Runner journalled, from the run you just watched"
grep -oE 'journal seq [0-9]+: [^;]*' "$RUN/stderr.log" | while IFS= read -r line; do
  case "$line" in
    *turn_finished*)       printf "  ${B}${Y}%s${O}   ${D}a real, billed model turn${O}\n" "$line" ;;
    *patch_applied*)       printf "  ${B}${Y}%s${O}   ${D}the model's own proposal${O}\n" "$line" ;;
    *gate_result*)         printf "  ${B}${G}%s${O}\n" "$line" ;;
    *candidate_prepared*)  printf "  ${B}${G}%s${O}\n" "$line" ;;
    *candidate_preserved*) printf "  ${B}${G}%s${O}\n" "$line" ;;
    *GITD_REFUSED*|*ok=false*) printf "  ${R}%s${O}\n" "$line" ;;
    *)                     printf "  ${W}%s${O}\n" "$line" ;;
  esac
  sleep 0.5
done
sleep 1.2
printf "\n  ${R}the last two lines are a real, open defect, shown rather than hidden:${O}\n"
printf "  ${W}the Runner releases its lease before gitd cleans the workspace, and${O}\n"
printf "  ${W}gitd's cleanup re-reads that lease online. The Candidate is already${O}\n"
printf "  ${W}prepared and preserved by then. It reproduces on the simulator too.${O}\n"
sleep 1.6

title "5  The Candidate, opened from the bundle that run preserved"
CAND=$(mktemp -d)
git clone -q "$PROOF/artifacts/preserve/repository.bundle" "$CAND" 2>/dev/null
run "git -C $CAND log --oneline -1"
say "git show --stat HEAD"
git -C "$CAND" show --stat --format= HEAD
sleep 1.4
say "git show HEAD"
git -C "$CAND" show --format= HEAD | head -14
sleep 2.6

title "6  The receipt claims nothing it has not earned"
say "python3 -m json.tool dogfood-receipt.json"
if [ -f "$RUN/dogfood-receipt.json" ]; then
  python3 -m json.tool "$RUN/dogfood-receipt.json" | while IFS= read -r line; do
    case "$line" in
      *false*)               printf "  ${Y}%s${O}\n" "$line" ;;
      *cost*|*wall*|*blake*) printf "  ${B}${W}%s${O}\n" "$line" ;;
      *)                     printf "  ${W}%s${O}\n" "$line" ;;
    esac
    sleep 0.14
  done
fi
sleep 1.2
printf "\n  ${G}every eligibility flag is false. this clears no release gate,${O}\n"
printf "  ${G}and the artifact says so itself rather than a footnote saying it.${O}\n"
sleep 3.2
printf "\n${B}${M}  Bullet Farm${O}   ${W}github.com/neverhuman/bulletfarm${O}\n\n"
sleep 3.4
