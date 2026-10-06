#!/usr/bin/env python3
"""Agrega censos F9 sin confundir falta de evidencia con efecto nulo."""
import argparse
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

HEX64 = re.compile(r"^[0-9a-f]{64}$")
HEX40 = re.compile(r"^[0-9a-f]{40}$")
WORKLOAD_STATES = {"completed", "unavailable", "out_of_scope", "observer_failure", "execution_failed"}
ELIGIBILITY = {"eligible", "not_intervenable", "unavailable", "out_of_scope"}
INTERVENTION_STATES = {"not_attempted", "confirmed", "unresolved", "execution_failed", "unavailable", "out_of_scope", "observer_failure"}
OBSERVER_STATES = {"preserved", "altered", "unresolved", "not_evaluated"}
CLASSIFICATION = {"behaviorally_relevant", "observed_only", "unresolved", "execution_failed", "unavailable", "out_of_scope", "observer_failure"}
WORKLOAD_FIELDS = {"workload_id", "run_id", "workload_status", "observer_effect", "graph_file", "graph_sha256", "census_sha256", "census_frozen_before_treatment", "census_commit", "plans_commit", "candidates"}
CANDIDATE_FIELDS = {"dependency_key", "eligibility", "intervention_status", "plan_file", "plan_sha256", "plan_frozen_before_treatment", "baseline_confirmations", "treatment_confirmations", "evidence_file", "evidence_sha256", "classification"}
COUNTS = (
    "observed_candidates", "eligible_candidates", "not_intervenable_candidates",
    "unavailable_candidates", "out_of_scope_candidates", "decidable_candidates",
    "behaviorally_relevant", "observed_only", "unresolved", "execution_failed",
    "observer_failure", "unavailable_interventions", "out_of_scope_interventions",
    "interventions_not_attempted",
)


def fail(message):
    raise ValueError(message)


def load_json(path):
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def canonical(obj):
    return json.dumps(obj, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def sha(data):
    return hashlib.sha256(data).hexdigest()


def valid_hash(value, length=64):
    return isinstance(value, str) and bool((HEX64 if length == 64 else HEX40).fullmatch(value))


def root_file(root, name, expected_sha, label):
    if not isinstance(name, str) or not name or Path(name).is_absolute():
        fail(f"{label}: ruta relativa requerida")
    if not valid_hash(expected_sha):
        fail(f"{label}: SHA-256 inválido")
    path = (root / name).resolve()
    if not path.is_relative_to(root):
        fail(f"{label}: ruta fuera de evidencia")
    if not path.is_file():
        fail(f"{label}: archivo no existe: {name}")
    if sha(path.read_bytes()) != expected_sha:
        fail(f"{label}: SHA-256 no coincide: {name}")
    return path


def validate_graph(graph, run_id):
    if graph.get("schema_version") != "f9.dependency_graph.v1":
        fail("Grafo F9.3 con esquema incorrecto")
    if graph.get("run_id") != run_id:
        fail("run_id de grafo diferente")
    if graph.get("claims", {}).get("causal_graph") is not False:
        fail("Grafo reclama causalidad")
    if graph.get("claims", {}).get("cross_source_total_order") is not False:
        fail("Grafo reclama orden total entre fuentes")
    deps = {}
    for node in graph.get("nodes", []):
        if node.get("node_type") != "dependency":
            continue
        key = node.get("dependency_key")
        if not valid_hash(key) or key in deps:
            fail("Identidad de dependencia inválida o repetida")
        deps[key] = node
    return deps


def census_digest(run_id, workload_id, keys):
    return sha(canonical({
        "run_id": run_id,
        "workload_id": workload_id,
        "dependency_keys": sorted(keys),
    }))


def validate_plan(repo, graph_path, plan_path, node, run_id, candidate):
    command = [
        sys.executable,
        str(repo / "tools/f9/intervention-plan/validate_plan.py"),
        "--graph", str(graph_path),
        "--plan", str(plan_path),
    ]
    outcome = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                             encoding="utf-8", timeout=30, check=False)
    if outcome.returncode:
        fail("InterventionPlan F9.4 inválido: " + (outcome.stdout + outcome.stderr).strip())
    plan = load_json(plan_path)
    if plan["dependency_key"] != node["dependency_key"] or plan["source_run_id"] != run_id:
        fail("Plan no corresponde a dependencia y run_id")
    if plan.get("confirmations", 0) < 2:
        fail("Plan sin confirmaciones mínimas")
    return plan


