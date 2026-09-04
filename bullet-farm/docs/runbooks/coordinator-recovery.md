# Coordinator recovery production

Status: **Linux component procedure; real-incident execution remains blocked**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-28

This runbook covers the production inspection/manifest producer and DF-R6
after a schema-2 Recovery generation is published. It does not authorize the
retained real incident, release evidence, or normal coordination dogfood.
Those gates remain in the
[full-product dogfood bridge](../assurance/full-product-dogfood-plan.md).

## Sealed inspection and authorization

`recovery-inspect` accepts only the three original forensic artifacts and an
explicit new output. It derives the trusted LF boundary, all artifact and OS
identities, record/claim inventories, frozen projection, lineage ranges,
incident time, and state/inventory digests. It creates nothing beneath the
coordinator directory and creates its mode-0400 output once:

```bash
bullet-family --root /absolute/family coord recovery-inspect \
  --interrupted-capture /absolute/incident/interrupted.partial \
  --tainted-generation /absolute/incident/tainted.jsonl \
  --frozen-live-source /absolute/family/.bullet-family/coord/events.jsonl \
  --output /absolute/recovery/inspection.json
```

All recovery documents live beneath an exact-owner mode-0700 directory. Every
input is a bounded, exact-owner, single-link, mode-0400 regular file. The
frozen live source alone may have the retained coordinator's exact-owner
legacy mode-0775 parent (or its already-tightened mode-0700 parent); every
other parent remains exact 0700. The bootstrap opens parents and inputs
without symlinks, performs two independent stable reads, and requires
canonical JSON with exactly one trailing LF.

Before authorization, produce and seal
`bullet.coord.recovery-bootstrap-provenance.v1`. It binds the bootstrap commit
and tree OIDs, archive SHA-256, Cargo.lock SHA-256, sorted source path/length/
SHA-256 inventory, exact Rust and Cargo versions, and executable length and
SHA-256. The `Cargo.lock` inventory entry must exactly equal the top-level
Cargo.lock SHA-256.

The Linux-only, policy-disabled producer accepts only the full reviewed commit,
canonical direct Cargo and Rustc ELF pathnames, and a new output outside the Hub
checkout:

```bash
bullet-family --root /absolute/family coord recovery-provenance \
  --bootstrap-commit <40-or-64-lowercase-hex> \
  --cargo-bin /absolute/direct/cargo \
  --rustc-bin /absolute/direct/rustc \
  --source-archive-output /absolute/recovery/bootstrap-source.tar \
  --output /absolute/recovery/bootstrap-provenance.json
```

It requires the ordinary primary checkout to be clean at that exact `HEAD`,
pins `/usr/bin/git` and both supplied tools through retained descriptors,
refuses replacement/alternate/lazy object authority, derives the tree, and
creates the same bounded raw `git archive` twice. It compares those complete
bytes, independently parses the tree and TAR path/mode/content inventories,
requires their exact file, directory, and mode equality plus exact source
length and SHA-256 equality with the raw Git blob contents,
refuses ambient `.git/info/attributes`, links, submodules, special entries,
`export-ignore`, `export-subst`, and every zero-byte source (V1 cannot represent
one), and copies the nonempty top-level `Cargo.lock` digest from that complete
sorted inventory. V1 admits at most 8,192 source files, 512-byte paths, 64 MiB
per file, 256 MiB in aggregate, and a 512 MiB running executable; the producer
also preflights the generic sealed canonical-document byte ceiling. Two bounded
`-Vv` probes must agree on host/release and the archived
`rust-toolchain.toml` channel. The producer stable-reads `/proc/self/exe` before
and after observation, revalidates every retained subject, then publishes one
canonical mode-0400 provenance output beneath an exact-euid mode-0700 parent.
`--source-archive-output` is also required: it is a distinct normalized
absolute sibling of that provenance output, outside the Hub checkout, and
receives the exact bounded raw archive as a mode-0400 single-link file. The
provenance output must be absent. The archive is created once, or an existing
archive is adopted only when its stable bytes exactly equal the regenerated
archive; a different existing archive refuses before provenance publication.
The producer never overwrites either path. It never opens or creates
coordinator state, builds, signs, contacts a remote, or reads a credential.

