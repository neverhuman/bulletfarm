# ADR 0020: W11 test-inventory baseline correction

Status: Accepted (DESIGNED; no implementation bytes)
Owner: Bullet Farm maintainers
Related: [0017 proof annex](0017-catalog-type-expression-proof-annex.md),
[0019 proof support correction](0019-w11-proof-support-correction.md)

## Context

Accepted ADR 0017 gives W11-ABC-i exclusive custody of its CI count and identity-digest constants,
but its exact implementation packet authorizes only the final Bullet Wire and total repins. It does
not authorize repairing a pre-existing Bullet Hub inventory drift. Two independent adjudications
therefore refused a six-constant edit without a narrow governance predecessor.

The frozen derivation subjects are:

| Subject | SHA-256 | LOC |
| --- | --- | ---: |
| `ops/ci/test-partitions.sh` | `dd647870cd88a117dbbb33900c10099cb6735246390bbbb615baf157f4697371` | 85 |
| `ops/ci/lib.sh` | `deab8c2b4ad4252aa075f5f6848c02ffe4b097134aa03093667594b4461ea5a4` | 460 |

The unchanged script runs `cargo nextest list --locked --workspace` separately for `all()`, the
frozen Hub filter, and the frozen Wire filter; projects matching test cases to
`<suite><TAB><identity><LF>` with its exact jq expression; and applies `LC_ALL=C sort -u`. The
current `lib.sh` literals are historically stale:

| Partition | Count | Identity SHA-256 |
| --- | ---: | --- |
| Hub | 492 | `253b0d4919da8db17d409349df7f175f643b77b89d4dabe00652fe52ae17b2e3` |
| Wire | 127 | `198de5ee3792b4f150d7a99194991d03682983d59d384d30c8e912b0775fb724` |
| Total | 619 | `9568c839b70e3ebfcd5abafe4fd6e6fe2dbca34a16a1207f452a902ad13bb7f9` |

Subtracting the exact frozen 19-row W11 delta from the live final enumeration reconstructs these
pre-W11 identity subjects:

| Partition | Count | Identity SHA-256 |
| --- | ---: | --- |
| Hub | 506 | `d2eb3a7fb3faf71f57c5f22c673d46e1c3fe19c72578467b62862e09b29e4dcf` |
| Wire | 200 | `ad537cf4f97cad05bbe634a54db46d438d9e3f6664db49816328a402c08838c8` |
| Total | 706 | `99e549781b165406993bde3eae6306857cb0b01c799d22f604f66ff96b6c8268` |

W11 A/B/C adds exactly 19 Wire identities and zero Hub identities. Their raw-sorted LF-final TSV
is 1,709 bytes with SHA-256
`fe11617d398bc649e0193c8508d84dd9fb147cb5125655dd3a59706a096aaa56`. Removing exactly those
rows from the final inventory reconstructs the pre-W11 Wire and total subjects above while leaving
Hub byte-identical. This proves the Hub change is historical reconciliation, not a W11 test claim.

## Decision

This decision narrowly supersedes only ADR 0017's assumption that Hub would remain unchanged during
W11-ABC-i. It authorizes W11-ABC-i to replace exactly the six count/digest literals in
`ops/ci/lib.sh`, atomically with the already-authorized A/B/C/D/I implementation union, with:

| Literal | Exact final value |
| --- | --- |
| `HUB_EXPECTED_TESTS` | `506` |
| `WIRE_EXPECTED_TESTS` | `219` |
| `TOTAL_EXPECTED_TESTS` | `725` |
| `HUB_EXPECTED_IDENTITIES_SHA256` | `d2eb3a7fb3faf71f57c5f22c673d46e1c3fe19c72578467b62862e09b29e4dcf` |
| `WIRE_EXPECTED_IDENTITIES_SHA256` | `36d5664975114e007735e214a3392443a23cb50c343d795e14f96fdb2275815d` |
| `TOTAL_EXPECTED_IDENTITIES_SHA256` | `3a0dd2c3388ab80149297aaae19d34921199193d6650af0cdd976be6edd949d1` |

All six values form one indivisible subject. A four-value Wire/total edit, a Hub-only commit, or any
intermediate green claim is forbidden. This decision authorizes no filter, jq projection, sorting,
test identity, test body, ignored-state, zero-suite allowlist, detector, hostile, path, or other
constant change. It grants no general future inventory repin. Any derivation-subject byte drift,
test-identity drift, count drift, digest drift, overlap, ignored test, uncovered identity, or zero-
suite drift reopens this decision before `lib.sh` may change.

## Mandatory proof and sequencing

Before acceptance, two independent read-only reviews must reproduce the scope analysis, exact
pre-W11/final values, and evidence ceiling. The accepted decision lands alone before `lib.sh` is
edited. Then the atomic W11 implementation proof must:

1. generate all three nextest JSON listings twice from the frozen source and obtain byte-identical
   raw-sorted TSV subjects;
2. prove JSON-declared counts equal TSV line counts, ignored count is zero, Hub/Wire intersection is
   empty, their union equals all 725 identities, and exactly the three frozen zero suites remain;
3. prove the 19-row delta is Wire-only, has the exact byte count and digest above, and reconstructs
   the exact pre-W11 subjects when removed;
4. run unchanged `ops/ci/test-partitions.sh` successfully after the six literal replacements and
   retain its count-neutral identity-substitution refusal; and
5. keep `ops/ci/lib.sh` below 500 physical lines and co-land it only in the reviewed A/B/C/D/I
   staged union.

Failure or ambiguity leaves the constants unchanged and W11 on HOLD.

## Evidence ceiling

Acceptance is **DESIGNED** governance only. It creates no test result, key, authority, Candidate,
Evidence, receipt, transaction, provider run, dogfood admission, live fact, or release eligibility.
The complete W11 union remains at most **COMPONENT_ONLY** after its full proof, and strict decoder
execution remains **UNPROVED** until authenticated W11-D.
