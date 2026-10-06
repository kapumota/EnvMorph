#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path

EXPECTED = {
    "csvkit-realdata",
    "jq-build-check",
    "fd-build-completions",
    "inih-meson-tests",
}

def canonical(obj):
    return json.dumps(
        obj,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")

def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()

def sha256_file(path):
    return sha256_bytes(Path(path).read_bytes())

def census_digest(run_id, workload_id, keys):
    return sha256_bytes(canonical({
        "run_id": run_id,
        "workload_id": workload_id,
        "dependency_keys": sorted(keys),
    }))

def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def completed_record(record, repo_root):
    graph_path = (repo_root / record["graph_file"]).resolve()
    if not graph_path.is_file():
        raise SystemExit(
            f"Grafo ausente para {record['workload_id']}: {graph_path}"
        )

    graph = load(graph_path)

    if graph.get("schema_version") != "f9.dependency_graph.v1":
        raise SystemExit("Schema de grafo F9.3 inesperado")

    if graph.get("run_id") != record["run_id"]:
        raise SystemExit("run_id de grafo inconsistente")

    claims = graph.get("claims", {})
    if (
        claims.get("causal_graph") is not False
        or claims.get("cross_source_total_order") is not False
        or claims.get("behavioral_relevance") is not False
    ):
        raise SystemExit("Grafo externo contiene claims no permitidos")

    candidates = []

    for node in graph.get("nodes", []):
        if node.get("node_type") != "dependency":
            continue

        candidates.append({
            "dependency_key": node["dependency_key"],
            "kind": node["kind"],
            "resource_type": node["resource_type"],
            "resource": node["resource"],
            "operation": node["operation"],
        })

    candidates.sort(key=lambda item: item["dependency_key"])
    keys = [item["dependency_key"] for item in candidates]

    graph_sha = sha256_file(graph_path)
    digest = census_digest(
        record["run_id"],
        record["workload_id"],
        keys,
    )

    if graph_sha != record["graph_sha256"]:
        raise SystemExit("SHA-256 de grafo inconsistente")

    if digest != record["census_sha256"]:
        raise SystemExit("Digest de censo inconsistente")

    if len(candidates) != record["candidate_count"]:
        raise SystemExit("Conteo de candidatos inconsistente")

    result = dict(record)
    result["candidates"] = candidates
    return result

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--records-dir", required=True)
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--protocol-commit", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--metrics", required=True)
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    records_dir = Path(args.records_dir)

    records = []
    for path in sorted(records_dir.glob("*.json")):
        records.append(load(path))

    ids = {record.get("workload_id") for record in records}
    if ids != EXPECTED or len(records) != 4:
        raise SystemExit(
            "Los records no coinciden con los cuatro workloads F8"
        )

    workloads = []
    statuses = {}
    total_candidates = 0

    for record in sorted(records, key=lambda item: item["workload_id"]):
        status = record["workload_status"]
        statuses[status] = statuses.get(status, 0) + 1

        if status == "completed":
            item = completed_record(record, repo_root)
            total_candidates += len(item["candidates"])
        else:
            item = dict(record)
            item["candidates"] = []

            if (
                item["graph_file"] is not None
                or item["graph_sha256"] is not None
                or item["census_sha256"] is not None
                or item["candidate_count"] != 0
            ):
                raise SystemExit(
                    "Workload no completado inventa grafo o candidatos"
                )

        workloads.append(item)

    census = {
        "schema_version": "f9.external_observation_census.v1",
        "campaign_id": "f9-8a-f8-external-v1",
        "observation_protocol_commit": args.protocol_commit,
        "corpus_source": "F8_frozen_external_corpus",
        "treatments_executed": False,
        "eligibility_assigned": False,
        "classifications_assigned": False,
        "f8_outcomes_used_for_candidate_selection": False,
        "workloads": workloads,
    }

    Path(args.output).write_text(
        json.dumps(census, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    metrics = {
        "schema_version": 1,
        "phase": "F9.8A",
        "status": "pass",
        "workloads_total": 4,
        "workload_statuses": dict(sorted(statuses.items())),
        "observed_candidates": total_candidates,
        "treatments_executed": False,
        "eligibility_assigned": False,
        "classifications_assigned": False,
        "f8_outcomes_used_for_candidate_selection": False,
    }

    Path(args.metrics).write_text(
        json.dumps(metrics, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
