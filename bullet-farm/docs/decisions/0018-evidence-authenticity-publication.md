# ADR 0018: Evidence authenticity and publication

Status: Accepted (DESIGNED; no implementation bytes)
Owner: Bullet Farm maintainers
Related: [0016 (legacy contract semantic closure)](0016-legacy-contract-semantic-closure.md), [0017 (catalog type-expression vocabulary)](0017-catalog-type-expression-vocabulary.md), and [0013 (operator decision register)](0013-operator-decision-register.md)

## Context and hard predecessors

ADR 0016 freezes the unsigned Evidence body and custody semantics. It leaves signer custody, raw gate observation, caller-free derivation, publication, replay, and release admission here. Current catalog Evidence/Proof records remain open, VerificationIntent is unsigned, and Kernel's fixture chain shares process/key custody and has no durable publication or restore authority. None is admissible Evidence.

The component dependency order is exact: accept corrected ADR 0017; land dormant W11 while its four legacy sentinels and 40 Rust/40 TypeScript open leaves remain byte-identical; accept and encode ADR 0016 LC-A through LC-D plus SD-B semantics; then let W12 atomically remove every public open leaf and publish zero-open component bytes. ADR 0016 remains authoritative for EvidenceV1, GateOutcome, EvidenceTier, and legacy semantic codes; a conflict is a HOLD, never an implicit wire change.

After ADR 0017 acceptance, this ADR may be independently accepted at **DESIGNED** once its own reviews pass; design acceptance neither waits for nor performs W11, W12, OD-L, signer activation, or final-lock admission. W12 may later encode accepted ADRs 0016–0018 and remains at most COMPONENT. Only after W12 creates the exact family-source bytes can authenticated OD-L approval, signer policy, final lock, activation, live use, or release proceed. These accepted design bytes create no wire, key, Evidence, ProofBundle, transaction, GateReceipt, or release eligibility.

## Canonical wire and bounds

`JCS(x)` is RFC 8785. Objects reject duplicate/unknown fields recursively; integers are exact IEEE 754 safe integers; text is NFC UTF-8. Catalog declarations and semantic sets sort by raw UTF-8 bytes. Displayed field order is schema/binding order only; JCS object keys always sort lexically.

The sole framed hash is `H(d,b)=BLAKE3("bullet-wire.v1\0" || LE64(|d|) || d || LE64(|b|) || b)`, where `\0` is one NUL byte. `Digest` is its 64 lowercase hex; `TaggedDigest` is `blake3:<Digest>`. `typed(p,d,b)=p+"_"+hex(H(d,b))`. No raw concatenation or caller-selected domain is valid.

Existing ADR 0016 IDs retain their prefixes. New full-width IDs are `VerificationFamilySourceSubjectId=vfs`, `VerificationSignerPolicyId=vsp`, `VerificationRetainedKeyIndexId=vki`, `VerificationIntentId=vfi`, `IdentityObservationId=ido`, `ArtifactOwnershipObservationId=aow`, `RuntimeObservationId=vro`, `GateExecutionResultId=ger`, `EvidenceLifecycleTransitionId=elt`, `ExposureSubjectId=exs`, `ApprovalReceiptId=apr`, `AuditLeafId=aul`, and `VerificationPublicationId=vpb`. `VerificationNonce` is `non_` plus 64 lowercase hex generated from 32 Kernel CSPRNG bytes. `ServiceIdentityId` is `svc_` plus 64 lowercase hex. Callers supply none of these authority/result identities.

`Timestamp` is Unix milliseconds in `0..=9007199254740991`; positive safe integers start at one; `ServiceUid` and `ServiceGid` are `1..=4294967294`; `ExitCode` is signed 32-bit; `Signal` is `1..=127`; `KeyId` is 1..128 ASCII bytes matching `[A-Za-z0-9][A-Za-z0-9._:/-]{0,127}`. Versioned records begin with `schema_version:"v1alpha1"`; embedded records forbid it. Required nullable fields are never omitted.

Short carrier payloads are at most 48,000 bytes, footer bytes at most 512, and compact tokens at most 65,536 ASCII bytes. Proof payloads are at most 524,288 and compact Proof tokens at most 700,416. With `B64(n)=4*floor(n/3)+{0,2,3}[n mod 3]`, v4.public length is exactly `10+B64(payload+64)+1+B64(footer)`; maxima are 64,780 and 699,830 respectively. Equality passes; one byte over any body/footer/token bound refuses before allocation or authentication.

## Acyclic family source and signer policy

```text
VerificationRepositorySourceRefV1 (E)
  repository:bullet-farm|bullet-git|bullet-kernel|bullet-portal
  commit_oid:GitOid; tree_oid:GitOid
VerificationFamilySourceSubjectV1 (V)
  verification_family_source_subject_id:VerificationFamilySourceSubjectId
  repositories:array<VerificationRepositorySourceRefV1,exactly 4>
  contract_catalog_digest:Digest; schema_bundle_digest:Digest
  generated_rust_digest:Digest; generated_typescript_digest:Digest
  toolchain_subject_digest:Digest; jeryu_subject_digest:Digest; provider_catalog_digest:Digest
  policy_snapshot_digest:Digest; policy_generation:PositiveSafeU64
```

Repositories occur exactly once in raw-name order and bind tagged commit/tree OIDs. The ID is `typed("vfs","bullet.verification-family-source.v1",JCS(body without ID))`; the full subject digest is `H("bullet.verification-family-source-body.v1",JCS(body))`. It and its referenced PolicySnapshot contain no family-lock, verification-signer-policy, OD-L, activation-receipt, or self digest.

Signer purposes are declared in this exact raw-ASCII order:

```text
artifact-ownership-observation-signing
identity-observation-signing
verification-evidence-signing
verification-intent-signing
verification-proof-bundle-signing
```

```text
VerificationSignerKeyV1 (E)
  key_id:KeyId; purpose:VerificationSignerPurposeV1; algorithm:"paseto-v4.public"
  principal_id:PrincipalId; service_identity_id:ServiceIdentityId; registered_uid:ServiceUid
  custody_subject_digest:Digest; public_key_lower_hex:Code<64>
  activates_at_unix_ms:Timestamp; expires_at_unix_ms:Timestamp
  revoked_at_unix_ms:Timestamp|null; retain_until_unix_ms:Timestamp
VerificationSignerPolicyV1 (V)
  verification_signer_policy_id:VerificationSignerPolicyId
  family_source_subject_id:VerificationFamilySourceSubjectId; family_source_subject_digest:Digest
  policy_snapshot_digest:Digest
  policy_generation:PositiveSafeU64; signer_policy_generation:PositiveSafeU64
  operator_decision_digest:Digest; activates_at_unix_ms:Timestamp; expires_at_unix_ms:Timestamp
  signer_keys:array<VerificationSignerKeyV1,5..40>
```

After W12, OD-L binds the family-source ID/digest and every key's purpose, fingerprint, principal, service, UID, custody subject, lifecycle, rotation, and rollback; it never names the final policy or lock. OD-L is authenticated only by a separately accepted `SignedVerificationOperatorApprovalV1` under `VerificationOperatorApprovalSignerPolicyV1`, fixed purpose `verification-operator-approval-signing`. That predecessor must freeze a purpose-fixed carrier and policy that bind the canonical OD-L body, exact source ID/full digest, validity/rollback, two distinct authorized approver principals and keys, key lifecycle/revocation, current-policy selection, signature/read-back, and cross-purpose refusal. `operator_decision_digest` is exactly that authenticated carrier's envelope digest. A Markdown row or raw/content digest is provenance only and can never populate it.

