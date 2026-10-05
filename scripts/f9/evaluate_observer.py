#!/usr/bin/env python3
import argparse
import json
import statistics
import subprocess
import time
from pathlib import Path

CASES = ["absolute", "relative", "dirfd", "missing", "forkexec"]

def run(cmd, timeout=30):
    return subprocess.run(
        cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )

def load_jsonl(path):
    events = []
    if not path.exists():
        return events
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            events.append(json.loads(line))
    return events

def observer_has(events, kind, resource, result):
    for e in events:
        if (
            e.get("kind") == kind
            and e.get("resource") == resource
            and e.get("result") == result
        ):
            return True
    return False

def strace_text(prefix):
    parts = []
    for p in sorted(prefix.parent.glob(prefix.name + "*")):
        if p.is_file():
            parts.append(p.read_text(encoding="utf-8", errors="replace"))
    return "\n".join(parts)

def timed(cmd, repetitions=5):
    values = []
    for _ in range(repetitions):
        start = time.perf_counter_ns()
        proc = subprocess.run(
            cmd,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=False,
            timeout=30,
        )
        end = time.perf_counter_ns()
        if proc.returncode != 0:
            raise RuntimeError(f"benchmark falló: {cmd}, rc={proc.returncode}")
        values.append((end - start) / 1_000_000.0)
    return values

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--observer", required=True)
    ap.add_argument("--micro", required=True)
    ap.add_argument("--raw", required=True)
    ap.add_argument("--metrics", required=True)
    ap.add_argument("--comparison", required=True)
    args = ap.parse_args()

    observer = str(Path(args.observer).resolve())
    micro = str(Path(args.micro).resolve())
    raw = Path(args.raw).resolve()
    metrics_path = Path(args.metrics)
    comparison_path = Path(args.comparison)

    raw.mkdir(parents=True, exist_ok=True)
    sandbox = raw / "sandbox"
    sandbox.mkdir(parents=True, exist_ok=True)
    (sandbox / "fixture.txt").write_text("envmorph-f9\n", encoding="utf-8")

    expected = {
        "absolute": [("file_open", str(sandbox / "fixture.txt"), "success")],
        "relative": [("file_open", str(sandbox / "fixture.txt"), "success")],
        "dirfd": [("file_open", str(sandbox / "fixture.txt"), "success")],
        "missing": [("file_open", str(sandbox / "missing.txt"), "error")],
        "forkexec": [
            ("exec", "/bin/cat", "success"),
            ("file_open", str(sandbox / "fixture.txt"), "success"),
        ],
    }

    rows = []
    all_ok = True

    for case in CASES:
        case_dir = raw / case
        case_dir.mkdir(parents=True, exist_ok=True)

        baseline = run([micro, case, str(sandbox)])
        (case_dir / "baseline.stdout").write_bytes(baseline.stdout)
        (case_dir / "baseline.stderr").write_bytes(baseline.stderr)

        observer_log = case_dir / "observer.jsonl"
        observed = run([
            observer,
            "--output", str(observer_log),
            "--",
            micro, case, str(sandbox),
        ])
        (case_dir / "observer.stdout").write_bytes(observed.stdout)
        (case_dir / "observer.stderr").write_bytes(observed.stderr)

        strace_prefix = case_dir / "strace"
        traced = run([
            "strace", "-ff", "-qq", "-s", "4096", "-yy",
            "-e", "trace=open,openat,creat,execve,execveat",
            "-o", str(strace_prefix),
            "--",
            micro, case, str(sandbox),
        ])
        (case_dir / "strace.stdout").write_bytes(traced.stdout)
        (case_dir / "strace.stderr").write_bytes(traced.stderr)

        events = load_jsonl(observer_log)
        trace_text = strace_text(strace_prefix)

        obs_matches = []
        strace_matches = []
        for kind, resource, result in expected[case]:
            obs_matches.append(observer_has(events, kind, resource, result))

            if case in ("relative", "dirfd") and resource.endswith("/fixture.txt"):
                needle = '"fixture.txt"'
            else:
                needle = resource

            ok = needle in trace_text
            if result == "error":
                ok = ok and "ENOENT" in trace_text
            strace_matches.append(ok)

        stdout_equal = (
            baseline.stdout == observed.stdout
            and baseline.stdout == traced.stdout
        )
        rc_equal = (
            baseline.returncode == observed.returncode
            and baseline.returncode == traced.returncode
            and baseline.returncode == 0
        )
        no_enforcement = (
            b"BLOQUEADO" not in observed.stderr
            and b"BLOQUEAR" not in observed.stderr
        )

        case_ok = (
            all(obs_matches)
            and all(strace_matches)
            and stdout_equal
            and rc_equal
            and no_enforcement
        )
        all_ok = all_ok and case_ok

        rows.append({
            "case": case,
            "observer_expectations": sum(1 for x in obs_matches if x),
            "expected_total": len(obs_matches),
            "strace_expectations": sum(1 for x in strace_matches if x),
            "stdout_equivalent": stdout_equal,
            "exit_code_equivalent": rc_equal,
            "observer_no_enforcement": no_enforcement,
            "observer_events_total": len(events),
            "status": "pass" if case_ok else "fail",
        })

    bench_dir = raw / "benchmark"
    bench_dir.mkdir(parents=True, exist_ok=True)
    bench_observer = bench_dir / "observer.jsonl"
    bench_strace = bench_dir / "strace"

    baseline_ms = timed([micro, "absolute", str(sandbox)])
    observer_ms = timed([
        observer, "--output", str(bench_observer), "--",
        micro, "absolute", str(sandbox)
    ])
    strace_ms = timed([
        "strace", "-ff", "-qq", "-s", "4096", "-yy",
        "-e", "trace=open,openat,creat,execve,execveat",
        "-o", str(bench_strace),
        "--",
        micro, "absolute", str(sandbox),
    ])

    base_med = statistics.median(baseline_ms)
    obs_med = statistics.median(observer_ms)
    strace_med = statistics.median(strace_ms)

    result = {
        "schema_version": 1,
        "phase": "F9.1A",
        "candidate": "agentguard-derived-ptrace-observer",
        "reference": "strace-ff",
        "status": "validated_candidate_not_final_backend" if all_ok else "failed",
        "f8_corpus_executed": False,
        "cases": rows,
        "summary": {
            "cases_total": len(rows),
            "cases_passed": sum(1 for r in rows if r["status"] == "pass"),
            "observer_expectations_total": sum(r["expected_total"] for r in rows),
            "observer_expectations_matched": sum(r["observer_expectations"] for r in rows),
            "strace_expectations_matched": sum(r["strace_expectations"] for r in rows),
            "stdout_equivalent_cases": sum(1 for r in rows if r["stdout_equivalent"]),
            "exit_code_equivalent_cases": sum(1 for r in rows if r["exit_code_equivalent"]),
        },
        "overhead_microbenchmark": {
            "repetitions": 5,
            "baseline_ms": baseline_ms,
            "observer_ms": observer_ms,
            "strace_ms": strace_ms,
            "baseline_median_ms": base_med,
            "observer_median_ms": obs_med,
            "strace_median_ms": strace_med,
            "observer_ratio_to_baseline": obs_med / base_med if base_med else None,
            "strace_ratio_to_baseline": strace_med / base_med if base_med else None,
        },
        "limitations": [
            "candidate captures only open/openat/creat and execve/execveat",
            "environment variables consumed in memory are not observable by syscall tracing",
            "vDSO accesses are outside this candidate",
            "path resolution is lexical and does not claim complete symlink identity",
            "overhead values are engineering microbenchmarks, not paper-level performance claims",
            "eBPF feasibility remains pending before final backend selection",
        ],
    }

    metrics_path.parent.mkdir(parents=True, exist_ok=True)
    metrics_path.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    with comparison_path.open("w", encoding="utf-8") as f:
        f.write("case\tobserver\tstrace\tstdout\texit\tstatus\n")
        for r in rows:
            f.write(
                f"{r['case']}\t"
                f"{r['observer_expectations']}/{r['expected_total']}\t"
                f"{r['strace_expectations']}/{r['expected_total']}\t"
                f"{'equal' if r['stdout_equivalent'] else 'different'}\t"
                f"{'equal' if r['exit_code_equivalent'] else 'different'}\t"
                f"{r['status']}\n"
            )

    if not all_ok:
        raise SystemExit("F9.1A comparison failed")

if __name__ == "__main__":
    main()
