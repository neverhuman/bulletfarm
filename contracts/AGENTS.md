# Hub contract boundary

Read the repository `AGENTS.md` first. Source contracts live in
`contracts/v1alpha1/`; generated bundles and bindings are listed in
`agent/generated-zones.toml` and must never be hand-edited.

Regenerate with `just contract-generate`. Prove byte drift with
`just contract-check`; changes to the formal contract gate also require
`just model-check`.