Only after that carrier verifies may policy ID be `typed("vsp","bullet.verification-signer-policy.v1",JCS(policy without ID))` and full policy digest be `H("bullet.verification-signer-policy-body.v1",JCS(policy))`. Last, the schema-3 family-lock external subject binds exact source ID/digest, authenticated OD-L envelope digest, policy path/ID/full digest/generations/window, and active PolicySnapshot. A later activation receipt binds the final lock but is never embedded back into source, OD-L, or policy. No signer activates before this complete read-back. This is the only construction order; until the separate approval-carrier decision and implementation are accepted, signer activation, final lock, live use, and release are cryptographically impossible.

Policy and key windows are nonempty and half-open. Keys sort by raw `(purpose,activates_at,key_id)`; each purpose has 1..8 nonoverlapping entries and exactly one selected entry. Require `activates < expires <= retain`; revocation is null or in `[activates,expires)`. Public material decodes to 32 nonzero bytes. Across all retained policy history, key IDs and decoded material are globally unique and never reused by another purpose or authority/release/audit/time/dogfood key; admission checks an immutable externally anchored retained-key index, not only the current policy. Concurrent selected purposes use five pairwise-distinct principals, services, UIDs, processes, key stores, and custody subjects; a rotation may retain only its purpose role while using a new key ID/material. Lookup by `(purpose,issuer,key_id)` returns exactly one retained entry. Secret bytes are never wire fields.

## Five purpose-fixed PASETO carriers

```text
SignedVerificationIntentV1 | SignedIdentityObservationV1 |
SignedArtifactOwnershipObservationV1 | SignedEvidenceV1 (V)
  issuer:ServiceIdentityId; key_id:KeyId; paseto:PasetoV4PublicShort
SignedProofBundleV1 (V)
  issuer:ServiceIdentityId; key_id:KeyId; paseto:PasetoV4PublicProof
```

Footer bytes are exactly JCS of `{"issuer":"<issuer>","key_id":"<key_id>","purpose":"<purpose>","schema_version":"v1alpha1"}`; that lexical key order is mandatory. Outer schema/issuer/key byte-equal the authenticated footer. Payload is exact JCS of the named claims. Purpose/assertion/envelope-domain rows are:

| Purpose | Implicit assertion | Envelope domain |
| --- | --- | --- |
| `artifact-ownership-observation-signing` | `bullet-farm.artifact-ownership-observation.v1alpha1` | `verification.artifact-ownership-observation-envelope.v1alpha1` |
| `identity-observation-signing` | `bullet-farm.identity-observation.v1alpha1` | `verification.identity-observation-envelope.v1alpha1` |
| `verification-evidence-signing` | `bullet-farm.evidence.v1alpha1` | `verification.evidence-envelope.v1alpha1` |
| `verification-intent-signing` | `bullet-farm.verification-intent.v1alpha1` | `verification.intent-envelope.v1alpha1` |
| `verification-proof-bundle-signing` | `bullet-farm.proof-bundle.v1alpha1` | `verification.proof-bundle-envelope.v1alpha1` |

`envelope_digest=H(domain,UTF8(compact_paseto))`; it never hashes an outer record or unsigned body. The private primitive uses fixed RFC-vector secret `b4cbfb43df4ce210727d953e4a713307fa19bb7d9f85041438d9e11b942a37741eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2`, public `1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2`, issuer `svc_aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa`, and key `verification-fixture-key-1`. These literal payload/token/envelope goldens are immutable:

```text
artifact-ownership-observation-signing | {"fixture":"artifact-ownership-observation-signing","schema_version":"v1alpha1"} | v4.public.eyJmaXh0dXJlIjoiYXJ0aWZhY3Qtb3duZXJzaGlwLW9ic2VydmF0aW9uLXNpZ25pbmciLCJzY2hlbWFfdmVyc2lvbiI6InYxYWxwaGExIn1KQY19ytLc92hCfDMG7awp94VuFpeDCbTBW-XvKSfOsOzP60MYX1RChA-2zYW04DzSxt_XFYlq6zMgNhxXzKUE.eyJpc3N1ZXIiOiJzdmNfYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSIsImtleV9pZCI6InZlcmlmaWNhdGlvbi1maXh0dXJlLWtleS0xIiwicHVycG9zZSI6ImFydGlmYWN0LW93bmVyc2hpcC1vYnNlcnZhdGlvbi1zaWduaW5nIiwic2NoZW1hX3ZlcnNpb24iOiJ2MWFscGhhMSJ9 | 0325bc700c5b7bb2ab5b97469dac7f581bf1fa30c193a305e1eb80490f8d3ac4
identity-observation-signing | {"fixture":"identity-observation-signing","schema_version":"v1alpha1"} | v4.public.eyJmaXh0dXJlIjoiaWRlbnRpdHktb2JzZXJ2YXRpb24tc2lnbmluZyIsInNjaGVtYV92ZXJzaW9uIjoidjFhbHBoYTEifXTt28ged02K_q_-r1mnfH-rdAuhcva71qFbtf-e1P_M6AeOsa-uJcndhZsxXh6D7PMjkN8MltqwhBimsLU3vwA.eyJpc3N1ZXIiOiJzdmNfYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSIsImtleV9pZCI6InZlcmlmaWNhdGlvbi1maXh0dXJlLWtleS0xIiwicHVycG9zZSI6ImlkZW50aXR5LW9ic2VydmF0aW9uLXNpZ25pbmciLCJzY2hlbWFfdmVyc2lvbiI6InYxYWxwaGExIn0 | 7fabb7aa7ee97d5b344356157fe45ddbbadb9e97ca0ae2f595979963fa077748
verification-evidence-signing | {"fixture":"verification-evidence-signing","schema_version":"v1alpha1"} | v4.public.eyJmaXh0dXJlIjoidmVyaWZpY2F0aW9uLWV2aWRlbmNlLXNpZ25pbmciLCJzY2hlbWFfdmVyc2lvbiI6InYxYWxwaGExIn3H_ZUhXtYzNyYEeFZkawaktpHuFCBedfsZa9wTW57wX8JVYK5YJHnYu1xHNpmnV-sXIZPV7MB2jrC7Nj4z78MG.eyJpc3N1ZXIiOiJzdmNfYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSIsImtleV9pZCI6InZlcmlmaWNhdGlvbi1maXh0dXJlLWtleS0xIiwicHVycG9zZSI6InZlcmlmaWNhdGlvbi1ldmlkZW5jZS1zaWduaW5nIiwic2NoZW1hX3ZlcnNpb24iOiJ2MWFscGhhMSJ9 | 2e393f9bb9b87d59f5333161af583bb3653ee8f04ed5efa15f5af0bb947a2067
verification-intent-signing | {"fixture":"verification-intent-signing","schema_version":"v1alpha1"} | v4.public.eyJmaXh0dXJlIjoidmVyaWZpY2F0aW9uLWludGVudC1zaWduaW5nIiwic2NoZW1hX3ZlcnNpb24iOiJ2MWFscGhhMSJ9joxK-RfXzayNsZX4gEKvffyZXFyEQ35Ku6SUUTb3cK9JReN0Dbhlqz7ZAmijTj-ElGRVwKieOFiZX_U91UKHAA.eyJpc3N1ZXIiOiJzdmNfYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSIsImtleV9pZCI6InZlcmlmaWNhdGlvbi1maXh0dXJlLWtleS0xIiwicHVycG9zZSI6InZlcmlmaWNhdGlvbi1pbnRlbnQtc2lnbmluZyIsInNjaGVtYV92ZXJzaW9uIjoidjFhbHBoYTEifQ | 5ee52784e9bdd0a612a2e0c5d487d6eb08da795c0626cb957ad0b70d4bb79359
verification-proof-bundle-signing | {"fixture":"verification-proof-bundle-signing","schema_version":"v1alpha1"} | v4.public.eyJmaXh0dXJlIjoidmVyaWZpY2F0aW9uLXByb29mLWJ1bmRsZS1zaWduaW5nIiwic2NoZW1hX3ZlcnNpb24iOiJ2MWFscGhhMSJ9U6kue75k5E6ctQQQVX_E9jhhJtxMjYT7oMygN0t1Vg_hjCnqyZqCGv7lbVWySDfH2G_3F2_HjQninIs1-OeAAw.eyJpc3N1ZXIiOiJzdmNfYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYWFhYSIsImtleV9pZCI6InZlcmlmaWNhdGlvbi1maXh0dXJlLWtleS0xIiwicHVycG9zZSI6InZlcmlmaWNhdGlvbi1wcm9vZi1idW5kbGUtc2lnbmluZyIsInNjaGVtYV92ZXJzaW9uIjoidjFhbHBoYTEifQ | ae35ecf7ae2de6363080d79404b0e46637e23a0643ec11439e90488ab2d863c3
```

