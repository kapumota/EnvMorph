#!/usr/bin/env python3
import argparse
import hashlib
import json
import posixpath
from collections import defaultdict
from pathlib import Path

SCHEMA_VERSION = "f9.dependency_event.v1"

def stable_hash(parts):
    payload = "\x1f".join("" if x is None else str(x) for x in parts)
    return hashlib.sha256(payload.encode("utf-8")).hexdigest()

def normalize_path(path, prefix, label):
    if not path:
        return path

    normalized = posixpath.normpath(path)

    if prefix:
        prefix = posixpath.normpath(prefix)
        if normalized == prefix:
            return label
        if normalized.startswith(prefix.rstrip("/") + "/"):
            suffix = normalized[len(prefix.rstrip("/")):]
            return label.rstrip("/") + suffix

    return normalized

def read_jsonl(path):
    result = []
    for index, line in enumerate(
        Path(path).read_text(encoding="utf-8").splitlines(),
        start=1,
    ):
        if not line.strip():
            continue
        try:
            item = json.loads(line)
        except json.JSONDecodeError as exc:
            raise SystemExit(f"{path}:{index}: JSON inválido: {exc}")
        if not isinstance(item, dict):
            raise SystemExit(f"{path}:{index}: se esperaba objeto JSON")
        result.append((index, item))
    return result

def normalize_ptrace(item, run_id, prefix, label):
    kind_raw = item.get("kind")

    if kind_raw == "file_open":
        kind = "file"
        resource_type = "path"
    elif kind_raw == "exec":
        kind = "executable"
        resource_type = "path"
    else:
        return None

    source_sequence = item.get("seq")
    pid = item.get("pid")
    ppid = item.get("ppid")
    operation = item.get("syscall")
    result = item.get("result")
    errno_value = item.get("errno")
    resource = normalize_path(item.get("resource", ""), prefix, label)

    if result not in ("success", "error"):
        raise SystemExit(f"Resultado ptrace inválido: {result}")

    return build_event(
        run_id=run_id,
        source="ptrace",
        source_sequence=source_sequence,
        pid=pid,
        ppid=ppid,
        kind=kind,
        resource_type=resource_type,
        resource=resource,
        operation=operation,
        result=result,
        errno_value=errno_value,
    )

def normalize_getenv(item, run_id):
    if item.get("kind") != "getenv":
        return None

    present = item.get("present")
    if not isinstance(present, bool):
        raise SystemExit("Evento getenv sin booleano present")

    return build_event(
        run_id=run_id,
        source="ld_preload_getenv",
        source_sequence=item.get("seq"),
        pid=item.get("pid"),
        ppid=item.get("ppid"),
        kind="environment",
        resource_type="environment_variable",
        resource=item.get("name", ""),
        operation="getenv",
        result="present" if present else "missing",
        errno_value=None,
    )

def build_event(
    run_id,
    source,
    source_sequence,
    pid,
    ppid,
    kind,
    resource_type,
    resource,
    operation,
    result,
    errno_value,
):
    if not isinstance(source_sequence, int) or source_sequence < 1:
        raise SystemExit(f"source_sequence inválido para {source}")
    if not isinstance(pid, int) or pid < 1:
        raise SystemExit(f"pid inválido para {source}")
    if ppid is not None and (not isinstance(ppid, int) or ppid < 0):
        raise SystemExit(f"ppid inválido para {source}")
    if not resource:
        raise SystemExit(f"resource vacío para {source}")
    if not operation:
        raise SystemExit(f"operation vacío para {source}")

    dependency_key = stable_hash([
        kind,
        resource_type,
        resource,
        operation,
    ])

    event_id = stable_hash([
        run_id,
        source,
        source_sequence,
        pid,
        kind,
        resource_type,
        resource,
        operation,
        result,
        errno_value,
    ])

    return {
        "schema_version": SCHEMA_VERSION,
        "event_id": event_id,
        "dependency_key": dependency_key,
        "run_id": run_id,
        "source": source,
        "source_sequence": source_sequence,
        "pid": pid,
        "ppid": ppid,
        "kind": kind,
        "resource_type": resource_type,
        "resource": resource,
        "operation": operation,
        "result": result,
        "errno": errno_value,
    }

