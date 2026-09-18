# Dogfood execution plan (v0 read-only, then v1 writing)

Status: **PROPOSED — planning artifact only. Claims no source path and requests no lane admission.**
Written: 2026-08-28 by `claude-dogfood-governance`
Authority: none. This page loses to `bullet-family check release --json`, to
[ADR 0015](../decisions/0015-dogfood-track.md), to
[ADR 0013](../decisions/0013-operator-decision-register.md), and to any independent review verdict.
Relationship to the frontier freeze: this document is written *under* the 2026-08-28T04:45Z freeze and
does not cross it. Nothing here is a successor feature lane; every packet below is a proposal awaiting
both the freeze lifting and a path-exact claim.

## 1. Why this page exists

Three separate audits, two independent HOLD verdicts, and one accepted containment packet have each
described part of the road to a dogfood run. Nobody has written the whole road down in one place with
current facts. This page does that, and states plainly which parts are already someone else's.

It also records a scope disagreement and its resolution. Two audits — the read-only driver audit
(2026-08-28T03:29Z) and the composition audit (04:31Z) — hold that the first packet **must not call
`bullet-gitd`, Runner gate execution, local-bare forge delivery, integration, observation, or
settlement**. A dogfood run that ships a change asserts every one of those. Rather than argue the point,
this plan **stages it**: a read-only v0 that both audits admit, and a writing v1 behind named closures.

## 2. Where a run actually stops today

Corrected against source, because several records disagree with the code:

| Operator state | Stops at | Code | Exit |
| --- | --- | --- | --- |
| No data dir (this host, now) | before step 1 | `POLICY_UNAVAILABLE` | 1 |
| Shipped v1alpha1 policy | step 1 `POLICY` | `POLICY_LIVE_ADMISSION_DISABLED` | 78 |
| Valid v1alpha2, no enrollment | step 2 `ENROLLMENT` | `ENROLLMENT_MISSING` | 78 |
| Everything else satisfied | **step 7 `PROBE_EXECUTION`** | **`RUNTIME_PROBE_UNAVAILABLE`** | 78 |

The last row is the one that matters, and it is **not** at `ADMISSION` as several records state. The
cause is `crates/application/src/live_conformance/probe_steps.rs:340-343`: `claude_probe` is
`#[cfg(not(test))] -> Err(unavailable("claude"))` because `bullet-harness-claude` is only a
`[dev-dependencies]` entry of `bullet-application`. Step 8 `RUNTIME_ADMISSION` is therefore **unreachable
in a production build**; `RUNTIME_PROBE_NOT_ADMISSIBLE` is observable only under `cfg(test)`.

`LiveStep::ALL` has **18** steps. `docs/architecture.md` still describes 13.

## 3. What exists, what is fail-closed, what is absent

**Exists and independently accepted:**

- The `DogfoodReadOnlyV0` transcript profile — a strict *parser* with closed tool-lifecycle identity.
  It has no exported process-execution path.
- Bubblewrap filesystem containment (`crates/harness-egress/src/filesystem.rs` + `filesystem/`), accepted
  2026-08-28T04:36Z. Descriptor-only argv, 22 typed denial reasons, FD-identity drift detection.

**Exists but fail-closed or unreachable:**

- The entire credential path inside containment: the field, builder, bind, env var and destination
  constant are all dead behind a blanket refusal, and re-killed by a second `let credential = None`.
- `PreparedSandbox::filesystem_command` — public, compiles, **zero callers, zero tests**. The
  nsenter-into-bwrap nesting has never been executed.
- `with_environment` on the containment profile accepts only the value that is already its default.

**Absent — all six:** a `dogfood` subcommand; any `DOGFOOD_RUN` symbol in Rust source; `check dogfood`;
a `GateProgram`/tool-pin concept; more than one entry in the sealed gate catalog; and any population of
`credential_targets`, which is `vec![]` in production.

## 4. Target v0 — a real model turn we can trust

One real read-only Claude turn inside the accepted containment yields exactly one validated
`PatchProposal` and a create-once receipt. **No writes.** Every Candidate, SCM, effect, verification,
live, profile and release eligibility bit is false *by type*, not by convention.

Exit contract, adopted from the driver audit: **78** designed neutral non-admission, **1** failure,
**0** only for a valid receipt.