These exercise the carrier primitive; SD-B3 additionally freezes five full semantically valid named-claims JCS/token/envelope fixtures before wrappers can ship. Wrappers reuse the crate-private W8-W10 key primitive, expose no generic/raw signer, and fix purpose/assertion/domain. Envelope bounds/current policy/key selection precede crypto; crypto precedes payload decode; canonical body, IDs/subjects, event time, current lifecycle, replay, and derivation follow. Genuine cross-purpose tokens and footer/assertion relabels always fail authentication.

## Intent and verifier-owned observations

```text
GateSpecRefV1 (E) gate_spec_id:GateId; gate_spec_digest:Digest
VerificationSignerRefV1 (E)
  purpose:VerificationSignerPurposeV1; principal_id:PrincipalId
  service_identity_id:ServiceIdentityId; key_id:KeyId
VerificationIntentV1 (V)
  verification_intent_id:VerificationIntentId; intent_sequence:PositiveSafeU64
  intent_nonce:VerificationNonce; restore_epoch:PositiveSafeU64
  candidate_id:CandidateId; candidate_manifest_digest:Digest; candidate_closure_digest:Digest
  author_lineage_digest:Digest; attempt_id:AttemptId; gate_specs:array<GateSpecRefV1,1..64>
  reconstruction_manifest_digest:Digest; environment_digest:Digest; toolchain_digest:Digest
  policy_snapshot_digest:Digest; policy_generation:PositiveSafeU64
  verification_signer_policy_id:VerificationSignerPolicyId
  verification_signer_policy_digest:Digest; verification_signer_policy_generation:PositiveSafeU64
  evidence_signer:VerificationSignerRefV1; proof_signer:VerificationSignerRefV1
  identity_observer:VerificationSignerRefV1; artifact_ownership_observer:VerificationSignerRefV1
  not_before_unix_ms:Timestamp; expires_at_unix_ms:Timestamp
```

Gate refs sort by `(gate_spec_id,gate_spec_digest)` and IDs are unique. Kernel resolves every immutable subject and selected signer, generates nonce, sets `intent_sequence=external high water + 1`, and constructs/signs the intent; no operator/workload field becomes a scheduling or outcome choice. The half-open window is nonempty, at most 86,400,000 ms, and lies inside PolicySnapshot, signer-policy, and all five selected-key windows. ID is `typed("vfi","bullet.verification-intent.v1",JCS(intent without ID))`. Durable consume/read-back of sequence, nonce, restore epoch, and envelope digest completes before verifier startup.

Runtime observation kinds are declared raw-ASCII as `peer_credential`, `process_identity`, `readback`, `resource`, `service_registration`, `timeout`:

```text
RuntimeObservationRefV1 (E) kind:RuntimeObservationKindV1; runtime_observation_id:RuntimeObservationId; body_digest:Digest
ServiceRegistrationObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; principal_id:PrincipalId
  service_identity_id:ServiceIdentityId; registered_uid:ServiceUid
  registration_generation:PositiveSafeU64; executable_digest:Digest; observed_at_unix_ms:Timestamp
PeerCredentialObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; listener_service_identity_id:ServiceIdentityId
  peer_service_identity_id:ServiceIdentityId; peer_pid:PositiveSafeU64; peer_uid:ServiceUid
  peer_gid:ServiceGid; boot_id_digest:Digest; process_start_ticks:SafeU64
  executable_digest:Digest; observed_at_unix_ms:Timestamp
ProcessIdentityObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; service_identity_id:ServiceIdentityId
  workcell_id:WorkspaceId; pidfd_subject_digest:Digest; cgroup_subject_digest:Digest
  executable_digest:Digest; argv_digest:Digest; environment_digest:Digest
  started_at_unix_ms:Timestamp; observed_at_unix_ms:Timestamp
ResourceObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; reservation_digest:Digest
  cpu_ms:SafeU64; memory_peak_bytes:SafeU64; pids_peak:SafeU64; disk_bytes:SafeU64
  egress_bytes:SafeU64; output_bytes:SafeU64; artifact_bytes:SafeU64
  unknown_liability:bool; observed_at_unix_ms:Timestamp
TimeoutObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; timer_boot_subject_digest:Digest
  grant_verified_elapsed_ms:SafeU64; deadline_after_ms:PositiveSafeU64
  observed_after_ms:SafeU64
  teardown_receipt_digest:Digest; observed_at_unix_ms:Timestamp
ReadbackObservationV1 (V)
  runtime_observation_id:RuntimeObservationId; state:contradictory|exact|unknown
  output_manifest_digest:Digest; artifact_manifest_digest:Digest
  classified_cas_readback_digest:Digest; observed_at_unix_ms:Timestamp
```

Each runtime ID is `typed("vro","bullet.verifier-runtime-observation.v1",JCS([kind,body without ID]))`; body digest is `H("bullet.verifier-runtime-observation-body.v1",JCS([kind,full body]))`. Only a distinct `bullet-verifier-supervisor` service creates these from registered `SO_PEERCRED`, pidfd/waitid, cgroup, monotonic timer, and CAS read-back. A crate-private constructor atomically persists body, event, audit leaf, and command receipt; mutation/workload APIs accept refs only. Strict registry/read-back decoders cannot dispatch, construct authority, or sign. Timer deltas satisfy `grant_verified_elapsed_ms <= observed_after_ms`, and timed-out results require `observed_after_ms >= deadline_after_ms`; resource counters are exact safe integers reconciled to the reservation, and unknown liability remains reserved. No host path, environment value, credential, secret, verdict, tier, success, or eligibility field is admitted.

