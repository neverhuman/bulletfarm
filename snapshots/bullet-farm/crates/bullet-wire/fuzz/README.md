# Corpus replay prerequisite

Status: deterministic component check; not release evidence.

This standalone crate (not a workspace member) replays checked-in
`decode_canonical_value` seeds against filename-bound admission or typed-refusal
outcomes. It rejects an unexpected corpus entry, a symlinked corpus or seed, an
over-bound seed, a seed whose path or identity changes while it is read, and an
outcome that differs from the committed expectation. On Unix the replay retains
the opened corpus directory, enumerates and opens seeds relative to that
descriptor with no-follow semantics, bounds each same-descriptor read to
1,048,577 bytes, and revalidates directory and seed identity before accepting
an outcome. Platforms without that custody implementation refuse the replay.

This harness does **not** implement cargo-fuzz targets, the pinned 100,000-run
nightly job, or closure item L-06. Those remain blocked, as do all Bullet
release profiles. Run the covered component check with:

```bash
cargo test --locked --manifest-path crates/bullet-wire/fuzz/Cargo.toml
cargo run --locked --manifest-path crates/bullet-wire/fuzz/Cargo.toml --bin replay
```
