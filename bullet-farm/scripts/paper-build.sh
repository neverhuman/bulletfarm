#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
paper_dir="$repo_root/docs/paper"
evidence="$paper_dir/evidence.json"
generated="$paper_dir/evidence.generated.tex"

for tool in jq pdflatex bibtex; do
  command -v "$tool" >/dev/null 2>&1 || {
    printf 'paper-build: required tool missing: %s\n' "$tool" >&2
    exit 1
  }
done
bash "$repo_root/ops/ci/strict-json.sh" "$evidence" >/dev/null || {
  echo "paper-build: evidence.json is ambiguous or invalid JSON" >&2
  exit 1
}

jq -e '
  .schema_version == "bullet.paper-evidence.v1" and
  (.snapshot.source_date_epoch | type == "number") and
  (.repositories | length == 4) and
  ([.repositories[].name] | sort == ["bullet-farm","bullet-git","bullet-kernel","bullet-portal"]) and
  (.maturity == ["Designed","Component-proved","Transaction-proved","Live/release-proved"])
' "$evidence" >/dev/null

macro_tmp="$(mktemp "$paper_dir/.evidence.generated.XXXXXX")"
trap 'rm -f "$macro_tmp"' EXIT
jq -r '[
  "% Generated from evidence.json by scripts/paper-build.sh. Do not edit.",
  "\\newcommand{\\EvidenceSnapshotId}{" + .snapshot.id + "}",
  "\\newcommand{\\EvidenceSnapshotDate}{" + (.snapshot.captured_at[0:10]) + "}",
  "\\newcommand{\\EvidenceStage}{" + .snapshot.stage + "}",
  "\\newcommand{\\EvidenceHubCommit}{" + ([.repositories[] | select(.name == "bullet-farm")][0].commit[0:12]) + "}",
  "\\newcommand{\\EvidenceKernelCommit}{" + ([.repositories[] | select(.name == "bullet-kernel")][0].commit[0:12]) + "}",
  "\\newcommand{\\EvidenceGitCommit}{" + ([.repositories[] | select(.name == "bullet-git")][0].commit[0:12]) + "}",
  "\\newcommand{\\EvidencePortalCommit}{" + ([.repositories[] | select(.name == "bullet-portal")][0].commit[0:12]) + "}",
  "\\newcommand{\\EvidenceDoctorOutcome}{" + .results.doctor.outcome + "}",
  "\\newcommand{\\EvidenceReleaseOutcome}{" + .results.release_gates.outcome + "}",
  "\\newcommand{\\EvidenceReleaseEvaluated}{" + (.results.release_gates.evaluated | tostring) + "}",
  "\\newcommand{\\EvidenceReleaseBlocked}{" + (.results.release_gates.blocked | tostring) + "}",
  "\\newcommand{\\EvidenceFastOutcome}{" + .results.fast.outcome + "}",
  "\\newcommand{\\EvidenceDemoOutcome}{" + .results.demo.outcome + "}",
  "\\newcommand{\\EvidenceFormalOutcome}{" + .results.formal.outcome + "}",
  "\\newcommand{\\EvidenceFormalModels}{" + (.results.formal.models | tostring) + "}",
  "\\newcommand{\\EvidenceAuditScore}{" + (.results.audit.score | tostring) + "}",
  "\\newcommand{\\EvidenceAuditRaw}{" + (.results.audit.raw_score | tostring) + "}",
  "\\newcommand{\\EvidenceAuditCaps}{" + (.results.audit.caps | tostring) + "}",
  "\\newcommand{\\EvidenceAuditHard}{" + (.results.audit.hard_findings | tostring) + "}",
  "\\newcommand{\\EvidenceAuditSoft}{" + (.results.audit.soft_findings | tostring) + "}",
  "\\newcommand{\\EvidenceTransactionOutcome}{" + .results.transaction_proof.outcome + "}"
] | .[]' "$evidence" >"$macro_tmp"
mv -f "$macro_tmp" "$generated"
trap - EXIT

SOURCE_DATE_EPOCH="$(jq -r '.snapshot.source_date_epoch' "$evidence")"
export SOURCE_DATE_EPOCH
export FORCE_SOURCE_DATE=1
export TZ=UTC
export LC_ALL=C
export TEXINPUTS="$paper_dir:"

cd "$paper_dir"
pdflatex -halt-on-error -file-line-error -interaction=nonstopmode bullet_farm_ieee.tex >/dev/null
bibtex bullet_farm_ieee >/dev/null
pdflatex -halt-on-error -file-line-error -interaction=nonstopmode bullet_farm_ieee.tex >/dev/null
pdflatex -halt-on-error -file-line-error -interaction=nonstopmode bullet_farm_ieee.tex >/dev/null
pdflatex -halt-on-error -file-line-error -interaction=nonstopmode executive_brief.tex >/dev/null
pdflatex -halt-on-error -file-line-error -interaction=nonstopmode executive_brief.tex >/dev/null

if [[ "${KEEP_PAPER_AUX:-0}" != "1" ]]; then
  rm -f -- bullet_farm_ieee.aux bullet_farm_ieee.bbl bullet_farm_ieee.blg \
    bullet_farm_ieee.log bullet_farm_ieee.out executive_brief.aux \
    executive_brief.log executive_brief.out
fi
