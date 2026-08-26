# Error and repair contract

Status: **normative for the Hub CLI; product API Problem Details remain Kernel-owned**  
Owner: Bullet Farm maintainers  
Last reviewed: 2026-08-25

`bullet-family` writes errors as `CODE: reason` and exits nonzero. The stable
uppercase code is the machine-facing classification; prose gives the exact
subject-specific reason. Never parse or branch on prose. A command exit or an
HTTP response is not evidence that the requested engineering outcome was
verified.

This document is the repair index consumed by `agent/exceptions.toml`. The
Kernel public API separately owns RFC 9457 Problem Details, request IDs, and
status codes; this Hub document does not invent or override that contract.

## Invalid input

Representative codes are `USAGE`, `INVALID_ARGUMENT`, `MISSING_OPTION`,
`INVALID_PATH`, `INVALID_FAMILY_LOCK`, `INVALID_RELEASE_BUNDLE`, and
`INVALID_GENERATED_CONTRACT`.

- Purpose: reject malformed or noncanonical input before mutation.
- Repair: use the command usage, strict schema, generated-zone command, or
  exact lock version named by the error. Do not hand-edit a generated subject.
- Proof: run the narrow command in `agent/test-map.json` for the rejected path.

## Conflict or changed subject

Representative codes are `CLAIM_OVERLAP`, `DIRTY_CHECKOUT`,
`CHECKOUT_CONFLICT`, `FUSION_DESTINATION_CONFLICT`,
`RELEASE_DESTINATION_EXISTS`, `GIT_REPOSITORY_CHANGED`, and
`CHECK_SUBJECT_MISMATCH`.

- Purpose: preserve one-writer, no-replace, and exact-subject guarantees.
- Repair: read current state, preserve foreign bytes, and create a new command
  only if the intended subject changed. Retrying the same mutation blindly is
  forbidden.
- Proof: coordinator status, exact OIDs/digests, and the owning transaction
  test must all agree.

## Unsupported or corrupt state

Representative codes are `UNSUPPORTED_SCHEMA`, `CORRUPT_COORD_LOG`,
`PARTIAL_COORD_WRITE`, `UNSUPPORTED_PLATFORM_CONTAINMENT`, and
`UNSUPPORTED_GIT_OBJECT_FORMAT`.

- Purpose: fail closed when persisted bytes or the platform cannot satisfy the
  admitted contract.
- Repair: preserve the subject, follow the schema-removal/setup-recovery
  runbook, or use an admitted platform. Never repair an append-only log by
  deleting the offending record.
- Proof: restore/integrity verification must pass before mutation resumes.

## Dependency unavailable

Representative codes are `FAMILY_ROOT_NOT_FOUND`, `FAMILY_MEMBER_MISSING`,
`SETUP_TOOL_UNAVAILABLE`, `SOURCE_METADATA_UNAVAILABLE`,
`RELEASE_VERIFIER_UNAVAILABLE`, and `CAPABILITY_UNPROBED`.

- Purpose: prevent ambient tools, credentials, services, or unsigned artifacts
  from being substituted for exact prerequisites.
- Repair: run `bullet-family doctor --json`, provision the pinned subject, and
  rerun only its mapped proof. An absent external signer, provider, forge, or
  platform remains `BLOCKED`; it is not a component failure to conceal.

## Verification failed

Representative codes are `PROOF_FAILED`, `GIT_VERIFICATION_FAILED`,
`RELEASE_SIGNATURE_INVALID`, `LOCKED_ARTIFACT_MISMATCH`, and
`RELEASE_EVIDENCE_SUBJECT_MISMATCH`.

- Purpose: distinguish process completion from exact-subject verification.
- Repair: inspect the raw proof and repair or replace the subject. Evidence for
  an older commit, Candidate, archive, configuration, policy, or environment
  cannot be reused.

## Outcome unknown

Representative codes are `COMMAND_TIMEOUT`, `RELEASE_PUBLICATION_UNKNOWN`, and
the durable command/effect state `UNKNOWN`.

- Purpose: preserve ambiguity when a write may have completed but its response
  was lost.
- Repair: reconcile using the original request identity and exact desired
  state. Do not retry the write, switch forge providers, or render green.
- Exit: only authoritative read-back may adopt the original effect or prove a
  typed refusal.

## Receipt missing

The release gate detail `no ... receipt is registered` and codes such as
`MSRV_GATE_MISSING` mean the component may exist but release evidence does not.

- Purpose: keep local tests, simulator output, and unsigned observations from
  becoming release authority.
- Repair: run the exact tagged proof, verify it under the external signer and
  trusted-time policy, then register the kind-specific receipt.
- Proof: `bullet-family check release --profile universal-v1 --receipts
  <admitted-absolute-registry> --json` remains authoritative and nonzero until
  every gate in that dependency closure is independently receipted.

## Escalation rule

Escalate when the same exact blocker survives its mapped repair, when current
state contradicts two authoritative observations, or when a recovery would
delete or overwrite unpreserved bytes. Include the stable code, exact subject,
command, exit status, and evidence location. Never relabel `UNKNOWN` as
`FAILED`, weaken a gate, or invent a receipt to make an escalation disappear.
