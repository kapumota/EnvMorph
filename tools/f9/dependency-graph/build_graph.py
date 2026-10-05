#!/usr/bin/env python3
import argparse
import hashlib
import json
from collections import defaultdict
from pathlib import Path

EVENT_SCHEMA = "f9.dependency_event.v1"
GRAPH_SCHEMA = "f9.dependency_graph.v1"

def stable_hash(parts):
    payload = "\x1f".join("" if x is None else str(x) for x in parts)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()

def load_events(path):
    events = []
    run_id = None

    for line_no, line in enumerate(
        Path(path).read_text(encoding="utf-8").splitlines(),
        start=1,
    ):
        if not line.strip():
            continue

        try:
            event = json.loads(line)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path}:{line_no}: JSON inválido: {exc}")

        if event.get("schema_version") != EVENT_SCHEMA:
            raise SystemExit(
                f"{path}:{line_no}: schema_version de evento inesperado"
            )

        if run_id is None:
            run_id = event.get("run_id")
        elif event.get("run_id") != run_id:
            raise SystemExit("El input mezcla run_id distintos")

        validate_event_identity(event, line_no)
        events.append(event)

    if not events:
        raise SystemExit("No hay eventos normalizados")

    return run_id, events

def validate_event_identity(event, line_no):
    required = {
        "schema_version",
        "event_id",
        "dependency_key",
        "run_id",
        "source",
        "source_sequence",
        "pid",
        "ppid",
        "kind",
        "resource_type",
        "resource",
        "operation",
        "result",
        "errno",
    }

    if set(event) != required:
        raise SystemExit(
            f"Evento {line_no} con campos inesperados"
        )

    expected_dependency = stable_hash([
        event["kind"],
        event["resource_type"],
        event["resource"],
        event["operation"],
    ])

    if event["dependency_key"] != expected_dependency:
        raise SystemExit(
            f"Evento {line_no} con dependency_key inconsistente"
        )

    expected_event = stable_hash([
        event["run_id"],
        event["source"],
        event["source_sequence"],
        event["pid"],
        event["kind"],
        event["resource_type"],
        event["resource"],
        event["operation"],
        event["result"],
        event["errno"],
    ])

    if event["event_id"] != expected_event:
        raise SystemExit(
            f"Evento {line_no} con event_id inconsistente"
        )

def process_node_id(run_id, pid):
    return stable_hash(["process", run_id, pid])

def dependency_node_id(dependency_key):
    return stable_hash(["dependency", dependency_key])

def resource_node_id(resource_type, resource):
    return stable_hash(["resource", resource_type, resource])

def edge_id(edge_type, source, target):
    return stable_hash(["edge", edge_type, source, target])

