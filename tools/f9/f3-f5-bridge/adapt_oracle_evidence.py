#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

SCHEMA = "f9.oracle_integration_evidence.v1"

def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def adapt_f3(data, confirmations):
    status = data.get("result")

    mapping = {
        "identical": (
            "observed_only",
            "f3_equivalence_reproduced",
        ),
        "equivalent": (
            "observed_only",
            "f3_equivalence_reproduced",
        ),
        "different": (
            "behaviorally_relevant",
            "f3_difference_reproduced",
        ),
        "missing": (
            "unresolved",
            "f3_missing_artifact",
        ),
        "error": (
            "unresolved",
            "f3_comparison_error",
        ),
    }

    if status not in mapping:
        raise SystemExit(f"Estado F3 no soportado: {status}")

    classification, reason = mapping[status]

    return {
        "schema_version": SCHEMA,
        "layer": "F3",
        "source_status": status,
        "classification": classification,
        "reason": reason,
        "evidence_level": "artifact_equivalence",
        "confirmations": confirmations,
        "details": {
            "oracle": data.get("oracle"),
            "metadata": data.get("metadata", {}),
        },
        "semantic_reimplementation": False,
        "causal_claim": False,
        "f8_corpus_executed": False,
    }

def adapt_f4(data, confirmations):
    status = data.get("trace_status")

    mapping = {
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
        "unresolved": (
            "unresolved",
            "f4_unresolved_trace",
        ),
    }

    if status not in mapping:
        raise SystemExit(f"Estado F4 no soportado: {status}")

    classification, reason = mapping[status]

    return {
        "schema_version": SCHEMA,
        "layer": "F4",
        "source_status": status,
        "classification": classification,
        "reason": reason,
        "evidence_level": "propagation_trace",
        "confirmations": confirmations,
        "details": {
            "first_observed_divergence":
                data.get("first_observed_divergence"),
            "absorption_boundaries":
                data.get("absorption_boundaries", []),
            "unresolved_count":
                data.get("unresolved_count"),
            "observations":
                data.get("observations", []),
        },
        "semantic_reimplementation": False,
        "causal_claim": False,
        "f8_corpus_executed": False,
    }

def adapt_f5(data):
    status = data.get("status")

    mapping = {
        "minimal": (
            "behaviorally_relevant",
            "f5_operational_sufficiency_reproduced",
        ),
        "no_observable_effect": (
            "observed_only",
            "f5_no_observable_effect",
        ),
    }

    if status not in mapping:
        raise SystemExit(f"Estado F5 no soportado: {status}")

    if (
        data.get("causality_claim")
        != "operational_sufficiency_not_physical_proof"
    ):
        raise SystemExit("F5 perdió su contrato de alcance causal")

    confirmations = data.get("confirmations")
    if not isinstance(confirmations, int) or confirmations < 2:
        raise SystemExit("F5 no aporta confirmaciones suficientes")

    classification, reason = mapping[status]

    return {
        "schema_version": SCHEMA,
        "layer": "F5",
        "source_status": status,
        "classification": classification,
        "reason": reason,
        "evidence_level": "operational_sufficiency",
        "confirmations": confirmations,
        "details": {
            "search_method": data.get("search_method"),
            "minimal_cardinality":
                data.get("minimal_cardinality"),
            "minimal_sets":
                data.get("minimal_sets", []),
            "essential_factors":
                data.get("essential_factors", []),
            "target_signature":
                data.get("target_signature"),
            "scope":
                data.get("scope"),
            "f5_causality_claim":
                data.get("causality_claim"),
        },
        "semantic_reimplementation": False,
        "causal_claim": False,
        "f8_corpus_executed": False,
    }

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--layer",
        choices=["F3", "F4", "F5"],
        required=True,
    )
    parser.add_argument("--input", required=True)
    parser.add_argument("--confirmations", type=int, default=2)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    data = load(args.input)

    if args.layer == "F3":
        if args.confirmations < 2:
            raise SystemExit("F3 requiere al menos dos confirmaciones F9")
        result = adapt_f3(data, args.confirmations)
    elif args.layer == "F4":
        if args.confirmations < 2:
            raise SystemExit("F4 requiere al menos dos confirmaciones F9")
        result = adapt_f4(data, args.confirmations)
    else:
        result = adapt_f5(data)

    Path(args.output).write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print(
        f"{result['layer']} -> "
        f"{result['classification']} "
        f"({result['source_status']})"
    )

if __name__ == "__main__":
    main()
