"""Typed whole-product assurance inventory validation."""

from __future__ import annotations

from collections import Counter
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from family import load_repositories, validate_adr, validate_corpus_source, validate_subject
from strict_io import (
    InventoryError, digest, exact_keys, expect_list, expect_object,
    load_relative, text, unique_objects,
)

EVIDENCE = ["DESIGNED", "COMPONENT", "TRANSACTION", "LIVE", "RELEASE"]
WAVES = [f"W{index}" for index in range(12)]
GAPS = [f"G{index}" for index in range(1, 19)]
SLICES = [f"V1-S{index}" for index in range(9)]
WORK_PACKAGES = [f"WP-{index:02d}" for index in range(1, 24)]
PRODUCT_PROFILES = [
    "self-hosted-v1", "evolution-v1", "provider-claude", "provider-codex",
    "provider-cursor", "provider-antigravity", "jeryu-forge-v1",
    "github-adapter-v1", "gitlab-adapter-v1", "gitlab-self-managed-v1",
    "platform-linux-x86_64", "platform-linux-aarch64", "platform-macos-x86_64",
    "platform-macos-aarch64", "platform-windows-x86_64", "universal-v1",
    "team-v1", "saga-v1",
]
DIAGNOSTIC_PROFILES = ["legacy-v1-26", "linux-preview"]
INVARIANT_FIELDS = {
    "control_ids", "documentation_anchor", "enforcement_target",
    "first_applicable_wave", "gate", "id", "introduced_in", "legacy_aliases",
    "lifecycle", "milestone", "owner", "proof_command", "statement",
    "threat_class", "tier", "trust_plane", "violation_event", "violation_mode",
}
SOURCE_NAMES = {
    "family_manifest", "corpus", "invariants", "receipt_kinds", "inventory_schema",
}


@dataclass(frozen=True)
class Inventory:
    ledger: dict[str, Any]
    corpus: dict[str, Any]
    invariants: dict[str, Any]
    receipt_kinds: list[str]
    reports: list[dict[str, Any]]
    source_digests: dict[str, str]
    corpus_counts: Counter[str]
    invariant_counts: Counter[str]
    gate_class_counts: Counter[str]
    receipt_kind_counts: Counter[str]
    subject_counts: Counter[str]


def _refs(value: Any, allowed: set[str], label: str, allow_empty: bool = False) -> list[str]:
    values = expect_list(value, label)
    if not values and not allow_empty:
        raise InventoryError(f"{label}: zero-item partition")
    if any(not isinstance(item, str) or item not in allowed for item in values):
        raise InventoryError(f"{label}: unknown or non-string reference")
    if len(values) != len(set(values)):
        raise InventoryError(f"{label}: duplicate reference")
    return values


def _evidence(value: Any, label: str) -> str:
    if value not in EVIDENCE:
        raise InventoryError(f"{label}: expected one of {EVIDENCE}")
    return value


def _source_contracts(ledger: dict[str, Any]) -> dict[str, dict[str, Any]]:
    contracts = expect_object(ledger["source_contracts"], "source_contracts")
    if set(contracts) != SOURCE_NAMES:
        raise InventoryError("source_contracts: missing or extra typed source")
    for name, raw in contracts.items():
        contract = expect_object(raw, f"source_contracts.{name}")
        exact_keys(contract, {"scope", "path", "schema"}, set(), f"source_contracts.{name}")
        if contract["scope"] not in {"hub", "family-root"}:
            raise InventoryError(f"source_contracts.{name}: unknown scope")
        text(contract["path"], f"source_contracts.{name}.path")
        text(contract["schema"], f"source_contracts.{name}.schema")
    return contracts


