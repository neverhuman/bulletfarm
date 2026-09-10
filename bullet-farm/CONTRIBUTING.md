# Contributing

Status: **contributor map; not install or release authority**  
Last reviewed: 2026-09-10

A clone of [neverhuman/bulletfarm](https://github.com/neverhuman/bulletfarm) is
the public entry. Member origins are `neverhuman/bullet-farm`,
`neverhuman/bullet-kernel`, `neverhuman/bullet-git`, and
`neverhuman/bullet-portal`. Do not create Git worktrees.

## Local console (unsigned)

Pin Rust **1.95.0**, Node **22.23.2**, and npm **10.9.8**. Then:

```bash
cd bullet-farm
just preview          # doctor BLOCKED / exit 3 is expected
just console -- --data-dir "$HOME/.local/state/bullet-operator-console"
```

Follow [`docs/runbooks/loopback-console.md`](docs/runbooks/loopback-console.md).
`just setup` is the blocked installer and still refuses on the schema-2
`family.lock`. Operating HOLD remains. This is not VERIFIED.

Do not paste bootstrap tokens, cookies, CSRF values, or home paths into issues
or pull requests.

## Packets

- At most four claimed files per change.
- Root is the Kernel integrator for `Cargo.toml`, generated contracts, and
  `talk.rs` (absent). Do not invent `bullet talk` / `ask` / `head`.
- Agents never write provider keys, enrollments, or operator-decision lines.
- Hosted merge is the five secretless lanes plus `required`. Do not add
  Tuiwright, Head Playwright, or `xbabe2-head` to workflows.

## Review

Open PRs against the member that owns the bytes. The aggregate snapshot is a
copy, not source authority.