```text
IdentitySubjectKindV1 = producer_identity | reconstruction_identity
IdentityObservationV1 (V)
  identity_observation_id:IdentityObservationId; subject_kind:IdentitySubjectKindV1
  verification_intent_id:VerificationIntentId; intent_envelope_digest:Digest
  candidate_id:CandidateId; attempt_id:AttemptId; workcell_id:WorkspaceId
  environment_digest:Digest; subject_principal_id:PrincipalId
  subject_service_identity_id:ServiceIdentityId; registered_uid:ServiceUid
  runtime_observations:set<RuntimeObservationRefV1,exactly 3>
  observed_at_unix_ms:Timestamp; observer_principal_id:PrincipalId
  observer_service_identity_id:ServiceIdentityId; observer_key_id:KeyId
  verification_signer_policy_id:VerificationSignerPolicyId
  verification_signer_policy_digest:Digest; verification_signer_policy_generation:PositiveSafeU64
ArtifactOwnershipObservationV1 (V)
  artifact_ownership_observation_id:ArtifactOwnershipObservationId
  verification_intent_id:VerificationIntentId; intent_envelope_digest:Digest
  candidate_id:CandidateId; attempt_id:AttemptId; workcell_id:WorkspaceId
  artifact_manifest_digest:Digest; artifact_owner_service_identity_id:ServiceIdentityId
  classified_cas_manifest_digest:Digest; readback:RuntimeObservationRefV1
  repository_id:RepositoryId; repository_subject_digest:Digest; observed_at_unix_ms:Timestamp
  observer_principal_id:PrincipalId; observer_service_identity_id:ServiceIdentityId
  observer_key_id:KeyId; verification_signer_policy_id:VerificationSignerPolicyId
  verification_signer_policy_digest:Digest; verification_signer_policy_generation:PositiveSafeU64
```

Identity refs contain exactly service-registration, peer-credential, and process-identity in kind order; ownership readback is exact. Observation IDs use `bullet.identity-observation.v1` and `bullet.artifact-ownership-observation.v1` over JCS without ID. Envelope/observer/intent/policy and all ref bodies agree. ADR 0016 custody facts are exactly producer identity, artifact ownership, reconstruction identity; fact digests are signed-envelope digests. They report facts only.

## Raw result and caller-free Evidence derivation

`GateProcessStateV1` catalog literals are raw-ASCII `cancelled,exited,infrastructure_failure,not_started,response_missing,signalled,timed_out,unsupported`; GateOutcome literals are `CANCELLED,FAIL,FLAKY,INFRA_ERROR,INVALIDATED,NOT_RUN,PASS,SUPERSEDED,TIMED_OUT,UNKNOWN,UNSUPPORTED`; EvidenceTier is `E0,E1,E2,E3,E4`. Semantic precedence below does not use enum ordinals.

```text
GateExecutionResultV1 (V)
  gate_execution_result_id:GateExecutionResultId
  verification_intent_id:VerificationIntentId; intent_envelope_digest:Digest
  candidate_id:CandidateId; attempt_id:AttemptId; workcell_id:WorkspaceId
  gate_spec_id:GateId; gate_spec_digest:Digest
  result_ordinal:PositiveSafeU64; predecessor_result_id:GateExecutionResultId|null
  predecessor_result_digest:Digest|null
  reconstruction_manifest_digest:Digest; environment_digest:Digest; toolchain_digest:Digest
  process_state:GateProcessStateV1; exit_code:ExitCode|null; term_signal:Signal|null
  discovered_count:SafeU64; passed_count:SafeU64; failed_count:SafeU64
  skipped_count:SafeU64; flaky_count:SafeU64
  output_manifest_digest:Digest; artifact_manifest_digest:Digest
  process_identity:RuntimeObservationRefV1; resource:RuntimeObservationRefV1
  readback:RuntimeObservationRefV1; timeout:RuntimeObservationRefV1|null
  started_at_unix_ms:Timestamp; completed_at_unix_ms:Timestamp
```

Counts are at most 1,000,000; checked `passed+failed+skipped=discovered`, `flaky<=discovered`. `not_started`, `unsupported`, `response_missing` require zero counts; first two require equal times. `exited` has exit only; `signalled` signal only; all other states neither. Initial result has ordinal one and null predecessor fields. A reconciliation result has predecessor ordinal plus one, exact predecessor ID/digest, and is permitted only to preserve UNKNOWN or resolve authenticated `response_missing`/unknown/contradictory read-back without rewriting history. Runtime refs have the implied kinds; timeout exists iff timed out; process/readback manifests and times agree; start is not after completion. ID is `typed("ger","bullet.gate-execution-result.v1",JCS(without ID))`; full digest is `H("bullet.gate-execution-result-body.v1",JCS(result))`. The supervisor constructs it from stored observations and wait/read-back facts; no signing, derivation, dispatch, or mutation API accepts caller JSON for this record.

```text
HumanAuthorizationRefV1 (E)
  approval_receipt_id:ApprovalReceiptId; approval_receipt_envelope_digest:Digest
  approval_signer_policy_digest:Digest
EvidenceLifecycleTransitionV1 (E)
  evidence_lifecycle_transition_id:EvidenceLifecycleTransitionId
  kind:initial|invalidated|superseded; ordinal:PositiveSafeU64
  predecessor_evidence_id:EvidenceId|null; predecessor_envelope_digest:Digest|null
  subject_change_decision_id:ContentId|null; subject_change_envelope_digest:Digest|null
  subject_change_signer_policy_digest:Digest|null
EvidenceEnvelopeClaimsV1 (V)
  verification_intent_id:VerificationIntentId; intent_envelope_digest:Digest
  candidate_manifest_digest:Digest; gate_spec_id:GateId; gate_spec_digest:Digest
  reconstruction_manifest_digest:Digest
  gate_execution_result_id:GateExecutionResultId; gate_execution_result_digest:Digest
  producer_identity_envelope_digest:Digest; reconstruction_identity_envelope_digest:Digest
  artifact_ownership_envelope_digest:Digest; human_authorization:HumanAuthorizationRefV1|null
  lifecycle_transition:EvidenceLifecycleTransitionV1
  policy_snapshot_digest:Digest; policy_generation:PositiveSafeU64
  verification_signer_policy_id:VerificationSignerPolicyId
  verification_signer_policy_digest:Digest; verification_signer_policy_generation:PositiveSafeU64
  evidence:EvidenceV1
```

Initial transition has ordinal one and all predecessor/change fields null. A successor has predecessor ordinal plus one, exact predecessor ID/envelope, and a current independently authenticated subject-change decision ID/envelope/signer-policy triple; kind determines only `SUPERSEDED` or `INVALIDATED`. Transition ID is `typed("elt","bullet.evidence-lifecycle-transition.v1",JCS(transition without ID))`. E4 requires a current two-person approval envelope and signer policy through `human_authorization`; until that predecessor is accepted, E4 is impossible. Other tiers require null human authorization.

The Evidence signer API accepts only authenticated intent/facts, internally loaded result/runtime bodies, exact immutable resolvers, and current policies. It has no caller Evidence, result, outcome, tier, reason, independence, overlap, lifecycle, or human-approval body argument. Initial derivation uses the first matching rule:

