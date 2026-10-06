#!/usr/bin/env python3
"""Verifica el agregador F9.7 con datos exclusivamente sintéticos."""
import copy
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

repo = Path(sys.argv[1]).resolve()
aggregator = repo / "tools/f9/metrics/aggregate_f9.py"
protocol = repo / "experiments/f9/metrics_protocol_v1.json"
fixture_events = repo / "tests/f9/dependency-graph/fixtures/events.jsonl"
builder = repo / "tools/f9/dependency-graph/build_graph.py"


def dump(path, data):
    path.write_text(json.dumps(data, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest(value):
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def candidate(key, eligibility, **kw):
    row = {
        "dependency_key": key, "eligibility": eligibility,
        "intervention_status": "not_attempted", "plan_file": None,
        "plan_sha256": None, "plan_frozen_before_treatment": False,
        "baseline_confirmations": 0, "treatment_confirmations": 0,
        "evidence_file": None, "evidence_sha256": None,
        "classification": None,
    }
    row.update(kw)
    return row


def create_plan(graph, node, root, suffix):
    from hashlib import sha256
    def h(items):
        text = "\x1f".join("" if x is None else str(x) for x in items)
        return sha256(text.encode("utf-8")).hexdigest()
    plan = {
        "schema_version": "f9.intervention_plan.v1", "plan_id": "",
        "source_run_id": graph["run_id"],
        "dependency_key": node["dependency_key"],
        "kind": node["kind"], "resource_type": node["resource_type"],
        "resource": node["resource"], "operation": node["operation"],
        "mechanism": "environment_override" if node["kind"] == "environment" else "isolated_text_file_replacement",
        "isolation": "workspace_copy", "baseline_value": "alpha", "treatment_value": "beta",
        "oracle_ref": "f9-synthetic:byte", "confirmations": 2,
        "result_blind_selection": True,
    }
    plan["plan_id"] = h([
        plan["source_run_id"], plan["dependency_key"], plan["kind"], plan["resource_type"],
        plan["resource"], plan["operation"], plan["mechanism"], plan["isolation"],
        plan["baseline_value"], plan["treatment_value"], plan["oracle_ref"],
        plan["confirmations"], plan["result_blind_selection"],
    ])
    name = f"plan-{suffix}.json"
    return name, dump(root / name, plan)


def create_evidence(root, suffix, layer, status, cls):
    name = f"evidence-{suffix}.json"
    evidence = {
        "schema_version": "f9.oracle_integration_evidence.v1",
        "layer": layer, "source_status": status, "classification": cls,
        "confirmations": 2, "semantic_reimplementation": False,
        "causal_claim": False, "f8_corpus_executed": False,
        "reason": "synthetic_ground_truth", "details": {},
        "evidence_level": "artifact_equivalence" if layer == "F3" else "propagation_trace",
    }
    return name, dump(root / name, evidence)


def call(suite, root, name, expect_pass=True):
    census = root / f"{name}-input.json"
    output = root / f"{name}-result.json"
    dump(census, suite)
    cmd = [sys.executable, str(aggregator), "--census", str(census),
           "--evidence-root", str(root), "--repo-root", str(repo),
           "--protocol", str(protocol), "--output", str(output)]
    p = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if expect_pass and p.returncode != 0:
        raise AssertionError(f"{name}: {p.stdout} {p.stderr}")
    if not expect_pass and p.returncode == 0:
        raise AssertionError(f"{name}: se aceptó una evidencia inválida")
    return json.loads(output.read_text(encoding="utf-8")) if expect_pass else None


with tempfile.TemporaryDirectory(prefix="envmorph-f9-7-") as td:
    root = Path(td)
    graph_file = root / "graph.json"
    subprocess.run([sys.executable, str(builder), "--events", str(fixture_events),
                    "--output", str(graph_file), "--metrics", str(root / "graph_metrics.json")],
                   check=True, stdout=subprocess.PIPE, text=True)
    graph = json.loads(graph_file.read_text(encoding="utf-8"))
    graph_sha = hashlib.sha256(graph_file.read_bytes()).hexdigest()
    nodes = {n["dependency_key"]: n for n in graph["nodes"] if n.get("node_type") == "dependency"}
    by_kind = {n["kind"]: n for n in nodes.values()}
    assert set(by_kind) == {"file", "environment", "executable"}

    rows = []
    for suffix, kind, layer, status, cls in (
        ("relevant", "environment", "F3", "different", "behaviorally_relevant"),
        ("stable", "file", "F4", "stable", "observed_only"),
    ):
        node = by_kind[kind]
        plan_file, plan_sha = create_plan(graph, node, root, suffix)
        ev_file, ev_sha = create_evidence(root, suffix, layer, status, cls)
        rows.append(candidate(
            node["dependency_key"], "eligible", intervention_status="confirmed",
            plan_file=plan_file, plan_sha256=plan_sha,
            plan_frozen_before_treatment=True, baseline_confirmations=2,
            treatment_confirmations=2, evidence_file=ev_file,
            evidence_sha256=ev_sha, classification=cls,
        ))
    rows.append(candidate(by_kind["executable"]["dependency_key"], "not_intervenable"))

    workload = {
        "workload_id": "synthetic-workflow-a", "run_id": graph["run_id"],
        "workload_status": "completed", "observer_effect": "preserved",
        "graph_file": graph_file.name, "graph_sha256": graph_sha,
        "census_sha256": digest({"run_id": graph["run_id"],
                                 "workload_id": "synthetic-workflow-a",
                                 "dependency_keys": sorted(nodes)}),
        "census_frozen_before_treatment": True,
        "census_commit": None, "plans_commit": None, "candidates": rows,
    }
    failed = {
        "workload_id": "synthetic-workflow-b", "run_id": "synthetic-failed-run",
        "workload_status": "observer_failure", "observer_effect": "unresolved",
        "graph_file": None, "graph_sha256": None, "census_sha256": None,
        "census_frozen_before_treatment": False,
        "census_commit": None, "plans_commit": None, "candidates": [],
    }
    suite = {"schema_version": "f9.metrics_census.v1", "campaign_id": "f9-7-synthetic",
             "corpus_role": "synthetic", "workloads": [workload, failed]}
    result = call(suite, root, "positive")
    assert result["totals"]["observed_candidates"] == 3
    assert result["totals"]["decidable_candidates"] == 2
    assert result["totals"]["behaviorally_relevant"] == 1
    assert result["totals"]["observed_only"] == 1
    assert result["totals"]["not_intervenable_candidates"] == 1
    assert result["workload_statuses"]["observer_failure"] == 1
    assert result["fractions"]["relevance_among_decidable"] == 0.5
    assert abs(result["fractions"]["decidability_among_observed"] - 2 / 3) < 1e-12
    assert result["scientific_claims"]["observer_completeness"] is False
    assert result["aggregator_executed_workloads"] is False
    assert result["external_corpus_data_consumed"] is False

    reordered = copy.deepcopy(suite)
    reordered["workloads"].reverse()
    reordered["workloads"][1]["candidates"].reverse()
    result2 = call(reordered, root, "reordered")
    assert result == result2, "Agregación no determinista ante orden de entrada"

    only_failure = copy.deepcopy(suite)
    only_failure["workloads"] = [copy.deepcopy(failed)]
    zero = call(only_failure, root, "zero_denominator")
    assert zero["fractions"]["decidability_among_observed"] is None
    assert zero["fractions"]["relevance_among_decidable"] is None

    for label in ("unresolved", "execution_failed", "unavailable", "out_of_scope", "observer_failure"):
        variant = copy.deepcopy(suite)
        candidate_row = variant["workloads"][0]["candidates"][0]
        candidate_row["intervention_status"] = label
        candidate_row["classification"] = label
        candidate_row["baseline_confirmations"] = 2
        candidate_row["treatment_confirmations"] = 1
        exception = {
            "schema_version": "f9.intervention_exception.v1",
            "status": label,
            "reason": "synthetic_controlled_failure",
            "plan_id": json.loads((root / candidate_row["plan_file"]).read_text(encoding="utf-8"))["plan_id"],
            "dependency_key": candidate_row["dependency_key"],
            "causal_claim": False,
            "f8_corpus_executed": False,
        }
        evidence_file = f"exception-{label}.json"
        candidate_row["evidence_file"] = evidence_file
        candidate_row["evidence_sha256"] = dump(root / evidence_file, exception)
        checked = call(variant, root, f"state_{label}")
        counter_name = {"unavailable": "unavailable_interventions",
                        "out_of_scope": "out_of_scope_interventions"}.get(label, label)
        assert checked["totals"][counter_name] == 1
        assert checked["totals"]["decidable_candidates"] == 1

    mutants = {}
    mutants["duplicate_candidate"] = copy.deepcopy(suite)
    mutants["duplicate_candidate"]["workloads"][0]["candidates"].append(copy.deepcopy(rows[0]))
    mutants["missing_candidate"] = copy.deepcopy(suite)
    mutants["missing_candidate"]["workloads"][0]["candidates"].pop()
    mutants["bad_graph_hash"] = copy.deepcopy(suite)
    mutants["bad_graph_hash"]["workloads"][0]["graph_sha256"] = "0" * 64
    mutants["bad_census_hash"] = copy.deepcopy(suite)
    mutants["bad_census_hash"]["workloads"][0]["census_sha256"] = "0" * 64
    mutants["missing_confirmation"] = copy.deepcopy(suite)
    mutants["missing_confirmation"]["workloads"][0]["candidates"][0]["treatment_confirmations"] = 1
    mutants["unfrozen_plan"] = copy.deepcopy(suite)
    mutants["unfrozen_plan"]["workloads"][0]["candidates"][0]["plan_frozen_before_treatment"] = False
    mutants["bad_evidence_hash"] = copy.deepcopy(suite)
    mutants["bad_evidence_hash"]["workloads"][0]["candidates"][0]["evidence_sha256"] = "0" * 64
    mutants["false_status"] = copy.deepcopy(suite)
    mutants["false_status"]["workloads"][0]["candidates"][0]["classification"] = "observed_only"
    mutants["observer_effect"] = copy.deepcopy(suite)
    mutants["observer_effect"]["workloads"][0]["observer_effect"] = "altered"
    mutants["foreign_path"] = copy.deepcopy(suite)
    mutants["foreign_path"]["workloads"][0]["graph_file"] = "../outside.json"
    mutants["fake_external"] = copy.deepcopy(suite)
    mutants["fake_external"]["corpus_role"] = "external"

    for name, invalid in mutants.items():
        call(invalid, root, "negative_" + name, expect_pass=False)

    print(f"F9.7 synthetic test: PASS ({8 + len(mutants)} escenarios)")
