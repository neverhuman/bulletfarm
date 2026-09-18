# Setup recovery drill

Status: **drill runnable on Linux GNU; the positive setup path is refused by design (schema 2), so recovery today is diagnosis, not repair**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-26
Component receipt baseline (minimum; replay current-head lanes before use): bullet-farm `d762f86` (`src/setup.rs`, `src/setup/transaction.rs`, `src/doctor`, `src/checkout`);
transaction rules in [`source-setup.md`](source-setup.md)

Run this drill after any setup crash, refusal, power loss, or "it printed an error after it said it
published". Its purpose is to classify the durable family state without destroying evidence, then either
verify the complete family or let the same authenticated setup transaction reconcile an exact partial
publication. It never deletes.

## 1. The four recovery classes

`bullet-family setup` (`src/setup.rs::run`) validates every fallible input first (platform containment,
admitted family-root descriptor, hub location, lock schema, Cargo/Node/npm subjects, environment), then runs
one transaction (`src/setup/transaction.rs`): members are cloned into a `0700`, descriptor-relative staging
directory named `.bullet-family-setup.checkout.<pid>.<nanoseconds>.<sequence>` beside the members, published with
no-replace rename, fsynced,
and finally the outer completion marker — the family-root `repos.manifest.toml`, byte-equal to the signed hub
manifest — is published last.

| State | Definition | Evidence |
| --- | --- | --- |
| **prior** | no new member directory, no staging directory, family-root `repos.manifest.toml` unchanged | `doctor --json` `family_layout` and the directory listing are what they were before the run |
| **recoverable exact partial** | the outer manifest is absent; one or more published member directories verify as ordinary clean clones at the signed lock's exact identities; every missing member is still absent; no staging orphan or conflicting entry exists | rerunning the *same authenticated schema-3 setup* verifies and reuses each exact member without replacement, creates only missing members, then publishes the manifest last |
| **complete next** | every locked member is an ordinary clean clone at its exact OID and the family-root manifest equals the hub manifest | `doctor --json` all `PASS` and `checkout verify` clean under a schema-3 lock |
| **indeterminate / orphan** | a `.bullet-family-setup.*` directory; an unverifiable, dirty, symlinked, or conflicting member; a manifest without every exact member; or an orphan left by bounded cleanup | **preserve; do not alter or retry around it; escalate with the observed paths and typed refusal** |

An error reported *after* the manifest was linked is indeterminate until the same exact `setup` and
`checkout verify` reconcile it. An error before manifest publication is **not necessarily prior state**:
no-replace member publication happens one member at a time, so exact members can already be durable. The
transaction's crash-boundary tests prove that a same-input rerun reuses those verified identities without
replacement and reaches one complete family; they also prove staging cleanup is bounded and may deliberately
preserve an unsafe orphan instead of guessing.

## 2. The drill

Run from the hub checkout. Under the checked-in schema-2 lock, the Bullet operations below refuse before product
state mutation. `cargo run` can still update ordinary compiler output under `target/`; use an already built exact
local binary when compiler-cache writes would interfere with evidence preservation.

1. **Diagnose.**

   ```bash
   cargo run --locked --quiet --bin bullet-family -- doctor --json; echo EXIT=$?
   ```

   Observed on this host (2026-08-25): `EXIT=3`, `"status": "BLOCKED"`. `doctor` reports its verdict in
   its exit status — 0 READY, 3 BLOCKED, the family's "diagnosed, not usable" code — so a script must treat
   3 as a diagnosis to read, not as a crash. Every other exit code is a typed `CoordError`. The check ids are
   `hub_checkout`, `toolchain`, `source_metadata`, `family_layout`, `member_oids`, `clean_checkouts`,
   `exact_family_authority`. Here `hub_checkout`, `toolchain`, and `family_layout` were `PASS`;
   `source_metadata` and `exact_family_authority` were `BLOCKED` ("family.lock schema 2 is diagnostic-only";
   "the diagnostic schema-2 lock cannot authenticate an install"); `member_oids` and `clean_checkouts` were
   `BLOCKED` because sibling members sit ahead of the schema-2 lock's pins and one member had active-lane
   edits (`DIRTY_CHECKOUT`). Read the `repair` field of each blocked check; the `clean_checkouts` repair is
   explicit: "finish and hand off active claims; do not clean, reset, or stage another agent's changes".

2. **Verify the family against the lock.**

   ```bash
   cargo run --locked --quiet --bin bullet-family -- checkout verify; echo EXIT=$?
   ```

   Observed: `UNSUPPORTED_SCHEMA: family.lock schema 2 is not installable; retain it for diagnosis, regenerate
   schema 3 from authenticated signed tags, and replace it atomically`, `EXIT=4`. That is the expected negative under the checked-in lock; with an
   admitted schema-3 lock this command is the clean-family verdict. Record the refusal, do not "fix" the lock
   ([`schema-removal.md`](schema-removal.md)).

