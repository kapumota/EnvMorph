#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import re
import subprocess
from collections import Counter
from pathlib import Path

KEY_COMMITS = [
    ("F9.8A census", "37a3fbf"),
    ("F9.8B policy", "d116cb8"),
    ("F9.8B plans", "15edeea"),
    ("F9.8C harness", "43608b8"),
    ("F9.8C assembly erratum", "a84479f"),
    ("F9.8C results", "7fb7a16"),
]

EXPECTED_TOTALS = {
    "observed_candidates": 12717,
    "eligible_candidates": 2,
    "behaviorally_relevant": 0,
    "observed_only": 2,
    "unresolved": 0,
    "execution_failed": 0,
    "observer_failure": 0,
    "decidable_candidates": 2,
}

def load(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def run(repo, *args):
    proc = subprocess.run(
        list(args),
        cwd=repo,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        check=False,
    )
    return proc

def full_sha(repo, ref):
    proc = run(repo, "git", "rev-parse", ref)
    if proc.returncode:
        raise SystemExit(f"No se pudo resolver {ref}")
    return proc.stdout.strip()

def is_ancestor(repo, left, right):
    return run(
        repo,
        "git",
        "merge-base",
        "--is-ancestor",
        left,
        right,
    ).returncode == 0

def tracked_file_sizes(repo):
    proc = run(repo, "git", "ls-files", "-z")
    if proc.returncode:
        raise SystemExit("git ls-files falló")

    result = []
    for raw in proc.stdout.split("\0"):
        if not raw:
            continue
        path = Path(repo) / raw
        if path.is_file():
            result.append((raw, path.stat().st_size))
    return result

def secret_findings(repo):
    patterns = [
        re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----"),
        re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
        re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{30,}\b"),
        re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
    ]

    findings = []
    proc = run(repo, "git", "ls-files", "-z")
    if proc.returncode:
        raise SystemExit("git ls-files falló")

    for raw in proc.stdout.split("\0"):
        if not raw:
            continue
        path = Path(repo) / raw
        if not path.is_file() or path.stat().st_size > 2_000_000:
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue

        for index, line in enumerate(text.splitlines(), start=1):
            if any(pattern.search(line) for pattern in patterns):
                findings.append({
                    "path": raw,
                    "line": index,
                })
    return findings

def count_home_paths(repo):
    proc = run(repo, "git", "grep", "-n", "-F", "/home/c-lara")
    if proc.returncode not in (0, 1):
        raise SystemExit("git grep de paths locales falló")
    lines = [
        line
        for line in proc.stdout.splitlines()
        if line.strip()
    ]
    return len(lines), lines[:50]

def raw_manifest_status(root):
    root = Path(root)
    if not root.is_dir():
        return {
            "present": False,
            "manifest_present": False,
            "checkable": False,
            "valid": None,
            "missing_or_mismatched_entries": None,
        }

    manifest = root / "RAW.sha256"
    if not manifest.is_file():
        return {
            "present": True,
            "manifest_present": False,
            "checkable": False,
            "valid": None,
            "missing_or_mismatched_entries": None,
        }

    proc = subprocess.run(
        ["sha256sum", "-c", "RAW.sha256"],
        cwd=root,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )

    failures = [
        line
        for line in proc.stdout.splitlines()
        if (
            "FAILED" in line
            or "FALL" in line
            or "No such file" in line
            or "no existe" in line
        )
    ]

    return {
        "present": True,
        "manifest_present": True,
        "checkable": True,
        "valid": proc.returncode == 0,
        "missing_or_mismatched_entries": failures[:100],
    }

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--summary", required=True)
    parser.add_argument("--result-metrics", required=True)
    parser.add_argument("--result-index", required=True)
    parser.add_argument("--observation-census", required=True)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    repo = Path(args.repo_root).resolve()

    summary = load(args.summary)
    result_metrics = load(args.result_metrics)
    result_index = load(args.result_index)
    observation_census = load(args.observation_census)

    totals = summary["totals"]

    for key, expected in EXPECTED_TOTALS.items():
        if totals.get(key) != expected:
            raise SystemExit(
                f"Métrica final cambió: {key}="
                f"{totals.get(key)} != {expected}"
            )

    if result_metrics["treatments_rerun"] is not False:
        raise SystemExit("F9.8C indica rerun de treatments")

    if result_index["plan_count"] != 2:
        raise SystemExit("Result index no contiene dos plans")

    classifications = Counter(
        item["classification"]
        for item in result_index["results"]
    )

    if classifications != Counter({"observed_only": 2}):
        raise SystemExit("Clasificaciones externas inesperadas")

    lineage = []
    resolved = []

    for label, short in KEY_COMMITS:
        full = full_sha(repo, short)
        lineage.append({
            "label": label,
            "short": short,
            "full": full,
        })
        resolved.append(full)

    if not all(
        is_ancestor(repo, left, right)
        for left, right in zip(resolved, resolved[1:])
    ):
        raise SystemExit("Genealogía F9.8A-C no es lineal")

    if full_sha(repo, "HEAD") != resolved[-1]:
        raise SystemExit("HEAD no corresponde al cierre F9.8C esperado")

    fractions = summary["fractions"]
    decidability = fractions["decidability_among_observed"]
    relevance = fractions["relevance_among_decidable"]

    expected_decidability = 2 / 12717

    if abs(decidability - expected_decidability) > 1e-15:
        raise SystemExit("Fracción de decidibilidad cambió")

    if relevance != 0.0:
        raise SystemExit("Fracción de relevancia cambió")

    workload_states = Counter(
        item["workload_status"]
        for item in observation_census["workloads"]
    )

    observer_states = Counter(
        item["observer_effect"]
        for item in observation_census["workloads"]
    )

    historical_raw = {
        "F9.2": raw_manifest_status(repo / ".envmorph/f9-2-evidence"),
        "F9.3": raw_manifest_status(repo / ".envmorph/f9-3-evidence"),
    }

    current_raw = {
        "F9.8A": raw_manifest_status(repo / ".envmorph/f9-8a-evidence"),
        "F9.8C": raw_manifest_status(repo / ".envmorph/f9-8c-evidence"),
    }

    event_id_contract = {
        "event_id_includes_ppid": False,
        "exact_dedup_identity_includes_ppid": True,
        "status": "frozen_as_documented_limitation",
        "remediation_in_f9_9": False,
    }

    sizes = tracked_file_sizes(repo)
    large_files = [
        {
            "path": name,
            "bytes": size,
        }
        for name, size in sizes
        if size > 10 * 1024 * 1024
    ]

    secrets = secret_findings(repo)
    home_count, home_examples = count_home_paths(repo)

    public_files = {}
    for name in (
        "README.md",
        "LICENSE",
        "LICENSE.md",
        "CITATION.cff",
        "SECURITY.md",
    ):
        public_files[name] = (repo / name).is_file()

    license_present = (
        public_files["LICENSE"]
        or public_files["LICENSE.md"]
    )

    gitignore = (repo / ".gitignore")
    ignored_envmorph = False

    if gitignore.is_file():
        probe = run(
            repo,
            "git",
            "check-ignore",
            "-q",
            ".envmorph/f9-private-probe",
        )
        ignored_envmorph = probe.returncode == 0

    public_blockers = []

    if not public_files["README.md"]:
        public_blockers.append("missing_README")
    if not license_present:
        public_blockers.append("missing_LICENSE")
    if not public_files["CITATION.cff"]:
        public_blockers.append("missing_CITATION")
    if secrets:
        public_blockers.append("secret_like_material")
    if large_files:
        public_blockers.append("tracked_files_over_10MiB")
    if not ignored_envmorph:
        public_blockers.append("dot_envmorph_not_ignored")
    if home_count:
        public_blockers.append(
            "active_or_historical_absolute_home_paths_need_release_review"
        )

    public_ready_now = len(public_blockers) == 0

    result = {
        "schema_version": "f9.audit_metrics.v1",
        "phase": "F9.9",
        "status": "pass",
        "scientific_results": {
            "observed_candidates": totals["observed_candidates"],
            "eligible_candidates": totals["eligible_candidates"],
            "behaviorally_relevant": totals["behaviorally_relevant"],
            "observed_only": totals["observed_only"],
            "unresolved": totals["unresolved"],
            "execution_failed": totals["execution_failed"],
            "observer_failure": totals["observer_failure"],
            "decidable_candidates": totals["decidable_candidates"],
            "decidability_among_observed": decidability,
            "relevance_among_decidable": relevance,
            "treatments_rerun_during_recovery": False,
        },
        "lineage": lineage,
        "workload_statuses_f9_8a": dict(
            sorted(workload_states.items())
        ),
        "observer_effects_f9_8a": dict(
            sorted(observer_states.items())
        ),
        "historical_raw_evidence": historical_raw,
        "current_raw_evidence": current_raw,
        "event_id_contract": event_id_contract,
        "known_observation_limits": [
            "direct_environ_reads_not_observed_by_getenv_interposition",
            "vdso_time_reads_not_covered_by_primary_ptrace_set",
            "mmap_and_cache_dependencies_not_complete",
            "preopened_file_descriptor_dependencies_not_complete",
            "path_normalization_is_lexical_not_filesystem_canonical",
            "no_cross_source_total_order",
            "observer_completeness_not_claimed",
            "ebpf_not_selected_on_current_host",
        ],
        "public_readiness": {
            "ready_now": public_ready_now,
            "recommended_visibility_change":
                "after_F9_10_scientific_freeze",
            "blockers": public_blockers,
            "files": public_files,
            "secret_like_findings": secrets,
            "tracked_files_over_10MiB": large_files,
            "dot_envmorph_ignored": ignored_envmorph,
            "absolute_home_path_occurrences": home_count,
            "absolute_home_path_examples": home_examples,
        },
        "claims": {
            "complete_dependency_observation": False,
            "physical_causality": False,
            "external_prevalence": False,
            "behavioral_hermeticity_gap_scalar": False,
            "universal_hermeticity": False,
            "cross_source_total_order": False,
        },
        "next_gate": "F9.10 scientific closure and release freeze",
    }

    Path(args.output).write_text(
        json.dumps(result, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    print("F9.9 audit model: PASS")
    print(
        "Public ready now:",
        "YES" if public_ready_now else "NO",
    )
    print(
        "Public blockers:",
        ",".join(public_blockers)
        if public_blockers
        else "none",
    )

if __name__ == "__main__":
    main()