This V1 document does not carry Git/Cargo/Rustc executable digests, build
command/environment, dependency-cache identity, target, or reproducibility
fields. Before authorization, a distinct admitted builder must extract the
exact recorded raw archive into a fresh non-worktree directory, use a separately
admitted credential-free dependency cache with isolated `CARGO_HOME` and
`CARGO_TARGET_DIR`, and run exact
`cargo build --locked --offline --release -p bullet-family --bin bullet-family`.
The independent reviewer repeats the commit/tree/archive/Cargo.lock/source and
tool-version derivation from separate custody and byte-compares that build with
the executable recorded by two byte-identical producer outputs. Until that
source-to-binary and exact tool-subject observation exists, producer success is
component evidence only and recovery remains blocked. Runtime independently
reads `/proc/self/exe` twice and binds the running executable to the sealed
document; it does not turn the remaining reviewer-verified facts into runtime
Git or build verification.

The independent reviewer—not the recovery operator—then authors a closed
`bullet.coord.recovery-authorization.v1` with decision `APPROVE`, the exact
inspection ID and sealed SHA-256, admitted operator identity and UID, reviewer
principal and recovery-key fingerprint, policy namespace, sealed provenance
SHA-256, stable decision time, renewable Unix issue/expiry, Linux boot ID,
exact initial time-namespace device/inode, and matching `CLOCK_BOOTTIME`
issue/expiry. The Unix and boot-time durations are equal, positive, and at
most 24 hours. Reaching either exact Unix or boot-time deadline is expired.

Before the reviewer authors that authorization, a trusted root supervisor—not
the recovery operator—publishes the current boot-scoped expectation at the
compiled path `/run/bullet/recovery-clock-v1.json`. Its parent must be
root:root mode-0755 and its single-link regular file root:root mode-0444. The
document is strict canonical JSON with exactly one trailing LF and this closed
shape:

```json
{"boot_id":"00000000-0000-4000-8000-000000000000","kind":"bullet.coord.recovery-clock-expectation.v1","schema_version":1,"time_namespace_device":1,"time_namespace_inode":1}
```

The create-once publisher derives the actual boot ID and retained host
time-namespace descriptor identity and does not accept those values from the
operator. It creates the absent compiled-path record or adopts an existing
record only when its bytes are exactly the expected canonical document. It
never deletes, replaces, or overwrites a differing or stale record. Such a
record requires disposable-lifecycle cleanup by the root package supervisor,
outside publisher authority, before the publisher can create a fresh record.
That packaged lifecycle remains absent from the current source subject and is
still blocked. The reviewer copies the exact published tuple into the signed
authorization. Runtime stable-reads the root record
twice around its clock sample, retains `/proc/self/ns/time` across the sample,
revalidates that same descriptor through `/proc/self/fd/<fd>` after the sample,
and requires its NSFS identity and exact `time:[inode]` magic-link type to equal
both the root expectation and signed authorization. The reader descriptor-walks
and validates `/`, the intentional root-owned `/run` tmpfs mount transition,
and `/run/bullet`; `NO_XDEV` forbids another mount transition beneath
`/run`. Missing, wrong-owner,
wrong-group, wrong-mode, linked, noncanonical, malformed, boot-stale, or
changing root expectations return `RECOVERY_POLICY_DISABLED` or a typed
boot/namespace refusal before mutation. The compiled policy digest binds this
exact expectation path and boot-ID/exact-time-type/nsfs/`CLOCK_BOOTTIME` trust
method, while the dynamic boot/device/inode tuple stays outside the stable
policy and operator-decision digests;
negative-offset and zero-offset child namespaces both refuse regardless of
their displayed clock values. The
reviewer uses the following closed sequence. Every value in angle brackets is
an explicit placeholder that must come from admitted policy and reviewed
custody; it is not an installed identity or operator fact:

```bash
bullet-family --root /absolute/family coord recovery-authorization-draft \
  --inspection /absolute/recovery/inspection.json \
  --bootstrap-provenance /absolute/recovery/bootstrap-provenance.json \
  --decision APPROVE \
  --recovery-operator <installed-recovery-operator> \
  --recovery-operator-uid <installed-recovery-operator-uid> \
  --reviewer-principal <installed-reviewer-principal> \
  --reviewer-fingerprint <installed-reviewer-fingerprint> \
  --policy-namespace <installed-policy-namespace> \
  --validity-window-ms <positive-at-most-86400000> \
  --output /absolute/recovery/authorization.json

bullet-family --root /absolute/family coord recovery-authorization-message \
  --authorization /absolute/recovery/authorization.json \
  --output /absolute/recovery/authorization-message.bin
```

The draft accepts only `APPROVE`. It reads the exact sealed inspection and
bootstrap provenance, requires every supplied identity to match installed
policy and operator/reviewer separation, and derives decision time, issue,
expiry, boot, and time-namespace facts from the trusted observation. No clock,
boot, namespace, private-key, repository, or result override is accepted. The
positive Unix/boottime window is equal and at most 24 hours. Immediately
before publishing, the producer re-observes that the authorization is current.

The message producer reads that sealed authorization and emits the exact
domain-separated binary signing message. It may contain NUL bytes and has no
terminal LF; do not render, re-encode, or edit it. Transfer only those exact
bytes through the admitted offline reviewer custody. The dedicated recovery
reviewer signs them outside Bullet and returns one raw 64-byte Ed25519
signature. The private key remains offline and is never supplied to this CLI,
committed, or passed to a provider. Import that raw signature without adding a
text encoding. Its input file must be mode-0400, single-link, and beneath an
exact-owner mode-0700 parent:

```bash
bullet-family --root /absolute/family coord \
  recovery-authorization-signature-import \
  --authorization /absolute/recovery/authorization.json \
  --signature /absolute/recovery/authorization-signature.ed25519 \
  --output /absolute/recovery/authorization-signature.json
```

Import verifies the exact 64 bytes against the installed reviewer public key,
the domain-separated message, the authorization digest, and installed reviewer
identity before sealing
`bullet.coord.recovery-authorization-signature.v1`. Draft, message, and import
outputs are distinct normalized absolute mode-0400 single-link files beneath
an exact-owner mode-0700 parent. Each output is create-once: an existing path,
even with equal bytes, refuses and is never replaced. Missing, duplicate,
unknown, relative, malformed, or wrong-policy input refuses; import also
refuses a wrong-length, wrong-key, or changed-body signature. Message and
import neither renew nor extend the signed window. Later recovery admission
independently refuses not-yet-valid or expired authority before mutation.
Listing these commands in the CLI usage inventory does not enable policy or
authorize recovery. The checked-in production policy still has no admitted
reviewer key or fingerprint and therefore returns `RECOVERY_POLICY_DISABLED`.

`recovery-manifest` stable-reads that authorization and inspection, rederives
the complete inspection from the three original artifacts, and refuses any
difference before creating the schema-2 manifest once:

```bash
bullet-family --root /absolute/family coord recovery-manifest \
  --inspection /absolute/recovery/inspection.json \
  --authorization /absolute/recovery/authorization.json \
  --authorization-signature /absolute/recovery/authorization-signature.json \
  --bootstrap-provenance /absolute/recovery/bootstrap-provenance.json \
  --interrupted-capture /absolute/incident/interrupted.partial \
  --tainted-generation /absolute/incident/tainted.jsonl \
  --frozen-live-source /absolute/family/.bullet-family/coord/events.jsonl \
  --output /absolute/recovery/manifest.json
```