1. `CANCELLED/VERIFICATION_CANCELLED`; then `TIMED_OUT/GATE_TIMED_OUT`; then    `INFRA_ERROR/VERIFIER_INFRASTRUCTURE_FAILURE`.
2. `NOT_RUN/ZERO_TESTS`, `ALL_TESTS_SKIPPED`, or `GATE_NOT_RUN`, in that order, for supported exited    zero-discovered, exited all-skipped, or not-started.
3. `UNSUPPORTED/GATE_UNSUPPORTED`.
4. `UNKNOWN/RESULT_MISSING`, `RESULT_CONTRADICTORY`, or `READBACK_UNKNOWN`, in that order, for    response-missing, authenticated contradictory readback, or authenticated unknown readback.
5. `FLAKY/FLAKY_TESTS`; then `FAIL/TEST_FAILURE`, `TESTS_SKIPPED`, `PROCESS_SIGNALLED`, or    `NONZERO_EXIT`, in that reason order.
6. `PASS` only for exact readback, exited zero, discovered positive, passed equals discovered, and    failed/skipped/flaky all zero. No other path is green.

Structurally invalid counts/result refuse issuance. Tier derives from authenticated conflict and holdout policy plus complete lineage. Any missing/stale/overlapping principal, service, session, workspace, owner, provider/model family, holdout custodian, lineage, or runtime fact refuses as `EVIDENCE_CUSTODY_UNKNOWN`; there is no UNKNOWN tier. Initial `started_at` is the minimum timestamp across result and every authenticated custody/runtime fact; `completed_at` is trusted database issuance time not earlier than their maximum. Successors retain original start and complete at trusted issuance. All PolicySnapshot/policy/key/current-time half-open windows are rechecked.

ADR 0016 EvidenceV1 remains exactly `schema_version,evidence_id,candidate_id,subject_hash,outcome,tier,reason_code,verifier_principal_id,environment_hash,custody,started_at_unix_ms,completed_at_unix_ms`. `evidence_id=typed("evd","bullet.evidence.v1",JCS(Evidence without ID))`. `subject_hash=H("bullet.evidence-subject.v1",JCS([intent_envelope_digest,candidate_manifest_digest,gate_spec_id,gate_spec_digest,reconstruction_manifest_digest,gate_execution_result_id,gate_execution_result_digest,producer_identity_envelope_digest,reconstruction_identity_envelope_digest,artifact_ownership_envelope_digest,human_authorization,lifecycle_transition,policy_snapshot_digest,policy_generation,verification_signer_policy_id,verification_signer_policy_digest,verification_signer_policy_generation]))`. Every Evidence field is recomputed; `evidence_body_digest=H("bullet.evidence-body.v1",JCS(Evidence))` before purpose-fixed signing.

## Total Proof coverage and authenticated ancillary refs

```text
ProofEvidenceRefV1 (E)
  evidence_id:EvidenceId; evidence_body_digest:Digest; evidence_envelope_digest:Digest
  gate_spec_id:GateId; gate_spec_digest:Digest; lifecycle_ordinal:PositiveSafeU64
ProofReviewRefV1 (E)
  review_receipt_id:ReviewReceiptId; review_envelope_digest:Digest; review_signer_policy_digest:Digest
ProofExposureRefV1 (E)
  exposure_subject_id:ExposureSubjectId; exposure_envelope_digest:Digest; exposure_signer_policy_digest:Digest
AuditMerkleSiblingV1 (E) side:left|right; digest:Digest
ProofAuditRefV1 (E)
  audit_batch_id:AuditBatchId; audit_batch_digest:Digest; audit_batch_signature_digest:Digest
  audit_anchor_digest:Digest
  publication_leaf_id:AuditLeafId; publication_leaf_digest:Digest; leaf_index:SafeU64
  inclusion_path:array<AuditMerkleSiblingV1,0..64>; audit_signer_policy_digest:Digest
ProofBundleV1 (V)
  proof_bundle_id:ProofBundleId; verification_intent_id:VerificationIntentId
  intent_envelope_digest:Digest; candidate_id:CandidateId; candidate_manifest_digest:Digest
  evidence_refs:array<ProofEvidenceRefV1,1..64>; review_refs:array<ProofReviewRefV1,0..64>
  exposure_refs:array<ProofExposureRefV1,0..64>; audit_refs:array<ProofAuditRefV1,1..64>
  coverage_policy_digest:Digest; coverage_digest:Digest
  policy_snapshot_digest:Digest; policy_generation:PositiveSafeU64
  verification_signer_policy_id:VerificationSignerPolicyId
  verification_signer_policy_digest:Digest; verification_signer_policy_generation:PositiveSafeU64
  proof_root:Digest; completed_at_unix_ms:Timestamp
```

Evidence refs are a bijection with intent GateSpecs and name exactly each gate's authenticated current lifecycle head; no missing, duplicate, stale, or alternate Evidence is allowed. Audit refs biject Evidence refs and prove each Evidence publication leaf into an authenticated batch/anchor, with every sibling direction verified. Review/exposure refs exactly equal the current PolicySnapshot-required sets; their independently authenticated signer policies must exist or the policy-required sets must be empty. Arrays sort by raw typed ID (Evidence primarily by gate ID) and are unique. `coverage_digest=H("bullet.verification-proof-coverage.v1",JCS([intent_envelope_digest,gate_specs,evidence_refs,required_review_refs,review_refs,required_exposure_refs,exposure_refs,audit_refs,coverage_policy_digest]))`, where required refs are exact current-policy resolver outputs and byte-equal actual ID sets.

All carriers authenticate and bind one intent/Candidate/policy. Missing or non-PASS Evidence may be truthfully bundled but can never satisfy a green GateReceipt. Proof material is the bundle without ID/root; `proof_root=H("bullet.verification-proof-root.v1",JCS(material))` and `proof_bundle_id=typed("prb","bullet.proof-bundle.v1",JCS(bundle without ID))`. Proof publication's own audit leaf is later bound by GateReceipt/release and cannot occur inside Proof, avoiding a hash cycle. The dedicated Proof signer accepts resolved refs, not caller bytes.

## Publication, replay, high water, and restore

Publication kinds are raw-ASCII `artifact-ownership-observation,evidence,gate-execution-result, identity-observation,proof-bundle,runtime-observation,verification-intent`.

