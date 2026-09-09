# Historical and operator-supplied media

Status: capture provenance and production-account qualification remain open.

Historical capture files are retained, unqualified local observations. Their
old authentication labels have not been independently established. They provide
no accepted production Bullet transaction, account qualification, or release
receipt. The current canonical `collection.json` is absent, so the no-argument
`just readme-live-check` correctly refuses with exit 78.

The current `readme-live-*` scripts process explicitly supplied capture data.
They perform no provider execution or authentication lookup and cannot establish
that an operator-supplied account, model or execution claim is true. The former
recorder and original metadata remain in retained source and audit records.

For a compatible, separately reviewed capture, the data stages accept explicit
absolute paths and fresh private output directories outside the source family:

```bash
just readme-live-record --from-capture /private/capture --staged-root /private/new-normalized
just readme-live-check --normalized-stage /private/new-normalized
just readme-live-render --from-captures /private/four-captures --staged-root /private/new-rendered \
  --tools-receipt /private/reviewed-tools.json
just readme-live-check --rendered-collection /private/new-rendered \
  --tools-receipt /private/reviewed-tools.json
```

Normalization verifies the declared data shape and byte identity. Collection
rendering requires the selected local tool receipt, pinned container and all
four compatible capture inputs. Its text playback and supplied screenshots do
not establish continuous native recording. The local tool receipt is not a
portable toolchain admission. Select a canonical, private file outside the source
family whose bytes match the reviewed `.config/readme-live-tools.sha256` pin.
The scripts refuse a missing or mismatched receipt. Normalization needs no tool
receipt. If a canonical collection is later present, checking it also requires
`just readme-live-check --tools-receipt /private/reviewed-tools.json`.

The [private capture and rendering runbook](../demo-gif/README.md) describes the
separate PTY capture, browser sampling and pixel-checked media tools. It records
lossless master preservation, GIF palette and timing limits, and the remaining
runtime qualification. Existing historical media and private captures require
separate review before any public export.

Ordinary documentation checks continue to reconstruct the accepted
[component media](../readme-media/README.md). A future accepted public live
collection must add its exact current-generation reconstruction to CI. No such
collection or production-account demonstration is admitted by this document.
