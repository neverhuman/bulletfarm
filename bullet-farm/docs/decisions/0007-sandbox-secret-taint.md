# ADR 0007: Sandbox, secrets, and tainted tool data

Status: Accepted
Owner: Bullet Kernel maintainers
Last reviewed: 2026-08-24
Applies to: provider, repository, and gate execution

## Decision

Linux is the strong-isolation reference. S1 is rootless OCI with namespaces, cgroup v2, seccomp,
read-only root, private HOME/tmp/XDG/cache, bounded mounts, and deny-by-default network. S2 is a
separate-kernel backend for untrusted or hidden-evaluation work.

Provider enclave, BulletGit writer, gate, verifier, broker, attestor, and observer receive distinct
minimal credentials and filesystems. Prompt/repository/provider/tool bytes are tainted and may only
propose typed intents. Gates are immutable argv-vector GateSpecs, never arbitrary shell text.

## Consequence

The host HOME and mutable PATH are not inherited. Containment certification is mandatory before
live admission.