def _validate_ledger(ledger: dict[str, Any]) -> tuple[
    dict[str, dict[str, Any]], dict[str, dict[str, Any]],
    dict[str, dict[str, Any]], dict[str, dict[str, Any]],
]:
    exact_keys(
        ledger,
        {
            "schema", "authority", "evidence_classes", "follow_on_phases",
            "source_contracts", "corpus_roles", "gaps", "waves", "slices",
            "work_packages", "release_profiles", "gate_bindings",
        },
        set(), "inventory",
    )
    if ledger["schema"] != "bullet.assurance-inventory.v1":
        raise InventoryError("inventory: unsupported schema")
    if ledger["authority"] != "planning-and-component-inventory-only":
        raise InventoryError("inventory: authority label drifted")
    if ledger["evidence_classes"] != EVIDENCE:
        raise InventoryError("inventory: evidence vocabulary drifted")
    if ledger["follow_on_phases"] != ["POST_V1", "RETIRED"]:
        raise InventoryError("inventory: follow-on phase vocabulary drifted")
    _source_contracts(ledger)

    gap_rows, gaps = unique_objects(ledger["gaps"], "id", "gaps")
    if list(gaps) != GAPS:
        raise InventoryError("gaps: expected exact ordered G1..G18")
    for row in gap_rows:
        exact_keys(row, {"id", "waves", "evidence_class"}, {"indirect_gate"}, f"gap {row['id']}")
        _refs(row["waves"], set(WAVES), f"gap {row['id']}.waves")
        _evidence(row["evidence_class"], f"gap {row['id']}.evidence_class")

    wave_rows, waves = unique_objects(ledger["waves"], "id", "waves")
    if list(waves) != WAVES:
        raise InventoryError("waves: expected exact ordered W0..W11")
    for row in wave_rows:
        exact_keys(
            row, {"id", "depends_on", "gaps", "slices", "work_packages", "evidence_class"},
            set(), f"wave {row['id']}",
        )
        dependencies = _refs(
            row["depends_on"], set(WAVES), f"wave {row['id']}.depends_on", allow_empty=True
        )
        if any(int(item[1:]) >= int(row["id"][1:]) for item in dependencies):
            raise InventoryError(f"wave {row['id']}: dependency is not earlier")
        _refs(row["gaps"], set(GAPS), f"wave {row['id']}.gaps")
        _refs(row["slices"], set(SLICES), f"wave {row['id']}.slices", allow_empty=True)
        _refs(
            row["work_packages"], set(WORK_PACKAGES),
            f"wave {row['id']}.work_packages", allow_empty=True,
        )
        _evidence(row["evidence_class"], f"wave {row['id']}.evidence_class")

    slice_rows, slices = unique_objects(ledger["slices"], "id", "slices")
    if list(slices) != SLICES:
        raise InventoryError("slices: expected exact ordered V1-S0..V1-S8")
    for row in slice_rows:
        exact_keys(row, {"id", "waves", "evidence_class"}, set(), f"slice {row['id']}")
        _refs(row["waves"], set(WAVES), f"slice {row['id']}.waves")
        _evidence(row["evidence_class"], f"slice {row['id']}.evidence_class")

    package_rows, packages = unique_objects(ledger["work_packages"], "id", "work_packages")
    if list(packages) != WORK_PACKAGES:
        raise InventoryError("work_packages: expected exact ordered WP-01..WP-23")
    for row in package_rows:
        exact_keys(
            row, {"id", "lifecycle", "waves", "follow_on", "owner", "evidence_class"},
            set(), f"work package {row['id']}",
        )
        routed = _refs(
            row["waves"], set(WAVES), f"work package {row['id']}.waves", allow_empty=True
        )
        follow_on = _refs(
            row["follow_on"], {"POST_V1", "RETIRED"},
            f"work package {row['id']}.follow_on", allow_empty=True,
        )
        if row["lifecycle"] == "ACTIVE" and not routed and follow_on != ["POST_V1"]:
            raise InventoryError(f"work package {row['id']}: active row is orphaned")
        if row["lifecycle"] == "RETIRED" and (routed or follow_on != ["RETIRED"]):
            raise InventoryError(f"work package {row['id']}: retired routing drifted")
        if row["lifecycle"] not in {"ACTIVE", "RETIRED"}:
            raise InventoryError(f"work package {row['id']}: unknown lifecycle")
        owner = text(row["owner"], f"work package {row['id']}.owner")
        if not owner.replace("-", "").isalnum() or not owner[0].islower():
            raise InventoryError(f"work package {row['id']}: invalid owner")
        _evidence(row["evidence_class"], f"work package {row['id']}.evidence_class")

    for gap in gap_rows:
        for wave_id in WAVES:
            if (wave_id in gap["waves"]) != (gap["id"] in waves[wave_id]["gaps"]):
                raise InventoryError(f"gap/wave edge is not bidirectional: {gap['id']} <-> {wave_id}")
    for group, field in ((slice_rows, "slices"), (package_rows, "work_packages")):
        for row in group:
            for wave_id in WAVES:
                if (wave_id in row["waves"]) != (row["id"] in waves[wave_id][field]):
                    raise InventoryError(f"{row['id']}/wave edge is not bidirectional: {wave_id}")

    profile_rows, profiles = unique_objects(ledger["release_profiles"], "name", "release_profiles")
    expected_profiles = PRODUCT_PROFILES + DIAGNOSTIC_PROFILES
    if list(profiles) != expected_profiles:
        raise InventoryError("release_profiles: exact profile order drifted")
    for row in profile_rows:
        exact_keys(
            row, {"name", "kind", "condition_gate", "current_result"}, set(),
            f"profile {row['name']}",
        )
        product = row["name"] in PRODUCT_PROFILES
        if row["kind"] != ("product" if product else "diagnostic"):
            raise InventoryError(f"profile {row['name']}: kind drifted")
        if row["current_result"] != "BLOCKED":
            raise InventoryError(f"profile {row['name']}: result drifted")
        if product:
            text(row["condition_gate"], f"profile {row['name']}.condition_gate")
        elif row["condition_gate"] is not None:
            raise InventoryError(f"profile {row['name']}: diagnostic has condition gate")

    gate_rows, gates = unique_objects(ledger["gate_bindings"], "id", "gate_bindings")
    if len(gates) != 46 or list(gates) != sorted(gates):
        raise InventoryError("gate_bindings: expected 46 sorted gates")
    for row in gate_rows:
        exact_keys(
            row, {"id", "gaps", "required_evidence_class", "receipt_kind"}, set(),
            f"gate {row['id']}",
        )
        _refs(row["gaps"], set(GAPS), f"gate {row['id']}.gaps")
        if row["required_evidence_class"] not in {"TRANSACTION", "LIVE", "RELEASE"}:
            raise InventoryError(f"gate {row['id']}: invalid evidence class")
    for profile in profile_rows:
        condition = profile["condition_gate"]
        if condition is not None and condition not in gates:
            raise InventoryError(f"profile {profile['name']}: condition gate is absent")
    for gap in gap_rows:
        direct = any(gap["id"] in gate["gaps"] for gate in gate_rows)
        indirect = gap.get("indirect_gate")
        if not direct and indirect is None:
            raise InventoryError(f"gap {gap['id']}: no direct or indirect gate")
        if indirect is not None and indirect not in gates:
            raise InventoryError(f"gap {gap['id']}: indirect gate is absent")
    return gaps, waves, profiles, gates