def validate_evidence(path, candidate, protocol):
    item = load_json(path)
    layer = item.get("layer")
    mapping = protocol["classification_map"].get(layer)
    if mapping is None:
        fail("Capa de oráculo no congelada")
    source_status = item.get("source_status")
    expected = mapping.get(source_status)
    if expected is None:
        fail("Estado del oráculo no congelado")
    if item.get("classification") != expected or candidate["classification"] != expected:
        fail("Clasificación no coincide con la salida del bridge F9.6")
    if item.get("schema_version") != "f9.oracle_integration_evidence.v1":
        fail("Evidencia no es salida de bridge F9.6")
    if item.get("semantic_reimplementation") is not False or item.get("causal_claim") is not False:
        fail("Evidencia viola límites semánticos")
    if item.get("f8_corpus_executed") is not False:
        fail("Evidencia contamina el baseline F8")
    if type(item.get("confirmations")) is not int or item["confirmations"] < 2:
        fail("Bridge F9.6 no acredita dos confirmaciones")
    return item


def empty_counts():
    return {key: 0 for key in COUNTS}


def fractions(counts):
    observed = counts["observed_candidates"]
    decided = counts["decidable_candidates"]
    return {
        "decidability_among_observed": None if not observed else decided / observed,
        "relevance_among_decidable": None if not decided else counts["behaviorally_relevant"] / decided,
        "denominators": {"observed_candidates": observed, "decidable_candidates": decided},
    }


def validate_candidate(row, root, repo, graph_path, graph_node, run_id, observer_effect, protocol):
    if not isinstance(row, dict) or set(row) != CANDIDATE_FIELDS:
        fail("Campos de candidato incompletos o inesperados")
    if row["dependency_key"] != graph_node["dependency_key"]:
        fail("dependency_key distinta del grafo")
    eligibility = row["eligibility"]
    status = row["intervention_status"]
    classification = row["classification"]
    if eligibility not in ELIGIBILITY or status not in INTERVENTION_STATES:
        fail("Elegibilidad o estado de intervención desconocido")
    if classification is not None and classification not in CLASSIFICATION:
        fail("Clasificación desconocida")
    b = row["baseline_confirmations"]
    t = row["treatment_confirmations"]
    if type(b) is not int or type(t) is not int or b < 0 or t < 0:
        fail("Confirmaciones inválidas")

    if eligibility != "eligible":
        if status != "not_attempted" or classification is not None or b or t:
            fail("Candidato no elegible fue interpretado como tratamiento")
        if any(row[x] is not None for x in ("plan_file", "plan_sha256", "evidence_file", "evidence_sha256")):
            fail("Candidato no elegible contiene plan o resultado")
        if row["plan_frozen_before_treatment"] is not False:
            fail("Congelación impropia en candidato no elegible")
        return

    if status == "not_attempted":
        if classification is not None or b or t or any(row[x] is not None for x in (
            "plan_file", "plan_sha256", "evidence_file", "evidence_sha256"
        )):
            fail("No intentado contiene evidencia de resultado")
        if row["plan_frozen_before_treatment"] is not False:
            fail("No intentado tiene plan reportado como ejecutado")
        return

    if row["plan_frozen_before_treatment"] is not True:
        fail("Tratamiento sin plan precongelado")
    plan_path = root_file(root, row["plan_file"], row["plan_sha256"], "plan")
    plan = validate_plan(repo, graph_path, plan_path, graph_node, run_id, row)
    if b > plan["confirmations"] or t > plan["confirmations"]:
        fail("Número de ejecuciones supera confirmaciones preregistradas")

    if status == "confirmed":
        if observer_effect != "preserved":
            fail("Efecto del observador impide confirmación")
        if b < 2 or t < 2:
            fail("Resultado decidido sin dos confirmaciones por variante")
        if classification not in ("behaviorally_relevant", "observed_only"):
            fail("Resultado confirmado requiere clase decidible")
        evidence_path = root_file(root, row["evidence_file"], row["evidence_sha256"], "evidence")
        evidence = validate_evidence(evidence_path, row, protocol)
        if evidence["confirmations"] > b or evidence["confirmations"] > t:
            fail("Confirmaciones de la evidencia no están respaldadas por el censo")
    else:
        if classification != status:
            fail("Abstención o fallo reclasificado indebidamente")
        failure_path = root_file(root, row["evidence_file"], row["evidence_sha256"],
                                 "evidence of abstention")
        failure = load_json(failure_path)
        schema = failure.get("schema_version")
        if schema == "f9.behavioral_classification.v1":
            if status != "unresolved" or failure.get("status") != status:
                fail("Clasificación F9.5 incongruente con abstención")
        elif schema == "f9.intervention_exception.v1":
            if failure.get("status") != status or not failure.get("reason"):
                fail("Excepción experimental sin estado o motivo coherente")
            if failure.get("causal_claim") is not False:
                fail("Excepción experimental contiene claim causal")
        else:
            fail("Contrato de abstención o fallo desconocido")
        if (failure.get("plan_id") != plan["plan_id"]
                or failure.get("dependency_key") != row["dependency_key"]):
            fail("Evidencia de abstención no pertenece a plan y dependencia")
        if failure.get("f8_corpus_executed") is not False:
            fail("Evidencia de abstención altera el baseline F8")


