# Local production prerequisite checkpoints

Status: **retained component observations; no release authority**

Owner: Bullet Farm maintainers

Last reconciled: 2026-09-08

The [active plan](full-product-dogfood-plan.md) and [gap register](product-gaps.md)
remain the current program. These dated observations retain their original
subjects and scope; a later commit needs its own proof. Older or failed
artifacts receive no credit merely because they are retained.

## Product-plan observations

The September 8 component checkpoint passed the Portal `family` lane's three
real-farmd browser tests on these exact source subjects:

| Member | Commit | Tree |
| --- | --- | --- |
| Portal | `b66a2053d25b5d306ee4c4b345bc6aa8ed34347c` | `3132496669137555cadf28c208a9658cb86b914c` |
| Kernel | `b179eafc2f86af5251d8b79a17106fe2b67e9ba7` | `99b6b38d0d45845dc098c31f70228e76f06033dd` |
| BulletGit | `48755d95cf8469d48e1a022f2f7223c07393d6a3` | `7f3e36d8b1f838774f84fe0a15ca02df0a855a48` |

From the Portal checkout, `bash scripts/ci-local.sh family` invokes
`ops/ci/real-farmd.sh` and `e2e/real-farmd.spec.ts`. The executed path connects
authenticated public commands, the UDS worker, product Runner/BulletGit,
immutable retained-artifact validation, durable `UNKNOWN`, and worker restart
read-back. Its provider is synthetic, its outer receipt is `UNSIGNED_FIXTURE`,
and its evidence is `COMPONENT_PROOF`; independent, transaction, and release
eligibility remain false. The browser uses Vite preview and real sibling farmd;
this observation supplies no packaged-origin or installation certification.

