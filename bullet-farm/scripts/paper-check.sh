#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
family_root="${BULLET_FAMILY_ROOT:-$(cd "$repo_root/.." && pwd -P)}"
paper_dir="$repo_root/docs/paper"
evidence="$paper_dir/evidence.json"
schema="$paper_dir/evidence.schema.json"
allow_dirty="${PAPER_ALLOW_DIRTY:-0}"

fail() { printf 'paper-check: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || fail "required tool missing: $1"; }
for tool in jq git sha256sum pdflatex bibtex pdfinfo pdffonts perl; do need "$tool"; done
bash "$repo_root/ops/ci/strict-json.sh" "$evidence" >/dev/null \
  || fail "evidence.json is ambiguous or invalid JSON"

jq -e '
  .schema_version == "bullet.paper-evidence.v1" and
  (.snapshot.id | length > 0) and
  (.snapshot.captured_at | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T")) and
  (.snapshot.source_date_epoch | type == "number") and
  (.snapshot.stage == "architecture-component-assurance") and
  (.snapshot.transaction_proof == false) and
  (.toolchain | type == "object") and
  ([.toolchain.rustc,.toolchain.cargo,.toolchain.git,.toolchain.just,
    .toolchain.jq,.toolchain.pdflatex,.toolchain.bibtex,.toolchain.poppler]
    | all(type == "string" and length > 0)) and
  (.commands | length > 0) and
  (.commands | all(
    (.repository as $repo | (["bullet-farm","bullet-kernel","bullet-git","bullet-portal"] | index($repo)) != null) and
    (.command | type == "string" and length > 0) and
    (.outcome | type == "string" and length > 0) and
    (.exit_code | type == "number") and
    (.boundary | type == "string" and length > 0)
  )) and
  (.repositories | length == 4) and
  ([.repositories[].name] | unique | length == 4) and
  (.maturity == ["Designed","Component-proved","Transaction-proved","Live/release-proved"]) and
  (.external_observations | length >= 4) and
  (.external_observations | all(
    (.name | type == "string" and length > 0) and
    (.subject | type == "string" and length > 0) and
    (.observed_at | type == "string" and test("^[0-9]{4}-[0-9]{2}-[0-9]{2}$")) and
    (.url | type == "string" and test("^https://") and . != "https://github.com" and . != "https://github.com/")
  )) and
  (([.external_observations[].name] | length) == ([.external_observations[].name] | unique | length)) and
  (.artifact_hashes | length > 0) and
  (.artifact_hashes | all(
    (.path | type == "string" and length > 0) and
    (.sha256 | type == "string" and test("^[0-9a-f]{64}$"))
  )) and
  (.results.release_gates.blocked == .results.release_gates.evaluated) and
  (.results.transaction_proof.outcome == "ABSENT")
' "$evidence" >/dev/null || fail "evidence.json violates the $schema contract or overstates maturity"

if [[ "$allow_dirty" != "1" ]]; then
  jq -e '
    .snapshot.family_clean == true and
    ([.repositories[].clean_at_capture] | all)
  ' "$evidence" >/dev/null || fail "evidence snapshot is not a clean four-repository capture"
fi

while IFS=$'\t' read -r name rel commit tree; do
  checkout="$family_root/$rel"
  [[ -d "$checkout/.git" ]] || fail "$name checkout missing: $checkout"
  actual_tree="$(git -C "$checkout" show -s --format=%T "$commit" 2>/dev/null)" || fail "$name commit is unavailable: $commit"
  [[ "$actual_tree" == "$tree" ]] || fail "$name snapshot tree mismatch: expected $tree, got $actual_tree"
  head="$(git -C "$checkout" rev-parse HEAD)"
  if [[ "$name" == "bullet-farm" && "$head" != "$commit" ]]; then
    git -C "$checkout" merge-base --is-ancestor "$commit" "$head" || fail "Hub publication head does not descend from evidence subject"
    disallowed="$(git -C "$checkout" diff --name-only "$commit..$head" -- . ':!Justfile' ':!docs/README.md' ':!docs/workplan.md' ':!docs/paper/**' ':!scripts/paper-build.sh' ':!scripts/paper-check.sh')"
    [[ -z "$disallowed" ]] || fail "Hub code changed after evidence subject: $disallowed"
  else
    [[ "$head" == "$commit" ]] || fail "$name snapshot/head mismatch: expected $commit, got $head"
  fi
  if [[ -n "$(git -C "$checkout" status --porcelain)" && "$allow_dirty" != "1" ]]; then
    fail "$name is dirty; freeze evidence only from clean canonical checkouts"
  fi
done < <(jq -r '.repositories[] | [.name,.path,.commit,.tree] | @tsv' "$evidence")

while IFS=$'\t' read -r rel expected; do
  actual="$(sha256sum "$repo_root/$rel" | awk '{print $1}')"
  [[ "$actual" == "$expected" ]] || fail "artifact hash mismatch for $rel"
done < <(jq -r '.artifact_hashes[] | [.path,.sha256] | @tsv' "$evidence")

tracked_paper="$(sha256sum "$paper_dir/bullet_farm_ieee.pdf" | awk '{print $1}')"
tracked_brief="$(sha256sum "$paper_dir/executive_brief.pdf" | awk '{print $1}')"
tracked_macros="$(sha256sum "$paper_dir/evidence.generated.tex" | awk '{print $1}')"
KEEP_PAPER_AUX=1 bash "$repo_root/scripts/paper-build.sh"
first_paper="$(sha256sum "$paper_dir/bullet_farm_ieee.pdf" | awk '{print $1}')"
first_brief="$(sha256sum "$paper_dir/executive_brief.pdf" | awk '{print $1}')"
first_macros="$(sha256sum "$paper_dir/evidence.generated.tex" | awk '{print $1}')"
[[ "$tracked_paper" == "$first_paper" ]] || fail "tracked paper PDF is stale relative to source/evidence"
[[ "$tracked_brief" == "$first_brief" ]] || fail "tracked brief PDF is stale relative to source/evidence"
[[ "$tracked_macros" == "$first_macros" ]] || fail "tracked evidence.generated.tex is stale relative to evidence.json"
KEEP_PAPER_AUX=1 bash "$repo_root/scripts/paper-build.sh"
[[ "$first_paper" == "$(sha256sum "$paper_dir/bullet_farm_ieee.pdf" | awk '{print $1}')" ]] || fail "paper PDF is nondeterministic under one source epoch"
[[ "$first_brief" == "$(sha256sum "$paper_dir/executive_brief.pdf" | awk '{print $1}')" ]] || fail "brief PDF is nondeterministic under one source epoch"

for log in "$paper_dir/bullet_farm_ieee.log" "$paper_dir/executive_brief.log"; do
  ! grep -Eq 'LaTeX Warning: (Citation|Reference).*undefined|multiply defined|There were undefined references' "$log" || fail "citation/reference warning in $(basename "$log")"
  ! perl -ne '$bad = 1 if /Overfull \\hbox \(([0-9.]+)pt too wide\)/ && $1 > 2; $bad = 1 if /Overfull \\vbox \(([0-9.]+)pt too high\)/ && $1 > 10; END { exit($bad ? 0 : 1) }' "$log" || fail "significant overfull box in $(basename "$log")"
done
! grep -Eq 'Warning--I didn.t find a database entry|Repeated entry|error message' "$paper_dir/bullet_farm_ieee.blg" || fail "BibTeX reported an unresolved or duplicate entry"
bib_entries="$(grep -c '^@' "$paper_dir/refs.bib")"
bbl_entries="$(grep -c '^\\bibitem' "$paper_dir/bullet_farm_ieee.bbl")"
[[ "$bib_entries" == "$bbl_entries" ]] || fail "bibliography has unused entries ($bib_entries source entries, $bbl_entries cited entries)"

perl -0777 -e '
  my %seen;
  while (/\@\w+\s*\{\s*([^,]+),(.*?)(?=\n\@|\z)/sg) {
    my ($key,$body)=($1,$2); die "duplicate bibliography key $key\n" if $seen{$key}++;
    next if $body =~ /keywords\s*=\s*\{internal\}/i;
    die "external reference $key lacks URL or DOI\n" unless $body =~ /(?:url|doi)\s*=/i;
    die "external reference $key lacks access date\n" unless $body =~ /urldate\s*=\s*\{\d{4}-\d{2}-\d{2}\}/i;
  }
' "$paper_dir/refs.bib" || fail "bibliography metadata preflight failed"

for pdf in bullet_farm_ieee executive_brief; do
  info="$(pdfinfo "$paper_dir/$pdf.pdf")"
  for field in Title Author Subject Keywords; do
    value="$(printf '%s\n' "$info" | sed -n "s/^$field:[[:space:]]*//p")"
    [[ -n "$value" ]] || fail "$pdf.pdf lacks PDF $field metadata"
  done
  printf '%s\n' "$info" | grep -q '^Page size:[[:space:]]*612 x 792 pts' || fail "$pdf.pdf is not US Letter"
  bad_fonts="$(pdffonts "$paper_dir/$pdf.pdf" | tail -n +3 | awk 'NF && $(NF-4) != "yes" {print}')"
  [[ -z "$bad_fonts" ]] || fail "$pdf.pdf contains unembedded fonts"
done

paper_pages="$(pdfinfo "$paper_dir/bullet_farm_ieee.pdf" | awk '/^Pages:/ {print $2}')"
brief_pages="$(pdfinfo "$paper_dir/executive_brief.pdf" | awk '/^Pages:/ {print $2}')"
(( paper_pages >= 12 && paper_pages <= 14 )) || fail "paper has $paper_pages pages; expected 12--14"
[[ "$brief_pages" == "4" ]] || fail "executive brief has $brief_pages pages; expected exactly 4"

git -C "$repo_root" diff --check
rm -f -- "$paper_dir"/bullet_farm_ieee.{aux,bbl,blg,log,out} "$paper_dir"/executive_brief.{aux,log,out}
printf 'paper-check: PASS (%s paper pages; %s brief pages; deterministic hashes %s / %s)\n' \
  "$paper_pages" "$brief_pages" "$first_paper" "$first_brief"
