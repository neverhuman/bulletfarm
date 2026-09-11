# Deep product, documentation, CI and media audit — 11 September 2026

Status: **audit and repair input; no release admission**  
Owner: Bullet Farm maintainers  
Observed: 2026-09-11 UTC

The published TUI GIF is almost entirely blank. The four member repositories have
green required checks on their recorded remote mains, but **all CI is not passing**:
scheduled jobs fail or have never run, and the public aggregate has no workflows.
The local source contains substantial conversation, authentication, terminal and
proof repairs. It still lacks the connected, installed Head-to-code transaction
needed for the requested production demonstration and daily CLI.

This audit supplies findings to the existing [G1–G18 register](product-gaps.md),
[full-product plan](full-product-dogfood-plan.md) and
[xbabe2 implementation order](xbabe2-development-closeout.md). It introduces no
completion dashboard, certification status, operator decision or permission to
lift Operating HOLD. Historical failures and refused requirements remain intact.

## Scope and method

The audit enumerates source Markdown with `rg` and Git, cross-checks the two
path sets, excludes ignored dependency/build caches, and records exact
bytes and SHA-256. The pre-edit baseline is **159 member Markdown files**, including
**107 files under member `docs/`**: Hub 112/92, Kernel 20/7, BulletGit 12/3 and
Portal 15/5, respectively. Kernel's count includes the untracked Tuiwright README.
All six additional family-root `docs/*.md` files are byte-identical to their Hub
`docs/spec/` counterparts; their equivalence is recorded explicitly. This replaces
the older 106-document enumeration as a dated observation, not a fixed CI constant.

Full-read completion, initial hashes and exact mirror relationships are recorded
in the [audit inventory](deep-audit-20260911.inventory.json). Large files were read
in bounded sections; truncated tool output was re-read before claiming coverage.
Coverage comprises 158 full-text member reads and one complete generated-corpus
typed-row comparison (all648 ordered fields/rows plus surrounding prose and
implemented claims). This is not independent acceptance of648 implementations. Source inspections targeted consequential capability
claims; this is not a claim to have manually reviewed every source line.

Three independent audit assignments covered Hub/Portal documentation,
Kernel/BulletGit documentation and implementation, and hosted CI. The integrator
inspected the actual public GIF, original terminal stream, capture implementation,
installed executable discovery, and reconciled results. GitHub reads used exact
API subjects and job logs. No provider, signing, administrative, merge or
publication action was performed by this audit. During reading, another agent
switched the Hub checkout to its media branch and restored it. Four documentation
hashes changed and returned; subsequent branch changes and a fifth changed
README are recorded in the completed audit inventory. Reads bind the enumerated
baseline, with exact later proposals identified separately. This audit does not claim a continuous source freeze.

The recorded requirement corpus remains **648 units: 33 implemented, 592 planned,
20 superseded and three refused**. These are existing dispositions, not a product
completion percentage. Newly added requirements need forward/reverse mapping;
old superseded/refused rows and retired WP-18 cannot disappear during regeneration.

## Source and evidence separation

| Member | Local audited HEAD | Local tree | Qualification boundary |
| --- | --- | --- | --- |
| Hub | `f43181f5f34a5c2f327d51cb893daa5fd8579e0b` | `107c9caac6b31defc8a77e0478211095d79baf09` | Full required passed here; later two-file UI dispatcher work has only focused tests/review. |
| Kernel | `899f878fc914533d4f5d44f5b67c6e4c462c3413` | `340228b5d765987110828e990222e8753f33c9d0` | Full required passed; new untracked Tuiwright workspace has separate component evidence. |
| BulletGit | `4e33103673f535fde871ef26cffe05460674293b` | `4f64a7e9a01684ebc726319497de4472dd11f5a3` | Existing remote required green; scheduled native/auditor/link failures remain. |
| Portal | `d00c88ae1122c7cbad13ff7228bdf9a24424d9a9` | `2d694b3575fa7b889f347394034210cb489b6e2a` | Actual admitted full required failed in lint after fast and build passed. |

Local Hub required r2 completed 600 fast and 219 contract identities, with all
mapped lanes. Kernel required r3 completed 1,256 fast and 34 contract identities,
with all mapped lanes. Historical receipts copied into an artifact directory
are not automatically evidence for either current source. These observations do
not qualify the new documentation commit or later dirty/untracked source. The
original Kernel required-r1 fast XML was overwritten before preservation; its
console and source archive survive. Retain this loss explicitly and rerun any
qualification that needs the missing artifact.