The accepted baseline repairs validate UTC timestamps in producer `+00:00` and fixture `Z` forms, checkpoint and close a quiescent WAL
ledger before immutable reads, await owned Gitd termination, and clean up CI workers. Kernel `Cargo.toml`/`Cargo.lock` now pin rusqlite
0.39.0 with bundled SQLite 3.51.3 and retained checked unsigned SQL conversions. The SQLite C source matches the vendor's fixed release
hash. The relevant Kernel consumers are `apps/bullet-runner/src/bin/bullet-command-worker/receipt/preservation.rs`,
`crates/adapters/src/sqlite/open.rs::close_quiescent`, and `crates/runner/src/gitd/session.rs`; Portal owns the CI fixture lifecycle in
`ops/ci/real-farmd.sh`. The bundled C SHA3-256 is `32d5424f97e0a7fc5ed2f6335afbb58be4e0298bd7117a34e39d345ff13d859e`, matching the [SQLite
3.51.3 release](https://sqlite.org/releaselog/3_51_3.html). Focused Runner library, SQLite adapter, and worker-receipt checks passed 103,
36, and 24 tests respectively; the Rust 1.95 workspace/all-targets build also passed. These observations do not constitute a release MSRV
receipt.

Hub `d3c4db3600e6f1d93800d258e3ac7c1a2b4db4f3` passed `required`: 770 Rust
tests, two formal models, and documentation/media checks. Its repaired
`scripts/demo.sh` captures and revalidates five Cargo executable subjects;
both existing launcher tests, including 35 fake-Cargo cases, passed.

The September 8 actual `just demo` and separate component-receipt verification
then passed on these newer Hub and Kernel subjects, with Portal and BulletGit
unchanged from the initial checkpoint above:

| Member | Commit | Tree |
| --- | --- | --- |
| Hub | `cdcbfd50e643635483acb427cf376f131684740d` | `b8a07dce4e284d1730f8a8e783bbf5444f6f2383` |
| Kernel | `071173a94e2d50d404aa07801a07064bd8a1e375` | `247dcc67595af8aa5f73420b7a78399494fed6ff` |

The demo disabled Portal startup and used synthetic execution and ephemeral
fixture authority. Receipt SHA256
`b379e69f9c82ad3ef1b13c16acd54fc52616e88da69048ef748ce0e0205ca2c1`
verified as `COMPONENT_PROOF`, with `EPHEMERAL_SELF_SIGNED` component signing
and `UNSIGNED_DIAGNOSTIC` verification trust. Transaction-gate and release-profile
eligibility remain false; this adds no native-provider, installation, or
independent transaction certification.

The following full-family attempt on those same four subjects completed
BulletGit `required`, Kernel `required` (1,067 selected standalone tests and
34 contract tests), and all nine Kernel family tests. Portal passed 131 Vitest
and 14 standalone Playwright tests, then its security lane refused two matches
of a public fixture key identifier in a retained receipt. The attempt failed;
its logs and original receipt remain preserved. Portal
`3cc19fd388dfbedfca689f1703f9dc719ea9ac23` (tree
`42a023a75de4955a1305f4e96057ddc2b85fad71`) now extends the default scanner
policy with one exact field-and-public-label match. Seven pinned canaries,
eight refusal mutations, and focused security and CI-policy checks passed.
The original failed-run receipt remains preserved unchanged; later runs produce
their own receipts. No artifact directory or whole field was excluded.

Two complete repaired full-family runs passed on September 8 at 04:07 and
04:18 UTC on Hub `cdcbfd50`, Kernel `071173a9`, BulletGit `48755d95`, and Portal `3cc19fd3`.
Each retained all eleven canonical report entries: 1,699 selected cases and
two formal models. Their canonical results match exactly; the observation digest is
`blake3:7a3d352c9fc6ef8fc579fe25728befe78ab83d98afd15c36860f7b1d2675bcbc`.
Each is an unsigned `DIAGNOSTIC_ONLY` observation with `release_authority=false`.
Real run metadata remains separately retained. These dated observations precede
this documentation commit; later source subjects require their own proof. Public App
identity, PR publication, the complete root workflow inventory, portable audit,
and branch protection remain blocked. WP-02 retains `COMPONENT` evidence in
the typed inventory; all G1–G18 remain `DESIGNED`, all release profiles remain
`BLOCKED`, and Operating HOLD remains effective. WP-22 retains its separate
publication component evidence.

The next admission prerequisite closes the incomplete single-location V1
initialization route before record-body reads or sidecar publication. Pristine
bootstrap now inventories both family metadata locations and binds permitted
interrupted stages and generation directories to the durable initialization
intent under the final lock. Retained incidents, foreign generations, ignored
stages, symlinks, and hardlinks cannot be treated as a pristine retry.
`src/coord/store/ledger/admission.rs` owns this guard; the existing initialization
journal still handles exact retries and process death. The final focused ledger
suite passed 51 tests, public coordinator lifecycle/CLI passed 11, and the
repository's two-pass Clippy policy passed. This is a `COMPONENT` prerequisite:
complete two-location inventory/replay/disposition/review records, Genesis-bound
admission references, and the operator checkpoint remain absent. Filesystem
absence does not prove that incident history never existed, and cooperative
same-UID checks do not supply production custody separation. All G1–G18 evidence
classes and all profile results in the typed inventory remain unchanged.

## Gap-register context

September 7 publication component (W0/WP-22; G1/G8/G12): the Hub now owns exact
source capture, deterministic aggregate templates, durable publication requests,
whole-history object scanning, atomic immutable-source/review refs, portable
split reconstruction, and an unsigned exact-event CI observation. Focused local
proof is component evidence only. Public App identity, PR publication, complete
hosted member/family/scheduled inventories, portable audit, protected integration,
and independent release campaigns remain required. No G-gap or product profile
is closed by the narrow publication bootstrap; see the
[runbook](../runbooks/publication.md).

September 8 connected component checkpoint (WP-02; G2/G3/G4/G13/G14): all
three Portal real-farmd family tests passed on exact Portal, Kernel, and
BulletGit subjects recorded in the [active plan](full-product-dogfood-plan.md).
The public command traversed the UDS worker and product Runner/BulletGit,
retained-artifact admission, durable `UNKNOWN`, and restart read-back. Strict UTC
timestamp validation, quiescent WAL shutdown, owned Gitd kill/wait, CI cleanup,
and the pinned SQLite 3.51.3 fix repaired observed baseline blockers. The
provider remains synthetic and the receipt `UNSIGNED_FIXTURE` /
`COMPONENT_PROOF`, with every higher eligibility flag false. This source
browser campaign uses Vite preview; packaged-origin and installation evidence
remain separate. WP-02 records `COMPONENT`, while all G1–G18 remain `DESIGNED`,
all profiles remain `BLOCKED`, and coordinator Operating HOLD remains effective.
A subsequent actual Hub demo and separate component-receipt verification passed
on Hub `cdcbfd50` and Kernel `071173a9`, with Portal `b66a2053` and BulletGit
`48755d95`; exact commits, trees, and receipt digest are in the active plan.
That demo used synthetic execution and ephemeral fixture authority, with
transaction-gate and release-profile eligibility false. The following family
attempt completed Kernel `required` (1,067 selected standalone and 34 contract
tests) and all nine Kernel family tests, then failed Portal security on a
public fixture key identifier. The original failed-run receipt remains preserved unchanged. Repaired
Portal `3cc19fd3` passed its seven pinned scanner canaries, eight refusal
mutations, and focused security and CI-policy checks. Two repaired full-family
runs then passed at 04:07 and 04:18 UTC with all eleven canonical report entries:
1,699 selected cases and two formal models per run. Their canonical results match
exactly; actual run metadata remains separately retained. These unsigned
`DIAGNOSTIC_ONLY` observations have `release_authority=false` and bind the dated
subjects in the active plan, preceding this documentation commit. No successful public CI,
protected integration, native-provider, installation, or transaction
certification is claimed.

September 8 coordinator prerequisite: the single-location V1 initialization
consumer now refuses before sidecar publication. The pristine initialization
path checks both metadata locations and exact interrupted-journal subjects;
retained incident material, foreign generations, and ignored or aliased stages
refuse. Focused ledger (51 tests), public lifecycle/CLI (11 tests), and mapped
Clippy checks passed. This closes an omitted-input bypass component; it does not
supply the complete two-location admission packet or lift Operating HOLD.
The [active plan](full-product-dogfood-plan.md) retains supervised upgrades before
execution migrations, the real Runner/API/Candidate path, all three initial
subscription providers and twelve tasks, publication/CI, and every later profile.
The typed inventory was revalidated together with this entry: all G1–G18 remain
`DESIGNED`, and all 18 product plus two diagnostic profiles remain `BLOCKED`.

## Shared prerequisite observations

September 8 schema prerequisite: authentic schema-22 migration prefixes now return `UPGRADE_REQUIRED` before writable source SQLite startup.
A private preflight preserves source WAL and rollback journals, rejects external super-journal recovery, and bounds snapshot input and
recovered size to 1 GiB. Focused adapter library (40), cross-process startup (5), and lease transaction (13) tests and all-target Clippy
passed after independent hostile review. The supervised upgrade command, maintenance custody, prefix-aware verified backup, high-water
enforcement, and rollback remain open; this startup preflight supplies none of their authority.

A subsequent serving-custody component retains a shared lock on the admitted main descriptor before preflight and throughout the ledger
lifetime. It supports the ext filesystem family and refuses unsupported filesystems or contention. Tests prove independent SQLite close,
duplicate descriptors, process death, and a forced creation-to-lock race; busy refusal preserves the exact inode. The focused custody
identity, 40 adapter, five cross-process, and 13 lease tests and Clippy passed with independent review. Online backups now use the same
shared custody and read-only SQLite, retaining the source guard through publication and bounded receipt read-back. Exclusive-owner and
missing-source refusal tests preserve the complete source/destination inventory. Prefix-aware upgrade backup, exclusive maintenance, and
generation replacement custody remain open.

Typed schema inspection now retains the verified catalog prefix, its digest, normalized authority and restore state. Serving and current
backup still require the current schema; backup receipts take their schema and restore subject from the inspected copy. Schema-22 integrity
requires the complete result `["ok"]`. The extended schema identity, 40 adapter tests, 18 integration tests and Clippy passed with
independent review. Authority is a sampled row: coherent upgrade admission still requires transaction and maintenance custody. This
component adds neither prefix-backup permission nor an upgrade or high-water authority.

Online backup now explicitly closes its admitted source on every producer result. A real SQLite close failure retains the connection and
custody until process exit and refuses new admissions before filesystem effects. Confirmed close followed by postflight or cleanup failure
reports the failure without poisoning later admissions. The extended child-process identity, 40 adapter tests, 18 integration tests and
Clippy passed. Existing in-flight operations and unrelated serving shutdown paths are outside this repair; prefix backup and exclusive
upgrade custody remain open.

A two-location preservation component now binds the complete outer and Hub inventories to one purpose-specific record. `bullet-family
preservation-bind` accepts supplied canonical mode-0600 observations in private mode-0700 parents and publishes a sealed mode-0400 record
outside the family. It never observes or moves the incident directories. Successful creation or exact-existing adoption requires retained
file and parent synchronization plus identity and byte read-back; persistent post-link sync failure refuses, retains the inode, and permits
exact retry after recovery. Five pair, ten publisher, eight sealed reader, and two CLI fixture tests passed. This is component evidence;
paired replay and historical dispositions, independent review and operator admission, locked Genesis references, and the complete
preservation fault journal remain open. Operating HOLD is unchanged.

A subsequent `preservation-bind replay` component consumes supplied sealed pair/request records and both complete retained ledger copies. It
binds exact copy bytes, replay projections, every historical claim, and ordered dispositions to one sealed `REPLAY_FACTS_ONLY` record, with
read-back and exact retry after synchronization failure. Recorded receipts must exist in the replayed history; other claims remain retained
for recovery. Missing, corrupt or interrupted histories, omitted claims, invented receipts, and unsafe or conflicting paths refuse. Six new
replay identities and 25 related tests passed, followed by combined Clippy and canonical inventory checks. The producer does not observe
incident directories or admit Genesis: complete incident-range handling, independent review, durable locked admission references and the
operator checkpoint remain required. Operating HOLD is unchanged.

A CI expectation component now derives 55 invocations from the reviewed 53 nested job definitions and exact eight-workflow inventory. The
canonical plan binds aggregate and member commits/trees, manifest and workflow digests, matrix values, dependencies, and runner selection.
`bullet-publish ci-plan` supports an admitted aggregate checkout or an existing publication store/request, without creating another checkout
or Git objects. Independent review against all eight source workflows and 29 publication tests passed. Its output explicitly contains no
execution evidence. The 27 required invocations and 28 scheduled invocations remain distinct; root workflow generation/activation, hosted
tool and worker admission, job observation envelopes, the stable final check, and additional family/MSRV and assurance campaigns remain
open.

A bootstrap CI diagnostic adapter now binds the selected immutable plan row, aggregate and member subjects, actual GitHub
event/workflow/run/attempt, and bounded member observation bytes. `ci-job-context` and `ci-job-observe` support only
`bullet-farm:REQUIRED:source_scan` under the existing bootstrap root job. The observer executes the exact source-bound semantic validator
and retains its output digests. The bootstrap job does not execute the nested source scan; both outputs state `execution_evidence=false`,
with no run or release verdict. Twenty-nine publication tests and both Clippy passes succeeded. This sampled validation does not attest
hosted provenance or isolate mutable same-UID tools. Generated root job execution, complete execution observations and the stable required
convergence check remain open. The measured Rust inventory is now 789 identities, partitioned into 570 Hub and 219 wire tests, with
precisely the six new replay identities added and no prior identity removed. The typed inventory remains unchanged: all 18 gaps are
`DESIGNED`, and all 18 product plus two diagnostic profiles remain `BLOCKED`.

The complete Kernel `required` check subsequently passed on commit
`3299443d4f15bb27e84aa8f4f70512ca800f4dd5`, tree
`1e260d91f13f462354bd5dcb6679240ad65f21c5`: 1,067 selected standalone tests,
34 contract tests, and the mapped lint, security and documentation checks.
Log SHA256: `da64c0cce7709258ecb861a9585f1ab3274cabaea2c874cd9790ebf5653fad28`.
Only that run's log and newly generated fast/contract reports receive current
credit; inherited coverage, family, faults and older observation files do not.

A later backup snapshot component was accepted on Kernel commit
`22d90d6a66f16624ccd1f60e6711576b8637de30`, tree
`5738a18bee863445f8dbc7c7444769ba4f14c6c6`. The actual backup source is now an
owned recovered private snapshot opened read-only; SQLite never opens the
original source for this backup path. Source custody and private directory
ownership survive inspection, publication, read-back and explicit close.
Nine backup tests, one schema-22 preservation test, 40 adapter tests, five
cross-process tests, 13 lease tests and strict Clippy passed. Empty and
unsupported sources still refuse. Failed close retains live handles and private
files until process exit; confirmed-close cleanup failure reports retained
paths and permits retry. Source capture and pre-cleanup directory identity
checks remain sampled under shared custody, with no hostile same-UID
containment, prefix allowance or exclusive upgrade authority. Complete
`required` proof on this newer commit remains pending; the preceding run
binds only its recorded older subject.


A subsequent CI execution component was accepted on Hub commit
`c64be759ab8a50313b3d7e98d5e7dc2c47730183`, tree
`fb825b4ff32e41812239a69b0335f3892c86bcae`. The root bootstrap job now executes
the reconstructed Hub's actual source-scan entrypoint, stages the exact member
diagnostic, and invokes the source-bound semantic validator. A separate
completion record binds successful bounded execution, selected source subjects,
tool and artifact digests; the earlier Rust validation output still correctly
states `execution_evidence=false`. A stable final job requires both run/attempt
artifact sets and successful predecessor/completion outputs; missing, skipped,
cancelled, neutral, malformed, stale, corrupt or extra results refuse.
All 47 actual wrapper fixtures and 29 publication tests passed, as did
ShellCheck, actionlint and exact inventory checks. The wrapper log SHA256 is
`f28f97d9d687855591debc602367c80ee506d71d6645f3a91b71eb14a3549497`;
the publication test log SHA256 is
`56bc5e369b252ee0ea8a5c153737859a1ca4b4a145a4f48e45dbed8cf92b0197`.
These are local fixture proofs, not GitHub executions. Completion remains
unsigned `DIAGNOSTIC_ONLY`, with `tool_closure_admitted=false` and no release
authority. This executes only one Hub source scan; all 53 job definitions / 55
invocations, family/MSRV additions, hosted capacity, portable Jankurai and
isolated certification workers still require implementation or qualification.
Later publication description edits require rebuilding the embedded template
catalog and rerunning the wrapper on their exact new subject.

Prefix-aware backup was then accepted on Kernel commit
`a6f11d03e053c550d86f7fda788714a9a5f7f447`, tree
`e1c962e59254d66760ca12ac4c41e04b63a25507`. The actual backup producer accepts
only inspected authentic schema 22 or current schema 23, verifies identical
source/copy schema state, digest and restore state, and reads back the exact
published bytes. Tests cover standalone, retained WAL, missing SHM, hot journal,
malformed catalogs, quarantine, exclusive custody, publication faults and
collision without source changes. All three new prefix tests, 43 adapter tests,
five cross-process tests, 13 lease tests and strict Clippy passed after independent
review. An initial diagnostic-string assertion failure is retained; the final
repair corrected expected Display text and strengthened the typed error check.
Actual Nextest listings contain 1,116 total / 1,070 standalone identities; removing
exactly these three additions reproduces both prior identity digests, with the
three egress, 34 contract and nine family identities unchanged.
Serving and ordinary restore still require the current schema, and quarantine
still refuses backup. A schema-22 receipt grants no rollback activation or upgrade
authority; prefix restore and durable backup/receipt retry remain open. Before
original writable access, supervised upgrade still needs exclusive maintenance
custody, a persistent serving gate and intent journal, external authority
high-water, transactional migration and verified quarantined rollback. Source
capture and cleanup retain the preceding component's sampled same-UID limits.
Complete required checks on the integrated subjects are a separate pending proof
at this checkpoint; earlier full checks do not transfer to later source trees.
The typed inventory was read and reconciled without changing its bytes: all
G1–G18 remain `DESIGNED`; all 18 product and two diagnostic profiles remain
`BLOCKED`, including the active post-V1 and retained retired dispositions.


Complete checks subsequently passed on the integrated prerequisite subjects:

- Kernel `e26398d4302f66e36e3eb2feb6bff221463cc77a`, tree
  `2aa8391d3ce33323d2c40d43bdaad80b8ecde80b`: 1,070 standalone and 34
  contract tests, plus mapped lint, security and documentation checks. Log SHA256
  `3d5256246746ebd0d83963f41648a336269886d8f4ef2648b1f7bbf00e6a7003`.
- Hub `43335886a9e6058038a93540df942fa2d9148f0a`, tree
  `f8216911e221f3ca1f2a2a8226f21e7a2dbb3e93`: 570 Hub and 219 wire
  tests, both formal models, lint, security, documentation and component media.
  Log SHA256 `7b5831572ceede079b51c52cae0a79b8d2ea220d5d62b5735cc089a9fe778624`.

Only newly generated reports and exact completed-run logs receive current
credit. Inherited coverage, family, fault and older observations are excluded.
Neither complete member check establishes hosted CI or a complete family run.
Later source changes require their own mapped verification and exact read-back.

BulletGit's actual staging entrypoint previously removed its upload directory
and then attempted to create `observations` without the parent. Commit
`c9a7b4aa8cdc6930837c3542536a1aec8e78ad51`, tree
`2dea0b26b06a5fe8fd78a7b64e41156258a1c3f2`, creates both private directories
before copying validated files. A real script fixture first reproduced the
failure, then passed source-only and report staging, exact bytes and modes,
retry cleanup, invalid inputs and parent-symlink refusal. It binds a retained
commit through read-only Git metadata; all fixture artifacts remain private.
Mapped artifact checks, ShellCheck and diff checks passed.

A subsequent BulletGit corpus-target repair was accepted on commit
`7c999f545a419d03fc94cdce2d804172bc3f0574`, tree
`61f9cf31409ae95d918f10c58a0a71f5b4fe18e7`. The actual replay helper now
uses the selected Cargo target as its parent, preserving the ordinary default,
both locked/offline commands and all six corpus checks. The mapped parity test
first reproduced private-target drift and then passed default and private paths,
including spaces, exact arguments and stop-on-failure behavior. This fixture
uses an explicit Cargo trace and claims no real corpus execution. Complete
BulletGit `required` on the accepted subject is pending at this checkpoint.
Both repairs remove concrete CI blockers; full root workflow activation and
hosted tool/worker admission remain open. The typed inventory was reconciled
without byte changes: all G1–G18 remain `DESIGNED` and all 20 profiles `BLOCKED`.


Verified prefix restore was accepted on Kernel commit
`8445f75d8550b5aab3559907cb8d67bc2f038fb2`, tree
`3fa7f9ecb2587347cad763b9476f4083b55c97f1`. The existing restore consumer now
admits only strict inspected schema 22 or 23 with an exact receipt. It preserves
schema and authority state, advances the restore epoch transactionally, and
returns success only after reading back the published quarantined output through
another private verification snapshot. SQLite never opens the backup source or
published output names. Explicit close failures retain live handles and files
until process exit and poison subsequent serving/backup/restore calls. Cleanup
failures retain evidence; post-publication failures never report success.
Five focused tests, 45 adapter tests, five cross-process tests, 13 lease tests and
strict Clippy passed after independent review. The initial Clippy dead-code
failure is retained; selecting the inspected current/prefix digest in the actual
consumer repaired it, followed by focused and strict Clippy success. Final focused
log SHA256 `fc3d2c61ffe3e6c5df9609672371f987956489a2774d15348445d85228278a94`;
final Clippy log SHA256
`358477f485a9e87d4b29d7cd509c3b19e2aaab5700e6564ac85da1c35b80907c`.
Actual Nextest contains 1,118 total / 1,072 standalone identities: two new restore
tests and one accurately renamed existing test reconstruct the prior inventories.
The three egress, 34 contract and nine family identities are unchanged. Capture,
read-back and cleanup remain sampled same-UID checks, not exclusive hostile
custody. Durable backup/receipt retry, maintenance custody and intent journal,
external authority high-water, migration and rollback activation remain open.
The following metadata commit updates only measured counts/digests and a stale
comment; complete Kernel proof on that integrated subject is pending here.

The sealed replay reader was accepted on Hub commit
`bd4c48944c68d48299e312994a70d56549b86346`, tree
`172ee3fcb162fa1e6c09805d7b8dada3461eba57`. The actual
`preservation-bind replay-verify --record ABS` command reads the sealed canonical
record, decodes its closed embedded request and shares the producer's complete
deterministic reconstruction. Whole-record comparison binds all retained inputs,
role-bound ledgers, projections, historical dispositions and the replay identity.
Final rereads of all three inputs and the record precede the fact-only response.
Nine replay tests (six retained producer and three new reader identities), two
CLI regressions, both Clippy passes and the canonical module inventory passed on
unchanged independently reviewed bytes. Replay log SHA256
`89c271f19f180455f350b3cc4d271c0e52a1f9941075938c2318751849b8a846`.
Actual Nextest contains 792 identities: 573 Hub plus 219 wire. Removing exactly
three additions reproduces the prior complete lists and digests. The reader
performs no writes, incident observation or recovery and grants no Genesis
admission. Complete incident-range handling, independent review and operator
binding, final initialization-lock persistence and crash journal remain open.
Complete Hub proof on this later integrated subject is pending here.

Complete BulletGit `required` then passed on the previously recorded commit
`7c999f545a419d03fc94cdce2d804172bc3f0574`, tree
`61f9cf31409ae95d918f10c58a0a71f5b4fe18e7`: 62 fast and 160 contract tests,
all six actual corpus outcomes, lint, security and documentation. Log SHA256
`6c4e3e08480915b873c54551531c8a86dca294ce7e51db61e1e6e42f180d46da`.
Only its three newly generated artifacts and exact run log receive new credit;
13 inherited artifacts are excluded. This is local member proof, not a hosted
or complete family result. The typed inventory was read and reconciled without
byte changes: all G1–G18 remain `DESIGNED`, all 20 profiles remain `BLOCKED`,
and active post-V1 plus retained retired dispositions remain unchanged.


## September 8 health-first staged backup checkpoint

The four preserved Kernel files were retained byte-for-byte, independently
reviewed by the `codex-health-review` lane, and committed as
`3efe25ec5d74fffb895b60dbb789e80e1a8a8636`, tree
`17f4e24d60b4c247c8c5fb70bd7118b67ba44472`. Both the output-copy and output-verify
SQLite connections now close explicitly before cleanup/publication. A failed
close retains the live connection and staging file, sets the process poison,
and still finalizes the source after dependent backup objects are gone. Restore
consumes the same staged lifecycle. Collision and cleanup errors preserve the
peer and primary error.

`cargo test --locked -p bullet-adapters --lib sqlite::backup::` passed all 16
selected tests with zero ignored. The new pairs exercise current schema 23 and
authentic prefix 22, copy/verify close failure with and without a primary error,
source shared custody and release, retained output handles, cleanup substitution,
publication collision, and process exit followed by retry. This does not prove
interruption during copy/publication or durable supervised upgrade recovery.
Strict all-targets adapter Clippy, targeted formatting and diff checks passed.
The cold focused command took 69.05 seconds, including 43.75 seconds compilation,
with peak RSS 658,068 KiB at two Cargo jobs; this is one local sample, not a hosted
capacity recommendation. Test log SHA256
`c19779de24d16cfa0099f90bb926ab8d89c1f2715e5ac2f41d32202060d94655`;
Clippy log SHA256
`6a46bf3e8fa434ef2daff19a0531dd89d9436f2948d203607dab3a4e5a875a32`.

The separately reviewed inventory commit
`b6a105b1ed12fd84d9abc1cc0554c6d052391c6b`, tree
`7d6da8a9d9d4a43fe0f18ab610db1f7519882d45`, declares 1,120 total and 1,074
standalone tests. The two new identities were measured from actual Nextest
output; retained complete JUnit and existing family/egress identities reproduce
every prior partition digest before the additions. The 34 contract, nine family
and three egress identities are unchanged. Full current workspace enumeration,
complete member checks and dependency-ordered family proof remain pending on the
final clean integration subjects. The earlier documentation-freshness failure
remains historical evidence.

The typed assurance inventory was reread and reconciled without byte changes:
all G1–G18 remain `DESIGNED`, all 18 product and two diagnostic profiles remain
`BLOCKED`, and active post-V1 plus retired dispositions remain present. This
checkpoint grants no coordinator admission, provider enrollment, hosted CI,
transaction, installation or release credit. The Operating HOLD remains effective.
