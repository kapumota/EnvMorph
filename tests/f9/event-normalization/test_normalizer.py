#!/usr/bin/env python3
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1]).resolve()
normalizer = root / "tools/f9/event-normalizer/normalize_events.py"
ptrace = root / "tests/f9/event-normalization/fixtures/ptrace.jsonl"
getenv = root / "tests/f9/event-normalization/fixtures/getenv.jsonl"

def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()

with tempfile.TemporaryDirectory() as td:
    td = Path(td)

    outputs = []

    for index in (1, 2):
        events = td / f"events-{index}.jsonl"
        deps = td / f"deps-{index}.json"
        metrics = td / f"metrics-{index}.json"

        subprocess.run(
            [
                sys.executable,
                str(normalizer),
                "--ptrace", str(ptrace),
                "--getenv", str(getenv),
                "--run-id", "fixture-run",
                "--output", str(events),
                "--dependencies", str(deps),
                "--metrics", str(metrics),
                "--path-prefix", "/tmp/f9-work",
                "--path-label", "@WORKSPACE@",
            ],
            check=True,
        )

        outputs.append((events, deps, metrics))

    assert sha(outputs[0][0]) == sha(outputs[1][0])
    assert sha(outputs[0][1]) == sha(outputs[1][1])
    assert sha(outputs[0][2]) == sha(outputs[1][2])

    events = [
        json.loads(line)
        for line in outputs[0][0].read_text(encoding="utf-8").splitlines()
        if line.strip()
    ]
    deps = json.loads(outputs[0][1].read_text(encoding="utf-8"))
    metrics = json.loads(outputs[0][2].read_text(encoding="utf-8"))

    assert metrics["input_events"] == 7
    assert metrics["normalized_events"] == 6
    assert metrics["duplicate_events_removed"] == 1
    assert metrics["dependency_count"] == 5
    assert metrics["cross_source_total_order_claimed"] is False
    assert metrics["f8_corpus_executed"] is False

    fixture_events = [
        e for e in events
        if e["resource"] == "@WORKSPACE@/fixture.txt"
    ]
    assert len(fixture_events) == 2
    assert len({e["event_id"] for e in fixture_events}) == 2
    assert len({e["dependency_key"] for e in fixture_events}) == 1

    missing = [
        e for e in events
        if e["resource"] == "@WORKSPACE@/missing.txt"
    ]
    assert len(missing) == 1
    assert missing[0]["result"] == "error"
    assert missing[0]["errno"] == 2

    env_present = [
        e for e in events
        if e["resource"] == "F9_GROUND_TRUTH"
    ]
    assert len(env_present) == 1
    assert env_present[0]["kind"] == "environment"
    assert env_present[0]["result"] == "present"

    env_missing = [
        e for e in events
        if e["resource"] == "F9_MISSING"
    ]
    assert len(env_missing) == 1
    assert env_missing[0]["result"] == "missing"

    dependency_by_resource = {
        d["resource"]: d for d in deps["dependencies"]
    }
    assert dependency_by_resource["@WORKSPACE@/fixture.txt"]["occurrences"] == 2

print("F9.2 fixture normalization: PASS")
