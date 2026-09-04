# Bullet Farm — IEEE white-paper record

Status: **historical summary; non-authoritative**  
Artifact availability: **the PDF, TeX, bibliography, and ZIP named below are not distributed by this repository**

The current IEEEtran preprint source lives at [`../paper/`](../paper/).
That source is not verified by the historically recorded hashes on this page
and is not release evidence.

---

This page records a separately produced **11-page IEEE two-column paper** with
**55 references** and approximately **7,250 words**:

> **Bullet Farm: A Transactional Multi-Frontier Architecture for Verified Autonomous Software Engineering**

The paper preserves the proposal’s defining contract—immutable planning, isolated writable attempts, permanent fencing, exact-Candidate evidence, protected delivery, and post-integration survival—while incorporating the subsequent lineage, independent-verification, sandbox, provider-governance, and external-effect corrections.  

## Artifact record

Earlier working material linked four machine-local `sandbox:` paths. Those
paths were never portable repository artifacts and have been removed. The
recorded hashes below are retained as historical metadata, not as verification
of files present in this checkout. A future publication must add the actual
files through the release process, verify them against their declared hashes,
and provide stable repository or release links.

## What the paper contains

The current-state survey distinguishes protocol-grade agent interfaces, structured headless CLIs, IDE-native agents, hosted change agents, open agent frameworks, and multi-agent orchestrators. It examines the present public integration contracts of Codex, Claude Code and the Claude Agent SDK, Cursor headless and ACP, and Google Antigravity headless operation rather than treating every harness as equivalent. ([OpenAI Developers][1])

It also covers representative adjacent systems and transitions, including:

* OpenHands and its Agent Server/SDK;
* SWE-agent and mini-SWE-agent;
* Agentless and AutoCodeRover;
* Aider;
* Cline, Continue, and Goose;
* GitHub Copilot’s hosted coding agent;
* Jules;
* Devin Local;
* Kiro;
* Roo Code;
* the Amazon Q Developer CLI transition;
* the Gemini CLI-to-Antigravity transition;
* Gas City as the principal orchestration case study;
* ACP and MCP as emerging interoperability layers.

The Gas City analysis is deliberately fair: it treats the project as a sophisticated orchestration SDK and a valuable source of real operational evidence—not as a straw man. Its public failures are reduced to recurring architectural classes involving ambiguous retries, incarnation confusion, schema availability, stale runtime identity, unsafe liveness inference, configuration blast radius, account drift, and projection inconsistency. The corresponding Bullet Farm rules are presented in a dedicated failure-to-design table. The underlying source review identified the broader issue: authority is not consistently bound to immutable incarnation, durable effect identity, exact evidence subject, runtime generation, and isolated writable ownership. 

## Core Bullet Farm contribution

The paper formalizes this lifecycle:

```text
Mission
  → immutable Plan Revision
  → atomic Work-Package graph
  → isolated Variant
  → incarnation-fenced Attempt
  → immutable Candidate
  → exact-subject Evidence
  → independent Review
  → brokered external Effect
  → protected Integration
  → post-merge Observation Window
```

Its five trust planes are:

1. **Control plane** — Missions, plans, graph revisions, leases, fences, commands, policies, budgets, routing, and quota.
2. **Execution plane** — credential-bearing provider enclave, untrusted repository sandbox, private clone, resource controls, checkpoints, and Candidate preparation.
3. **Verification plane** — independent runners, clean reconstruction, separate caches and credentials, protected test custody, and verifier attestations.
4. **Effect and delivery plane** — last-moment fence validation, narrowly scoped credentials, expected-state writes, remote read-back, and effect receipts.
5. **Evidence and audit plane** — proof bundles, content-addressed artifacts, causal events, evidence invalidation, retention, and tamper-evident audit history.

The paper also introduces or formalizes:

* permanent, never-reused Variant fence epochs;
* a complete immutable Authority Token;
* private writable clones rather than shared writable worktrees;
* Candidate ancestry and rebase invalidation;
* Evidence trust levels E0–E4;
* four-valued observation rather than optimistic liveness inference;
* successor-Attempt scope amendment;
* exact logical-effect identity over at-least-once transport;
* `OUTCOME_UNKNOWN` handling for ambiguous remote writes;
* provider capability negotiation and conformance;
* quota as a multi-window vector;
* deterministic tools and economy-model tiers before frontier escalation;
* blind, multidimensional review independence;
* protected merge-group verification;
* post-integration survival as part of completion;
* explicit falsification conditions under which the design must be rejected.

## Quality and build verification

The final reviewer-style internal score was **92/100** for a systems architecture and vision paper. The strongest dimensions were:

| Dimension                            | Assessment                         |
| ------------------------------------ | ---------------------------------- |
| Problem significance and novelty     | Excellent                          |
| Current harness survey               | Strong and primary-source grounded |
| Distributed-systems correctness      | Strong                             |
| Architecture completeness            | Strong                             |
| Security and trust boundaries        | Strong                             |
| Comparative rigor                    | Strong                             |
| Falsifiability and evaluation design | Strong                             |
| Limitations and claim discipline     | Excellent                          |
| IEEE writing and presentation        | Strong                             |
| Reference quality                    | Strong                             |

The principal deduction is explicit in the paper: **Bullet Farm has not yet been implemented or experimentally validated**. Consequently, the paper does not claim measured superiority, zero regressions, perfect autonomy, or exactly-once physical side effects. It instead defines a stronger systems contract and a reproducible program capable of validating—or falsifying—that contract.

Final artifact checks included:

* successful IEEEtran compilation;
* no undefined references or citations;
* 11 pages on US Letter;
* 55 bibliography entries;
* embedded and subsetted Type 1 fonts;
* PDF title, author, subject, and keyword metadata;
* render inspection of all 11 pages;
* no visible clipping, overlapping elements, black glyph boxes, or broken equations;
* validated wide tables and the five-plane architecture figure;
* PDF preflight confirming a readable, unencrypted, non-scanned document.

### Historically recorded hashes

```text
PDF:
dcbe7afad7320e88cd4e5d3a9e29aef564506eeaa3586613be9dd7680b5150fb

LaTeX:
9461662526a683ea335d80a92eccc044e6515365e81f1cc260d3ceec18341bb7

BibTeX:
aff2d8d1de0bc7e2d11dbe8be0c415ba68181c86b6eef442faf7930edaa6471c

ZIP:
423b9abcd0d562d160a5def3cfbe4f33075ba51a34600c5e2ed37ac604eb68c3
```

[1]: https://developers.openai.com/codex/cli "https://developers.openai.com/codex/cli"
