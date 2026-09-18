# Live provider conformance — admission, ratification, and the nightly lane

Status: **runbook for an operator act; no live receipt exists yet**
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-26
Component receipt baseline (minimum; replay current-head lanes before use): bullet-kernel `ba485d5`+ (signed launch grants, egress isolation, live-conformance
path), bullet-farm `bf5c642`+ (policy v1alpha2 rule), ADRs 0011 and 0012.

A future provider process may be dispatched only when Kernel admission holds a real runtime
observation, a signed launch grant bound to the durable active lease, the exact executable
digest and policy, and an egress-isolation receipt from a namespace whose default-drop ruleset
was probed before the child started. The checked-in policy
(`policy/v1alpha1/policy.json`, generation 1) disables live admission by schema rule. Even a
structurally valid v1alpha2 policy currently reaches only the default production-adapter
observation port, which refuses with `RUNTIME_PROBE_UNAVAILABLE` before operator-key read,
graph/Mission, lease, or nonce writes, egress preparation, or child spawn. The product CLI does
open its SQLite ledger before entering this orchestration; the refusal makes no claim that the
database file is absent. Nothing in this runbook changes either boundary.

## 1. What runs today without any operator act

```bash
cd ../bullet-kernel
bash ops/ci/egress.sh                       # 3/3 live isolation proofs on a Linux host with unshare/slirp4netns/nft
BULLET_LIVE_PROVIDERS=claude,codex,cursor,agy bash ops/ci/nightly.sh   # exit 78: every provider refused, zero spawns
```

The nightly positive halves target a marker script, so any spawn under the checked-in policy is
detected and fails the lane. `POLICY_LIVE_ADMISSION_DISABLED` at `POLICY` is the expected,
neutral outcome. The aggregate script returns 78 when any positive half is neutrally refused,
0 only when every selected provider returns a PONG receipt, and 1 when a refusal test, tool,
execution, or spawn check fails. The four-selector CLI matrix proves neutral 78, an exact typed
Admission refusal receipt, no operator-key file, and no spawn marker. A separate application
policy test with a production adapter proves Mission/events remain empty, `LEASE` is `NOT_RUN`,
no launch-grant nonce exists, and a rejecting egress backend is never called. Only a strict
`cfg(test)` dispatcher synthesizes the observation needed to exercise the later mint → verify
grant → egress → dispatch → canary scan → PONG machinery; that is component proof, not a real
probe.

## 2. Prepare the operator-owned candidate inputs; this is not admission

1. Generate the authority key (private half stays at `<data-dir>/authority/launch-grant.key`,
   mode 0600, never copied or committed):

   ```bash
   cd ../bullet-kernel
   cargo run --locked -p bullet -- authority keygen --data-dir /abs/path/bullet-data
   ```

   The command prints the public key and a ready-to-paste `IssuerKeyV1` JSON with
   `key_purpose = authority-signing`, `algorithm = paseto-v4.public`, `audiences = ["provider-runner"]`.

2. Prepare a candidate policy: copy `policy/v1alpha1/policy.json`, set `schema_version` to
   `v1alpha2`, `policy_generation` to `2`, append the `IssuerKeyV1` to `issuer_keys`, and set
   `sandbox_policy.live_admission_enabled` to `true`. Leave every other field unchanged;
   `route_policy.evolutionary_authority` must stay `false`. The hub validator accepts exactly this
   shape and nothing looser (`LIVE_ADMISSION_REQUIRES_GENERATION`, `LIVE_ADMISSION_REQUIRES_RUNNER_KEY`,
   `UNSAFE_POLICY` otherwise). Store it outside the repositories, e.g.
   `/abs/path/bullet-data/policy/policy.json`.

3. Follow [ADR 0013 OD-A](../decisions/0013-operator-decision-register.md) for the common-policy witness and a
   separate enrollment witness for each selected provider. The witness must bind the operator-owned path,
   owner/mode, policy digest, schema-3 trust anchor, key fingerprint/custody, signer/approver, exact executable
   path/digest, service identity, credential handle, budget, validity, revocation, and rollback.