def evaluate(suite, root, repo, protocol):
    if not isinstance(suite, dict) or set(suite) != {"schema_version", "campaign_id", "corpus_role", "workloads"}:
        fail("Campos de censo inesperados")
    if suite["schema_version"] != "f9.metrics_census.v1":
        fail("Esquema de censo desconocido")
    if not isinstance(suite["campaign_id"], str) or not suite["campaign_id"]:
        fail("campaign_id vacío")
    role = suite["corpus_role"]
    if role not in ("synthetic", "external"):
        fail("Rol de corpus inválido")
    workloads = suite["workloads"]
    if not isinstance(workloads, list) or not workloads:
        fail("Censo sin workloads")
    if role == "external" and {w.get("workload_id") for w in workloads} != set(protocol["external_corpus"]):
        fail("El corpus externo no coincide con los cuatro workloads F8 congelados")
    if role == "external" and len(workloads) != len(protocol["external_corpus"]):
        fail("Corpus externo con workloads duplicados")

    seen_workloads = set()
    reports = []
    totals = empty_counts()
    workload_states = Counter()

    for workload in workloads:
        if not isinstance(workload, dict) or set(workload) != WORKLOAD_FIELDS:
            fail("Campos de workload inesperados")
        wid = workload["workload_id"]
        run_id = workload["run_id"]
        if not isinstance(wid, str) or not wid or wid in seen_workloads:
            fail("workload_id inválido o duplicado")
        seen_workloads.add(wid)
        if not isinstance(run_id, str) or not run_id:
            fail("run_id inválido")
        status = workload["workload_status"]
        oe = workload["observer_effect"]
        if status not in WORKLOAD_STATES or oe not in OBSERVER_STATES:
            fail("Estado del workload o observador desconocido")
        candidates = workload["candidates"]
        if not isinstance(candidates, list):
            fail("candidates debe ser lista")
        counts = empty_counts()
        workload_states[status] += 1

        if status != "completed":
            if candidates or any(workload[x] is not None for x in (
                "graph_file", "graph_sha256", "census_sha256", "census_commit", "plans_commit"
            )) or workload["census_frozen_before_treatment"] is not False:
                fail("Workload no ejecutable inventa censo o grafo")
            if oe not in ("unresolved", "not_evaluated"):
                fail("Workload fallido no puede reclamar observer_effect preserved")
            reports.append({"workload_id": wid, "workload_status": status,
                            "observer_effect": oe, "counts": counts, "fractions": fractions(counts)})
            continue

        if workload["census_frozen_before_treatment"] is not True:
            fail("Censo no congelado antes del tratamiento")
        graph_path = root_file(root, workload["graph_file"], workload["graph_sha256"], "graph")
        graph = load_json(graph_path)
        nodes = validate_graph(graph, run_id)
        if workload["census_sha256"] != census_digest(run_id, wid, nodes):
            fail("Censo no coincide con hash preregistrado")
        if role == "external" and (
            not valid_hash(workload["census_commit"], 40)
            or not valid_hash(workload["plans_commit"], 40)
        ):
            fail("Faltan los dos commits de congelación pre-treatment")
        if role == "synthetic" and (
            workload["census_commit"] is not None or workload["plans_commit"] is not None
        ):
            fail("El corpus sintético no debe simular commits de congelación")

        ids = [candidate.get("dependency_key") for candidate in candidates]
        if len(ids) != len(set(ids)) or set(ids) != set(nodes):
            fail("No hay igualdad entre censo de candidatos y dependencias observadas")
        counts["observed_candidates"] = len(nodes)

        for candidate in candidates:
            validate_candidate(candidate, root, repo, graph_path,
                               nodes[candidate["dependency_key"]], run_id, oe, protocol)
            eligibility = candidate["eligibility"]
            counts[{
                "eligible": "eligible_candidates",
                "not_intervenable": "not_intervenable_candidates",
                "unavailable": "unavailable_candidates",
                "out_of_scope": "out_of_scope_candidates",
            }[eligibility]] += 1
            if candidate["intervention_status"] == "not_attempted":
                counts["interventions_not_attempted"] += 1
            kind = candidate["classification"]
            if kind:
                count_key = {
                    "unavailable": "unavailable_interventions",
                    "out_of_scope": "out_of_scope_interventions",
                }.get(kind, kind)
                counts[count_key] += 1
                if kind in ("behaviorally_relevant", "observed_only"):
                    counts["decidable_candidates"] += 1

        if sum(counts[x] for x in (
            "eligible_candidates", "not_intervenable_candidates",
            "unavailable_candidates", "out_of_scope_candidates"
        )) != counts["observed_candidates"]:
            fail("Censo incompleto")
        if counts["behaviorally_relevant"] + counts["observed_only"] != counts["decidable_candidates"]:
            fail("Denominador decidible inconsistente")
        for key in COUNTS:
            totals[key] += counts[key]
        reports.append({"workload_id": wid, "workload_status": status,
                        "observer_effect": oe, "counts": counts, "fractions": fractions(counts)})

    reports.sort(key=lambda x: x["workload_id"])
    return {
        "schema_version": "f9.metrics_summary.v1",
        "campaign_id": suite["campaign_id"],
        "corpus_role": role,
        "workloads_total": len(reports),
        "workload_statuses": dict(sorted(workload_states.items())),
        "totals": totals,
        "fractions": fractions(totals),
        "per_workload": reports,
        "scientific_claims": {
            "observer_completeness": False,
            "physical_causality": False,
            "external_prevalence": False,
            "behavioral_hermeticity_gap_scalar": False,
            "cross_source_total_order": False,
        },
        "aggregator_executed_workloads": False,
        "external_corpus_data_consumed": role == "external",
        "denominator_zero_policy": "null_not_zero",
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--census", required=True)
    parser.add_argument("--evidence-root", required=True)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--protocol", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()
    root = Path(args.evidence_root).resolve(strict=True)
    repo = Path(args.repo_root).resolve(strict=True)
    try:
        summary = evaluate(load_json(Path(args.census)), root, repo, load_json(Path(args.protocol)))
    except (ValueError, KeyError, TypeError, OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"CENSUS REJECTED: {error}")
    Path(args.output).write_text(json.dumps(summary, indent=2, sort_keys=True, ensure_ascii=False) + "\n", encoding="utf-8")
    print("F9.7 census validation: PASS")


if __name__ == "__main__":
    main()