`recover-rollover` accepts the complete authority closure again. It verifies
the sealed documents, signature, installed policy, running executable,
authority-derived manifest, and fresh source inspection. After the lower
creation-free exact-topology preflight and legacy read lease, it revalidates
boot, time-namespace, wall-clock rollback, and `CLOCK_BOOTTIME` expiry once
more immediately before stable authority may create `LOCK`, tighten the legacy
0775 root, sync, or otherwise mutate:

```bash
bullet-family --root /absolute/family coord recover-rollover \
  --manifest /absolute/recovery/manifest.json \
  --inspection /absolute/recovery/inspection.json \
  --authorization /absolute/recovery/authorization.json \
  --authorization-signature /absolute/recovery/authorization-signature.json \
  --bootstrap-provenance /absolute/recovery/bootstrap-provenance.json \
  --interrupted-capture /absolute/incident/interrupted.partial \
  --tainted-generation /absolute/incident/tainted.jsonl \
  --frozen-live-source /absolute/family/.bullet-family/coord/events.jsonl
```

Producing these documents does not authorize execution.

## Preconditions

- Work from an exact-owner family root and a clean, reviewed component subject.
  Recovery documents require 0700; the retained coordinator root may be exact
  legacy 0775 until the stable-lock authority tightens it. Do not use the live
  incident for rehearsal.
- The bootstrap must contain the independently accepted dedicated reviewer
  public key and matching fingerprint. A build without them returns
  `RECOVERY_POLICY_DISABLED`; that is the current checked-out fail-closed state.
- The trusted root supervisor must publish the boot-scoped clock expectation
  described above. Missing or untrusted custody is policy-disabled; the
  operator must not create, rewrite, or supply that record.
- Process-visible numeric root custody or a bare inherited descriptor is not
  by itself host-supervisor authority. Production enablement remains blocked
  on an admitted packaged supervisor and runtime-dependency subject; fixed host
  user, mount, time, PID, and procfs visibility; a sanitized loader/environment;
  anti-ptrace, dumpability, and same-UID injection controls; blocked capability
  and namespace creation; and an authenticated supervisor clock/descriptor
  channel. That packaging and confinement proof is absent from the current
  source subject, so the engineering packet remains component-policy-disabled
  even when its local path and descriptor checks pass.
- `CURRENT`, the Recovery manifest, the retired legacy source, and every
  retained artifact must pass their existing locked replay checks.
- Use Linux. Other platforms return `COORD_RECOVERY_PLATFORM_UNSUPPORTED`
  before document I/O or coordinator creation until native proof exists.
- Choose new normalized absolute output paths. Producer outputs are created
  once as mode-0400 regular files and are never overwritten.
- The recovery operator and the independent reviewer are different people.
  The reviewer authors the approval from the proof result; the producer does
  not accept an actor, repository, claim, commit, decision, count, or result
  override on the command line.

Stop immediately on a dirty or changed subject, `UNKNOWN`, a failed/skipped/
zero test, stale watermark, missing exact receipt, unexpected file, or typed
refusal. Preserve the files and ledger bytes. Never retry by constructing a
different semantic request under the same incident.

## Closed sequence

Set explicit absolute paths in the shell without using a provider home or
credential directory:

```bash
bullet-family --root /absolute/family coord recovery-plan \
  --output /absolute/recovery/plan.json

bullet-family --root /absolute/family coord recovery-proof \
  --plan /absolute/recovery/plan.json
```

The proof response's `projection` is the exact `rpf_<digest>` receipt ID. The
fixed in-process proof appends only PASS with six checks; there is no CLI shape
for FAILED, SKIP, UNKNOWN, zero checks, or a caller-selected result.

