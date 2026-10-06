#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

def read_jsonl(path):
    events = []
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
        events.append(event)
    return events

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--metrics", required=True)
    parser.add_argument("--exclude-path", action="append", default=[])
    parser.add_argument("--exclude-env", action="append", default=[])
    args = parser.parse_args()

    excluded_paths = set(args.exclude_path)
    excluded_env = set(args.exclude_env)

    kept = []
    removed = []

    for event in read_jsonl(args.input):
        resource_type = event.get("resource_type")
        resource = event.get("resource")

        reason = None

        if resource_type == "path" and resource in excluded_paths:
            reason = "instrumentation_path_exact"

        if (
            resource_type == "environment_variable"
            and resource in excluded_env
        ):
            reason = "instrumentation_environment_exact"

        if reason is None:
            kept.append(event)
        else:
            removed.append({
                "event_id": event.get("event_id"),
                "resource": resource,
                "reason": reason,
            })

    Path(args.output).write_text(
        "".join(
            json.dumps(item, sort_keys=True, separators=(",", ":")) + "\n"
            for item in kept
        ),
        encoding="utf-8",
    )

    metrics = {
        "schema_version": 1,
        "input_events": len(kept) + len(removed),
        "kept_events": len(kept),
        "removed_events": len(removed),
        "removed": removed,
        "policy": "exact_identity_only",
        "outcome_dependent_filtering": False,
        "f8_outcomes_consulted": False,
    }

    Path(args.metrics).write_text(
        json.dumps(metrics, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
