# Live provider conformance — admission, ratification, and the nightly lane

Status: **runbook for an operator act; no live receipt exists yet**
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-25
Component receipt baseline (minimum; replay current-head lanes before use): bullet-kernel `ba485d5`+ (signed launch grants, egress isolation, live-conformance
path), bullet-farm `bf5c642`+ (policy v1alpha2 rule), ADRs 0011 and 0012.

A provider process is dispatched only when Kernel admission holds two pieces of evidence — a
signed launch grant bound to the durable active lease, the exact executable digest and the
policy, and an egress-isolation receipt from a namespace whose default-drop ruleset was
probed before the child started — and only when the loaded policy enables live admission.
The checked-in policy (`policy/v1alpha1/policy.json`, generation 1) does not, by schema rule.
Nothing in this runbook changes that file.

## 1. What runs today without any operator act

```bash
cd ../bullet-kernel
bash ops/ci/egress.sh                       # 3/3 live isolation proofs on a Linux host with unshare/slirp4netns/nft
BULLET_LIVE_PROVIDERS=claude,codex,cursor,agy bash ops/ci/nightly.sh   # exit 0: every provider refused, zero spawns
```

The nightly positive halves target a marker script, so a spawn under the checked-in policy is
detected and fails the lane. `POLICY_LIVE_ADMISSION_DISABLED` is the expected, neutral outcome.
The fake-provider path (mint → verify grant → admit signed → egress sandbox → admit egress →
dispatch → canary scan → PONG match) is exercised by the Kernel test suite with a test-only
policy seam; the production loader never accepts such a policy.

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
v1alpha2 shape but does not yet implement that complete external anchor, so live release admission remains
BLOCKED. A self-created key/policy plus a forged operator-looking line cannot qualify a LIVE_PROOF.

## 3. Run the real conformance lane

```bash
cd ../bullet-kernel
BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/path/bullet-data/policy/policy.json \
BULLET_LIVE_PROVIDERS=claude,codex,cursor,agy bash ops/ci/nightly.sh
```

Real mode is explicit: it refuses to start without an absolute `BULLET_POLICY_PATH`, resolves each
provider binary to an absolute symlink-free path, and keeps receipts under
`target/live/<provider>/<utc>/`. Each provider gets one turn — "Reply with the single word PONG and
nothing else." — in read-only mode inside the egress sandbox under a grant that expires within
15 seconds. A single provider can be run directly:

```bash
cargo run --locked -p bullet -- provider live-conformance \
  --data-dir /abs/path/bullet-data --provider claude --executable "$(readlink -f "$(command -v claude)")"
```

Exit 0 = PONG-shaped diagnostic output until the local anchor-admission and semantic-registration work below
lands; 78 = policy refusal (neutral); anything else = a named failing step in the
receipt (`POLICY`, `OPERATOR_KEY`, `LEASE`, `ADMISSION`, `MINT`, `VERIFY_GRANT`, `ADMIT_SIGNED`,
`EGRESS_PREPARE`, `ADMIT_EGRESS`, `REQUIRE_DISPATCH`, `DISPATCH`, `CANARY_SCAN`, `PONG_MATCH`). A step that
did not run is `NOT_RUN`, never omitted. Reason codes: `LAUNCH_GRANT_*`, `POLICY_*`,
`ADMISSION_REFUSED`, `SECRET_CANARY_EXPOSURE`, `EGRESS_*`.

## 4. What a receipt does and does not prove

Today a PONG-shaped result is **unregistered diagnostic input**, not LIVE_PROOF: schema-3 machine admission of
the external policy/enrollment anchor and the provider receipt's kind-specific semantic registration are still
local engineering gaps. After those controls land, an independently ratified native run may produce
LIVE_PROOF-class evidence for **provider conformance of one adapter on one host under one policy generation**:
the exact binary digest, grant, egress receipt, and policy digest are sealed in it. It is not a five-plane
transaction, forge evidence, or release evidence. The `release.provider.<name>` gates accept only that
machine-admitted and semantically registered form. Spend is bounded by the grant's budget fields (default cap
50 000 micro-USD per run).

## 5. Rollback

Delete or replace the ratified policy file and rerun the lane: every positive half returns to
`POLICY_LIVE_ADMISSION_DISABLED`. Revoking the key (`revoked_at_unix_ms` in `issuer_keys`) has the
same effect through `LIVE_ADMISSION_REQUIRES_RUNNER_KEY`. Grants already minted expire within 15 s
and their nonces are single-use.