The log line is an auditable coordination witness, never runtime authority. Before a real run, machine admission
must verify and read back every bound subject through the schema-3 trust anchor. The current loader validates the
v1alpha2 shape, but the product then refuses because no separately granted and contained real runtime probe is
implemented. The complete external anchor is also absent, so live release admission remains BLOCKED. A
self-created key/policy plus a forged operator-looking line cannot qualify a LIVE_PROOF.

## 3. Reproduce the current real-mode refusal

```bash
cd ../bullet-kernel
BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/path/bullet-data/policy/policy.json \
BULLET_LIVE_PROVIDERS=claude,codex,cursor,agy bash ops/ci/nightly.sh
```

Nightly real mode refuses to start without an absolute `BULLET_POLICY_PATH`, requires each named
provider binary on `PATH`, and canonicalizes that path internally. The direct CLI's
`--executable` is optional and otherwise resolves the provider default on `PATH`; when supplied,
the argument must be absolute and is canonicalized. With the checked-in policy either interface
stops at `POLICY`; with a structurally valid v1alpha2 policy it stops at `ADMISSION` before
operator-key read, graph/Mission, lease, or nonce writes, egress preparation, or process startup.
The command is useful only to reproduce and inspect that sealed refusal today; it cannot send a
read-only PONG turn. A single provider can be checked directly:

```bash
cargo run --locked -p bullet -- provider live-conformance \
  --data-dir /abs/path/bullet-data --provider claude --executable "$(readlink -f "$(command -v claude)")"
```

Exit 78 is the neutral designed refusal: `POLICY_LIVE_ADMISSION_DISABLED` at `POLICY` for the
checked-in policy, or `RUNTIME_PROBE_UNAVAILABLE` at `ADMISSION` for a structurally valid
v1alpha2 policy. Production adapters cannot currently return exit 0. The receipt retains all 13
steps (`POLICY`, `OPERATOR_KEY`, `LEASE`, `ADMISSION`, `MINT`, `VERIFY_GRANT`, `ADMIT_SIGNED`,
`EGRESS_PREPARE`, `ADMIT_EGRESS`, `REQUIRE_DISPATCH`, `DISPATCH`, `CANARY_SCAN`, `PONG_MATCH`).
For v1alpha1, `POLICY` is `REFUSED` and the other 12 records are `NOT_RUN`. For valid v1alpha2
with a default production adapter, `POLICY` is `PASS`, `ADMISSION` is `REFUSED`, and the other
11 records—including schema-earlier `OPERATOR_KEY` and `LEASE`—are `NOT_RUN`: the safety
observation executes early but maps to the existing `ADMISSION` slot. Once a real observation
producer is implemented, any non-neutral error remains a named failing step rather than a
skipped green.

## 4. What a receipt does and does not prove

Today production cannot produce a PONG-shaped result. PONG receipts from the strict test-only
dispatcher are component evidence, not a provider observation or LIVE_PROOF. A real read-only
probe needs its own grant and containment design; schema-3 machine admission of the external
policy/enrollment anchor and kind-specific semantic receipt registration are also still local
engineering gaps. After those controls land, an independently ratified native run may produce
LIVE_PROOF-class evidence for **provider conformance of one adapter on one host under one policy generation**:
the exact binary digest, grant, egress receipt, and policy digest are sealed in it. It is not a five-plane
transaction, forge evidence, or release evidence. The `release.provider.<name>` gates accept only that
machine-admitted and semantically registered form. Spend is bounded by the grant's budget fields (default cap
50 000 micro-USD per run).

## 5. Rollback

Delete or replace the ratified policy file and rerun the lane: every positive half returns to
`POLICY_LIVE_ADMISSION_DISABLED`. Until a real observation producer exists, a structurally valid
v1alpha2 policy refuses earlier with `RUNTIME_PROBE_UNAVAILABLE`, so the operator key is not read
and no grant is minted. A future admitted probe path must re-establish key revocation, short grant
expiry, and one-use nonce proof before it may dispatch.