The independent reviewer compares the plan, retained artifacts, repository
observation, and proof result. They then create a strict RFC-8785 canonical
document containing exactly these fields and no others:

```json
{"decision":"APPROVE","evidence_subject_blake3":"blake3:<64 lowercase hex>","kind":"recovery_review_approval_v1","plan_id":"rcp_<64 lowercase hex>","proof_receipt_ids":["rpf_<64 lowercase hex>"],"reviewer":"independent-reviewer","schema_version":1}
```

The subject and plan IDs must be copied from `plan.json`; the proof list must
contain exactly the proof response ID in sorted order. The reviewer must not
equal `recovery_orchestrator` in the plan. Seal the canonical file before use:

```bash
chmod 0400 /absolute/recovery/approval.json

bullet-family --root /absolute/family coord recovery-review \
  --plan /absolute/recovery/plan.json \
  --approval /absolute/recovery/approval.json

bullet-family --root /absolute/family coord recovery-request \
  --plan /absolute/recovery/plan.json \
  --approval /absolute/recovery/approval.json \
  --output /absolute/recovery/adoption-request.json
```

Inspect the produced request before the only explicit adoption mutation. It
must bind the exact proof and review envelope offsets and the complete
post-review watermark:

```bash
bullet-family --root /absolute/family coord adopt \
  --request /absolute/recovery/adoption-request.json

bullet-family --root /absolute/family coord status --json --all
```

Restart the process and run the same status command again. Every adopted claim
must remain `recovered_receipted` with the same request, adoption, commit,
tree, proof-subject, review-subject, and complete watermark identities.

## Replay and conflict rules

- An exact proof or review retry retains the locked Recovery authority checks,
  reconstructs the pre-request full watermark, and returns the stored result
  before forensic/Git producer work, the clock, or append.
- Different canonical producer bytes under the same deterministic request ID
  return `COORD_REQUEST_CONFLICT` and do not call the clock or grow the segment.
- A stale full watermark returns `STALE_COORD_WATERMARK` before append.
- The adoption consumer byte-compares the complete canonical request on retry.
- Approval is APPROVE-only. Unknown fields, duplicate keys, noncanonical JSON,
  another mode, symlinks, relative paths, and reviewer/orchestrator equality
  refuse before mutation.
- A not-yet-valid authorization always refuses. An expired authorization may
  only verify and return `ALREADY_CURRENT` for the exact already-published
  generation; it cannot inspect a replacement subject or reach mutation.
- A signed renewal changes only boot/time-namespace execution custody and its
  Unix/boottime window. It preserves the stable APPROVE decision subject,
  decision time, manifest, and generation ID, so an active renewal can resume
  the exact exchanged or writer-wait topology. It cannot select a new
  inspection, provenance, actor, policy, or generation.
- Missing, wrong-key, changed-body, same-actor, provenance/executable mismatch,
  contradictory Cargo.lock provenance, boot/time-namespace change,
  root-expectation custody/type mismatch, wall-clock rollback, over-24-hour,
  or either-clock expiry for non-current authority remains policy-disabled or
  a typed refusal.

## Component validation

Run these from `bullet-farm` on the frozen subject:

```bash
cargo test -p bullet-family recovery_production --lib -- --nocapture
cargo test -p bullet-family --test coord_recovery_producers -- --nocapture
cargo test -p bullet-family coord:: --lib -- --nocapture
cargo test -p bullet-family \
  --test coord --test coord_receipt_scopes \
  --test coord_rollover --test coord_recovery_producers -- --nocapture
cargo test -p bullet-wire --test canonical_hostile -- --nocapture
git diff --check
```

This is unsigned component observation. DF-R7a still requires a fresh
owner-0700 rehearsal with the publication/append crash matrix and independent
bundle review. DF-R7b still requires exact live-input hash comparison and
explicit independent approval. Neither this procedure nor a green component
test authorizes the frozen real incident or advances a release profile.
