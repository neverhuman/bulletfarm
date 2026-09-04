# bullet-farm operations

Keep each CI entrypoint in `ops/ci/` and expose it through `scripts/ci-local.sh`.
Required lanes must run from the canonical checkout without fetching source.
Never publish, mirror, tag, or mutate forge state from a proof lane.
