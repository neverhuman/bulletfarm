//! Historical gate rows plus the conditions selected by `universal-v1`.
//! Each table is sorted by gate id; the parent module's tests enforce exact
//! profile coverage, gap coverage, vocabulary, and command existence.

use super::{ClaimRow, Owner, command, none};

const PROVIDER_WHY: &str = "The bounded offline parser and the zero-spawn neutral refusal prove neither schema-3 policy/enrollment-anchor admission nor the live adapter, its credentials, quota, isolation, patch output, onboarding, or sealed receipt registration; only a semantically admitted PONG receipt under an operator-ratified policy is LIVE_PROOF for this adapter.";
const PROVIDER_ACCEPTANCE: &str = "First land hostile-tested schema-3 policy and provider-enrollment-anchor admission, provider-specific onboarding/runtime probing, and semantic sealed-receipt registration. Then perform the operator act in `docs/runbooks/live-conformance.md` (authority keygen, a ratified policy generation with an active provider-runner key, the `AGENT_CHAT.md` ratification line), run the real-mode lane, and register the sealed exact adapter/version/profile receipt covering isolation, failure, quota, and patch conformance.";
const PROVIDER_OWNER: Owner = Owner::LocalThenExternal {
    offline: "schema-3 policy/enrollment-anchor admission, provider onboarding/runtime probing, and semantic sealed-receipt registration are local engineering",
    external: "the operator ratifies the policy and enrollment, supplies the exact provider executable/profile/credentials, and runs the native live conformance lane",
};
const PROVIDER_NOTE: &str = "real mode refuses to start without an absolute operator-ratified policy path; the current loader only checks structure and does not admit the external anchor, and under the checked-in v1alpha1 policy the lane exits 78 (neutral refusal) and spawns nothing";
const PROFILE_WHY: &str = "A dependency graph selects a condition; it does not establish the independently trusted, exact-subject semantics that condition names.";
const PROFILE_ACCEPTANCE: &str = "Register exact current-family evidence only after signer lifecycle, dependency closure, schema-3 family, policy/toolchain/environment fingerprints, trusted time, replay state, and the condition-specific semantic validator all pass.";
const PROFILE_EXISTS: &str = "an explicit profile graph and bounded structural registry parser that remain fail-closed without external trust, replay state, current-family admission, and condition-specific semantics";
const PROFILE_OWNER: Owner = Owner::LocalThenExternal {
    offline: "condition-specific semantic validators and typed refusals are local engineering",
    external: "independently admitted signer roots, trusted time, replay state, exact signed family subjects, credentials, services, and target platforms",
};

