# Bullet Farm hub command surface.

default:
    @just --list

[positional-arguments]
setup *args:
    #!/bin/bash
    exec /bin/bash scripts/setup.sh "$@"

[positional-arguments]
coord *args:
    cargo run --locked --quiet --bin bullet-family -- coord "$@"

demo:
    bash scripts/demo.sh

proof-transaction-offline:
    bash scripts/proof-transaction-offline.sh

preview:
    bash scripts/preview.sh

dev:
    bash scripts/dev.sh

readme-record:
    bash scripts/readme-record.sh

readme-render:
    bash scripts/readme-render.sh

readme-check:
    bash scripts/readme-check.sh

fast:
    bash scripts/ci-local.sh fast

lint:
    bash scripts/ci-local.sh lint

docs:
    bash scripts/ci-local.sh docs

check:
    bash scripts/ci-local.sh required

contract:
    bash scripts/ci-local.sh contract

family-contract:
    bash scripts/ci-local.sh family-contract

contract-generate:
    cargo run --locked --quiet -p bullet-wire --bin bullet-contract -- generate --root .

contract-check:
    cargo run --locked --quiet -p bullet-wire --bin bullet-contract -- check --root .

model-check:
    bash formal/model-check.sh

security:
    bash scripts/ci-local.sh security

release-truth:
    bash scripts/release-truth.sh write

audit:
    bash scripts/ci-local.sh audit

toolchain-pinned:
    bash scripts/ci-local.sh toolchain-pinned

paper:
    bash scripts/paper-build.sh

paper-check:
    bash scripts/paper-check.sh

[positional-arguments]
ci-doctor lane="all":
    bash scripts/ci-doctor.sh "$1"

hooks-install:
    git config --local core.hooksPath ops/git-hooks

family:
    bash scripts/ci-local.sh family

verify: check

farmd:
    bash scripts/farmd.sh

portal:
    bash scripts/portal.sh

check-family:
    bash scripts/ci-local.sh family

[positional-arguments]
lock-generate tag subjects:
    cargo run --locked --quiet --bin bullet-family -- lock generate --tag "$1" --subjects "$2"

[positional-arguments]
lock-verify tag:
    cargo run --locked --quiet --bin bullet-family -- lock verify --tag "$1"