def _validate_corpus(
    root: Path, corpus: dict[str, Any], ledger: dict[str, Any],
    repositories: dict[str, Path],
) -> tuple[Counter[str], Counter[str]]:
    exact_keys(corpus, {"schema", "corpus", "units"}, set(), "corpus")
    if corpus["schema"] != ledger["source_contracts"]["corpus"]["schema"]:
        raise InventoryError("corpus: schema drifted")
    source_rows, sources = unique_objects(corpus["corpus"], "key", "corpus.sources")
    roles = expect_object(ledger["corpus_roles"], "corpus_roles")
    if set(sources) != set(roles):
        raise InventoryError("corpus: source roles are missing or orphaned")
    family_root = root.parent
    for source in source_rows:
        exact_keys(source, {"key", "path", "title"}, set(), f"corpus source {source['key']}")
        if roles[source["key"]] not in {
            "historical-provenance", "analysis-source", "design-source",
        }:
            raise InventoryError(f"corpus source {source['key']}: unknown role")
        text(source["title"], f"corpus source {source['key']}.title")
        validate_corpus_source(family_root, source["path"], f"corpus source {source['key']}")

    unit_rows, units = unique_objects(corpus["units"], "id", "corpus.units")
    counts: Counter[str] = Counter()
    subjects: Counter[str] = Counter()
    for unit in unit_rows:
        exact_keys(
            unit, {"id", "doc", "ref", "unit", "disposition", "anchor"},
            {"partial", "note"}, f"corpus unit {unit['id']}",
        )
        if unit["doc"] not in sources:
            raise InventoryError(f"corpus unit {unit['id']}: unknown source")
        text(unit["ref"], f"corpus unit {unit['id']}.ref")
        text(unit["unit"], f"corpus unit {unit['id']}.unit")
        disposition = unit["disposition"]
        expected = {
            "IMPLEMENTED": "test", "PLANNED": "wave",
            "REFUSED": "adr", "SUPERSEDED": "adr",
        }.get(disposition)
        if expected is None:
            raise InventoryError(f"corpus unit {unit['id']}: unknown disposition")
        anchor = expect_object(unit["anchor"], f"corpus unit {unit['id']}.anchor")
        if anchor.get("kind") != expected:
            raise InventoryError(f"corpus unit {unit['id']}: disposition/anchor mismatch")
        if expected == "wave":
            exact_keys(anchor, {"kind", "value"}, set(), f"corpus unit {unit['id']}.anchor")
            if anchor["value"] not in WAVES:
                raise InventoryError(f"corpus unit {unit['id']}: stale wave")
        elif expected == "adr":
            exact_keys(anchor, {"kind", "value"}, set(), f"corpus unit {unit['id']}.anchor")
            validate_adr(root, anchor["value"], f"corpus unit {unit['id']}.anchor")
        else:
            validate_subject(
                anchor, repositories, f"corpus unit {unit['id']}.anchor", {"test"}
            )
            subjects["test"] += 1
        partial = unit.get("partial")
        if partial is not None:
            partial = expect_object(partial, f"corpus unit {unit['id']}.partial")
            validate_subject(
                partial, repositories, f"corpus unit {unit['id']}.partial", {"symbol", "test"}
            )
            subjects[partial["kind"]] += 1
        counts[disposition] += 1
        counts[f"doc:{unit['doc']}"] += 1
    if len(units) != 648:
        raise InventoryError(f"corpus: expected 648 units, found {len(units)}")
    if counts["IMPLEMENTED"] == 0 or counts["PLANNED"] == 0 or counts["doc:paper"] == 0:
        raise InventoryError("corpus: zero implemented, planned, or paper partition")
    return counts, subjects


