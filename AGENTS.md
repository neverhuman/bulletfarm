# bullet-farm Agent Instructions

Read `SPLIT.md` first. This repository is the public hub of the Bullet Farm
split family.
Discover the outermost family root from `repos.manifest.toml`. Read its
`AGENT_CHAT.md` before every claim and edit, and use `bullet-family coord` for
machine-enforced claims. Never commit a machine-local absolute chat path.

Read `agent/JANKURAI_STANDARD.md` next.
Do not edit outside requested ownership. Run the mapped test lane before the
final response.

- Canonical local Jeryu slug: `root/bullet-farm`.
- Do not add committed cross-repo `path = "../..."` dependencies.
- Do not hand-edit generated artifacts listed in `agent/generated-zones.toml`.
- Run `bash scripts/ci-local.sh required` before handing off changes.
- Spec documents under `docs/spec/` are design corpus, not runtime authority.