def build_graph(run_id, events):
    by_pid = defaultdict(list)

    for event in events:
        by_pid[event["pid"]].append(event)

    observed_pids = set(by_pid)
    nodes = []
    process_identity_conflicts = 0
    cross_source_process_nodes = 0

    for pid in sorted(by_pid):
        group = by_pid[pid]
        ppid_values = sorted({
            e["ppid"] for e in group
            if e["ppid"] is not None
        })
        sources = sorted({e["source"] for e in group})

        if len(ppid_values) > 1:
            process_identity_conflicts += 1

        if len(sources) > 1:
            cross_source_process_nodes += 1

        nodes.append({
            "node_id": process_node_id(run_id, pid),
            "node_type": "process",
            "pid": pid,
            "ppid_values": ppid_values,
            "sources": sources,
            "identity_scope": "run_id_plus_pid",
        })

    dependency_groups = defaultdict(list)

    for event in events:
        dependency_groups[event["dependency_key"]].append(event)

    for dependency_key in sorted(dependency_groups):
        group = dependency_groups[dependency_key]
        first = group[0]

        semantic = {
            (
                e["kind"],
                e["resource_type"],
                e["resource"],
                e["operation"],
            )
            for e in group
        }

        if len(semantic) != 1:
            raise SystemExit(
                f"dependency_key {dependency_key} agrupa semánticas distintas"
            )

        nodes.append({
            "node_id": dependency_node_id(dependency_key),
            "node_type": "dependency",
            "dependency_key": dependency_key,
            "kind": first["kind"],
            "resource_type": first["resource_type"],
            "resource": first["resource"],
            "operation": first["operation"],
        })

    resource_groups = {}

    for event in events:
        key = (event["resource_type"], event["resource"])
        resource_groups[key] = {
            "resource_type": event["resource_type"],
            "resource": event["resource"],
        }

    for key in sorted(resource_groups):
        item = resource_groups[key]
        nodes.append({
            "node_id": resource_node_id(
                item["resource_type"],
                item["resource"],
            ),
            "node_type": "resource",
            "resource_type": item["resource_type"],
            "resource": item["resource"],
        })

    observed_edges = defaultdict(list)

    for event in events:
        source = process_node_id(run_id, event["pid"])
        target = dependency_node_id(event["dependency_key"])
        observed_edges[(source, target)].append(event)

    edges = []

    for key in sorted(observed_edges):
        source, target = key
        group = observed_edges[key]
        sequences = defaultdict(list)

        for event in group:
            sequences[event["source"]].append(event["source_sequence"])

        edges.append({
            "edge_id": edge_id(
                "observed_dependency",
                source,
                target,
            ),
            "edge_type": "observed_dependency",
            "source_node": source,
            "target_node": target,
            "occurrence_count": len(group),
            "event_ids": sorted(e["event_id"] for e in group),
            "sources": sorted({e["source"] for e in group}),
            "results": sorted({e["result"] for e in group}),
            "source_sequences": {
                source_name: sorted(values)
                for source_name, values in sorted(sequences.items())
            },
        })

    for dependency_key in sorted(dependency_groups):
        first = dependency_groups[dependency_key][0]
        source = dependency_node_id(dependency_key)
        target = resource_node_id(
            first["resource_type"],
            first["resource"],
        )

        edges.append({
            "edge_id": edge_id(
                "targets_resource",
                source,
                target,
            ),
            "edge_type": "targets_resource",
            "source_node": source,
            "target_node": target,
        })

    parent_pairs = set()

    for child_pid in sorted(by_pid):
        ppid_values = {
            e["ppid"] for e in by_pid[child_pid]
            if e["ppid"] is not None
        }

        if len(ppid_values) != 1:
            continue

        parent_pid = next(iter(ppid_values))

        if parent_pid in observed_pids and parent_pid != child_pid:
            parent_pairs.add((parent_pid, child_pid))

    for parent_pid, child_pid in sorted(parent_pairs):
        source = process_node_id(run_id, parent_pid)
        target = process_node_id(run_id, child_pid)

        edges.append({
            "edge_id": edge_id(
                "parent_of",
                source,
                target,
            ),
            "edge_type": "parent_of",
            "source_node": source,
            "target_node": target,
        })

    nodes.sort(key=lambda n: (n["node_type"], n["node_id"]))
    edges.sort(key=lambda e: (e["edge_type"], e["edge_id"]))

    process_count = sum(
        1 for node in nodes
        if node["node_type"] == "process"
    )
    dependency_count = sum(
        1 for node in nodes
        if node["node_type"] == "dependency"
    )
    resource_count = sum(
        1 for node in nodes
        if node["node_type"] == "resource"
    )

    observed_edge_count = sum(
        1 for edge in edges
        if edge["edge_type"] == "observed_dependency"
    )
    target_edge_count = sum(
        1 for edge in edges
        if edge["edge_type"] == "targets_resource"
    )
    parent_edge_count = sum(
        1 for edge in edges
        if edge["edge_type"] == "parent_of"
    )

    return {
        "schema_version": GRAPH_SCHEMA,
        "run_id": run_id,
        "nodes": nodes,
        "edges": edges,
        "metrics": {
            "events_total": len(events),
            "process_nodes": process_count,
            "dependency_nodes": dependency_count,
            "resource_nodes": resource_count,
            "observed_dependency_edges": observed_edge_count,
            "dependency_resource_edges": target_edge_count,
            "parent_edges": parent_edge_count,
            "process_identity_conflicts": process_identity_conflicts,
            "cross_source_process_nodes": cross_source_process_nodes,
            "f8_corpus_executed": False,
        },
        "claims": {
            "causal_graph": False,
            "cross_source_total_order": False,
            "behavioral_relevance": False,
        },
    }

def validate_graph(graph):
    if graph["schema_version"] != GRAPH_SCHEMA:
        raise SystemExit("schema_version de grafo inválido")

    node_ids = [node["node_id"] for node in graph["nodes"]]
    edge_ids = [edge["edge_id"] for edge in graph["edges"]]

    if len(node_ids) != len(set(node_ids)):
        raise SystemExit("node_id duplicado")

    if len(edge_ids) != len(set(edge_ids)):
        raise SystemExit("edge_id duplicado")

    node_set = set(node_ids)

    for edge in graph["edges"]:
        if edge["source_node"] not in node_set:
            raise SystemExit("Arista con source_node inexistente")
        if edge["target_node"] not in node_set:
            raise SystemExit("Arista con target_node inexistente")

    if graph["claims"]["causal_graph"] is not False:
        raise SystemExit("F9.3 no puede declarar causal_graph")

    if graph["claims"]["cross_source_total_order"] is not False:
        raise SystemExit("F9.3 no puede declarar orden total cross-source")

    if graph["claims"]["behavioral_relevance"] is not False:
        raise SystemExit("F9.3 no puede declarar relevancia conductual")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--events", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--metrics", required=True)
    args = parser.parse_args()

    run_id, events = load_events(args.events)
    graph = build_graph(run_id, events)
    validate_graph(graph)

    Path(args.output).write_text(
        json.dumps(
            graph,
            sort_keys=True,
            separators=(",", ":"),
        ) + "\n",
        encoding="utf-8",
    )

    Path(args.metrics).write_text(
        json.dumps(
            {
                "schema_version": 1,
                "phase": "F9.3",
                "run_id": run_id,
                "status": "pass",
                **graph["metrics"],
                **{
                    "causal_graph_claimed": graph["claims"]["causal_graph"],
                    "cross_source_total_order_claimed":
                        graph["claims"]["cross_source_total_order"],
                    "behavioral_relevance_claimed":
                        graph["claims"]["behavioral_relevance"],
                },
            },
            indent=2,
            sort_keys=True,
        ) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
