#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path

SCHEMA_VERSION = "f9.intervention_plan.v1"

FORBIDDEN_FIELDS = {
    "expected_result",
    "expected_behavior",
    "expected_difference",
    "expected_classification",
}

REQUIRED_FIELDS = {
    "schema_version",
    "plan_id",
    "source_run_id",
    "dependency_key",
    "kind",
    "resource_type",
    "resource",
    "operation",
    "mechanism",
    "isolation",
    "baseline_value",
    "treatment_value",
    "oracle_ref",
    "confirmations",
    "result_blind_selection",
}

def stable_hash(parts):
    payload = "\x1f".join("" if x is None else str(x) for x in parts)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()

def expected_plan_id(plan):
    return stable_hash([
        plan["source_run_id"],
        plan["dependency_key"],
        plan["kind"],
        plan["resource_type"],
        plan["resource"],
        plan["operation"],
        plan["mechanism"],
        plan["isolation"],
        plan["baseline_value"],
        plan["treatment_value"],
        plan["oracle_ref"],
        plan["confirmations"],
        plan["result_blind_selection"],
    ])

def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def dependency_nodes(graph):
    return {
        node["dependency_key"]: node
        for node in graph["nodes"]
        if node.get("node_type") == "dependency"
    }

def validate(plan, graph):
    problems = []

    extra_forbidden = FORBIDDEN_FIELDS & set(plan)
    if extra_forbidden:
        problems.append(
            "campos de resultado prohibidos: "
            + ",".join(sorted(extra_forbidden))
        )

    if set(plan) != REQUIRED_FIELDS:
        missing = REQUIRED_FIELDS - set(plan)
        extra = set(plan) - REQUIRED_FIELDS
        if missing:
            problems.append(
                "faltan campos: " + ",".join(sorted(missing))
            )
        if extra:
            problems.append(
                "campos no permitidos: " + ",".join(sorted(extra))
            )

    if problems:
        return problems

    if plan["schema_version"] != SCHEMA_VERSION:
        problems.append("schema_version inválido")

    if plan["source_run_id"] != graph.get("run_id"):
        problems.append("source_run_id no coincide con el grafo")

    deps = dependency_nodes(graph)
    node = deps.get(plan["dependency_key"])

    if node is None:
        problems.append("dependency_key no observada")
    else:
        for field in ("kind", "resource_type", "resource", "operation"):
            if plan[field] != node.get(field):
                problems.append(
                    f"{field} no coincide con dependencia observada"
                )

    if plan["baseline_value"] == plan["treatment_value"]:
        problems.append("baseline_value y treatment_value deben diferir")

    if not isinstance(plan["confirmations"], int) or plan["confirmations"] < 2:
        problems.append("confirmations debe ser >= 2")

    if plan["result_blind_selection"] is not True:
        problems.append("result_blind_selection debe ser true")

    if not isinstance(plan["oracle_ref"], str) or not plan["oracle_ref"]:
        problems.append("oracle_ref vacío")

    if plan["isolation"] != "workspace_copy":
        problems.append("isolation no soportado")

    mechanism = plan["mechanism"]

    if mechanism == "environment_override":
        if not (
            plan["kind"] == "environment"
            and plan["resource_type"] == "environment_variable"
            and plan["operation"] == "getenv"
        ):
            problems.append(
                "environment_override incompatible con la dependencia"
            )

    elif mechanism == "isolated_text_file_replacement":
        if not (
            plan["kind"] == "file"
            and plan["resource_type"] == "path"
            and plan["resource"].startswith("@WORKSPACE@/")
        ):
            problems.append(
                "isolated_text_file_replacement requiere archivo bajo @WORKSPACE@"
            )

    else:
        problems.append("mechanism no soportado")

    if plan["plan_id"] != expected_plan_id(plan):
        problems.append("plan_id inconsistente")

    return problems

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--graph", required=True)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    graph = load_json(args.graph)
    plan = load_json(args.plan)

    problems = validate(plan, graph)

    result = {
        "schema_version": 1,
        "status": "eligible" if not problems else "invalid",
        "problems": problems,
        "dependency_key": plan.get("dependency_key"),
        "plan_id": plan.get("plan_id"),
        "behavioral_relevance_evaluated": False,
        "f8_corpus_executed": False,
    }

    if args.output:
        Path(args.output).write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

    if problems:
        for problem in problems:
            print(problem)
        raise SystemExit(2)

    print("InterventionPlan: ELIGIBLE")

if __name__ == "__main__":
    main()
