# ADR 0010: Supply-chain and release policy

Status: Accepted
Owner: Bullet Farm maintainers
Last reviewed: 2026-08-24
Applies to: build, CI, and release

## Decision

Pin Rust, Node, Java/TLC, actions, dependencies, and certified external binaries. Required builds
are locked/offline where supported. Secret, dependency, vulnerability, source, and license scanners
are installed and blocking. Trust-boundary parsers receive fuzz/property/sanitizer coverage.

Releases produce signed SBOM, SLSA provenance, reproducibility evidence, and immutable artifacts.
Signer/builder expectations, revocation, supported versions, disclosure, and rollback are explicit.

Release receipts use the frozen canonical TOML receipt and signer-policy contracts. The receipt binds its exact
kind, family, tag, hub commit/tree, tool binary/version, profile, configuration, subject, result digest, ordered
timestamps, signer identity, and the domain-separated digest of the entire admitted policy. The policy is an
explicit absolute-path trust-root input, scopes each Ed25519 signer to exact receipt kinds and a validity interval,
and fixes the OpenSSH namespace `bullet-farm-release-receipt-v1`. Environment variables, default search paths,
workspace files, and a policy bundled with the receipt never become implicit trust.

Contract verification is not semantic adjudication. It does not establish trusted current time, provision an
independent policy, prove that a claimed test or live effect happened, or clear a release gate. Each kind needs its
own independent exact-subject verifier and an externally provisioned policy before a real receipt can be registered.

## Consequence

Missing tools and scanner failures are failures, not skips. Wave 2 supply-chain acceptance remains
required before a production claim.
