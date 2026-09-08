"""Hostile in-memory mutations for the typed assurance inventory."""

from __future__ import annotations

import copy
from collections.abc import Callable

from model import (
    Inventory, _validate_invariants, _validate_ledger, _validate_reports,
)
from strict_io import InventoryError, unique_objects


def _must_refuse(label: str, action: Callable[[], object]) -> None:
    try:
        action()
    except InventoryError:
        print(f"self-test: PASS {label}")
        return
    raise InventoryError(f"self-test: hostile mutation passed: {label}")


def run(baseline: Inventory) -> None:
    cases = 0

    def check_ledger(mutator: Callable[[dict], None]) -> None:
        document = copy.deepcopy(baseline.ledger)
        mutator(document)
        _validate_ledger(document)

    _must_refuse("missing G5", lambda: check_ledger(lambda d: d["gaps"].pop(4)))
    cases += 1
    _must_refuse(
        "duplicate G1",
        lambda: check_ledger(lambda d: d["gaps"].append(copy.deepcopy(d["gaps"][0]))),
    )
    cases += 1
    _must_refuse(
        "one-way gap/wave edge",
        lambda: check_ledger(lambda d: d["waves"][0]["gaps"].clear()),
    )
    cases += 1
    _must_refuse(
        "unknown evidence label",
        lambda: check_ledger(lambda d: d["slices"][0].update(evidence_class="IMPLEMENTED")),
    )
    cases += 1
    _must_refuse(
        "missing WP-23", lambda: check_ledger(lambda d: d["work_packages"].pop())
    )
    cases += 1
    _must_refuse(
        "unknown top-level field",
        lambda: check_ledger(lambda d: d.update(unexpected=True)),
    )
    cases += 1
    _must_refuse(
        "missing profile condition gate",
        lambda: check_ledger(
            lambda d: d["release_profiles"][0].update(
                condition_gate="release.profile.absent"
            )
        ),
    )
    cases += 1

    def duplicate_corpus_unit() -> None:
        document = copy.deepcopy(baseline.corpus)
        document["units"].append(copy.deepcopy(document["units"][0]))
        unique_objects(document["units"], "id", "corpus.units")

    _must_refuse("duplicate corpus unit", duplicate_corpus_unit)
    cases += 1

    def check_invariants(mutator: Callable[[dict], None]) -> None:
        document = copy.deepcopy(baseline.invariants)
        mutator(document)
        _validate_invariants(document, baseline.ledger)

    _must_refuse(
        "enforced invariant without proof",
        lambda: check_invariants(
            lambda d: next(
                entry for entry in d["entries"] if entry["lifecycle"] == "enforced"
            ).update(proof_command="")
        ),
    )
    cases += 1

    profiles = {item["name"]: item for item in baseline.ledger["release_profiles"]}
    gates = {item["id"]: item for item in baseline.ledger["gate_bindings"]}

    def check_reports(
        mutator: Callable[[list], None],
        gate_mutator: Callable[[dict], None] | None = None,
    ) -> None:
        reports = copy.deepcopy(baseline.reports)
        bindings = copy.deepcopy(gates)
        mutator(reports)
        if gate_mutator is not None:
            gate_mutator(bindings)
        _validate_reports(reports, profiles, bindings, baseline.receipt_kinds)

    _must_refuse("missing profile report", lambda: check_reports(lambda rows: rows.pop()))
    cases += 1
    _must_refuse(
        "zero-gate profile partition",
        lambda: check_reports(lambda rows: rows[0].update(gates=[])),
    )
    cases += 1

    def mutate_class(reports: list) -> None:
        reports[0]["gates"][0]["class"] = "LIVE"

    _must_refuse("runtime gate class drift", lambda: check_reports(mutate_class))
    cases += 1

    def mutate_kind(bindings: dict) -> None:
        gate = next(value for value in bindings.values() if value["receipt_kind"] is not None)
        gate["receipt_kind"] = "invented-kind"

    _must_refuse("unknown receipt kind", lambda: check_reports(lambda _: None, mutate_kind))
    cases += 1
    print(f"self-test: PASS {cases}/{cases} typed hostile mutations")