| ID | Packet | Owner | Size |
| --- | --- | --- | --- |
| V0-1 | Fallible dispatch seam + process-tree supervision | **claimed** by the composition-audit lane | — |
| V0-2 | `crates/harness-claude/src/dogfood.rs` — bounded read-only argv, dogfood profile, enrolled version, one validated proposal | free | M |
| V0-3 | `crates/application/src/dogfood.rs` — strict intent + create-once receipt | free | M |
| V0-4 | Credential projection into the sandbox | **unowned; v0's true blocker** | L |
| V0-5 | Wire containment into the live dispatch path | unowned | M |
| V0-6 | Close the containment proof honestly | unowned | S |

### V0-1 is not ours, and it fixes a real bug

`PreparedSandbox` places entered commands in the *namespace holder's* process group, while `capture_turn`
kills `-child_pid` assuming the child leads its own group. Under a timeout a descendant can retain
stdout/stderr while the reader join waits, until the outer sandbox later drops. Sequence behind this
packet; do not race it.

### V0-2 — the read-only dispatch path

New file only. `src/dispatch.rs` and `src/session.rs` keep `ConformanceV1`; the frozen conformance path
must not move.

Argv: `-p <prompt> --output-format stream-json --verbose --permission-mode plan --tools Read,Glob,Grep
--json-schema <proposal schema> --max-budget-usd <cap>`, plus `--strict-mcp-config
--disable-slash-commands --setting-sources ""`. Parse with `TranscriptProfile::DogfoodReadOnlyV0` and the
**enrolled** runtime version, never the frozen observed constant.

Reuse rather than rebuild: `proposal::schema_source()` already yields the schema that
`harness-antigravity` feeds to `--json-schema`; `ArgvBuilder::build_with_admission`, `capture_turn` and
`scan_events` are the existing chokepoints.

Fix while here: the turn reduction currently discards cost, wall time, native session id and three
artifact digests.

Negative tests that keep it honest: argv contains exactly the admitted flags and nothing else; a tool
outside the allowlist refuses; a wrong enrolled version poisons at `system/init`; a missing
`structured_output` is a typed failure; two proposal carriers are ambiguous and one is refused; zero
proposals never becomes a fabricated empty proposal; a canary planted in the parent environment never
appears in captured events; the kill switch refuses before spawn.

### V0-3 — intent and receipt

Strict `DogfoodReadOnlyIntentV0` and create-once `DogfoodReadOnlyReceiptV0`, written through the existing
descriptor-bound create-once storage. **Do not retrofit the `bullet-command-worker` receipt** — it is
closed around `run_demo`, a simulator transcript, Candidate artifacts and UNKNOWN settlement.

Negative tests: the serialized record carries no key matching `sig|signature|mac|seal`; its class is none
of the nine release receipt kinds; a body claiming `true` on any eligibility flag is **refused, not
accepted**; an unknown field is refused; writing over an existing path fails; the mode is exactly 0600.

### V0-4 — credential projection, the blocker

The accepted containment refuses all credential paths at **four independent layers**: a blanket refusal
before any file is opened; a second `let credential = None` that makes deleting the first a silent no-op;
a `uid != 0` requirement; and an ancestor-custody requirement. Claude's OAuth files are uid 1000 under
uid-1000 directories, so they fail three of the four. They also cannot be smuggled in as runtime files —
those destinations are restricted to a closed allowlist.

**Reuse, do not rebuild.** `crates/harness-core/src/admission/credentials.rs` already implements exactly
"admit these files and nothing else": a grant carrying source, target and expected digest; staging into a
fresh owner-private HOME; bounded file count and size; and a receipt that deliberately omits the host
source. The two subsystems do not know about each other. The cheapest correct path is to stage with the
existing helper and bind the resulting private HOME into the sandbox, rather than growing a second
credential model inside the containment crate.

Two constraints to design around: the existing binds are read-only, and Claude rewrites its credential
file on token refresh; and the containment's validation module and its integration test are both within
forty lines of the authored-file cap, so this change will require a module split.

### V0-6 — make the containment proof falsifiable

The accepted canary is real but has three weaknesses worth closing: it tests existence rather than
attempting a read; it is capability-gated twice and **returns silently** when namespaces are unavailable,
so in a restricted environment it passes without asserting anything; and there is no uncontained control,
so it demonstrates the projection is closed without falsifying "the fixture would have passed anyway."

