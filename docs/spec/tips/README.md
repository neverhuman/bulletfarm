# Supplied `tips/*.md` source set

These three files are byte-for-byte copies of the complete source set supplied in
`/home/ubuntu/bullet/tips` on 2026-09-18. They are retained together so a fresh reviewer
can verify that the repository did not silently omit or rewrite an input document.

| File | SHA-256 | Role |
| --- | --- | --- |
| `BULLETFARM_FINAL_ENGINEERING_SPEC.md` | `1d8064b40041e9383b66b790eb8eaa41924657db5f5d1bb0a5373fc5c501b9eb` | Canonical 3.0 specification; also copied to the parent directory for the active documentation link. |
| `BULLETFARM_FINAL_ENGINEERING_SPEC(1).md` | `ec3d305487dcd715df3b8b17050aaaf21144f664b0f53919334feaa246cfeec8` | Earlier complete adjudication and provenance. It does not create a second active backlog or override the canonical specification. |
| `START_HERE.md` | `0d7652b43504949fa2e71999bf2cb6f1b7ab25971e2334d1fb3db5f9e4740632` | Original build-contract entry point and provenance. |

`START_HERE.md` says the supplied package contains seven schemas, nine examples,
`backlog.json`, `requirements-validation.txt`, `validate_package.py`, and
`VALIDATION_REPORT.json`. Those files are absent from the supplied `tips/` directory.
Their absence is tracked as GAP-002 in the active [closure plan](../../closure-plan.md).
No repository check claims that the unavailable validator or its reported 90 static checks
has run against this code.

The canonical specification governs conflicts. The `(1)` document's 24 core packages,
seven `BF3-X` experiments, and 80 `Q3` scenarios are preserved as design provenance; the
canonical document adjudicates them into BF3-001–030, 90 acceptance criteria, and the
AT/HF/CF scenario inventory used by current development.
