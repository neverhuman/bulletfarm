# Bullet Farm — family root

This directory is the split-family **container** (not a Git repository). It holds four independent
checkouts, the coordination log, and the planning corpus. Nothing here is runtime authority; the
authoritative answers live inside the repositories and are indexed by
[`bullet-farm/docs/README.md`](bullet-farm/docs/README.md).

**Public index:** [https://github.com/neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm)
(not `neverhuman/bullet-farm`). That GitHub repository publishes the hub checkout only. It is a
discovery/PR mirror, not source authority, and it does not make a trusted family install.

| Path | What it is | Authority |
| --- | --- | --- |
| `bullet-farm/` | Public hub: installer/setup, family lock, contracts (`bullet-wire`), policy, CI composition, release checks | product (Rust) |
| `bullet-kernel/` | Control plane: ledger, leases/fences, admission, runner/verifier/effects boundaries, farmd | product (Rust) |
| `bullet-git/` | BulletGit: private clones, journal/CAS, generations, Candidates, `bullet-gitd` | product (Rust) |
| `bullet-portal/` | Operations portal (Vite/React), projection-only | product (TS) |
| `repos.manifest.toml` | Family membership; must never depend on sibling paths | product |
| `AGENTS.md` | Family rules (zero worktrees, Jeryu via pinned tags, coordination) | rules |
| `AGENT_CHAT.md` | Live append-only multi-agent coordination log (cut over 2026-08-25T09:29Z); machine claims live in `.bullet-family/coord/events.jsonl` via `bullet-family coord` | log |
| `AGENT_CHAT.archive/` | Frozen prior log (`2026-08-25T0929Z.md`, 19,707 lines). Do not rewrite | archive |
| `NEXT_EVOLUTION_PLAN.md` | Frozen scored G1–G15 campaign. Loses to the Hub closure roadmap and `check release` | historical planning |
| `.l7-bundle/` | Archived audit bundle and the 2026-08-24T14:04Z admitted live demo receipt | evidence archive |

## Which document answers what

| Question | Read |
| --- | --- |
| How done is the first GA profile, exactly, with receipts? | `bullet-farm/docs/assurance/closure-roadmap.md`, then `bullet-family check release --profile self-hosted-v1 --receipts <absolute-registry> --json` (27 selected gates) |
| What is the plan to ship, and what can only the operator do? | `bullet-farm/docs/assurance/closure-roadmap.md` (Waves 0–11) and `bullet-farm/docs/decisions/0013-operator-decision-register.md` |
| What is the historical scoring methodology behind the closure work? | `bullet-farm/docs/assurance/path-to-100.md` — frozen scoring snapshot; use `closure-roadmap.md` for current execution order |
| What is still blocked for a release, and why? | `bullet-farm/docs/release.md` |
| What does "evolutionary multi-agent" mean here, normatively? | `bullet-farm/docs/architecture/evolutionary-control.md` |
| Why those choices — method rationale, role catalogue, fitness/selection design, work items, operator decisions? | `TEAM_PLAN_CLAUDE.md` (planning artifact; §10 is the 2026-08-25 addendum and scorecard) |
| Which controls are enforced vs planned? | `bullet-farm/docs/assurance/invariant-registry.md` + generated crosswalk |
| How do agents coordinate? | `bullet-farm/docs/runbooks/fleet.md` (claims, heartbeats, handoffs, receipts) |
| How do I set up from source, and why is a hub-only install refused today? | `bullet-farm/docs/runbooks/source-setup.md` |

## Planning corpus at this root (provenance, not authority)

| File | Role | Status | sha256 (first 16) |
| --- | --- | --- | --- |
| `TEAM.md` | Red-team of the Centerrail spec; C1–C12 hardenings; V1 boundary | historical input; contains ANSI corruption and a disproved Gas Town #742 / “687 respawns” attribution—use the corrected R3 paper for incident claims | `013f19032017ea5e` |
| `TEAM_EVOLUTION_PLAN.md` | codex-plan: TeamRecipe genome, MAP-Elites/ASHA/islands, Gate 0 / Phase 1C | superseded by `evolutionary-control.md` + `TEAM_PLAN_CLAUDE.md` | `354311fb02850296` |
| `TEAM_GAP_CLOSURE_PLAN.md` | codex-plan gap-closure program (2026-08-24 12:36Z) | superseded by the Hub `closure-roadmap.md` | `50ba29aa97ff06f0` |
| `TEAM_PLAN_CLAUDE.md` | claude-orch synthesis: roles, evolutionary loop, work items, exit gates, operator decisions, 2026-08-25 addendum | planning provenance; loses to the Hub `closure-roadmap.md` | (changes with edits) |
| `NEXT_EVOLUTION_PLAN.md` | 2026-08-25 scored closure campaign; one upstream Jeryu source plus signed runtime; GitHub/GitLab adapters | frozen planning provenance; loses to the Hub `closure-roadmap.md` | (changes with edits) |
| `docs/CENTERRAIL_FINAL_ADAPTIVE_MULTI_FRONTIER_ENGINEERING_SPEC.md` | the original spec | historical provenance (also hashed under `bullet-farm/docs/spec/`) | `4356a1338989987f` |
| `docs/GASTOWN_OPEN_ISSUES_RISK_AUDIT_FOR_CENTERRAIL.md` | Gas Town risk audit R1–R38 | historical provenance | `615117b7aa47359f` |
| `docs/git_role.md` | BulletGit capability-secure repository design | historical provenance | `bc517cec12584c47` |
| `docs/POTENTIAL_DRAFT.md` | adjudicated red-team additions A1–A7 and the doctrine layer | input adopted into `TEAM_PLAN_CLAUDE.md` §10.3; mirrored + hashed under `bullet-farm/docs/spec/` (2026-08-25) | `cdb9e8972fd0df06` |
| `docs/nightshift.md` | release-truth UX principle ("display the exact claim that remains unproved") | input adopted into `TEAM_PLAN_CLAUDE.md` §10.3 (WI-35, shipped as `bullet-farm/docs/assurance/release-truth.generated.md`); mirrored + hashed under `bullet-farm/docs/spec/` (2026-08-25) | `c92ca3ff3fd5a79d` |
| `docs/paper.md` | historical paper summary | byte-identical to the sanitized hub copy `bullet-farm/docs/spec/paper.md` since 2026-08-25 (the four machine-local links and the unhedged completion claim were removed); the current preprint is `bullet-farm/docs/paper/` | `1e1a8bd8300a688f` |

## Proof from the hub

```bash
cd bullet-farm
cargo run --locked --quiet --bin bullet-family -- doctor --json   # BLOCKED is honest today
just fast && just contract && just check-family
bullet-family check release --profile self-hosted-v1 \
  --receipts /absolute/admitted-registry --json                    # exit 3, 27 gates, until receipts exist
```

## Open operator decisions

The single register is `bullet-farm/docs/decisions/0013-operator-decision-register.md` (supersedes the
lists formerly kept here, in `docs/assurance/product-gaps.md`, and in `TEAM_PLAN_CLAUDE.md` §10.5).
Headline items: (1) ratify policy generation 2 with a `provider-runner` authority key — procedure in
`bullet-farm/docs/runbooks/live-conformance.md`; (2) after Waves 4 and 5, approve one operator-named
immutable Jeryu build and protected test repository with distinct short-lived broker, attestor,
integrator, and observer custody as specified by ADR 0013; (3) a GitHub App test repository; and
(4) signed schema-3 lock inputs and release-signing custody. The register, not this index, owns the
credential procedure and current decision status.