## 5. Target v1 — the writing half

Gated on the four closures in §6. Packets: the gate-program and tool-pin concept; both gate executors;
three scoped gates; preservation and gate context on the attempt; the adapter bridge; the driver;
candidate-ref-only delivery; and the `DOGFOOD_RUN` record with its hub refusal proof.

`crates/domain/src/gates.rs` is clean, unclaimed and untouched since 2026-08-25 — the least contended
piece of the whole objective. `bullet-git` is clean and uncontested, and already exposes the apply
operation the writing leg needs.

## 6. The four closures that gate v1

1. **Credential projection / broker custody** — §V0-4. Unowned.
2. **A distinct runner service identity** over the peer-authenticated daemon socket. Missing: the RPC for
   kernel-side probe and launch-grant mint and read-back, a dogfood command subject, and the binding from
   a kernel-selected command to Mission, package, Variant and Attempt. Two specific defects: the heartbeat
   *records* a freeze but does not kill an in-flight synchronous turn, and the live-conformance path
   acquires an Attempt but never terminally releases it — `Succeeded` means a Candidate, so an honest
   proposal-only terminal state is needed.
3. **A durable vector budget.** The launch issuer's budget reservation is only a hash of maxima: there is
   no durable reservation, invocation debit, cost or wall settlement, or unknown-usage retention.
4. **A genuine provider-conformance turn** preceding dogfood. Enrollment, `--version` output and the
   dogfood parser must never be converted into conformance evidence. Requires promoting the Claude harness
   from a dev-dependency and un-gating the probe — but **paired** with an offline command mode that shares
   no network and prepares no proxy, because today's probe prepares the normal proxy boundary and so its
   denial label does not prove the proxy route was absent.

## 7. Three defects that exist right now

Harmless only because the single catalog gate is `grep`, which emits nothing and ignores its environment.
Each becomes live the moment a real gate runs.

1. **A reachable panic.** The gate output truncator slices a `String` at a byte offset that need not be a
   character boundary. Clippy diagnostics routinely contain multi-byte characters.
2. **A credential leak into the gate child.** The runner builds the gate command with no environment
   clearing, so a cargo gate would execute build scripts and procedural macros with every provider
   credential in scope.
3. **A false failure after a green run.** The post-gate untracked-file check runs without excluding
   ignored paths, so a passing test run that writes a build directory into the worktree fails the attempt
   immediately afterwards.

One correctness note in the other direction: the containment binds the workspace **read-only**, which is
right — [ADR 0001](../decisions/0001-provider-execution-mode.md) makes providers proposers — and means v0
needs no change there.

## 8. Documentation drift to correct in passing

`docs/architecture.md` describes 13 live steps against an 18-step enum; several records place the
production refusal at ADMISSION rather than PROBE_EXECUTION; `docs/cli.md` exposes no dogfood command;
`docs/testing.md` documents only the synthetic simulator dogfood, which must never be reused as evidence
for a live one; and `docs/egress-isolation.md` does not describe the filesystem containment at all.

## 9. Verification discipline

Focused crate proof first, then the owning repository's fast lane under `umask 077` in a private cargo
target directory — never inside `.git`, and never the hub's `check fast|required`, which refuses on any
dirty checkout. Use the package-scoped formatter; a bare formatter invocation on a crate root follows the
module tree and reformats other lanes' files.

A future read-only dogfood lane runs only credential-free component subjects. A host lane missing any of
the identity, broker, provider, certification or reservation inputs **exits typed neutral 78 and never
counts green.**

The test inventory gates everything: five pinned counts plus six identity digests, partitions that must
sum exactly and stay pairwise disjoint, and digest drift on even a count-neutral rename. Counts can only
be re-derived on a clean tree.

## 10. What this plan will not do

Cross the freeze. Convert enrollment, version output or the parser into conformance evidence. Flip a
release gate. Add a dogfood variant to the release gate class or receipt-kind vocabulary. Sign an
observation with the launch-grant key. Retrofit the demo worker receipt. Touch the Jeryu forge. Let a
model choose gate arguments. Let Bullet merge into a real repository — it delivers a candidate ref and a
human merges.