def _validate_invariants(document: dict[str, Any], ledger: dict[str, Any]) -> Counter[str]:
    exact_keys(document, {"entries", "registry_version", "schema_version"}, set(), "invariants")
    if document["schema_version"] != ledger["source_contracts"]["invariants"]["schema"]:
        raise InventoryError("invariants: schema drifted")
    text(document["registry_version"], "invariants.registry_version")
    rows, _ = unique_objects(document["entries"], "id", "invariants.entries")
    counts: Counter[str] = Counter()
    for row in rows:
        exact_keys(row, INVARIANT_FIELDS, set(), f"invariant {row['id']}")
        lifecycle = row["lifecycle"]
        if lifecycle not in {"enforced", "planned"}:
            raise InventoryError(f"invariant {row['id']}: unknown lifecycle")
        wave = row["first_applicable_wave"]
        if type(wave) is not int or f"W{wave}" not in WAVES:
            raise InventoryError(f"invariant {row['id']}: unknown first wave")
        for key in ("owner", "gate", "statement", "documentation_anchor"):
            text(row[key], f"invariant {row['id']}.{key}")
        enforcement, proof = row["enforcement_target"], row["proof_command"]
        if lifecycle == "enforced" and (not enforcement or not proof):
            raise InventoryError(f"invariant {row['id']}: enforced row lacks implementation/proof")
        if lifecycle == "planned" and (enforcement or proof):
            raise InventoryError(f"invariant {row['id']}: planned row claims implementation/proof")
        counts[lifecycle] += 1
        counts[f"owner:{row['owner']}"] += 1
    if counts["enforced"] == 0 or counts["planned"] == 0:
        raise InventoryError("invariants: zero enforced or planned partition")
    return counts


def _receipt_kinds(schema: dict[str, Any], ledger: dict[str, Any]) -> list[str]:
    if schema.get("schema_version") != ledger["source_contracts"]["receipt_kinds"]["schema"]:
        raise InventoryError("schema bundle: schema_version drifted")
    try:
        gate = schema["schemas"]["GateReceiptV1"]["properties"]["receipt_kind"]["enum"]
        spec = schema["schemas"]["ReleaseGateSpecV1"]["properties"]["receipt_kind"]["enum"]
    except (KeyError, TypeError) as error:
        raise InventoryError("schema bundle: receipt-kind enum is absent") from error
    if gate != spec or not isinstance(gate, list) or len(gate) != len(set(gate)):
        raise InventoryError("schema bundle: receipt-kind enums disagree or duplicate")
    if any(not isinstance(item, str) or not item for item in gate):
        raise InventoryError("schema bundle: invalid receipt kind")
    return gate


