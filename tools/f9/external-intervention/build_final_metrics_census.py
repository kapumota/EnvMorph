#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--observation-census", required=True)
    parser.add_argument("--eligibility", required=True)
    parser.add_argument("--plan-index", required=True)
    parser.add_argument("--result-index", required=True)
    parser.add_argument("--census-commit", required=True)
    parser.add_argument("--plans-commit", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    census = load(args.observation_census)
    eligibility_rows = [
        json.loads(line)
        for line in Path(args.eligibility).read_text(
            encoding="utf-8"
        ).splitlines()
        if line.strip()
    ]
    plan_index = load(args.plan_index)
    result_index = load(args.result_index)

    eligibility = {
        (row["workload_id"], row["dependency_key"]): row
        for row in eligibility_rows
    }
    plans = {
        (row["workload_id"], row["dependency_key"]): row
        for row in plan_index["plans"]
    }
    results = {
        (row["workload_id"], row["dependency_key"]): row
        for row in result_index["results"]
    }

    workloads = []

    for source in census["workloads"]:
        wid = source["workload_id"]
        status = source["workload_status"]

        if status != "completed":
            if source["candidates"]:
                raise SystemExit(
                    f"Workload no completado {wid} contiene candidatos"
                )

            workloads.append({
                "workload_id": wid,
                "run_id": source["run_id"],
                "workload_status": status,
                "observer_effect": source["observer_effect"],
                "graph_file": None,
                "graph_sha256": None,
                "census_sha256": None,
                "census_frozen_before_treatment": False,
                "census_commit": None,
                "plans_commit": None,
                "candidates": [],
            })
            continue

        candidates = []

        for observed in source["candidates"]:
            key = (wid, observed["dependency_key"])

            if key not in eligibility:
                raise SystemExit(
                    f"Falta elegibilidad para {wid} {key[1]}"
                )

            row = eligibility[key]

            if row["eligibility"] == "eligible":
                if key not in plans or key not in results:
                    raise SystemExit(
                        f"Eligible sin plan o resultado {wid} {key[1]}"
                    )

                plan = plans[key]
                result = results[key]

                candidate = {
                    "dependency_key": observed["dependency_key"],
                    "eligibility": "eligible",
                    "intervention_status":
                        result["intervention_status"],
                    "plan_file": plan["plan_file"],
                    "plan_sha256": plan["plan_sha256"],
                    "plan_frozen_before_treatment": True,
                    "baseline_confirmations":
                        result["baseline_confirmations"],
                    "treatment_confirmations":
                        result["treatment_confirmations"],
                    "evidence_file": result["evidence_file"],
                    "evidence_sha256":
                        result["evidence_sha256"],
                    "classification":
                        result["classification"],
                }
            else:
                candidate = {
                    "dependency_key": observed["dependency_key"],
                    "eligibility": row["eligibility"],
                    "intervention_status": "not_attempted",
                    "plan_file": None,
                    "plan_sha256": None,
                    "plan_frozen_before_treatment": False,
                    "baseline_confirmations": 0,
                    "treatment_confirmations": 0,
                    "evidence_file": None,
                    "evidence_sha256": None,
                    "classification": None,
                }

            candidates.append(candidate)

        workloads.append({
            "workload_id": wid,
            "run_id": source["run_id"],
            "workload_status": status,
            "observer_effect": source["observer_effect"],
            "graph_file": source["graph_file"],
            "graph_sha256": source["graph_sha256"],
            "census_sha256": source["census_sha256"],
            "census_frozen_before_treatment": True,
            "census_commit": args.census_commit,
            "plans_commit": args.plans_commit,
            "candidates": candidates,
        })

    suite = {
        "schema_version": "f9.metrics_census.v1",
        "campaign_id": "f9-8c-f8-external-v1",
        "corpus_role": "external",
        "workloads": workloads,
    }

    Path(args.output).write_text(
        json.dumps(suite, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
