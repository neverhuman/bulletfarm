# 0011 — Signed launch grants and provider egress isolation

Status: Accepted (contract and Kernel components landed 2026-08-25; live admission remains policy-refused)
Owner: Bullet Farm maintainers
Related: 0001 (providers propose, BulletGit writes), 0005 (signed authority and key lifecycle), 0007 (sandbox, secrets, tainted tool data)

## Decision

A provider process may be dispatched only when the Kernel's admission carries two pieces of evidence, each
clearing exactly one blocker and nothing else:

1. **A signed launch grant** (`SignedLaunchGrantV1`, `bullet-wire`): a PASETO v4.public token whose claims
   (`LaunchGrantClaimsV1`) bind the durable active lease (mission, repository, graph revision, work package,
   Variant, Attempt, fence, runner identity/epoch, workspace and nonce digest, authority epoch, freeze
   generation), the evaluated provider admission (provider, adapter, profile, model, credential generation,
   protocol, absolute executable path and BLAKE3 digest, descriptor and capability digests), the policy
   (snapshot digest, generation, sandbox manifest digest, environment digest, admitted gate ids) and the
   budget (reservation id, invocations, wall clock, cost). Audience `provider-runner`, operation
   `launch-provider`, footer purpose `launch-grant-signing`, implicit assertion
   `bullet-farm.launch-grant.v1alpha1`, TTL ≤ 15 s, single-use nonce. Keys resolve through the policy's
   `authority-signing` issuer keys; no new key purpose exists. The Kernel verifies with a vendored,
   golden-pinned implementation (no path dependency) and clears `SIGNED_ADMISSION_UNAVAILABLE` only when
   every binding equals its own observed values and the nonce has never been spent.
2. **An egress-isolation receipt** (`bullet-harness-egress`): the provider runs inside a fresh user+network
   namespace with an nftables default-drop ruleset that permits only loopback and TCP to a host-side
   allow-listing CONNECT proxy; eight in-namespace probes must prove direct internet, the host's forge
   port, decoy host ports and DNS are refused before the child starts; the sealed receipt (allowlist,
   ruleset and probe digests) clears `EGRESS_ISOLATION_UNAVAILABLE` only when every containment probe is
   refused/unreachable.

Minting happens only from the durable active lease, in-process, with an operator-held 0600 key
(`bullet authority keygen | mint-launch-grant`); there is no HTTP minting route. The launch grant never
authorizes lease acquisition, Attempt advance, release, mutation or effects.

## Policy consequence

`v1alpha1` policy forbids `sandbox_policy.live_admission_enabled = true` (`UNSAFE_POLICY`), and the Kernel's
loader enforces the same rule. A fully valid grant therefore ends in `POLICY_LIVE_ADMISSION_DISABLED` today
without spending its nonce. Live provider execution requires an operator-ratified policy generation that
registers a `provider-runner` authority key and enables live admission; that ratification is an operator
act recorded in the coordination log, never an agent edit.

## Rejected alternatives

- A shared environment token (`BULLET_LIVE_ADMISSION`): not admission; removed in kernel `50e9217`.
- A default-off Cargo feature: static and unbound to lease, executable or policy.
- Reusing mutation authority claims for launch: the nine mutation operations bind BulletGit/effect subjects,
  not a process launch; conflating them would let a workspace grant spawn a provider.
- Mounting removed farmd lease routes to obtain a lease for minting: transport carries no identity
  (see RUNNER-FARMD-LEASE-ROUTE-AUDIT-R1); a separate signed lease-transport authority is the predecessor.

## Evidence

Hub `a2d6b2a` (contract, golden `5f89dde4…`), consumer syncs kernel `28938b4` / git `08179e3` /
portal `9a9da85`, kernel `d388733` (verifier, issuer, policy loader, admission, CLI, egress crate;
410/410 tests, egress live lane 3/3 on the development host).