```text
VerificationIntentConsumeV1 (E)
  intent_sequence:PositiveSafeU64; nonce:VerificationNonce; restore_epoch:PositiveSafeU64
  intent_id:VerificationIntentId; intent_envelope_digest:Digest
VerificationRetainedKeyEntryV1 (E)
  purpose:VerificationSignerPurposeV1; principal_id:PrincipalId
  service_identity_id:ServiceIdentityId; key_id:KeyId; public_key_lower_hex:Code<64>
  custody_subject_digest:Digest; source_policy_id:VerificationSignerPolicyId
  source_policy_digest:Digest; source_policy_generation:PositiveSafeU64
VerificationRetainedKeyIndexV1 (V)
  verification_retained_key_index_id:VerificationRetainedKeyIndexId; generation:PositiveSafeU64
  prior_index_digest:Digest|null; entries:array<VerificationRetainedKeyEntryV1,5..65536>
VerificationPublicationSubjectV1 (E tagged union; discriminator subject_kind)
  {subject_kind:"artifact-ownership-observation",artifact_ownership_observation_id:ArtifactOwnershipObservationId} |
  {subject_kind:"evidence",evidence_id:EvidenceId} |
  {subject_kind:"gate-execution-result",gate_execution_result_id:GateExecutionResultId} |
  {subject_kind:"identity-observation",identity_observation_id:IdentityObservationId} |
  {subject_kind:"proof-bundle",proof_bundle_id:ProofBundleId} |
  {subject_kind:"runtime-observation",runtime_observation_id:RuntimeObservationId} |
  {subject_kind:"verification-intent",verification_intent_id:VerificationIntentId}
VerificationPublicationEntryV1 (E)
  publication_id:VerificationPublicationId; publication_sequence:PositiveSafeU64
  kind:VerificationPublicationKindV1; logical_key_digest:Digest
  subject:VerificationPublicationSubjectV1; object_digest:Digest; audit_leaf_id:AuditLeafId
  audit_leaf_digest:Digest; prior_chain_digest:Digest; chain_digest:Digest
VerificationIntentReplaySegmentV1 (V)
  generation:PositiveSafeU64; prior_segment_digest:Digest|null
  entries:array<VerificationIntentConsumeV1,0..65536>
VerificationPublicationSegmentV1 (V)
  generation:PositiveSafeU64; prior_segment_digest:Digest|null
  entries:array<VerificationPublicationEntryV1,0..65536>
VerificationHighWaterV1 (V)
  generation:PositiveSafeU64; verification_signer_policy_digest:Digest
  verification_signer_policy_generation:PositiveSafeU64; restore_epoch:PositiveSafeU64
  retained_key_index_id:VerificationRetainedKeyIndexId; retained_key_index_digest:Digest
  intent_sequence_high_water:SafeU64; intent_segment_digest:Digest
  publication_sequence_high_water:SafeU64; publication_segment_digest:Digest
  publication_chain_digest:Digest; updated_at_unix_ms:Timestamp; checksum:Digest
```

Retained-key entries sort by raw `(purpose,service_identity_id,key_id)` and are globally unique by key ID and decoded material. Index ID is `typed("vki","bullet.verification-retained-key-index.v1",JCS(index without ID))`; full digest is `H("bullet.verification-retained-key-index-body.v1",JCS(index))`. Genesis has generation one/null prior; successors increment generation, bind the prior full digest, and may only append entries. Intent segment entries sort by raw nonce; nonce/ID/envelope tuples are unique across every retained segment, and their sequence set is contiguous when ordered numerically. Publication segment entries sort by logical-key digest; logical keys are globally unique and their sequence set is contiguous. A full 65,536-entry segment seals; its successor increments generation and binds the prior segment digest. Segment digests are respectively `H("bullet.verification-intent-replay-segment.v1",JCS(segment))` and `H("bullet.verification-publication-segment.v1",JCS(segment))`; all prior segments remain immutable and reachable. Replay lookup uses durable unique indices but is verified by traversing all sorted set manifests and replaying publication entries by sequence. Empty genesis segment has generation one, null prior, and empty entries.

Logical key is `H("bullet.verification-publication-logical-key.v1",JCS([kind,tuple]))`, with exact tuple: intent `[intent_id]`; runtime `[intent_id,runtime_kind,runtime_id]`; identity `[intent_id,subject_kind]`; ownership `[intent_id,candidate_id,artifact_manifest_digest]`; result `[intent_id,gate_spec_id,result_ordinal]`; Evidence `[intent_id,gate_spec_id,lifecycle_ordinal]`; Proof `[intent_id,coverage_digest]`. The tagged subject's `subject_kind` must byte-equal entry kind. Audit-leaf material is exactly `[publication_sequence,kind,logical_key_digest,subject,object_digest]`; `audit_leaf_digest=H("bullet.verification-publication-audit-leaf.v1",JCS(material))` and `audit_leaf_id=typed("aul","bullet.verification-publication-audit-leaf-id.v1",JCS(material))`. The leaf contains no publication ID or chain head. Publication ID uses `typed("vpb","bullet.verification-publication.v1",JCS(entry without ID/chain_digest))`, so no digest is circular.

Chain genesis is `H("bullet.verification-publication-chain-genesis.v1",b"")`; each step is `H("bullet.verification-publication-chain-step.v1",JCS([prior_chain_digest,publication_sequence,logical_key_digest,subject,object_digest,audit_leaf_id,audit_leaf_digest]))`. Exact retry returns stored bytes. Same logical key/different digest is conflict/quarantine. Lost response is UNKNOWN until CAS, ledger, event, outbox, audit leaf, segments, chain, and external high water agree. One serialized database transaction writes object bytes/digest, entry, event, outbox, and audit leaf or none; external advance/read-back follows. Crash between them enters RECOVERING.

High-water checksum is `H("bullet.verification-high-water.v1",JCS(record without checksum))`. Descriptor-admitted canonical regular file is exactly `/var/lib/bullet-authority/verification-high-water.v1.json`, on a separately mounted non-restored filesystem, parent mode 0700 and file mode 0600, owned by registered Kernel authority UID, opened beneath/no-symlink with one exclusive lock. Advance uses same-directory create, write, fsync, rename-no-replace/replace under lock, directory fsync, reopen/inode+bytes read-back; every frame and deadline is bounded. Missing/corrupt/regressed/ahead state refuses except explicit empty install.

Kernel generates nonce and enforces `intent_sequence=high_water+1`; durable nonce consume precedes startup. High-water generation equals prior generation plus one on every advance; the external tuple and reachable manifests may only extend. Before any database restore is exposed, external `restore_epoch` increments and is read back; every unresolved intent, grant, credential, activation, and cache from older epochs is invalid. Reconciliation rebuilds segments/chain from immutable CAS, ledger, event, outbox, audit and remote read-back, compares exact external bytes, and requires independent recovery approval. Unknown liability remains reserved; neither restore nor policy rollback removes nonce/publication history.

## Release-registry graph and validation

Successor `ReleaseRegistryObjectKindV1` raw-ASCII-merges current values with exactly `approval-receipt,artifact-ownership-observation,audit-anchor,audit-batch,audit-batch-signature,audit-leaf,evidence,exposure-subject,gate-execution-result,identity-observation,operator-decision,policy-snapshot,proof-bundle,review-receipt,runtime-observation,verification-family-source,verification-intent,verification-retained-key-index,verification-signer-policy`. Verification intent/fact/Evidence/Proof use their purpose-fixed envelope digest. Approval/review/exposure use the authenticated envelope digest fixed by their accepted predecessor; `operator-decision` is exactly the verified `SignedVerificationOperatorApprovalV1` envelope and its accepted two-person signer policy, never Markdown. Audit-batch signature uses its detached-signature object digest; audit batch, anchor, and leaf use their exact accepted body/root/leaf digests. Runtime/result/source/PolicySnapshot/signer-policy/key-index use the full-body digest fixed above or by their predecessor. Every raw digest becomes `subject_digest="blake3:"+digest`; no generic signed-JSON digest substitutes. `subject_id=typed("cnt","bullet.release-verification-subject.v1",JCS([object_kind,native_subject_id,subject_digest]))` and registry `object_id` equals it. Object path is exactly `verification/<object-kind>/<native-typed-id>.json`, relative, normalized, and beneath-only.

