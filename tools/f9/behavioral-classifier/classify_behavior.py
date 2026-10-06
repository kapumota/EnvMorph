#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

SCHEMA_VERSION = "f9.behavioral_classification.v1"

def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def load_runs(paths):
    return [load_json(path) for path in paths]

def validate_run(run, plan, variant):
    if run.get("plan_id") != plan["plan_id"]:
        raise SystemExit("Evidencia con plan_id diferente")
    if run.get("variant") != variant:
        raise SystemExit("Evidencia con variant incorrecto")
    if run.get("behavioral_relevance_evaluated") is not False:
        raise SystemExit("La evidencia F9.4 ya contiene clasificación indebida")
    if run.get("f8_corpus_executed") is not False:
        raise SystemExit("La evidencia indica ejecución del corpus F8")

def stdout_signature(runs):
    values = {run["stdout_sha256"] for run in runs}
    if len(values) != 1:
        return None
    return next(iter(values))

def classify(plan, baseline, treatment):
    required = plan["confirmations"]

    result = {
        "schema_version": SCHEMA_VERSION,
        "plan_id": plan["plan_id"],
        "dependency_key": plan["dependency_key"],
        "status": "unresolved",
        "reason": "insufficient_confirmations",
        "oracle_ref": plan["oracle_ref"],
        "required_confirmations": required,
        "baseline_confirmations": len(baseline),
        "treatment_confirmations": len(treatment),
        "baseline_consistent": False,
        "treatment_consistent": False,
        "baseline_signature": None,
        "treatment_signature": None,
        "causal_claim": False,
        "f8_corpus_executed": False,
    }

    if plan["oracle_ref"] != "micro:stdout":
        result["reason"] = "unsupported_oracle"
        return result

    if len(baseline) < required or len(treatment) < required:
        return result

    if any(run["exit_code"] != 0 for run in baseline + treatment):
        result["reason"] = "execution_failed"
        return result

    baseline_sig = stdout_signature(baseline)
    treatment_sig = stdout_signature(treatment)

    result["baseline_consistent"] = baseline_sig is not None
    result["treatment_consistent"] = treatment_sig is not None
    result["baseline_signature"] = baseline_sig
    result["treatment_signature"] = treatment_sig

    if baseline_sig is None or treatment_sig is None:
        result["reason"] = "within_variant_instability"
        return result

    if baseline_sig != treatment_sig:
        result["status"] = "behaviorally_relevant"
        result["reason"] = "oracle_difference_reproduced"
    else:
        result["status"] = "observed_only"
        result["reason"] = "oracle_equivalence_reproduced"

    return result

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True)
    parser.add_argument("--baseline", action="append", default=[])
    parser.add_argument("--treatment", action="append", default=[])
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    plan = load_json(args.plan)
    baseline = load_runs(args.baseline)
    treatment = load_runs(args.treatment)

    for run in baseline:
        validate_run(run, plan, "baseline")

    for run in treatment:
        validate_run(run, plan, "treatment")

    result = classify(plan, baseline, treatment)

    Path(args.output).write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(
        f"{result['status']}: {result['reason']}"
    )

if __name__ == "__main__":
    main()
