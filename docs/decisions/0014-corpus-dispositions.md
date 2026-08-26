# ADR 0014 — Corpus dispositions: what "addressed" means for the historical vision

Status: **PROPOSED** (operator ruling recorded 2026-08-25; register maintained by the generated
corpus-coverage instrument; not runtime, release, or scoring authority)
Applies to: `policy/corpus-coverage-v1.json`, `docs/assurance/corpus-coverage.generated.md`,
`scripts/corpus-coverage.sh`, `src/check/corpus/`.

## Decision

The historical vision corpus — the Centerrail spec, `git_role.md`, the Gas Town risk audit,
`nightshift.md`, `POTENTIAL_DRAFT.md`, the paper, and `architecture/evolutionary-control.md` — is
provenance, not authority (`docs/spec/README.md`). It nevertheless states requirements the family
has either built, planned, replaced, or rejected. To make "the whole vision is addressed" a checkable
claim instead of a sentiment, **every normative unit of the corpus carries exactly one disposition**
in `policy/corpus-coverage-v1.json`:

| Disposition | Meaning | Required anchor |
| --- | --- | --- |
| `IMPLEMENTED` | code exists and a named test in a proof lane proves it | a test (or symbol) that resolves at HEAD |
| `PLANNED` | not, or not fully, implemented; owned by a closure-roadmap wave | `W0`…`W11` heading in `docs/assurance/closure-roadmap.md` |
| `SUPERSEDED` | the family replaced the mechanism by a reviewed decision | an ADR file, and a row in the register below |
| `REFUSED` | the family rejects the mechanism by a reviewed decision | an ADR file, and a row in the register below |

Two numbers are reported and never conflated: **addressed** (units with any disposition — 100 % by
construction once the policy validates) and **implemented** (units at `IMPLEMENTED`). Nothing here
moves a release gate, a scorecard row, or a profile.

## Rules

1. A `SUPERSEDED` or `REFUSED` row is valid only if its id is listed in the register below with the
   ruling ADR; the instrument fails closed otherwise (`CORPUS_COVERAGE_ANCHOR`).
2. `IMPLEMENTED` requires an anchor that resolves in the named checkout; an absent sibling checkout
   makes the anchor *unverifiable*, never resolved.
3. A disposition changes only with the anchor that justifies it; the generated page is a
   drift-gated zone (`scripts/corpus-coverage.sh check`).
4. Refusal reasons come from existing decisions: ADR 0001 (read-only providers; the kernel applies
   patches), ADR 0002 (Jeryu forge requirements), the family zero-worktree rule (`AGENTS.md`), and
   spec §45 (V1 boundary). A refusal with no ruling decision is not admissible.

## Register of superseded and refused units

Maintained alongside the policy; ids are quoted exactly so the instrument can check them.

