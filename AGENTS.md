# Bullet Farm Split Family

`/home/ubuntu/bullet` is the split-family container. It is not a Git repository.
Every product repo lives as an independent checkout under this directory.

Read `repos.manifest.toml` first. Read and append `/home/ubuntu/bullet/AGENT_CHAT.md`
before every claim, edit, and handoff. Re-read immediately before writing.
Repository-local `AGENTS.md` files add path-specific rules; this family rule
always remains in force.

## Zero-new-worktree rule

Do not create Git worktrees anywhere, for any reason. Work only in the claimed
canonical primary checkout of each member repo.

## Family map

| Repo | Role |
| --- | --- |
| `bullet-farm` | Public hub, installer, onboarding, family lock, spec corpus. Public GitHub index: `https://github.com/neverhuman/bulletfarm` (not `neverhuman/bullet-farm`) |
| `bullet-kernel` | Control-plane modular monolith and trust-boundary bins |
| `bullet-git` | BulletGit types, capability API, journal, proof roots |
| `bullet-portal` | Vite + React operations portal |

## Jeryu

Consume Jeryu through pinned tags. The only permitted Jeryu family is
`/home/ubuntu/jain-split/jeryu-split`. Do not recreate `/home/ubuntu/jeryu-split`.
Do not add committed `path = "../..."` dependencies.

## Proof

From the hub:

```bash
cargo run --locked --quiet --bin bullet-family -- doctor --json
just fast
just demo
```

`just setup` refuses first unless `BULLET_SETUP_ADMITTED_BIN` and the explicit absolute
Cargo/Node/npm subjects are supplied. Once those bootstrap inputs exist, the selected external CLI
rejects the checked-in schema-2 `family.lock` with `UNSUPPORTED_SCHEMA`; see
`bullet-farm/docs/runbooks/source-setup.md`. The family index is `README.md` in this directory.
