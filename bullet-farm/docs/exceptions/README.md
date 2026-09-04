# Dated exceptions

An exception is a reviewed, expiring explanation for a boundary that cannot
yet meet its normal rule. It is never an authority token, test waiver, release
receipt, or permission to translate `UNKNOWN` into success.

Every exception record must name:

- the exact file, operation, tool, provider, or platform subject;
- an owner and review date;
- the invariant that remains enforced;
- the narrow reason the normal rule cannot currently apply;
- an expiry date and removal trigger;
- a migration or containment plan; and
- an executable proof lane whose failure invalidates the exception.

Exceptions must not hide missing credentials, unsigned tools, failing tests,
unsupported containment, release blockers, or product/runtime semantics such
as `STALE`, `UNKNOWN`, and provisional filesystem generations. Those states are
real and remain visible. Expired exceptions fail closed until removed or
re-approved with fresh evidence.

The Hub's typed repair taxonomy is in [`../errors.md`](../errors.md), with its
machine-readable exception surface in `agent/exceptions.toml`.
