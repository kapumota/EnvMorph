#!/usr/bin/env python3
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1]).resolve()
builder = root / "tools/f9/dependency-graph/build_graph.py"
events = root / "tests/f9/dependency-graph/fixtures/events.jsonl"

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

with tempfile.TemporaryDirectory() as td:
    td = Path(td)

    outputs = []

    for index in (1, 2):
        graph = td / f"graph-{index}.json"
        metrics = td / f"metrics-{index}.json"

        subprocess.run(
            [
                sys.executable,
                str(builder),
                "--events", str(events),
                "--output", str(graph),
                "--metrics", str(metrics),
            ],
            check=True,
        )

        outputs.append((graph, metrics))

    assert sha(outputs[0][0]) == sha(outputs[1][0])
    assert sha(outputs[0][1]) == sha(outputs[1][1])

    graph = json.loads(outputs[0][0].read_text(encoding="utf-8"))
    metrics = json.loads(outputs[0][1].read_text(encoding="utf-8"))

    assert metrics["events_total"] == 4
    assert metrics["process_nodes"] == 2
    assert metrics["dependency_nodes"] == 3
    assert metrics["resource_nodes"] == 3
    assert metrics["observed_dependency_edges"] == 3
    assert metrics["dependency_resource_edges"] == 3
    assert metrics["parent_edges"] == 1
    assert metrics["process_identity_conflicts"] == 0
    assert metrics["cross_source_process_nodes"] == 1
    assert metrics["causal_graph_claimed"] is False
    assert metrics["cross_source_total_order_claimed"] is False
    assert metrics["behavioral_relevance_claimed"] is False
    assert metrics["f8_corpus_executed"] is False

    observed = [
        e for e in graph["edges"]
        if e["edge_type"] == "observed_dependency"
    ]

    repeated = [
        e for e in observed
        if e["occurrence_count"] == 2
    ]

    assert len(repeated) == 1
    assert len(repeated[0]["event_ids"]) == 2
    assert repeated[0]["source_sequences"]["ptrace"] == [1, 2]

    assert any(
        e["edge_type"] == "parent_of"
        for e in graph["edges"]
    )

print("F9.3 fixture graph: PASS")