| Unit id | Disposition | Ruling | Reason |
| --- | --- | --- | --- |
| `spec.s18.13.live-portal-terminal` | REFUSED | `0001-provider-execution-mode.md` | Portal is projection-only; interactive/tmux/PTY agent driving refused family-wide (AGENTS.md zero-worktree rule) |
| `spec.s18.4.pty-mirror` | SUPERSEDED | `0001-provider-execution-mode.md` | Providers run read-only structured turns; no PTY plane exists in the family |
| `spec.s18.5.pty-authority-rule` | SUPERSEDED | `0001-provider-execution-mode.md` | Moot without a PTY plane; structured transcripts are the only session evidence |
| `spec.s18.6.dialog-controller` | SUPERSEDED | `0001-provider-execution-mode.md` | Screen-recognition automation replaced by structured permission/plan events (harness-sim models them as typed) |
| `spec.s21.1.tool-request` | SUPERSEDED | `0001-provider-execution-mode.md` | Interactive agent tool gateway replaced by read-only turns plus typed PatchProposal |
| `spec.s21.2.policy-decision` | SUPERSEDED | `0001-provider-execution-mode.md` | No agent tool execution to police; scope is enforced on the proposal by BulletGit |
| `spec.s21.3.command-parsing` | SUPERSEDED | `0001-provider-execution-mode.md` | Gates are immutable argv vectors (ADR 0007); no shell text is ever parsed or executed |
| `spec.s21.5.tool-receipts` | SUPERSEDED | `0001-provider-execution-mode.md` | Agent tool receipts replaced by gate receipts with framed argv digests (verifier aggregate) |
| `spec.s31.kernel-crates` | SUPERSEDED | `0014-corpus-dispositions.md` | Split into four repos (kernel 27 crates, git 4, hub bullet-wire, portal); ops/ci shell lanes are gates, not control |
| `spec.s33.8.pty-golden-tests` | REFUSED | `0001-provider-execution-mode.md` | Providers run structured read-only turns; no PTY driving or screen recognition |
| `spec.s35.e10` | SUPERSEDED | `0001-provider-execution-mode.md` | Providers get no tool authority; bullet-mcpd is a read-only farmd query server, not a gateway |
| `spec.s35.e16` | SUPERSEDED | `0001-provider-execution-mode.md` | Bounded read-only turns replace supervised interactive sessions; attach/peek may return as observation in W9 |
| `spec.s40.9.behavior-enforcement` | SUPERSEDED | `0001-provider-execution-mode.md` | No tool requests reach the kernel; behavior is evaluated on proposals and candidates (detector scaffold) |
| `spec.s43.3.trust-boundaries` | SUPERSEDED | `0003-five-trust-planes.md` | Five trust planes with principal separation |
| `spec.s45.v1-boundary` | SUPERSEDED | `0014-corpus-dispositions.md` | Profile ladder: self-hosted-v1 (Ubuntu, Claude, Jeryu) then evolution, provider/forge/platform slices, universal, team, saga |
| `spec.s48.6.live-terminal` | SUPERSEDED | `0001-provider-execution-mode.md` | No PTY sessions; observation-only attach/peek is a W9 item |
| `spec.s5.seven-planes` | SUPERSEDED | `0003-five-trust-planes.md` | Seven functional planes retained; transaction authority collapsed to five domains |
| `gastown.s10.do-not-port` | REFUSED | `0001-provider-execution-mode.md` | tmux/worktree/PTY driving refused family-wide; ADR 0001 argv admission denies worktree and tmux tokens. |
| `gastown.s13.issue-index` | SUPERSEDED | `0014-corpus-dispositions.md` | Replaced by pinned competitor-snapshot.md subjects; comparison claims forbidden until matched corpus. |
| `potential.c8.any-two-providers` | SUPERSEDED | `0001-provider-execution-mode.md` | universal-v1 requires all four providers; no receipt substitutes for another. |
| `potential.s4.6.five-screen-cockpit` | SUPERSEDED | `0014-corpus-dispositions.md` | Portal defines fifteen surfaces (projections::surfaces); five-screen cockpit replaced. |
| `potential.s4.8.build-plan` | SUPERSEDED | `0014-corpus-dispositions.md` | Replaced by closure-roadmap Waves 0-11 and explicit-profile release check. |
| `paper.hashes.historical` | SUPERSEDED | `0014-corpus-dispositions.md` | Hashes are historical metadata; HISTORICAL_ARTIFACTS.sha256 binds Markdown only. |

## Independent audit — 2026-08-26

Method: every `IMPLEMENTED` row (163 at hub `8590f0c`) was audited read-only by a Codex-family model
(codex-cli 0.149.1, `exec --sandbox read-only`, judging from code and tests only, planning documents withheld;
seed sample 40 rows with `random.seed(20260826)`, then the remaining 123 in three batches). Verdicts:
AGREE 33, PARTIAL 125, DISAGREE 5, UNVERIFIABLE 0.
Rule applied: the lower disposition stands — every PARTIAL/DISAGREE/UNVERIFIABLE row became `PLANNED`, its test
retained as the `partial` anchor, its wave taken from the nearest PLANNED rows of the same section, and the
auditor's reason recorded in the row's `note`. 130 rows moved; 33 remain `IMPLEMENTED`.
Nobody re-scored their own rows: the seed authors were Claude-family agents, the auditor was not.

## Consequences

- "100 % of `docs/*.md` addressed" becomes `bullet-family` output, not prose.
- The page can never show more implemented than the tests prove; it can show less.
- Units the roadmap does not own yet fail validation until a wave or a ruling is named — that is
  the intended pressure.
