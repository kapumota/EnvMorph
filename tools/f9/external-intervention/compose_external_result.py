#!/usr/bin/env python3
import argparse
import hashlib
import json
from pathlib import Path

MAPPING = {
    "stable": (
        "observed_only",
        "f4_stable_trace_reproduced",
    ),
    "absorbed": (
        "behaviorally_relevant",
        "f4_absorbed_difference_reproduced",
    ),
    "persistent": (
        "behaviorally_relevant",
        "f4_persistent_difference_reproduced",
    ),
}

def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def semantic_signature(report):
    return {
        "trace_status": report.get("trace_status"),
        "first_observed_divergence":
            report.get("first_observed_divergence"),
        "absorption_boundaries":
            report.get("absorption_boundaries"),
        "unresolved_count": report.get("unresolved_count"),
        "observations": [
            {
                "position": item.get("position"),
                "label": item.get("label"),
                "oracle": item.get("oracle"),
                "equivalence": item.get("equivalence"),
                "propagation": item.get("propagation"),
            }
            for item in report.get("observations", [])
        ],
    }

def write_exception(
    output,
    plan,
    status,
    reason,
    baseline_confirmations,
    treatment_confirmations,
):
    item = {
        "schema_version": "f9.intervention_exception.v1",
        "plan_id": plan["plan_id"],
        "dependency_key": plan["dependency_key"],
        "status": status,
        "reason": reason,
        "baseline_confirmations": baseline_confirmations,
        "treatment_confirmations": treatment_confirmations,
        "causal_claim": False,
        "f8_corpus_executed": False,
    }

    Path(output).write_text(
        json.dumps(item, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    return item

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True)
    parser.add_argument("--observer-effect", required=True)
    parser.add_argument("--run", action="append", default=[])
    parser.add_argument("--f4", action="append", default=[])
    parser.add_argument("--bridge-evidence")
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    plan = load(args.plan)
    runs = [load(path) for path in args.run]

    baseline_ok = sum(
        1
        for run in runs
        if run.get("variant") == "baseline"
        and run.get("exit_code") == 0
        and run.get("timed_out") is False
    )
    treatment_ok = sum(
        1
        for run in runs
        if run.get("variant") == "treatment"
        and run.get("exit_code") == 0
        and run.get("timed_out") is False
    )

    if args.observer_effect != "preserved":
        item = write_exception(
            args.output,
            plan,
            "observer_failure",
            f"observer_effect_{args.observer_effect}",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    if baseline_ok < 2 or treatment_ok < 2:
        item = write_exception(
            args.output,
            plan,
            "execution_failed",
            "external_execution_failed_or_timed_out",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    if len(args.f4) != 2:
        item = write_exception(
            args.output,
            plan,
            "unresolved",
            "missing_f4_confirmations",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    reports = [load(path) for path in args.f4]

    if any(
        report.get("trace_status") == "unresolved"
        or report.get("unresolved_count", 0) > 0
        for report in reports
    ):
        item = write_exception(
            args.output,
            plan,
            "unresolved",
            "f4_unresolved_trace",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    signatures = [
        semantic_signature(report)
        for report in reports
    ]

    if signatures[0] != signatures[1]:
        item = write_exception(
            args.output,
            plan,
            "unresolved",
            "cross_confirmation_trace_instability",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    status = reports[0].get("trace_status")

    if status not in MAPPING:
        item = write_exception(
            args.output,
            plan,
            "unresolved",
            f"unsupported_f4_status_{status}",
            baseline_ok,
            treatment_ok,
        )
        print(json.dumps(item, sort_keys=True))
        return

    if not args.bridge_evidence:
        raise SystemExit("Falta bridge evidence para resultado decidible")

    bridge = load(args.bridge_evidence)
    expected_classification, expected_reason = MAPPING[status]

    if (
        bridge.get("schema_version")
        != "f9.oracle_integration_evidence.v1"
        or bridge.get("layer") != "F4"
        or bridge.get("source_status") != status
        or bridge.get("classification") != expected_classification
        or bridge.get("reason") != expected_reason
        or bridge.get("confirmations") != 2
        or bridge.get("semantic_reimplementation") is not False
        or bridge.get("causal_claim") is not False
    ):
        raise SystemExit("Bridge F9.6 inconsistente")

    result = {
        "schema_version": "f9.external_intervention_result.v1",
        "plan_id": plan["plan_id"],
        "dependency_key": plan["dependency_key"],
        "status": "confirmed",
        "classification": bridge["classification"],
        "reason": bridge["reason"],
        "source_status": status,
        "baseline_confirmations": baseline_ok,
        "treatment_confirmations": treatment_ok,
        "semantic_signature_sha256": hashlib.sha256(
            json.dumps(
                signatures[0],
                sort_keys=True,
                separators=(",", ":"),
            ).encode("utf-8")
        ).hexdigest(),
        "causal_claim": False,
    }

    Path(args.output).write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(json.dumps(result, sort_keys=True))

if __name__ == "__main__":
    main()
