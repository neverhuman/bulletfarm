# BulletFarm acceptance status on merged `main`

Status date: 2026-09-18. Evidence baseline:
`neverhuman/bulletfarm@1ea7aa60f536e3f3fe8ca186acd64b8433acc294` (the reviewed,
rebase-merged result of PR #1). This ledger applies the acceptance text in the
[canonical specification](spec/BULLETFARM_FINAL_ENGINEERING_SPEC.md) to code and tests on that
exact baseline. The documentation PR that adds this file changes no runtime source.

`D` means the complete criterion is implemented and a current-main test exercises it. `P` means
some reusable behavior exists but the criterion is not satisfied. `M` means the required behavior
or proof is missing. There are no `D` criteria yet. A package cannot be called complete until all
three of its criteria are `D` and every linked AT/HF/CF scenario passes against one immutable
candidate. File and line references below are anchors on the evidence baseline; an absence is
bounded by the complete active module list, route list or schema cited in that row.

## Per-criterion ledger

| Package | AC01 | AC02 | AC03 | Evidence on `main` and remaining requirement |
| --- | :---: | :---: | :---: | --- |
| BF3-001 | M | P | P | The check entry point builds and tests but has no known-good/injected-defect behavioral oracle ([`scripts/check:11`](../scripts/check#L11)). HOLD and the gap inventory keep unresolved authority non-live; the import report records reusable files, but neither behavior has a current acceptance test. |
| BF3-002 | P | M | P | Strict JSON rejects duplicate keys ([`src/digest.rs:15`](../src/digest.rs#L15)); commands reject missing IDs and wrong versions and hash exact request bytes ([`src/commands.rs:11`](../src/commands.rs#L11)). Canonical schemas, cycles, mandatory-check validation and role-specific job variants are absent from the active modules ([`src/lib.rs:1`](../src/lib.rs#L1)). |
| BF3-003 | M | P | M | Same ID/body replay and changed-body conflict exist for receipt-only commands ([`src/commands.rs:33`](../src/commands.rs#L33)), with a current HTTP replay test ([`tests/core.rs:29`](../tests/core.rs#L29)). The four-table schema has no audit, outbox, jobs, candidates or evidence, so atomic state/outbox rollback and structural immutability are missing ([`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)). |
| BF3-004 | M | P | P | Loopback host/origin/bearer checks, private bootstrap, revocation and actor-scoped operation lookup exist ([`src/api.rs:49`](../src/api.rs#L49), [`src/commands.rs:84`](../src/commands.rs#L84)). Membership, capabilities, current-access checks for repository artifacts, runner enrollment and the human-only authority model are absent from the schema ([`migrations/001_core.sql:4`](../migrations/001_core.sql#L4)). |
| BF3-005 | M | M | M | The complete active schema contains no grants, holds, limits, usage or completion reserve ([`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)). |
| BF3-006 | M | M | M | The active module list contains no plan/task graph or Git metadata activation authority ([`src/lib.rs:1`](../src/lib.rs#L1)); readiness, known-revert handling and expected-old ref updates are unimplemented. |
| BF3-007 | M | M | M | Commands deliberately return `NOT_IMPLEMENTED` rather than dispatching a bounded adapter ([`src/commands.rs:58`](../src/commands.rs#L58)); no deterministic provider/forge adapter or framed job fixture is active ([`src/lib.rs:1`](../src/lib.rs#L1)). |
| BF3-008 | M | P | P | Human stop signals one process group and reports an unknown survivor ([`src/runner.rs:183`](../src/runner.rs#L183)). It does not prove filesystem/network/credential isolation, exact sandbox incarnation, descendant cleanup or native-daemon containment. |
| BF3-009 | P | M | M | The separate coordination board provides tested overlap rejection and release ([`tests/cli_board.rs:75`](../tests/cli_board.rs#L75)), but it writes `bf.sqlite` directly ([`src/board.rs:692`](../src/board.rs#L692)). There is no transactional all-resource allocator, pending-change/occupancy separation, generation fencing or all-job recovery. |
| BF3-010 | M | M | M | `bf run` validates a provider name and then exits as unimplemented ([`src/main.rs:259`](../src/main.rs#L259)); no transport/profile has conformance, freshness, cancellation, accounting or credential-boundary proof. Live use also remains under HOLD. |
| BF3-011 | M | M | M | The four-table schema has no Candidate, check obligation, evidence or verifier job relations ([`migrations/001_core.sql:16`](../migrations/001_core.sql#L16)); author-independent verification and prepublication/CI obligation separation are absent. |
| BF3-012 | M | M | M | PR code performs read-only `gh pr list` observation ([`src/prs.rs:134`](../src/prs.rs#L134)); no privileged safe importer, durable publication outbox, lost-response reconciliation or remote-head/human-edit protection exists. |
| BF3-013 | M | M | M | The React page labels a demo and fake PRs ([`web/src/App.tsx:112`](../web/src/App.tsx#L112)), but its project/draft/work routes are not registered by the server ([`src/api.rs:34`](../src/api.rs#L34)). No fixture journey or installed live recovery demonstration completes. |
| BF3-014 | M | P | P | Board claims reject path overlap while disjoint paths can proceed, and PRs are observed across board repositories ([`src/prs.rs:89`](../src/prs.rs#L89)). Applicable cross-domain grants, accountable scheduler waits, owner lanes, fairness and external-work reconciliation remain absent. |
| BF3-015 | P | M | M | A human-only stop gate and recorded process-group stop exist ([`src/main.rs:245`](../src/main.rs#L245)), but there is no old-writer fencing with exact checkpoint, common return verification, takeover generation or respecification allowance. |
| BF3-016 | M | M | M | Discovery labels four local CLI families, but no coding-provider adapter is qualified at all ([`src/agents.rs:49`](../src/agents.rs#L49)); exact-checkpoint restart, second-provider policy and harness-family independence are missing. |
| BF3-017 | M | M | M | The web page renders draft/planning controls, but calls routes the server does not expose ([`web/src/App.tsx:45`](../web/src/App.tsx#L45), [`src/api.rs:34`](../src/api.rs#L34)). There are no persistent conversations, mission jobs, bounded planning modes, acceptance mapping or memory authority. |
| BF3-018 | P | P | P | The embedded React shell has keyboard controls, status and reconnect UI ([`web/src/App.tsx:116`](../web/src/App.tsx#L116); the API bounds blocking calls with 32 slots ([`src/api.rs:77`](../src/api.rs#L77)). Its projects/drafts/work/events reads return the fallback, expected-version commands are not enforced and CLI/TUI still bypass the hub, so no criterion is complete. |
| BF3-019 | M | M | M | There is no integration module or schema state beyond the skeleton list and four tables ([`src/lib.rs:1`](../src/lib.rs#L1), [`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)); issuer/target validity, native merge capability and known-revert invalidation are absent. |
| BF3-020 | M | M | M | The active schema records no usage, metering, costs, attention windows or routing calibration ([`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)). |
| BF3-021 | M | P | P | Repository migration preservation records unavailable material and provenance outside the runtime, but product restore/retention is absent from the active schema ([`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)). No paused restore, publisher revocation/rotation, safe cleanup or portable controller export has a current test. |
| BF3-022 | M | M | M | Current CI runs the ordinary single-candidate check entry point ([`.github/workflows/ci.yml:25`](../.github/workflows/ci.yml#L25)); there is no two-owner/two-runner/two-provider fault campaign, independent rebuild package or representative pilot. |
| BF3-023 | M | M | M | The active module/schema surface has no shadow, learning, promotion, lesson or rollback state ([`src/lib.rs:1`](../src/lib.rs#L1), [`migrations/001_core.sql:1`](../migrations/001_core.sql#L1)). |
| BF3-024 | M | M | M | Grok is only discovered/probed as a local process/tool ([`src/main.rs:321`](../src/main.rs#L321)); Grok Build has no executor adapter or qualification evidence. |
| BF3-025 | M | M | M | There is no production pipeline integration, artifact/environment approval binding, credential separation or observed-versus-triggered deployment state in the active modules ([`src/lib.rs:1`](../src/lib.rs#L1)). |
| BF3-026 | M | M | M | There is no Slack transport, signature/replay identity mapping or thin-client authority path in the active modules ([`src/lib.rs:1`](../src/lib.rs#L1)). |
| BF3-027 | M | M | M | There is no Grok Bot or SMS capability/notification adapter in the active modules ([`src/lib.rs:1`](../src/lib.rs#L1)); unsupported API and approval-disclosure behavior have not been evaluated. |
| BF3-028 | M | M | M | OpenJarvis is absent from the active package dependencies ([`Cargo.toml:11`](../Cargo.toml#L11)); compile, dependency, cancellation, stream, timeout quality and cost evidence have not been collected. |
| BF3-029 | M | M | M | No measured architecture hypothesis, experiment budget, federation prototype or retain/reject decision exists; the active package remains one small monolith ([`src/lib.rs:1`](../src/lib.rs#L1)). |
| BF3-030 | M | M | M | Current forge support is read-only PR observation ([`src/prs.rs:134`](../src/prs.rs#L134)); no evidence-selected forge-native change, maintainer/conformance receipt or compatibility proof exists. |

## Named follow-ups and owner state

This is the issue queue while GitHub issues remain empty. “Proposed” records the board handoff; it
does not create a claim. Every owner must acquire a non-overlapping board claim before writing.

| Follow-up | BF3 scope | Current evidence/gap | Owner and ordering |
| --- | --- | --- | --- |
| Command/package/installer identity | Consolidation before G0 | Cargo package and binary remain `bf` ([`Cargo.toml:1`](../Cargo.toml#L1)); installed `bulletfarm` still launches the archived product. | **Proposed: `codex-consolidation`**. First runtime PR after this documentation PR merges. |
| `--data-dir` and alias equivalence | Consolidation/BF3-004 | CLI and spawned hub pass one directory ([`src/main.rs:124`](../src/main.rs#L124), [`src/main.rs:332`](../src/main.rs#L332)), while board helpers read `BF_DATA_DIR` independently ([`src/board.rs:698`](../src/board.rs#L698)). | **Proposed: `codex-consolidation`**, in the same command-identity PR; test both aliases against one temporary and preserved data directory. |
| Retired-repository PR deduplication | Consolidation/BF3-014 | PR discovery uses each observed origin verbatim ([`src/prs.rs:89`](../src/prs.rs#L89)). | **Proposed: `codex-consolidation`**, in command-identity PR; map exactly five retired identities and preserve unrelated ones. |
| `bf run` bounded adapter | BF3-007–010 | Explicit stub at [`src/main.rs:259`](../src/main.rs#L259). | **Proposed: `grok-4.6`**, only after command-identity PR merges and a new claim is acquired; fixtures and secret-free qualification precede any HOLD-gated live call. |
| React project/draft/work/event service | BF3-004, BF3-017–018 | Calls at [`web/src/App.tsx:45`](../web/src/App.tsx#L45) have no routes in [`src/api.rs:34`](../src/api.rs#L34). | **Owner role: next BF3-017/018 integrator; currently unassigned.** Start only after BF3-002–004 authority/contracts land. |
| Codex session over-count and wrapper/child dedup | BF3-014, BF3-018 | Discovery begins PID-keyed ([`src/agents.rs:49`](../src/agents.rs#L49)); its stress test currently accepts 200 Codex PIDs as 200 agents ([`tests/cli_agents.rs:132`](../tests/cli_agents.rs#L132)). | **Owner role: next discovery integrator; currently unassigned.** Bind a logical session to provider session ID plus process-start identity and retain a bounded observation set. |
| Transcript path coverage | BF3-014, BF3-018 | Claude registry paths receive special resolution ([`src/live.rs:335`](../src/live.rs#L335)); four payload shapes are parsed ([`src/live.rs:389`](../src/live.rs#L389)). Stale paths, PID reuse and every provider's real path rules lack end-to-end coverage. | **Owner role: next discovery integrator; currently unassigned.** Pair with session-identity work so paths cannot attach to a reused PID. |
| First PR-refresh timeout settling | BF3-018 | A first timeout returns before setting `prs_settled`, causing repeated blocking waits ([`src/live.rs:185`](../src/live.rs#L185)). | **Owner role: next observation integrator; currently unassigned.** Add a timing test and settle after the first bounded wait. |
| One hub/database migration | BF3-003–004, BF3-009, BF3-021 | Board writes `bf.sqlite` ([`src/board.rs:692`](../src/board.rs#L692)); hub opens its own store. | **Owner role: G0 storage integrator; currently unassigned.** Fingerprint/back up/import paused, then fence the old writer; never silently reuse migration number 1. |

The remaining package work is assigned by role and dependency in the
[complete closure plan](closure-plan.md). Keeping a role explicitly unassigned is a visible blocker,
not permission to create a parallel PR. The active PR queue must return to zero after each slice.