def _validate_reports(
    reports: list[dict[str, Any]], profiles: dict[str, dict[str, Any]],
    gates: dict[str, dict[str, Any]], receipt_kinds: list[str],
) -> tuple[Counter[str], Counter[str]]:
    if len(reports) != len(profiles):
        raise InventoryError("runtime reports: missing or extra profile report")
    report_profiles: set[str] = set()
    seen_gates: set[str] = set()
    for report in reports:
        exact_keys(
            report, {"schema_version", "command", "tier", "profile", "status", "gates"},
            set(), "runtime report",
        )
        profile = text(report["profile"], "runtime report.profile")
        if profile not in profiles or profile in report_profiles:
            raise InventoryError(f"runtime reports: unknown or duplicate profile {profile}")
        if (
            report["schema_version"] != 3 or report["command"] != "check"
            or report["tier"] != "RELEASE" or report["status"] != "BLOCKED"
        ):
            raise InventoryError(f"runtime report {profile}: header/result drifted")
        report_profiles.add(profile)
        rows, indexed = unique_objects(report["gates"], "id", f"runtime report {profile}.gates")
        condition = profiles[profile]["condition_gate"]
        if condition is not None and condition not in indexed:
            raise InventoryError(f"runtime report {profile}: condition gate missing")
        for row in rows:
            exact_keys(
                row, {"id", "status", "class", "detail", "repair", "subjects"}, set(),
                f"runtime gate {row['id']}",
            )
            gate_id = row["id"]
            if gate_id not in gates:
                raise InventoryError(f"runtime report {profile}: orphan gate {gate_id}")
            if (
                row["status"] != "BLOCKED"
                or row["class"] != gates[gate_id]["required_evidence_class"]
            ):
                raise InventoryError(f"runtime gate {gate_id}: result/class drifted")
            if not isinstance(row["subjects"], list) or row["subjects"]:
                raise InventoryError(f"runtime gate {gate_id}: unexpected evidence subjects")
            text(row["detail"], f"runtime gate {gate_id}.detail")
            text(row["repair"], f"runtime gate {gate_id}.repair")
            seen_gates.add(gate_id)
    if report_profiles != set(profiles):
        raise InventoryError("runtime reports: profile union drifted")
    if seen_gates != set(gates):
        raise InventoryError(
            f"runtime reports: gate union drifted; missing {sorted(set(gates) - seen_gates)}"
        )
    classes: Counter[str] = Counter()
    kinds: Counter[str] = Counter()
    for gate in gates.values():
        classes[gate["required_evidence_class"]] += 1
        kind = gate["receipt_kind"]
        if kind is None:
            if gate["id"] != "release.receipt-contracts":
                raise InventoryError(f"gate {gate['id']}: null receipt kind is not admitted")
        elif kind not in receipt_kinds:
            raise InventoryError(f"gate {gate['id']}: unknown receipt kind {kind}")
        else:
            kinds[kind] += 1
    if set(kinds) != set(receipt_kinds):
        raise InventoryError("gate bindings: receipt kind has zero reverse edges")
    return classes, kinds


def load_and_validate(root: Path, reports: list[dict[str, Any]]) -> Inventory:
    ledger, ledger_bytes = load_relative(root, "policy/assurance-inventory-v1.json")
    ledger = expect_object(ledger, "inventory")
    _, _, profiles, gates = _validate_ledger(ledger)
    contracts = _source_contracts(ledger)
    manifest_contract = contracts["family_manifest"]
    repositories, manifest_bytes = load_repositories(root.parent, manifest_contract["schema"])
    source_digests = {"policy/assurance-inventory-v1.json": digest(ledger_bytes)}
    source_digests["../repos.manifest.toml"] = digest(manifest_bytes)

    documents: dict[str, dict[str, Any]] = {}
    for name in ("corpus", "invariants", "receipt_kinds"):
        contract = contracts[name]
        loaded, raw = load_relative(root, contract["path"])
        documents[name] = expect_object(loaded, name)
        source_digests[contract["path"]] = digest(raw)
    schema_contract = contracts["inventory_schema"]
    schema, raw_schema = load_relative(root, schema_contract["path"])
    schema = expect_object(schema, "inventory schema")
    if schema.get("$id") != "https://schemas.bullet.farm/assurance/bullet.assurance-inventory.v1.schema.json":
        raise InventoryError("inventory schema: $id drifted")
    source_digests[schema_contract["path"]] = digest(raw_schema)

    corpus_counts, subject_counts = _validate_corpus(
        root, documents["corpus"], ledger, repositories
    )
    invariant_counts = _validate_invariants(documents["invariants"], ledger)
    receipt_kinds = _receipt_kinds(documents["receipt_kinds"], ledger)
    gate_counts, kind_counts = _validate_reports(reports, profiles, gates, receipt_kinds)
    return Inventory(
        ledger, documents["corpus"], documents["invariants"], receipt_kinds, reports,
        source_digests, corpus_counts, invariant_counts, gate_counts, kind_counts,
        subject_counts,
    )