Native namespaces are `transaction:vfi_*` for intents; `observation:ido_*`, `observation:aow_*`, `observation:vro_*`, `observation:ger_*`, and `observation:exs_*`; `evidence:evd_*`; `proof-bundle:prb_*`; `policy:vsp_*`, `policy:vki_*`, and imported PolicySnapshot/OD-L IDs; `configuration:vfs_*`; `review:rev_*`/`review:apr_*`; and `audit-anchor:aub_*`/`audit-anchor:aul_*`. Descriptor read-back recomputes canonical bytes, purpose/body digest, tagged mapping, subject/object ID, path, and exactly-one graph edge. It includes family source, OD-L, signer policy/key index, intent, every referenced runtime/fact/result/Evidence/Proof, human/lifecycle/review/exposure subjects, their audit inclusions, current PolicySnapshot, verification request, GateReceipt, trusted time, and replay state with no orphan. Request and receipt subject sets are identical. A Proof audit leaf is a GateReceipt subject; it is not inside Proof.

Legal order is signed intent -> durable consume -> supervisor runtime/result -> signed facts -> derived signed Evidence -> create-once/read-back/audit -> complete derived signed Proof -> create-once/read-back/audit -> registry full-chain verification -> distinct SSH-Ed25519 `gate-attestor` GateReceipt -> trusted-time/signature/replay admission. Current time rechecks loaded canonical PolicySnapshot/signer-policy IDs, full digests, generations and windows. Retained keys verify history only. A signature authorizes no mutation; a valid later signature repairs nothing.

Validation precedence is bounded outer envelope; current policy; exact purpose/principal/service/key; crypto; canonical payload; IDs/digests/subjects; event/current time; derivation/custody; durable replay/read-back; total Proof coverage; GateReceipt/release semantics. Stable families are `VERIFICATION_ENVELOPE_INVALID`, `VERIFICATION_SIGNER_POLICY_INVALID`, `VERIFICATION_SIGNER_KEY_UNKNOWN`, `VERIFICATION_SIGNER_KEY_WRONG_PURPOSE`, `VERIFICATION_SIGNER_KEY_INACTIVE`, `VERIFICATION_SIGNATURE_INVALID`, `VERIFICATION_CLAIMS_INVALID`, `VERIFICATION_SUBJECT_MISMATCH`, `VERIFICATION_TIME_INVALID`, `GATE_RESULT_INVALID`, `EVIDENCE_DERIVATION_MISMATCH`, `EVIDENCE_CUSTODY_UNKNOWN`, `VERIFICATION_INTENT_REPLAYED`, `VERIFICATION_PUBLICATION_CONFLICT`, `EVIDENCE_PUBLICATION_CONFLICT`, `PROOF_PUBLICATION_CONFLICT`, `VERIFICATION_HIGH_WATER_INVALID`, `VERIFICATION_RESTORE_RECONCILIATION_REQUIRED`, `VERIFICATION_READBACK_UNKNOWN`, and `RELEASE_EVIDENCE_CHAIN_INVALID`. No error is PASS.

## Exact cap-safe implementation packets

Every ordinary lane owns at most four exact paths; every changed Rust/test/doc finishes below 500 physical LOC. Each intermediate committed head compiles and its applicable doctors pass. No abbreviated root/test path grants authority. Accepted ADR 0017, dormant W11 with four unchanged legacy sentinels, and accepted/encoded ADR 0016 LC-A through LC-D plus SD-B are hard W12 predecessors. SD-B0 accepts this one path at DESIGNED only after independent custody/security, wire/Kernel, and release/registry reviews. OD-L and `SignedVerificationOperatorApprovalV1` are not ADR-acceptance or W12 predecessors and receive no implementation packet here; a separate accepted decision must freeze their exact carrier, signer policy, and paths after W12 supplies the source subject.

Pre-W12 generated sharding is structural and byte-reviewed:

1. **W12-G0:** `crates/bullet-wire/src/contract_bindings.rs`, new    `crates/bullet-wire/src/contract_bindings/shards.rs`, new    `crates/bullet-wire/src/contract_bindings/shards/tests.rs`, `crates/bullet-wire/src/contract_tool.rs`.
2. **W12-SR0:** new `contracts/generated/rust/schema_bundle/legacy_0.rs`, `legacy_1.rs`, `legacy_2.rs`,    and `contracts/v1alpha1/rust-shard-staging-manifest.json`; production root remains unchanged.
3. **W12-SR1:** `contracts/generated/rust/schema_bundle.rs`, new    `contracts/generated/rust/schema_bundle/verification.rs`, `contracts/v1alpha1/bundle-manifest.json`,    and deletion of `contracts/v1alpha1/rust-shard-staging-manifest.json`.
4. **W12-ST0:** new `contracts/generated/typescript/schemaBundle/legacy0.ts`, `legacy1.ts`,    `legacy2.ts`, and `contracts/v1alpha1/typescript-shard-staging-manifest.json`.
5. **W12-ST1:** `contracts/generated/typescript/schemaBundle.ts`, new    `contracts/generated/typescript/schemaBundle/verification.ts`, `contracts/v1alpha1/bundle-manifest.json`,    and deletion of `contracts/v1alpha1/typescript-shard-staging-manifest.json`.

G0 must generate legacy roots byte-identically until explicit staging. Stage manifests bind every new byte while normal generation/check remains unchanged; activation atomically switches one language root/manifest and deletes its stage. Roots and all shards finish below 500 LOC, public exports remain identical, two clean generations agree, and manifest inventory has no orphan.

W12 publication is one coordinated atomic commit composed of disjoint claims, never broken intermediate heads: **W12-A** owns `contracts/v1alpha1/contract-catalog.json`, `crates/bullet-wire/src/catalog/records.rs`, `crates/bullet-wire/src/catalog/constraints/legacy_core.rs`, and `crates/bullet-wire/src/catalog/constraints.rs`; **W12-B** owns `contracts/v1alpha1/schema-bundle.json`, `contracts/generated/rust/schema_bundle/verification.rs`, `contracts/generated/typescript/schemaBundle/verification.ts`, and `contracts/v1alpha1/bundle-manifest.json`. It encodes accepted ADRs 0016-0018, closes all leaves, generates twice, and freezes new catalog/schema/Rust/TypeScript sentinels. W12 produces COMPONENT bytes only; it neither creates nor authenticates OD-L, activates a signer, or admits a final lock.

Farm component lanes then proceed:

