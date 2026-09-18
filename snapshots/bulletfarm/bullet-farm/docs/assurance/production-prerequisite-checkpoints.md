# Local production prerequisite checkpoints

Status: **retained component observations; no release authority**

Owner: Bullet Farm maintainers

Last reconciled: 2026-09-09

The [active plan](full-product-dogfood-plan.md) and [gap register](product-gaps.md)
remain the governing program. Every observation applies to its named source
subject; a later commit requires its own proof.

## Retained September 8 observations

The original checkpoint text is preserved byte-for-byte, in order, in
[production prerequisites and backup observations](prerequisite-observations-20260908.md)
and [publication, health and local proof observations](health-observations-20260908.md).
The two files contain 440 and 452 lines. Their concatenation has SHA256
`b19704fca07fd128ba691084604342f5f5355a3c1035d93a2d6fe3838e45dff5`.
Historical failures and narrower receipts retain their original status.

## September 9 health checkpoint

The [baseline manifest](health-audit-baseline.json) and
[complete repair queue](health-repair-queue.jsonl) were committed at
`dd01f6fdd9d6e3fce52c39a2eb7f7645bbb49d56`, tree
`c0e42543ab52ffa416254a438fc4660f07f53727`. They preserve all 165 original
findings and 28 cap occurrences from the four September 8 scans, including
original dirty source subjects. All 193 entries remain unresolved triage inputs;
these are auditor findings, not 193 demonstrated product defects. No higher
unqualified score, source move or exception silently closes an entry.
Independent queue review SHA256:
`1333640e48bcf603286a4ebfcfba888b972087cd6f27d40053fce2808012d82a`.

Kernel formatting-only packets are committed as
`b524f89e2ff066031522b2c1f4a25ba61acf4fa2`,
`c0cb715da5efc540a80f9e73d9c9ced12b84b565` and
`a7e1fd32aa9be56dae699e761f863232459d9920`. Their final tree is
`76f265b5c9d1835e72b09669927745ce2cd0716a`.
Independent review SHA256:
`9d1d9158710ea81fc80632363bd4803386f623f04b315204fcba256383816a59`.

The subsequent complete Kernel required run failed after 411.718 seconds.
All 1,074 fast tests, formatting and strict Clippy passed, then the inventory
check refused a renamed test identity. The remaining lint checks, contracts,
security and documentation were not reached. Its retained log SHA256 is
`83a4a21d4cf98b4a81e38fa3e135099dc3ca6533c4e546a9522391ec129a7f09`;
independent failed-run read-back SHA256 is
`7091cef66abe7e36087060025a87009d5b0d6980463dac75d35cbd5e507bf4b2`.

The inventory repair is committed as
`21e504a486f6e1d3aa94dfcd4036a967bdd31d6a`, tree
`2a766426fd56a12e84d9bbd2346c6330d5a9a371`. The entire process-group source
matches its prior bytes after replacing only the helper and test identifiers.
Two identity digests and the registered Gitd test-source anchors were updated;
all counts and partition filters remain unchanged. Actual inventory checking
passed for 1,120 identities, with ShellCheck and five source-anchor controls.
Enumeration supplies no new test-execution credit. Packet SHA256
`c9d5fe6df92ac90ff6ad6b362cdb56415106f5c80cdd5f946fc35d2eb377721b`;
independent runtime review SHA256
`6b4921dfbdadb602cb6ca8f3dc92a217effe5bfb1c27d4941f07d7d4a9155e0a`.
The complete required-r4 run on this clean subject passed in 470.949 seconds,
with 1,074 fast and 34 contract tests: 1,108 distinct completed cases. All
required lanes and source/tool/configuration guards passed. Result SHA256
`d1c24c163ad2e86aeedb24ebeaad8c1ea7e49acdcd65375d4470ebef2463a739`;
independent review SHA256
`0a8242c4a00465dfc279cb7c9189462edad1b538246b35ccff462ee78e2a9967`.
The earlier required-r3 command passed but concurrent configuration changes
invalidated its guard; it remains diagnostic evidence only. Neither run supplies
family, egress, hosted CI or release-campaign acceptance.

Jankurai's browser-storage repair is committed locally as
`c995f83837a1554a25291842e253c16a5aa82b53`, tree
`6baf198de1d96278770851bd3b4dacc545c8c404`. All 65 workspace tests, including
ten new regression groups, and strict Clippy passed. Accessor names no longer
supply credential evidence by themselves; unsafe values, supported operations
and conservative HTML fallback remain covered. There is no blanket CSRF-name
exemption or complete JavaScript value-flow/security certification. Packet SHA256
`12f99ff0959c620f53865d26a870134802f2ccfd7aff11e3d7d94bdc850a1b98`;
independent runtime review SHA256
`b3fac53af144740598d2a337632c7be0872cc4092e4814f4f2a04322c6abacd1`.