Portal's actual monitored required run completed 360 fast tests and the production
build, then failed at `ops/ci/source-custody-test.mjs:174` with status 75 where the
fixture expected zero. Contract, security and docs did not execute. The nested
fixture's own proof completed successfully; its historical verifier inherited
the outer `BULLET_CI_SOURCE_SESSION`, which belongs to another checkout. Static
call-chain review identifies the foreign-session refusal as the cause; the old
assertion discarded stderr, so no missing diagnostic is invented. Repair fixture
environment separation and retain diagnostics while preserving production
foreign-session refusal. Then repeat the entire admitted proof on the new source.

An earlier component regression made the Linux monitor detect a real write-and-restore
during compilation despite equal ending bytes and compiler exit zero. The later
admitted Portal required run failed before reaching that regression; it retained
continuous monitor events, unchanged input controls and final process/lock
read-back. This bounded Linux proof does not supply hosted or native-platform
admission automatically.

Actual Tuiwright component execution completed **12 selected/12 passed**, with
75 artifacts and 354 ordered events. It exercised real Bullet PTYs, six-client
blocked-startup cases, navigation/resize/detach and process/terminal custody.
Eight admission negatives also passed. Installed mode still refuses with exit
78. It does not establish signed installation, real provider use, all terminal
suites, p95 latency, accessibility or media qualification. The Kernel CI route
and Rust result validator are still missing; the Hub dispatcher correctly refuses
when that route is absent.

## Why the published TUI GIF is white

