# Start here — BulletFarm build contract 3.0

Read `BULLETFARM_FINAL_ENGINEERING_SPEC.md`. Part I compares the supplied plans; Parts II–IV define the one target to build; Part V and the appendices define build order, scenarios and contracts. Archived plans are provenance, not competing defaults.

## What is here

A detailed design, seven strict v3 schemas, nine synthetic examples, structural SQL, 22 core plus eight deferred work packages, 80 required future runtime scenarios, source/decision preservation, comparative scores and a static validator. There is no finished Rust orchestrator, live grant, production credential, certified sandbox or measured performance claim.

## Begin without keys

Use the existing authorized Git/CI workflow. Inspect existing source first and preserve conforming modules. Implement BF3-001 through the dependency-ready G0 packages using fakes. Define ordinary Rust types and one command/state transaction kernel. Build the first live boundary only after identity, budget, isolation, job recovery and independent checks exist. One selected executor is enough for G1; actual two-provider coding and two people are required for G2.

The first-party stack is Rust plus React/TypeScript/Vite. External certified harnesses may use other languages. One clean-install SQLite store and one allocator are defaults; retain a conforming existing backend by an explicit decision rather than implement two.

## Check this package

```bash
python3 -m pip install -r requirements-validation.txt
python3 validate_package.py
```

This checks schemas, examples, graphs, score arithmetic, hashes and selected SQLite constraints. It does not run the product scenarios. Read `VALIDATION_REPORT.json` for the exact result.

## One agent iteration

Choose a dependency-ready work package from `backlog.json`. Map its proposed paths onto real code; bind owner and grants only for live actions. Write the negative fixture, implement its acceptance, run actual repository checks, obtain independent review and update a truthful `BUILD_CHECKPOINT.json`. Nulls in this package are unresolved environment inputs, not permission for unlimited spending. A source comment or old PASS never supplies a current grant.

## Do not lose these distinctions

A Job is supervised execution; a Task is the promise. Writer authority is not physical occupancy. A pending-change reservation outlives the author. A draft PR may exist before CI-only gates, but it is not review-ready until required preparation passes. Historical merge does not prove validity after a known revert. A model acting for an admin is not a human approval. Profiles are configuration, not permission. Unknown spend/effects remain unknown until reconciled.

## Independent rebuild test

A fresh capable agent gets this kit and an empty tree without previous conversations. It builds the fake-backed command/claim/candidate/verification/effect/status loop. A separate reviewer executes the acceptance cases and additional negatives. Record missing assumptions. That rebuild has not been performed by document generation.
