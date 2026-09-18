# Bullet Farm paper and executive brief

Status: **Stage-1 architecture/component-assurance preprint; not release evidence**
Last reviewed: 2026-08-25 (snapshot `20260825-stage1-r3`; R4 merge adds the verified liveness-class issues #3584/#4737/#4091 to the issue audit, the exact TLC model boundary, evolutionary control as design, a red-team self-review appendix, and claim-site citations; R5 adds the value/risk framing below; no evidence input changed by either)

This directory holds the citable IEEEtran paper and one-column executive brief:

> Bullet Farm: A Software-Change Transaction Processor for Heterogeneous Agents

The author is **Bullet Farm Maintainers**. A green compile does not authorize a
command, Candidate, Evidence result, effect, installer, or release.

## Value and risk framing (R5)

A first-time reader must be able to state, within two minutes, what Bullet Farm
is, what it is worth if it works, what is proven today, and what the biggest
concerns are. Three places carry that and must stay consistent with each other
and with `evidence.json`:

1. the brief's front-page **Decision summary** (what it is / what it is worth if
   it works / what is proven today / biggest concerns / what must happen before
   launch);
2. the manuscript's abstract and the **Value and Risk at a Glance** table, the
   last subsection of the Research Contract; and
3. **Biggest Concerns, Limitations, and Artifact** in the evaluation section,
   which ranks the top eight concerns, each with its mitigation and the exact
   evidence that would retire it.

Upside language is bounded by the same maturity vocabulary as everything else:
it says what the design removes and what becomes measurable, never that any of
it is measured. Comparative or economic claims stay in Stage 2. When a concern
is retired by a receipt, update the ranked list, the glance table, and the
brief in the same change.

## Build and preflight

From the Hub root:

```bash
just paper
just paper-check
```

`just paper` generates shared LaTeX macros from `evidence.json` and builds both
PDFs under one source epoch. `just paper-check` is strict: it rejects dirty or
mismatched subjects, stale input hashes, bibliography/reference problems,
layout warnings, missing metadata, unembedded fonts, wrong page size/count, and
nondeterministic bytes. `PAPER_ALLOW_DIRTY=1 bash scripts/paper-check.sh` exists
only for local layout development; it is not publication preflight.

The evidence manifest follows `bullet.paper-evidence.v1`. Both documents
consume `evidence.generated.tex`; edit `evidence.json`, never generated macros.
PDF hashes are reported by the successful strict check. Neither PDF is a family
release subject.

## Authority

Write inventory rows only from the active
[`closure roadmap`](../assurance/closure-roadmap.md), generated
[`release truth`](../assurance/release-truth.generated.md),
[`../release.md`](../release.md), and the operator index
[`../assurance/product-gaps.md`](../assurance/product-gaps.md). The superseded
[`v1-closure-plan.md`](../assurance/v1-closure-plan.md) is provenance, not a
current inventory authority. Pin competitor
subjects from
[`../assurance/competitor-snapshot.md`](../assurance/competitor-snapshot.md)
and the dated landscape appendix. The non-authoritative opportunity backlog is
[`../workplan.md`](../workplan.md); it cannot change the active closure roadmap.

Stage 2, including any matched benchmark or measured superiority claim, is
deferred until a signed connected `TRANSACTION_PROOF` exists. Historical
Centerrail material under [`../spec/`](../spec/) remains provenance, not
runtime fact.