The exact public asset at aggregate
[`3b7b474b163b513bdc94a6e8a026a392dcad3aa8`](https://github.com/neverhuman/bulletfarm/commit/3b7b474b163b513bdc94a6e8a026a392dcad3aa8)
was downloaded and compared with the local file. Both hashes are
`e1c3b09627e6c05893137e26ce7f6e2d78391c071a3bff444a664844b7bfb557`.

| Observation | Result and implication |
| --- | --- |
| Actual encoded file | 11,610 bytes, 1920×1080 logical screen, three frames, 27.77 seconds. The URL works; the recorded content is defective. |
| Decoded frames | First two are blank except a cursor; the last contains only the detach message. At least 99.80% of every frame is RGB `236,239,244` (`#eceff4`). |
| Original cast | 153 output events, 2,987 decoded output bytes; no CONNECTING/HOLD/LIVE/Mission/Control Tower labels and no truecolor controls. Missing panes are already absent before GIF encoding. |
| Nested PTY | `scripts/media/narrate-operator-tui.py` forks a new PTY but never sets its window size or propagates resize. A reproduction of that same primitive observed 0 columns × 0 lines. |
| Theme/geometry | `record-tui.sh` and the renderer select `github-light`; the recorded 1913×1078 raster is expanded to a 1920×1080 logical screen. This is not the requested native capture geometry. |
| Activity provenance | Existing manifest identifies provider `none`, runtime `0.0.0`, no Candidate and no release admission. It cannot demonstrate Codex/Claude/Cursor coding. |
| Existing checker | Actual `readme-real-check.sh` invocation returned PASS for this blank asset. It checks structural/provenance properties without proving visible product behavior. Its `>` byte comparison also incorrectly permits exactly 50,000,000 bytes. |

The 0×0 nested PTY is a source-backed causal diagnosis supported by a present
reproduction and the original stream. Historical inner-PTY dimensions were not
recorded, so it is not a direct measurement of the old process. Merely changing
the palette cannot recover missing terminal output. The narrator also ignores
the child's wait status and uses timed keys instead of visible-state assertions;
its final wait is not bounded by the earlier loop deadline.

Preserve the broken GIF, cast, manifests and checker PASS as regression evidence.
Repair capture in tracked Rust supervision, set/verify PTY geometry before exec,
propagate resize, require actual screen and durable-state observations, preserve
exit/termination custody, and reject blank/wrong-theme/wrong-subject exports.
Colorful synthetic fixtures can test capture; they cannot be the replacement
production demonstration. A concurrent [Hub PR11](https://github.com/neverhuman/bullet-farm/pull/11) at
`c58fc5b943e161120a667b8339e4943f8b86a1c5` proposes an outer-PTY/dark-theme/entropy
repair and a new HOLD-only clip. Review and reuse accepted repairs; it is not the
public aggregate asset above. Independent decoding confirms28 dark TUI frames,
11.49seconds and226,592bytes, but its manifest still records provider none, no
Candidate, dirty source and no release eligibility. It retains padded geometry.
Before integration, make scripted keys/Ctrl+C explicit to the operator story: the
new recorder currently injects them into every generic narration. Correct the
stale PR body and scope the truecolor syntax gate so it does not reject valid
monochrome product behavior. Syntax entropy alone does not verify visible content.
The paired full-recording/excerpt contract remains in
the [work order](xbabe2-development-closeout.md#real-1080p-capture-as-maintained-source).

## Every hosted CI lane and protection gap

The current YAML enumeration has **53 job definitions and 55 expanded cells**:
27 required-workflow cells and 28 scheduled cells. Required and scheduled are
different event sets. All 27 required cells pass on recorded remote member mains.
For scheduled cells on those mains, six pass, six fail and 16 are unexecuted
(eight Hub and eight Kernel). An older Hub manual run separately has three passes
and five failures. These counts must be regenerated when workflows change.

| Repository | Remote main | Current required result | Scheduled / protection |
| --- | --- | --- | --- |
| Aggregate | `3b7b474b163b513bdc94a6e8a026a392dcad3aa8` | No workflows/checks/runs | Unprotected main. |
| Hub | `4bfaefb2fcd1cc133c929a8a3f9a1ea392daa98d` | [success](https://github.com/neverhuman/bullet-farm/actions/runs/34540198530) | No current-head scheduled run; main unprotected. |
| Kernel | `bd2d7b70e76eaecbfd1e783b36d73a212969fc44` | [success](https://github.com/neverhuman/bullet-kernel/actions/runs/34525485342) | Scheduled workflow has zero runs; protection requires zero reviews. |
| BulletGit | `4e33103673f535fde871ef26cffe05460674293b` | [success](https://github.com/neverhuman/bullet-git/actions/runs/34478528041) | Four current scheduled failures; main unprotected. |
| Portal | `00b13bfa299a80f1b390505663eff7b4c6c35a8e` | [success](https://github.com/neverhuman/bullet-portal/actions/runs/34511594652) | Two current scheduled failures; protection requires zero reviews. |

All 11 latest failed scheduled job logs were fetched and inspected:

| Job | First failure | Required repair |
| --- | --- | --- |
| [Hub coverage, older head](https://github.com/neverhuman/bullet-farm/actions/runs/34512762593/job/102990830294) | Missing `BULLET_PUBLICATION_GITLEAKS`; 371 passed, one failed, 447 unexecuted. | Provision the admitted scanner for the actual selected canary; rerun the full unchanged inventory. |
| [Hub external links, older head](https://github.com/neverhuman/bullet-farm/actions/runs/34512762593/job/102990830400) | Installer does not support pinned Lychee 0.24.0 with fallback disabled. | Use a supported checksum-verified installation and execute all links. |
| [Hub Windows, older head](https://github.com/neverhuman/bullet-farm/actions/runs/34512762593/job/102990830590) | Checkout rejects `fixtures/hostile/cases/nul.json`. | Rename through its generator, preserving hostile bytes and refusal assertions. |
| [Hub macOS, older head](https://github.com/neverhuman/bullet-farm/actions/runs/34512762593/job/102990830618) | Python 3.12.12 unavailable for selected arm64 runner. | Admit an available exact runtime and prove the whole native lane. |
| [Hub auditor, older head](https://github.com/neverhuman/bullet-farm/actions/runs/34512762593/job/102990830772) | Auditor unavailable, exit 78; audit never runs. | Provision verified admitted Jankurai and execute committed policy. |
| [BulletGit Windows](https://github.com/neverhuman/bullet-git/actions/runs/34512765642/job/102990817254) | Custody refusal 75; precise failed predicate absent. | Native owner/ACL/file identity, exclusion and stale/race semantics with useful diagnostics. |
| [BulletGit links](https://github.com/neverhuman/bullet-git/actions/runs/34512765642/job/102990817284) | Wrong `bullet-farm/bullet-git` GitHub destination, HTTP 404. | Correct actual destinations and rerun the full inventory. |
| [BulletGit auditor](https://github.com/neverhuman/bullet-git/actions/runs/34512765642/job/102990817357) | Auditor unavailable, exit 78. | Provision and execute actual audit; unavailable remains non-passing. |
| [BulletGit macOS](https://github.com/neverhuman/bullet-git/actions/runs/34512765642/job/102990817449) | Bash 3 cannot execute dynamic-descriptor syntax. | Qualified Bash or equivalent compatible custody; prove BSD process/filesystem behavior. |
| [Portal coverage](https://github.com/neverhuman/bullet-portal/actions/runs/34512768413/job/102990721314) | Lines 82.82% below 88% floor. | Integrate meaningful production/failure tests and rerun current-source coverage without lowering the floor. |
| [Portal Windows](https://github.com/neverhuman/bullet-portal/actions/runs/34512768413/job/102990721852) | Custody refusal 75, then missing-artifact errors. | Equivalent native custody; retain primary failure and valid diagnostics before artifact staging. |

New local Portal code also requires source admission and a monitor/tool profile
that current hosted YAML does not provision. Its monitor is Linux-specific while
scheduled jobs still include macOS and Windows. These are static current-source
integration blockers, distinct from older observed hosted failures. The Portal
`history-links-audit` job runs npm audit, not the separately required Jankurai
audit; add actual auditor execution and regenerate the inventory.

All four member CI workflows already declare `merge_group`. There are no recorded
merge-group or cron executions in the available run inventories; three scheduled
workflow executions were manual. Preserve the actual event identity. GitHub's
[merge-queue contract](https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/configuring-pull-request-merges/managing-a-merge-queue)
requires the separate merge-group trigger; its presence alone does not qualify it.

Kernel/Portal enforce several useful protections but require zero approvals and
do not require approval of the latest push. Prepare exact administrative packets
for all five mains, including at least one independent review, latest substantive
change approval by another person, stale dismissal, conversation resolution,
administrator enforcement, force/deletion prevention and qualified queue behavior.
The historical Hub merge refusal still needs its stated approval mechanism.

Successful GitHub conclusions and artifact metadata were read; successful artifact
payloads were not independently revalidated in this audit. Complete CI acceptance
requires actual current-source execution and validated artifacts, including the
generated aggregate, rather than reusing this status snapshot as test evidence.

## Runtime gaps that block the daily product

These findings bind the local commits above. Concurrent unreviewed Kernel
task-admission changes began after the source reads. Kernel PR15 at9c589fa proposes
nonce/quota binding and v2 dispatch; stacked PR16 at95784a60 proposes worker changes.
They branch from older bd2d7b70 and omit root-only startup/authentication/alias/proof
repairs. Reconcile exact accepted source and complete independent review plus
crash/retry/full checks before crediting a combined implementation. These proposed
repairs are not the full Head, scheduler or native lifecycle.

| Boundary | Current fact | Work required |
| --- | --- | --- |
| Entry point | Clap requires a subcommand; `bulletfarm` target exists in source but is absent from PATH. | Equivalent default conversation plus explicit chat/setup/serve, stable piped/JSON behavior and installed alias. |
| Conversation | Exact durable message/journal/owner/cursor mechanisms exist; Head reads report runtime binding required. | One durable Head turn per conversation, pinned input/history/config/budget, streamed output and ambiguous-start recovery. |
| Runnable task | Accepted task storage reports `CODING_BINDING_ADMISSION_UNAVAILABLE`; dependency evidence unavailable. | Server-owned run/nonce/reservation/outbox and current dependency/capacity/budget admission. |
| Limits | Coding task validation allows up to 16 provider invocations. | Enforce accepted eight-call budget across Head, guidance, repair, escalation and provider/account switches. |
| Native execution | Generic signed-in adapter uses raw `Command::output`, inherited environment and unsupported lifecycle controls. | Pin real native protocol/runtime/account, bounded stream/process custody, effective model/effort and acknowledged control/recovery. |
| Stop | CLI reports `STOP_UNIMPLEMENTED`. | Queued cancellation, persistent STOPPING, observed termination, retained unknown liability; pause/freeze remain separate. |
| Finalization | Preservation and settlement primitives exist. | One atomic Candidate/Attempt/lease/reservation/audit/verifier-outbox transaction before cleanup. |
| Verification | Product verifier unconditionally exits 2 with admission unavailable. | Real signed-intent consumer, independent workcell/UID/keys/artifacts and exact gate execution. |
| Integration | Exact-subject and expected-old authority mechanisms exist. | Independent accepted evidence, exact approval, protected update and response-loss reconciliation/read-back. |
| Installation | Local executables exist; no matching service units were observed. | Signed profile package, immutable install, supervised separate identities, persistent state and full lifecycle qualification. |
| Team | PostgreSQL adapter is a DSN scaffold. | Authority implementation, remote workers/verifiers, fencing/partitions/failover before saga. |

The inspected generic adapter uses Codex read-only exec and Cursor text/plan mode;
Cursor does not receive its selected model argument. It reports no qualified native
version/session/turn, and interrupt/terminate/resume are unsupported. This audit
did not invoke it or establish public-task reachability. Installed signed-in
terminals and valid subscriptions do not qualify this missing Bullet execution path.

Current official interfaces reinforce the need for separate adapters:
[Codex App Server](https://learn.chatgpt.com/docs/app-server) supports the local
stdio protocol and version-specific generated schemas; its WebSocket transport is
experimental and unsupported. Use local stdio beneath Bullet's authenticated
access plane. [Cursor ACP](https://cursor.com/docs/cli/acp) has explicit session,
permission and blocking-extension exchanges. [Claude streaming](https://code.claude.com/docs/en/headless)
has its own native lifecycle. Qualify installed versions and controls separately;
do not infer Antigravity compatibility or interchangeable vendor flags.

Proposal contracts also disagree: both allow 128 operations and one MiB per write;
Kernel has no inspected aggregate-byte check while BulletGit caps writes at 32 MiB.
Kernel caps gates at 16; BulletGit's validator has no count maximum. BulletGit's
transport allows roughly twice the decoded size plus overhead, while JSON control
characters can require sixfold escaping. Generate compatible operation/gate/UTF-8/
metadata/preimage/transport limits and test producer→wire→consumer boundaries.

## Required failure matrix

Execute this matrix in addition to the existing twelve-boundary transaction
campaign. Synthetic fault results and installed/provider results remain distinct.

| Boundary | Required negative/recovery scenarios |
| --- | --- |
| Submission | Response loss before/after commit, duplicates, empty-cache discovery, exact retry after settlement. |
| Journal | Reload during commit, detach after durable settlement, two tabs, archived recovery, storage failures. |
| Ownership | Unauthorized reads, cookie replacement, stale CSRF, delayed 401/403, cross-owner caches/SSE, missing session acknowledgement. |
| Navigation | Thread switch, stale response, multiple loaded pages, ordinary refresh, focus/scroll/draft preservation. |
| Streaming | Gaps, duplicates, malformed frames, reconnect, revocation and bounded authoritative resynchronization. |
| TUI | Six clients, blocked credentials/network, first paint, resize, independent detach and complete terminal restoration. |
| Admission | Stale task/dependency revisions, capacity, unknown quota, deadlines, verifier backpressure and no budget reset. |
| Native execution | Ambiguous start, surviving descendants, stale prompts, unsupported controls, supervisor death and unknown termination. |
| Settlement | Fault at each atomic-finalization boundary, preservation failure, release response loss and cleanup refusal. |
| Review/effects | Self-review, changed Candidate, stale approval, wrong destination/base, lost update response and authoritative reconciliation. |
| Channels | Unlinked/unauthorized events, duplicates, reconnect, revocation, polling restart and ambiguous outbound delivery. |
| Installation | Two clean installs, reboot, upgrade/interruption, backup/restore, rollback, retained uninstall and disaster recovery. |
| Media | Tool drift, blank/zero-size PTY, frame collision/drop, timing errors, disk exhaustion/interruption, oversized output and mismatched identities. |

## Documentation repairs and full-program obligations

Correct current instructions while preserving dated historical evidence:

- Reconcile the ACTIVE execution plan's original-generation prerequisite with the
  reviewed fresh-generation proposal and continuing original-incident obligation.
  Remove current advice to run coordinator verbs during Operating HOLD.
- Correct effect-reconciliation advice that equates APPLIED with dispatch or
  recommends a new request key after UNKNOWN. Preserve exact identity and read-back.
- Amend ADR0018's proposed migration24 filename: current migration24 is operator
  sessions and history reaches27. Use the next supported append-only migration.
- Align policy rollback guidance with immutable successor generations and authority
  high-water; never silently delete/replace a ratified policy in place.
- Update source-origin prose in member SPLIT files, release/setup instructions,
  schema/backup ranges, toolchain pins and current CLI grammar.
- Distinguish saved human messages from Head execution in loopback/MCP instructions;
  remove stale claims that the signed-in adapter refuses before all starts.
- Replace static green badges and claims of visible TUI activity with source-bound
  evidence links and honest qualification labels.
- Repair the spec crosswalk's literal unexpanded totals, stale source/test pins and
  obsolete projection claims. Active session/context/scale requirements cannot be
  discarded by a stale DEFER label. Regenerate authored-source inventories normally.
- Reconcile live-conformance step counts with actual selected identities. Keep
  old media renderer limitations explicit until replaced and independently qualified.
- Repair compiled generated-release claims that call PONG live coding proof or
  conflate the1.95MSRV with selected1.97.1tooling; regenerate from authored source.
- Document IndexedDB recovery retention: browser state is not authority, but its
  exact prepared/unresolved requests and completion links are not disposable.

Every G1–G18, W/DF/WP, active Nightshift NS-0–NS-10 requirement, profile, lifecycle
and publication obligation remains in scope. Full cognition includes T0–T5,
context successors/compression, dissent/fusion, routing exclusions, negative
knowledge/quota, struggle, all behavior/context inventories, proof invalidation,
composition/conflict forecasting and causal lineage. Dormant libraries, catalogs
or attractive screens do not establish runtime enforcement. Preserve all84 behavior
rules,23 context kinds and20 Git capabilities. Nightshift also requires count and
monotonic-time fairness, scoped clocks, claim citations validated against returned
tools, exact routing/focus, verified ref retirement, and crypto-erasure covering
backups/projections/logs/caches while retaining immutable identity/tombstones.

The current contract census still has40 open catalog leaves,40 Rust dynamic values
and40 TypeScript open records. Catalog W11 preserves legacy sentinels while its
resolver/duplicate-aware byte parsers/strict immutable generation are qualified;
W12 removes the open leaves together. These catalog packet names are distinct from
roadmap Wave11 team/saga. ADR0018 additionally requires five purpose-separated
signers, OS-fact-derived evidence, exact gate coverage, atomic publication and
independent high-water read-back/restore; signer presence is not acceptance.

Matched studies, calibration, independent holdouts, contamination and missingness
controls, all-in cost, frozen MOME/ASHA search/archive/allocation, shadow/canary/drift/
rollback and the required procedural-guidance pilot remain owed. Guidance stays
disabled until independently qualified under the same task budget. Preserve the
no-guidance/raw/generated/equal-budget-no-graph comparison and adverse results.

All four provider, four forge and five platform slices remain independent.
Ubuntu/Claude/protected Jeryu remains `self-hosted-v1`; ordinary GitHub source
delivery does not certify the GitHub App effect adapter. Compose compatible
receipts into `universal-v1`; certify team before saga. Retain all 18 product
profiles and the two explicitly diagnostic profiles.

Active WP obligations still include wire/source publication, preservation ancestry
and rehearsal, managed/connect-existing Jeryu, signed auditor distribution,
reproducible paper/brief and immutable PDFs, comparative evidence, independent
security/accessibility, two independent rubric-preserving re-scores, stranger
installation and lifecycle acceptance. Run seven-day observation only after actual
integrated changes exist. No superiority claim precedes accepted matched evidence.

The detailed implementation sequence, user experience, negative-test campaign,
operator packets and final acceptance are in the
[existing xbabe2 work order](xbabe2-development-closeout.md). A passing component,
new plan or reviewed document closes only its own bounded requirement.

The inventory binds private retained evidence by basename and SHA-256; it is not
a portable execution receipt. Full raw logs and artifacts remain in the private
xbabe2 custody directory until an exact reviewed preservation/publication packet
makes them independently available. No source audit can guarantee that no further
defect exists; every newly discovered active obligation enters the same registers.