6. **SD-B2:** `crates/bullet-wire/src/dogfood/grant_signing.rs`, new    `crates/bullet-wire/src/verification/carrier.rs`, new    `crates/bullet-wire/src/verification/carrier/tests.rs`, `crates/bullet-wire/src/lib.rs`.
7. **SD-B3:** new `crates/bullet-wire/src/verification/policy.rs`, new    `crates/bullet-wire/src/verification/policy/tests.rs`, new    `fixtures/canonical/verification-authenticity-v1.json`, `crates/bullet-wire/src/verification.rs`.
8. **SD-B4:** new `crates/bullet-wire/src/verification/facts.rs`, new    `crates/bullet-wire/src/verification/fact_signing.rs`, new    `crates/bullet-wire/src/verification/fact_signing/tests.rs`, `crates/bullet-wire/src/verification.rs`.
9. **SD-B5:** new `crates/bullet-wire/src/verification/runtime.rs`, new    `crates/bullet-wire/src/verification/derive.rs`, new    `crates/bullet-wire/src/verification/derive/tests.rs`, `crates/bullet-wire/src/verification.rs`.
10. **SD-B6:** new `crates/bullet-wire/src/verification/record_signing.rs`, new     `crates/bullet-wire/src/verification/chain.rs`, new     `crates/bullet-wire/src/verification/chain/tests.rs`, `crates/bullet-wire/src/verification.rs`.
11. **SD-B7:** `crates/bullet-wire/src/release/evidence.rs`, `crates/bullet-wire/src/release/fields.rs`,     `crates/bullet-wire/src/release/validate.rs`, `crates/bullet-wire/tests/release_registry_contract.rs`.
12. **SD-B8:** `src/family_lock/schema/external.rs`, `src/family_lock/schema.rs`,     `src/family_lock/schema/tests.rs`, `src/family_lock.rs`.

**SD-BI-n** co-lands with each new nested-test lane and owns only `ops/ci/lib.sh` and the accepted ADR 0017 cap-safe `crates/bullet-wire/tests/canonical_hostile/inventory_tests.rs`; it updates exact inventory/count/digest sentinels so every intermediate head is green and makes no semantic change. **W12-GI** uses those same two support paths and co-lands with W12-G0.

Kernel must first extract, not copy, secure high-water storage:

13. **K-SDB0:** new `crates/adapters/src/external_high_water/mod.rs`, new     `crates/adapters/src/external_high_water/storage.rs`, new     `crates/adapters/src/external_high_water/tests.rs`, `crates/adapters/src/lib.rs`.
14. **K-SDB0A:** `crates/adapters/src/authority_high_water/mod.rs`,     `crates/adapters/src/authority_high_water/storage.rs`,     `crates/adapters/src/authority_high_water/tests.rs`, `crates/adapters/src/lib.rs`.
15. **K-SDB1:** `crates/domain/src/schema_bundle.rs` only.
16. **K-SDB2:** new `crates/application/src/verification.rs`, new     `crates/application/src/verification/intent.rs`, new     `crates/application/src/verification/publication.rs`, `crates/application/src/lib.rs`.
17. **K-SDB3:** new `crates/verifier/src/supervisor.rs`, new     `crates/verifier/src/runtime_observation.rs`, `crates/verifier/src/lib.rs`, new     `crates/verifier/tests/runtime_observation.rs`.
18. **K-SDB4:** new `db/migrations/0024_verification_publication.sql`,     `crates/adapters/src/sqlite/migrations/catalog.rs`,     `crates/adapters/src/sqlite/migrations/tests/late.rs`, `crates/adapters/src/sqlite/mod.rs`.
19. **K-SDB5:** new `crates/adapters/src/sqlite/verification.rs`, new     `crates/adapters/src/sqlite/verification/publication.rs`, new     `crates/adapters/tests/verification_publication.rs`, `crates/adapters/src/sqlite/mod.rs`.
20. **K-SDB6:** new `crates/adapters/src/verification_high_water/mod.rs`, new     `crates/adapters/src/verification_high_water/tests.rs`,     `crates/adapters/src/external_high_water/mod.rs`, `crates/adapters/src/lib.rs`.
21. **K-SDB7:** `crates/verifier/src/signed_chain.rs`, `crates/verifier/src/signed_chain/records.rs`, `crates/verifier/src/signed_chain/validation.rs`, `crates/verifier/src/signed_chain/crypto.rs`; it imports Farm wire/carriers and removes duplicate fixture authority. **K-SDB7H** co-lands with only `crates/verifier/tests/signed_chain.rs`.

Distinct signer binaries are separate lanes: **K-SDB8** owns `crates/observer/Cargo.toml`, `crates/observer/src/lib.rs`, root `Cargo.toml`, and `Cargo.lock`; **K-SDB9I** owns new `apps/bullet-identity-observer/Cargo.toml`, new `apps/bullet-identity-observer/src/main.rs`, root `Cargo.toml`, and `Cargo.lock`; **K-SDB9A** owns new `apps/bullet-artifact-observer/Cargo.toml`, new `apps/bullet-artifact-observer/src/main.rs`, root `Cargo.toml`, and `Cargo.lock`; **K-SDB9P** owns new `apps/bullet-proof-attestor/Cargo.toml`, new `apps/bullet-proof-attestor/src/main.rs`, root `Cargo.toml`, and `Cargo.lock`. **K-SDB9E** owns `apps/bullet-verifier/Cargo.toml`, `apps/bullet-verifier/src/main.rs`, `apps/bullet-verifier/tests/bin.rs`, and deletion of `apps/bullet-verifier/src/bin/fixture_main.rs`; **K-SDB9EH** deletes only `apps/bullet-verifier/tests/fixture_bin.rs`. Each binary has one compile-time purpose and cannot select another role.

Release admission is likewise exact: **SD-B9** owns new `src/check/semantic_registry/evidence_chain.rs`, `src/check/semantic_registry.rs`, `tests/check_cli/semantic_registry.rs`, and new `tests/check_cli/semantic_registry/evidence_chain.rs`; **SD-B10** owns `src/check/semantic_registry/kinds.rs`, `src/check/semantic_registry/validation.rs`, `src/check/semantic_registry/admission.rs`, and `tests/check_cli/semantic_registry/hostile.rs`. Detached registry/time/replay signatures remain separate existing authorities. No lane creates a shadow DTO/root or imports a sibling checkout.

## Hostile proof and evidence ceiling

Tests pin strict catalog closure, every ID/domain/body/footer/token/envelope golden, payload/token bounds, key uniqueness/custody/lifecycle edge, purpose relabel/cross-use, exact result/count/readback derivation, every subject/time/lineage substitution, E4/lifecycle authorization, total GateSpec coverage, ancillary signatures/inclusion proofs, exact retry/conflict, concurrent duplicate, lost response, ENOSPC/crash at every atomic boundary, segment rollover, restart/restore/rollback, high-water descriptor attack, every registry object exactly once, no orphan, and GateReceipt laundering refusal. Zero/skipped tests, missing tool, or unavailable predecessor fails.

Private proof includes focused/full Farm wire, canonical-hostile, registry, Clippy/fmt/diff/LOC, contract-tool and two clean generations, then focused/full Kernel verifier/publication/high-water, restart/restore/fault, Clippy/fmt, and semantic release fixtures using unique targets.

ADR review is **DESIGNED**. W12/Farm/Kernel packets are at most **COMPONENT_ONLY**. **TRANSACTION** requires distinct installed UIDs and external key custody, a signed exact-family LocalBareForge chain, durable read-back/audit anchor, and fault suite. **RELEASE** additionally requires current OD-D/OD-E plus accepted `SignedVerificationOperatorApprovalV1` and `VerificationOperatorApprovalSignerPolicyV1` authentication of OD-L, family/signer/time/replay subjects, semantic GateReceipt, and selected profile. Architecture prose, fixture keys, raw Markdown, content digests, or local keys never raise these ceilings.