The TypeScript boundary repair is committed locally as
`ace7c01eea36bf2b6c9dbf9057bea72ab1997f87`, tree
`46f5ab47dcb609cb1e0eba486e39b7ad5efad594`. All 123 workspace tests, including
eight new regression groups, and strict Clippy passed. Primitive `unknown` is
distinguished from unsafe assertions and subsequent narrowing; bounded aliases,
nested assignments and division/regex controls retain unsafe detections. This
is not a type checker or a complete control-flow proof. Packet SHA256
`9494fdceffd42cfbcb872c8ad736fff1dcd78cbf1e9d386585a25acde5d6b9a8`;
independent runtime review SHA256
`2a999f4581a9a3a17c0c87825dfcf988160059f99cd36203b72a956b9426dd9e`.
Both auditor repositories' required wrappers run metadata only; their full tests
and strict Clippy were executed separately. Neither packet updates selected
immutable dependencies or admits a portable auditor artifact.

Jankurai's tool-execution repair is committed as
`0444744ffb1b41e0b09769f1f7d7e7e61e57c31c`, tree
`e529eae8193b4c6a475d48f9e03091b8f2e379f5`. All 76 analyzer workspace tests,
including 11 new regression groups, and strict Clippy passed. A supported,
reachable CI declaration supplies route information only; inert catalogs,
comments or command strings cannot establish executed artifacts. Actual admitted
execution observations remain required. Packet SHA256
`55bcf80a4d1a2feaad569e69f5a24522857159ea866f66506895ec225a2150f5`;
independent runtime review SHA256
`184ff7d29b9341ec475da3c4f809022485e16539478158f854d8532a2198477f`.
Core still selects earlier dependency commits; this local repair does not
qualify the selected executable or a portable auditor.

Hub's capture-normalization packet is committed as
`73fb192b546b754dd3c7913d83616dd240be4223`, tree
`9580689beabbfb7499e2a7ef9e9a626029b4581b`. All 15 custody groups and ShellCheck
passed. Literal captured replies are retained; unmeasured model, account,
authentication, source and transaction facts remain unknown. Packet SHA256
`d44fb2e57e613a1d63fab37385b43a4d4c32204bbf3e00ca34d5132113df5003`;
independent review SHA256
`ba1495b247c72a8df8a31ef93926d4c133a1ba093e26dd87ce31fa91a577ce50`.
This committed intermediate packet refuses complete media qualification.
The subsequent offline renderer is under review; current Hub full proof waits
for its integration, reviewed capture/tool inputs and complete reconstruction.
Historical version-1 media retains its original bytes and limited provenance.

BulletGit's test-only import repair is committed as
`f0c8605595564bf990b6a52735e8710b4e9a6440`, tree
`61a422cd305dd2a967e0c2f8093fe9a46dc23b1a`. Existing generation tests, strict
Clippy and formatting passed; runtime behavior is unchanged. Independent source
review SHA256:
`4c28e68850425317a0eb0e4ae867ab050f6c1b4603cdbf48bbf455d8a771f4a2`.
The complete required-r3 run passed 222 tests but used the checkout Cargo target;
its independent review explicitly retained the private-target gap. A subsequent
cold run, required-r5, passed all required lanes in 135.802 seconds with a new
0700 private target, unchanged source/tool/configuration guards and 2,208,353,374
logical build-output bytes. Its raw log SHA256 is
`92784ec15929ead6948695f548b2ce962051b53d42362b8e75910e69c624499c`;
independent review SHA256 is
`2f4021dd0d730c3cf03a36f0c46c3a03e4701b9f70317c672a9830152dd196a3`.
Nextest's separate
repository-local report store is preserved; it is not the Cargo build target.

The intervening required-r4 run failed before compilation when secret scanning
matched a generated Rustdoc search-index fragment from required-r3. Its redacted
report SHA256 is
`42962df41e763682bc9e59d551387c92615d6a15defd80d7a06d7e892e3858de`.
All 964 generated documentation entries were retained outside the checkout,
with an unchanged preservation manifest SHA256
`d14bd0f6ae9b7bc137865da24de712970c54148b10385d0eaab7055a4b9bcf5b`.
No source-scanning exemption was added. Newly generated private documentation
scans clean, but the old search-index filename/bytes were not reproduced; its
exact false-positive disposition remains unconfirmed. Failed-run reports and
older successful receipts are retained separately from current evidence.

Typed inventory SHA256 remains
`d0df5aea6d15366246b8b58ae998605c8f30240be2859eb593bb567e9f16326f`.
All G1–G18 remain `DESIGNED`, all 18 product and two diagnostic profiles remain
`BLOCKED`, and the coordinator Operating HOLD remains effective. Complete
current-family checks, qualified score-90 audits and hosted CI remain open.
The primary JeRyu aggregate still requires administrator provisioning; local
review commits do not establish private or public integration authority.