def exact_identity(event):
    return (
        event["source"],
        event["source_sequence"],
        event["pid"],
        event["ppid"],
        event["kind"],
        event["resource_type"],
        event["resource"],
        event["operation"],
        event["result"],
        event["errno"],
    )

def validate_event(event):
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
            "Evento con campos inválidos: "
            + ",".join(sorted(set(event) ^ required))
        )

    if event["schema_version"] != SCHEMA_VERSION:
        raise SystemExit("schema_version inválido")

    for key in ("event_id", "dependency_key"):
        value = event[key]
        if len(value) != 64 or any(c not in "0123456789abcdef" for c in value):
            raise SystemExit(f"{key} inválido")

    if event["source"] not in ("ptrace", "ld_preload_getenv"):
        raise SystemExit("source inválido")

    if event["kind"] not in ("file", "executable", "environment"):
        raise SystemExit("kind inválido")

    if event["resource_type"] not in ("path", "environment_variable"):
        raise SystemExit("resource_type inválido")

    if event["result"] not in ("success", "error", "present", "missing"):
        raise SystemExit("result inválido")

def aggregate_dependencies(events):
    groups = defaultdict(list)

    for event in events:
        groups[event["dependency_key"]].append(event)

    result = []

    for key in sorted(groups):
        group = groups[key]
        first = group[0]
        result.append({
            "dependency_key": key,
            "kind": first["kind"],
            "resource_type": first["resource_type"],
            "resource": first["resource"],
            "operation": first["operation"],
            "occurrences": len(group),
            "sources": sorted({e["source"] for e in group}),
            "pids": sorted({e["pid"] for e in group}),
            "results": sorted({e["result"] for e in group}),
        })

    return result

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--ptrace", required=True)
    parser.add_argument("--getenv", required=True)
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--dependencies", required=True)
    parser.add_argument("--metrics", required=True)
    parser.add_argument("--path-prefix")
    parser.add_argument("--path-label", default="@WORKSPACE@")
    args = parser.parse_args()

    candidates = []

    for _, item in read_jsonl(args.ptrace):
        event = normalize_ptrace(
            item,
            args.run_id,
            args.path_prefix,
            args.path_label,
        )
        if event is not None:
            candidates.append(event)

    for _, item in read_jsonl(args.getenv):
        event = normalize_getenv(item, args.run_id)
        if event is not None:
            candidates.append(event)

    seen = set()
    events = []
    duplicate_events_removed = 0

    for event in candidates:
        validate_event(event)
        identity = exact_identity(event)

        if identity in seen:
            duplicate_events_removed += 1
            continue

        seen.add(identity)
        events.append(event)

    events.sort(
        key=lambda e: (
            e["source"],
            e["pid"],
            e["source_sequence"],
            e["event_id"],
        )
    )

    dependencies = aggregate_dependencies(events)

    output = Path(args.output)
    output.write_text(
        "".join(
            json.dumps(event, sort_keys=True, separators=(",", ":")) + "\n"
            for event in events
        ),
        encoding="utf-8",
    )

    Path(args.dependencies).write_text(
        json.dumps(
            {
                "schema_version": 1,
                "run_id": args.run_id,
                "dependency_count": len(dependencies),
                "dependencies": dependencies,
            },
            indent=2,
            sort_keys=True,
        ) + "\n",
        encoding="utf-8",
    )

    metrics = {
        "schema_version": 1,
        "run_id": args.run_id,
        "input_events": len(candidates),
        "normalized_events": len(events),
        "duplicate_events_removed": duplicate_events_removed,
        "dependency_count": len(dependencies),
        "sources": sorted({e["source"] for e in events}),
        "cross_source_total_order_claimed": False,
        "f8_corpus_executed": False,
    }

    Path(args.metrics).write_text(
        json.dumps(metrics, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
