# Historical BulletFarm archive

This branch preserves superseded implementations and source evidence. Its instructions,
plans, grants and reviews are historical records; they do not govern current development
or constitute current approvals. Development occurs only on `neverhuman/bulletfarm/main`.

`snapshots/<repository>/` contains each source main tree byte-for-byte at the inventory.
`ref-index.json` maps every inventoried ref to an immutable `archive/<repository>/...`
tag with its original Git object ID and authorship. `records/` records source inventories
and GitHub exports; more evidence is appended as capture completes. PR heads are under
`archive/<repository>/pull/<number>/head`, with their review evidence in
`records/<repository>/pulls/<number>/`. Local reflog and branch refs retain original objects.

No local Git configuration, credential stores or live coordination database is published.
An independent private backup outside the development checkout retains local state.
GitHub records are historical evidence, not newly imported native GitHub reviews.
Retirement remains blocked until every available export and restoration gate passes.