const fn profile_condition(id: &'static str, gap_ids: &'static [&'static str]) -> ClaimRow {
    ClaimRow {
        id,
        gap_ids,
        claim: "No current independently admitted receipt establishes this selected release-profile condition.",
        why: PROFILE_WHY,
        acceptance: PROFILE_ACCEPTANCE,
        exists: PROFILE_EXISTS,
        owner: PROFILE_OWNER,
        next: none("the renderer supplies the exact selected-profile command"),
    }
}

pub(crate) const CONDITION_ROWS: &[ClaimRow] = &[
    profile_condition(
        "release.profile.evolution-v1",
        &["G11", "G13", "G14", "G15"],
    ),
    profile_condition("release.profile.github-adapter-v1", &["G7"]),
    profile_condition("release.profile.gitlab-adapter-v1", &["G16"]),
    profile_condition("release.profile.gitlab-self-managed-v1", &["G16"]),
    profile_condition("release.profile.jeryu-forge-v1", &["G6"]),
    profile_condition("release.profile.platform-linux-aarch64", &["G9", "G10"]),
    profile_condition("release.profile.platform-linux-x86_64", &["G9", "G10"]),
    profile_condition("release.profile.platform-macos-aarch64", &["G9", "G10"]),
    profile_condition("release.profile.platform-macos-x86_64", &["G9", "G10"]),
    profile_condition("release.profile.platform-windows-x86_64", &["G9", "G10"]),
    profile_condition("release.profile.provider-antigravity", &["G5"]),
    profile_condition("release.profile.provider-claude", &["G5"]),
    profile_condition("release.profile.provider-codex", &["G5"]),
    profile_condition("release.profile.provider-cursor", &["G5"]),
    profile_condition("release.profile.saga-v1", &["G18"]),
    profile_condition(
        "release.profile.self-hosted-v1",
        &[
            "G1", "G2", "G3", "G5", "G6", "G8", "G9", "G10", "G13", "G14",
        ],
    ),
    profile_condition("release.profile.team-v1", &["G17"]),
    profile_condition(
        "release.profile.universal-v1",
        &["G1", "G2", "G3", "G5", "G6", "G7", "G8", "G9", "G10", "G16"],
    ),
];

pub(crate) const NATIVE_ROWS: &[ClaimRow] = &[
    ClaimRow {
        id: "release.package-linux-x86_64",
        gap_ids: &["G9"],
        claim: "No signed Ubuntu 24.04 x86_64 package with the embedded Portal, services, migrations, sandbox assets, and guest image exists.",
        why: "The portable profile needs the concrete native archive and its supply-chain subjects; a platform condition or an internal builder test is not a package.",
        acceptance: "Build the exact tagged x86_64-unknown-linux-gnu package inside the admitted different-identity builder, semantically verify every subject, and register its signed receipt.",
        exists: "a quarantined internal one-target builder component exercised by focused tests; the public `release build` command refuses before parsing or output because different-identity containment is absent",
        owner: Owner::LocalThenExternal {
            offline: "the different-identity builder boundary and package semantic verifier are Hub release engineering",
            external: "exact signed family subjects, protected signing roots, and the admitted Linux builder environment",
        },
        next: none(
            "the public `release build` command returns RELEASE_BUILD_CONTAINMENT_UNAVAILABLE before parsing or output; no admitted package-build command exists yet",
        ),
    },
    ClaimRow {
        id: "release.systemd-v1",
        gap_ids: &["G1", "G9"],
        claim: "No native systemd install, upgrade, activation, rollback, uninstall, and non-destructive retention receipt exists.",
        why: "A signed archive is not an operable Linux release until native lifecycle transitions preserve the prior or whole next generation and read their service state back.",
        acceptance: "Run two clean Ubuntu 24.04 installs plus upgrade, activation, rollback, uninstall, retention, and disaster drills from signed package bytes and register the exact receipt.",
        exists: "release and setup runbooks plus structural bundle verification; no activation helper, generation lifecycle, clean-host drill, or signed lifecycle receipt",
        owner: Owner::LocalThenExternal {
            offline: "the narrow activation service, generation lifecycle, rollback, and fault harness are Hub release engineering",
            external: "two clean Ubuntu 24.04 hosts, signed package bytes, protected signing roots, and trusted time",
        },
        next: none(
            "no typed native activation or lifecycle-drill command exists; source setup and archive extraction are not installers",
        ),
    },
];

const fn provider(
    id: &'static str,
    claim: &'static str,
    exists: &'static str,
    lane: &'static str,
) -> ClaimRow {
    ClaimRow {
        id,
        gap_ids: &["G5"],
        claim,
        why: PROVIDER_WHY,
        acceptance: PROVIDER_ACCEPTANCE,
        exists,
        owner: PROVIDER_OWNER,
        next: command(lane, PROVIDER_NOTE),
    }
}

pub(crate) const ROWS: &[ClaimRow] = &[
    ClaimRow {
        id: "release.backup-restore",
        gap_ids: &["G3", "G9"],
        claim: "Nobody has restored a tagged backup and read the restored state back as an exact-subject release receipt.",
        why: "A backup whose restore has never been observed protects nothing; the pointed-at bytes, not the database, are the recovery subject.",
        acceptance: "Run the tagged backup/restore suite from release bytes and register a signed receipt binding backup digest, restore epoch, and the exact four-repository subjects.",
        exists: "Kernel `798f0c8` `farm backup|restore` with an exact receipt and quarantined offline restore (COMPONENT_PROOF); restored state is not admitted for production use",
        owner: Owner::LocalThenExternal {
            offline: "the backup/restore suite and its receipt shape are Kernel code (V1-S2)",
            external: "release bytes from signed tagged subjects (G9) and production restore admission (G3)",
        },
        next: command(
            "cd ../bullet-kernel && cargo run --locked -p bullet -- farm backup --database /abs/ledger.sqlite --output /abs/snapshot.sqlite --receipt /abs/backup-receipt.json",
            "component backup with an exact receipt; `farm restore --backup … --receipt … --destination …` restores into quarantine only, and neither run is a tagged-bytes release receipt",
        ),
    },
    ClaimRow {
        id: "release.checksums",
        gap_ids: &["G9"],
        claim: "No checksum set is admitted for the exact artifact set selected by any release profile.",
        why: "Without checksums an installer cannot tell substituted bytes from released bytes.",
        acceptance: "Generate and re-read checksums for the requested profile's exact target set and bind the profile id, closure digest, targets, and checksums in its signed manifest; `universal-v1` must bind all five targets.",
        exists: "hub `a1b7ab3` + `ab1fd8f` build and re-read a BLAKE3 checksum manifest for one unsigned Linux x86_64 component bundle; it is not a signed checksum set admitted for any profile, and the four additional universal targets have no checksum subjects",
        owner: Owner::LocalThenExternal {
            offline: "checksum generation and aggregation for the exact targets selected by the requested profile are hub engineering (V1-S7)",
            external: "the requested profile's exact archives and signed manifest; `universal-v1` additionally needs its four additional target archives",
        },
        next: none("no profile-preserving checksum producer or admission command exists"),
    },
    ClaimRow {
        id: "release.fault-suite",
        gap_ids: &["G2", "G3"],
        claim: "Nobody has crashed the tagged family at every SQLite/WAL/CAS/journal/generation boundary and read back either the prior or the whole next state.",
        why: "Crash safety that was only reasoned about is not crash safety; one unobserved boundary can lose or duplicate an effect.",
        acceptance: "Run the tagged crash-boundary and recovery fault suite from release bytes and register its exact signed receipt.",
        exists: "Kernel atomic lease/command/outbox, injected setup transaction boundaries (hub `94b6549`), and BulletGit prior-or-whole-next recovery (`274fd6d`, `f551736`) as COMPONENT_PROOF",
        owner: Owner::LocalThenExternal {
            offline: "the crash-boundary and recovery fault suite is Kernel/BulletGit engineering (V1-S2/S3)",
            external: "the tagged release bytes the suite must run from",
        },
        next: none(
            "`just model-check` covers only the two bounded formal protocols (lease/fence/reclaim, command/effect ambiguity), not the crash-boundary suite",
        ),
    },
    ClaimRow {
        id: "release.forge.github-app",
        gap_ids: &["G7"],
        claim: "No GitHub App has integrated an exact Candidate into a protected test repository and read the result back.",
        why: "GitHub is an effect adapter; until a lost response reconciles to UNKNOWN and read-back adopts the original OID, every GitHub effect is unproved.",
        acceptance: "Configure the GitHub App test repository with branch protection and register exact-subject dispatch, check, integration, read-back, and reconciliation receipts.",
        exists: "a specified, uncertified effect adapter (ADR 0002, ADR 0008); no App, test repository, or credential",
        owner: Owner::LocalThenExternal {
            offline: "the typed GitHub App capability, delivery, check, integration, read-back, reconciliation, and semantic receipt-admission paths are effects/Hub engineering",
            external: "the operator-provisioned GitHub App, branch-protected test repository, and separate delivery, attestation, integration, and observation credentials",
        },
        next: none("no typed GitHub App command exists"),
    },
    ClaimRow {
        id: "release.forge.jeryu",
        gap_ids: &["G6"],
        claim: "No protected Jeryu integration has run with restored operator authentication and produced a read-back receipt.",
        why: "Jeryu is the source forge; an integration that never read its own result back cannot claim protected-ref safety.",
        acceptance: "With operator-restored authentication run read-only probes, then one exact protected integration with UNKNOWN/read-back reconciliation, and register the receipt; never modify the running forge.",
        exists: "a local bare-forge component and a typed-quarantine Jeryu adapter; no authenticated read-back",
        owner: Owner::LocalThenExternal {
            offline: "typed read-only probes, protected integration/read-back reconciliation, and semantic receipt admission are effects/Hub engineering",
            external: "operator-restored scoped authentication on the unmodified pinned forge and an operator-named test repository",
        },
        next: none(
            "the read-only probes that come first have no typed command yet; never modify the running forge",
        ),
    },
    ClaimRow {
        id: "release.installable-lock",
        gap_ids: &["G1"],
        claim: "The checked-in family.lock is still schema 2, which every installer path rejects by design.",
        why: "Installation from the hub cannot start until a lock carries authenticated Jeryu sources and signed exact subjects.",
        acceptance: "Publish immutable signed member tags, generate the schema-3 lock with Jeryu URL/slug, exact tree OIDs, lockfiles, and artifact digests, and commit it under a signed hub tag.",
        exists: "schema-3 generation and strict verification code (`bullet-family lock generate|verify`), descriptor-relative setup, and the honest schema-2 refusal (`doctor` reports `UNSUPPORTED_SCHEMA`)",
        owner: Owner::External(
            "operator publishes signed immutable member tags with an authenticated Jeryu URL/slug; the schema-3 lock can only be generated from those",
        ),
        next: command(
            "bullet-family lock generate --tag <prospective-version> --subjects <absolute-path>",
            "at the clean prospective Hub HEAD, generates the schema-3 lock from authenticated signed non-Hub tags for that version; commit the lock and sign the Hub tag last, then verify that exact tag. Until those subjects exist `bullet-family doctor --json` reports the schema-2 refusal, and that refusal is correct",
        ),
    },
    ClaimRow {
        id: "release.installer-twice",
        gap_ids: &["G1"],
        claim: "Nobody has run the signed prebuilt installer twice in a fresh HOME from tagged hub-only bytes.",
        why: "An operator-selected external executable that has not itself passed signed package admission is not an installer receipt; idempotence and exact clean member OIDs must be observed, not assumed.",
        acceptance: "From a signed prebuilt bullet-family binary and tagged bytes, run setup twice in a fresh HOME and register exact clean member OIDs, zero tracked changes, and no worktrees.",
        exists: "two-run component fixture with local source transport and sealed Linux Cargo/Node/Bash/npm subjects (hub `94b6549`, `7efe2f3`); `scripts/setup.sh` stays a pre-admission source wrapper",
        owner: Owner::LocalThenExternal {
            offline: "admission of every remaining Git/helper subject, production Jeryu transport, and the exact two-run installer receipt path are Hub engineering",
            external: "a signed prebuilt `bullet-family` from tagged bytes, authenticated Jeryu sources, protected signer roots, and two clean fresh-HOME hosts",
        },
        next: none(
            "`scripts/setup.sh` delegates to an operator-selected external `bullet-family` and passes explicit Cargo/Node/npm paths, but that executable is not yet authenticated as a signed release package; running the wrapper twice is not the installer receipt",
        ),
    },
    ClaimRow {
        id: "release.jankurai-90",
        gap_ids: &["G8"],
        claim: "No pinned Jankurai report reads 90 or better with zero caps and zero hard findings for the exact release subjects.",
        why: "Hard findings and caps are release blockers; a machine-local or skip-green audit lane cannot substitute for the pinned receipt.",
        acceptance: "Run pinned Jankurai 1.6.11 with zero skips against the exact tagged subjects and register the score/cap/hard-finding receipt at 90 or better with zero caps and zero hard findings.",
        exists: "pinned local jankurai 1.6.11 lane that fails closed below its floor and ratchets upward only (`just audit`); no hosted checksum-pinned artifact",
        owner: Owner::LocalThenExternal {
            offline: "reduce hard findings and caps to zero and raise the score to 90 with the pinned local binary",
            external: "a checksum-pinned hosted CI Jankurai artifact and the exact tagged subjects",
        },
        next: command(
            "just audit",
            "pinned local lane with an upward-only floor; a local binary is not the CI artifact and this checkout is not the tagged subject",
        ),
    },
    ClaimRow {
        id: "release.manifest-non-circular",
        gap_ids: &["G9"],
        claim: "No signed final profile manifest binds the hub tag and selected artifact closure without embedding its own digest.",
        why: "A manifest that includes its own digest cannot be checked; a manifest without a signature cannot be trusted.",
        acceptance: "Generate the final manifest binding profile id, closure digest, hub tag, schema-3 lock, byte-sorted selected targets, and archive/SBOM/provenance digests, then sign it with the protected release key; self-hosted-v1 requires its one Ubuntu x86_64 entry and universal-v1 requires five entries.",
        exists: "hub `release verify` checks an exact non-circular signed five-target manifest against a preassembled fixture (`352f963`); no builder or signer",
        owner: Owner::LocalThenExternal {
            offline: "the profile-preserving manifest producer and semantic admission path are Hub release engineering",
            external: "the requested profile's exact artifacts plus the protected release signing key and signer identity",
        },
        next: none(
            "no profile-preserving manifest producer or signer exists; the current schema-2/five-target verifier is a quarantined universal-envelope component and cannot admit self-hosted-v1",
        ),
    },
    ClaimRow {
        id: "release.package-matrix",
        gap_ids: &["G9"],
        claim: "No release profile has a signed, lifecycle-smoked archive set for its exact selected targets; self-hosted-v1 selects one Ubuntu x86_64/systemd archive and later universal-v1 selects five targets.",
        why: "A release is the exact packaged bytes selected by its profile; the Vite preview lane and a separately built farmd are not package evidence.",
        acceptance: "Build and lifecycle-smoke the requested profile's exact tagged archive set with the Portal embedded, then register every archive digest; self-hosted-v1 requires one Ubuntu x86_64/systemd archive and universal-v1 requires all five targets.",
        exists: "hub `a1b7ab3` + `ab1fd8f` emit one unsigned Linux x86_64 component archive with the committed Portal embedded and all eight release binaries; it is not a signed self-hosted-v1 package, while the four additional universal-v1 targets are absent",
        owner: Owner::LocalThenExternal {
            offline: "profile-preserving packaging and lifecycle-smoke commands are Hub release engineering (V1-S7)",
            external: "the protected signer and exact build hosts for the selected profile; universal-v1 additionally needs its four non-first-GA target hosts",
        },
        next: none(
            "no profile-preserving package producer or lifecycle-smoke admission command exists; the frozen five-target verifier cannot admit self-hosted-v1",
        ),
    },
    ClaimRow {
        id: "release.platform-containment",
        gap_ids: &["G10"],
        claim: "No release profile has an admitted containment or typed mutation-refusal receipt for its exact selected platform set.",
        why: "Linux needs a production containment receipt; any selected platform without an equivalent backend must fail closed, and that target-specific refusal is itself a receipt.",
        acceptance: "For each requested platform profile, bind the exact target and profile-closure digest to either its production containment receipt or its fail-closed mutation-refusal receipt; `universal-v1` must admit all five target bindings, while `self-hosted-v1` selects Linux x86_64 only.",
        exists: "Linux user+net namespace, slirp4netns, nftables, and CONNECT-proxy isolation proofs (Kernel `d388733`, `ops/ci/egress.sh`); no non-Linux backend or refusal receipt",
        owner: Owner::LocalThenExternal {
            offline: "profile-preserving containment certification, typed mutation-refusal, and semantic receipt-admission commands are platform/Hub engineering",
            external: "independently controlled target hosts and the requested profile's exact packaged bytes; `universal-v1` additionally requires all five target bindings",
        },
        next: none("no profile-preserving containment certification command exists"),
    },
    ClaimRow {
        id: "release.provenance",
        gap_ids: &["G9"],
        claim: "No signed build provenance statement is admitted for the exact artifacts selected by any release profile.",
        why: "Without provenance a consumer cannot connect archive bytes to the hub tag, lock, and toolchains that produced them.",
        acceptance: "Produce provenance bound to the exact profile, closure digest, hub tag, lock, toolchains, and selected archive digests, sign it, and re-read it with the release verifier; `universal-v1` must bind all five targets.",
        exists: "the quarantined internal Linux component writes and re-reads unsigned in-toto provenance bound to its one-target component subject; `release verify` re-reads provenance inside a signed bundle; no signed selected-profile provenance is admitted, and the four additional universal target subjects are absent",
        owner: Owner::LocalThenExternal {
            offline: "the profile-preserving provenance producer and semantic admission path are Hub release engineering",
            external: "the protected release key, independent build-attestor identity, and requested profile's exact selected archive digests; `universal-v1` additionally needs its four additional target archives",
        },
        next: none(
            "no profile-preserving provenance producer or admission command exists; the current schema-2/five-target verifier is a quarantined universal-envelope component and cannot admit self-hosted-v1",
        ),
    },
    provider(
        "release.provider.antigravity",
        "No admitted Antigravity binary has produced a sealed PONG receipt through the live-conformance path; the path exists (Kernel `ba485d5`, `b4735da`, `0d848f6`; `docs/runbooks/live-conformance.md`) but every run so far refused at POLICY_LIVE_ADMISSION_DISABLED under the checked-in v1alpha1 policy, so the offline structured-output subset and the zero-spawn neutral refusal are the only Antigravity evidence.",
        "Kernel `5badc85` bounded structured-output subset; the common policy→key→lease→admission→grant→egress→read-only-turn→canary→receipt path with the v1alpha2 loader mirror (`0d848f6`); every run refuses before spawn under the checked-in policy",
        "cd ../bullet-kernel && BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/bullet-data/policy/policy.json BULLET_LIVE_PROVIDERS=agy bash ops/ci/nightly.sh",
    ),
    provider(
        "release.provider.claude",
        "No admitted Claude binary has produced a sealed PONG receipt through the live-conformance path; the path exists (Kernel `ba485d5`, `b4735da`, `0d848f6`; `docs/runbooks/live-conformance.md`) but every run so far refused at POLICY_LIVE_ADMISSION_DISABLED under the checked-in v1alpha1 policy, so the offline stream-JSON subset, the deep fake-process proof, and the zero-spawn neutral refusal are the only Claude evidence.",
        "Kernel `c34d578` bounded stream-JSON subset and the only deep positive fake-process proof; the common policy→key→lease→admission→grant→egress→read-only-turn→canary→receipt path with the v1alpha2 loader mirror (`0d848f6`); every run refuses before spawn under the checked-in policy",
        "cd ../bullet-kernel && BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/bullet-data/policy/policy.json BULLET_LIVE_PROVIDERS=claude bash ops/ci/nightly.sh",
    ),
    provider(
        "release.provider.codex",
        "No admitted Codex binary has produced a sealed PONG receipt through the live-conformance path; the path exists (Kernel `ba485d5`, `b4735da`, `0d848f6`; `docs/runbooks/live-conformance.md`) but every run so far refused at POLICY_LIVE_ADMISSION_DISABLED under the checked-in v1alpha1 policy, so the offline App Server JSONL subset and the zero-spawn neutral refusal are the only Codex evidence.",
        "Kernel `ca376e4` bounded App Server JSONL subset; the common policy→key→lease→admission→grant→egress→read-only-turn→canary→receipt path with the v1alpha2 loader mirror (`0d848f6`); every run refuses before spawn under the checked-in policy",
        "cd ../bullet-kernel && BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/bullet-data/policy/policy.json BULLET_LIVE_PROVIDERS=codex bash ops/ci/nightly.sh",
    ),
    provider(
        "release.provider.cursor",
        "No admitted Cursor binary has produced a sealed PONG receipt through the live-conformance path; the path exists (Kernel `ba485d5`, `b4735da`, `0d848f6`; `docs/runbooks/live-conformance.md`) but every run so far refused at POLICY_LIVE_ADMISSION_DISABLED under the checked-in v1alpha1 policy, so the offline ACP subset and the zero-spawn neutral refusal are the only Cursor evidence.",
        "Kernel `ea89929` bounded ACP subset; the common policy→key→lease→admission→grant→egress→read-only-turn→canary→receipt path with the v1alpha2 loader mirror (`0d848f6`); every run refuses before spawn under the checked-in policy",
        "cd ../bullet-kernel && BULLET_LIVE_REAL=1 BULLET_POLICY_PATH=/abs/bullet-data/policy/policy.json BULLET_LIVE_PROVIDERS=cursor bash ops/ci/nightly.sh",
    ),
    ClaimRow {
        id: "release.receipt-contracts",
        gap_ids: &["G12"],
        claim: "The receipt verifier has only ever checked fixtures; no external allowed-signers policy or real signed release receipt has been provisioned.",
        why: "A verifier without a provisioned signer policy and a real receipt is a component, not release evidence.",
        acceptance: "Implement and hostile-test each selected receipt kind's semantic verifier, then provision the external signer policy, trusted-time observation, replay state, and exact tagged receipts; both this admission gate and the requested profile-condition receipt must pass, and neither can substitute for the other.",
        exists: "strict canonical receipt/policy verifier with exact OpenSSH signer/namespace/interval checks (hub `143f8b9`) plus the MSRV receipt-admission path (`d762f86`); only fixtures have ever been checked",
        owner: Owner::LocalThenExternal {
            offline: "kind-specific semantic validators and typed admission for the exact requested profile closure are Hub engineering",
            external: "an independently provisioned allowed-signers policy, trusted time, replay/high-water state, and real signed receipts from tagged bytes",
        },
        next: none(
            "legacy-v1-26 has no admission command; conformant profiles render their exact selected-registry command",
        ),
    },
    ClaimRow {
        id: "release.rust-msrv-1-95",
        gap_ids: &["G9"],
        claim: "No admitted Rust 1.95 build-and-test receipt is available to this profile; conformant profiles use their selected semantic registry, while legacy-v1-26 deliberately keeps this gate static.",
        why: "MSRV is a release promise; a local toolchain match today is not a tagged-bytes build receipt, and a self-selected signer or a generic result digest cannot clear the gate.",
        acceptance: "Provision the selected semantic registry with role-separated source-tag, build-attestor, and trusted-time roots, then supply the receipt binding the schema-3 lock, clean signed subjects, dependency-lock digests, exact Rust 1.95 rustc/cargo bytes, and zero-skip build/test observations for all three Rust workspaces.",
        exists: "the selected-registry structural admission path and local lanes that already build under rustc 1.95.0 (`rust-toolchain.toml`, `scripts/ci-doctor.sh`); legacy-v1-26 ignores registry contents and no profiled path reads the legacy fixed descriptor",
        owner: Owner::LocalThenExternal {
            offline: "the admission path and the 1.95 lane are hub code; every local lane already runs under rustc 1.95.0",
            external: "a selected semantic registry with three distinct admitted signer roots, an independently signed time observation, and the schema-3 lock the receipt must bind",
        },
        next: none(
            "legacy-v1-26 has no admission command; conformant profiles render their exact selected-registry command",
        ),
    },
    ClaimRow {
        id: "release.rust-pinned-1-97-1",
        gap_ids: &["G9"],
        claim: "Nobody has built and tested the exact tagged family with pinned Rust 1.97.1 and registered the receipt.",
        why: "The pinned toolchain is the second required build; one toolchain receipt never covers the other.",
        acceptance: "Build and test the exact tagged family with the admitted pinned Rust 1.97.1 toolchain under cargo --locked and register the receipt.",
        exists: "the explicit `just toolchain-pinned` Rust 1.97.1 build/test lane emits a subject-, tool-, argv-, environment-, and output-bound machine-local observation; it is unsigned diagnostic input, and no condition-specific admission path or release receipt exists",
        owner: Owner::LocalThenExternal {
            offline: "the pinned 1.97.1 lane exists; its condition-specific semantic admission path remains hub engineering",
            external: "the exact tagged family bytes and the same signer/time roots as the MSRV receipt",
        },
        next: command(
            "just toolchain-pinned",
            "builds and tests this Hub under pinned Rust 1.97.1 and emits an unsigned observation only; it cannot clear the release gate",
        ),
    },
    ClaimRow {
        id: "release.sbom",
        gap_ids: &["G9"],
        claim: "No SBOM set is admitted for the exact artifacts selected by any release profile.",
        why: "Consumers cannot audit or respond to a vulnerability in bytes whose contents were never enumerated.",
        acceptance: "Generate and validate CycloneDX and SPDX SBOMs for the requested profile's exact target set and bind profile id, closure digest, targets, and SBOM digests in the signed manifest; universal requires all five targets.",
        exists: "hub `a1b7ab3` + `ab1fd8f` generate and re-read a CycloneDX 1.6 SBOM with typed component, package URL, and admitted-license fields for one unsigned Linux x86_64 component bundle; no signed SBOM set is admitted for any profile, and the four additional universal target subjects are absent",
        owner: Owner::LocalThenExternal {
            offline: "SBOM generation and validation over the requested profile's exact built archives",
            external: "the requested profile's exact archives and signed manifest that binds each SBOM digest; `universal-v1` additionally needs its four additional target archives",
        },
        next: none("no profile-preserving SBOM producer or admission command exists"),
    },
    ClaimRow {
        id: "release.scan.dependency",
        gap_ids: &["G8", "G9"],
        claim: "No pinned dependency scan receipt exists for the exact release lockfiles.",
        why: "A scan run on a working tree at some earlier commit says nothing about the tagged lockfiles.",
        acceptance: "Run the admitted cargo-deny 0.19.8 against the exact tagged lockfiles and register its receipt.",
        exists: "pinned cargo-deny 0.19.8 checks advisories, licenses, bans, and sources against this checkout after refreshing and age-checking RustSec; no tagged-family scan receipt exists",
        owner: Owner::LocalThenExternal {
            offline: "the pinned four-category cargo-deny lane over this checkout",
            external: "the exact tagged lockfiles of all three Rust workspaces",
        },
        next: command(
            "just security",
            "runs current-tree secrets, canary, cargo-deny advisories/licenses/bans/sources, and strict workflow analysis on this checkout only; not the exact tagged family",
        ),
    },
    ClaimRow {
        id: "release.scan.license",
        gap_ids: &["G8", "G9"],
        claim: "No pinned license policy scan receipt exists for the exact release artifacts.",
        why: "License violations discovered after tagging invalidate the release bytes.",
        acceptance: "Run the admitted license scanner against the exact archives and SBOMs and register its receipt.",
        exists: "pinned cargo-deny 0.19.8 checks the configured license policy for this checkout together with advisories, bans, and sources; no exact-archive or SBOM receipt exists",
        owner: Owner::LocalThenExternal {
            offline: "the pinned cargo-deny license-policy lane over this checkout",
            external: "the exact tagged archives and SBOMs it must run against",
        },
        next: command(
            "just security",
            "checks the checkout's lockfile license policy; exact tagged archives, their SBOMs, and an admitted release receipt remain",
        ),
    },
    ClaimRow {
        id: "release.scan.secret",
        gap_ids: &["G8", "G9"],
        claim: "No pinned secret scan receipt exists for the exact tagged trees.",
        why: "A canary or credential in tagged bytes is unrecoverable once published.",
        acceptance: "Run the admitted gitleaks 8.21.2 against every exact tagged tree and register its receipt.",
        exists: "pinned gitleaks 8.21.2 scans the current Hub source and lockfiles before dependency resolution, and a secret-shaped canary proves genuine findings fail; no exact tagged-family receipt exists",
        owner: Owner::LocalThenExternal {
            offline: "the pinned current-tree scan and genuine-finding canary",
            external: "every exact tagged tree of the four repositories",
        },
        next: command(
            "just security",
            "scans this current Hub tree and proves a canary is detected; the exact tagged trees of all four repositories remain",
        ),
    },
    ClaimRow {
        id: "release.scan.workflow",
        gap_ids: &["G8", "G9"],
        claim: "No pinned workflow policy scan receipt exists for the exact workflow bytes.",
        why: "Hosted workflows are a supply-chain surface; unpinned or over-permissioned steps break provenance.",
        acceptance: "Run the admitted zizmor 1.25.2 against the exact workflow bytes and register its receipt.",
        exists: "pinned zizmor 1.25.2 runs offline with ignores disabled and strict collection on this checkout; actionlint 1.7.8 and repository policy meta-tests cover the same workflow inventory in lint",
        owner: Owner::LocalThenExternal {
            offline: "the pinned zizmor lane over this checkout",
            external: "the exact tagged workflow bytes",
        },
        next: command(
            "just security",
            "runs strict workflow analysis on this checkout only; not the exact tagged workflow bytes or a hosted release receipt",
        ),
    },
    ClaimRow {
        id: "release.signatures",
        gap_ids: &["G9"],
        claim: "No requested profile artifact set carries signatures from admitted protected release keys.",
        why: "An unsigned archive, checksum set, SBOM, or manifest can be replaced by anyone who can write to the download path.",
        acceptance: "Sign every archive, checksum set, SBOM, provenance statement, and final manifest selected by the requested profile, bind its closure digest and target set, and re-read them with the release verifier; universal requires the five-target set.",
        exists: "hub `release verify` checks detached signatures and exact Ed25519 signer status inside an assembled bundle; nothing signs",
        owner: Owner::LocalThenExternal {
            offline: "the profile-preserving signing plan, producer, and signature-admission path are Hub release engineering",
            external: "protected release keys, a distinct signer identity, and the requested profile's exact artifact set",
        },
        next: none(
            "no profile-preserving signer or signature-admission command exists; the current schema-2/five-target verifier is a quarantined universal-envelope component and cannot admit self-hosted-v1",
        ),
    },
    ClaimRow {
        id: "release.transaction-demo",
        gap_ids: &["G2"],
        claim: "Nobody has produced a signed five-plane TRANSACTION_PROOF from `just demo`; the demo ends in self-signed component evidence.",
        why: "The offline transaction is the baseline every live and release gate builds on; component evidence renamed is still not a transaction.",
        acceptance: "Run the connected tagged demo through real child boundaries and the protected local forge simulator, ending in one signed TRANSACTION_PROOF covering authority, runner death/salvage, independent verification, ambiguous-effect reconciliation, protected integration, preservation, and truthful projection.",
        exists: "`just demo` runs the Kernel simulator and emits a self-signed COMPONENT_PROOF receipt; atomic lease/command/outbox, fail-closed `bullet-gitd`, the fixture E2 verifier, and Portal PENDING→UNKNOWN remain component evidence",
        owner: Owner::Local(
            "Kernel + BulletGit + Portal engineering (V1-S4); the TRANSACTION_PROOF is credential-free and offline",
        ),
        next: command(
            "just demo",
            "today ends in self-signed component evidence from the simulator; a connected receipt must replace that evidence, not rename it",
        ),
    },
];