3. **Look for staging or a recoverable exact partial publication.**

   ```bash
   ls -a "$(dirname "$PWD")" | grep -F '.bullet-family-setup.' ; echo "staging dirs above (none = ok)"
   ```

   Observed: none. If one exists, it is an in-progress transaction or a cleanup-bounded orphan: leave it,
   record its name and mtime, and hand the family to the orchestrator. If no staging directory exists but
   the outer manifest is absent, do not classify the state as prior merely because setup returned an error.
   Record every present member path. With the same authenticated schema-3 lock, setup admits an existing
   member only after verifying its exact signed identity and cleanliness; it refuses conflicting state.

4. **Exercise both refused setup entrypoints without inventing bootstrap authority.**

   ```bash
   (cd /tmp && /abs/path/to/bullet-farm/scripts/setup.sh --offline); echo EXIT=$?
   ```

   Observed: `setup: SETUP_BOOTSTRAP_UNAVAILABLE: operator-pre-admitted bootstrap unavailable`, `EXIT=4`, no
   staging directory created, and no ambient Cargo shim executed. This is exactly what
   `ops/ci/setup-refusal.sh` asserts. Automation branches on the stable code, not the prose detail. It is
   intentionally earlier than
   the Rust schema check: the wrapper requires `BULLET_SETUP_ADMITTED_BIN` to name an absolute canonical,
   non-symlink executable outside the source family and also requires explicit absolute
   `BULLET_SETUP_CARGO_BIN`, `BULLET_SETUP_NODE_BIN`, and `BULLET_SETUP_NPM_CLI` values. It does not discover,
   authenticate, or resolve those subjects, and no signed prebuilt installer is published.
   The wrapper accepts only no arguments or exactly one `--offline`; every
   other tail returns `SETUP_ARGUMENT_INVALID`, `EXIT=4`, before external
   bootstrap selection. Its Linux launcher is the fixed `/bin/bash`, including
   through `just setup`, so an absent or hostile `PATH` cannot select a shell.

   The source-built binary's direct refusal is the separate schema check:

   ```bash
   cargo run --locked --quiet --bin bullet-family -- setup \
     --root /abs/path/to/family --source jeryu --offline; echo EXIT=$?
   ```

   In the canonical family this returns `UNSUPPORTED_SCHEMA`, `EXIT=4`, before tool admission or staging. An
   operator-selected external binary with all four explicit wrapper subjects would reach the same Rust ordering,
   but that selection is not signed package admission and must not be recorded as installer evidence. On a
   non-Linux-GNU host the first Rust check instead answers `UNSUPPORTED_PLATFORM_CONTAINMENT`
   ([`platform-refusal.md`](platform-refusal.md)).

5. **Confirm the hub itself is intact.**

   ```bash
   cargo run --locked --quiet --bin bullet-family -- hub check; echo EXIT=$?
   ```

   Observed: `hub-check: ok`, `EXIT=0`.

## 3. What you must not do

- Do not delete, move, or overwrite a published member, a staging directory, or an orphan to force a rerun.
  An exact partial member is intentional recoverable state and must be reused; an unverifiable member or
  staging orphan is evidence to preserve. Only the same authenticated setup transaction may distinguish
  those cases by verifying the existing identities and refusing conflicts.
- Do not `git reset`, `git clean`, or re-clone a dirty shared checkout to satisfy `clean_checkouts`; that is
  another agent's active claim. `doctor` says so in its repair text.
- Do not edit `family.lock` or the family-root `repos.manifest.toml` by hand; both are authority inputs and
  the transaction compares the root manifest byte-for-byte with the signed hub manifest.
- Do not treat `doctor` as repair: it diagnoses and grants no install authority.

## 4. Limits of this drill

- The positive path (a real two-run setup in a fresh home ending in complete-next state) and the injected
  boundary-recovery matrix are component proof only: the fixtures use `LocalTransport` and test-only
  validators/verifiers (`tests/setup.rs`, `tests/setup_signed.rs`, `src/setup/transaction/tests.rs`), not
  production `JeryuTransport`/`SetupValidator`. No crash-injected recovery has been run from tagged bytes on
  a fresh host; gate `release.installer-twice` is `BLOCKED`.
- With the schema-2 lock, `checkout verify` cannot return a clean verdict on any host, so the "complete next"
  branch of this drill is currently unreachable in the canonical family. That is honest, not a defect of the
  drill.
- Same-UID mutation during the path-based Git child remains outside the descriptor boundary
  ([`source-setup.md`](source-setup.md)); a drill cannot detect that race after the fact.
