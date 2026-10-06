#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
from pathlib import Path

BINDINGS = {
    "locale": "LC_ALL",
    "timezone": "TZ",
    "tmpdir": "TMPDIR",
}

def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()

def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def matrix_value(text, key):
    match = re.search(
        rf'^{re.escape(key)}\s*=\s*"([^"]*)"\s*$',
        text,
        re.M,
    )
    if not match:
        raise SystemExit(f"Falta {key} en matrix_manifest")
    return match.group(1)

def workload_spec(text, workload_id):
    for block in text.split("[[workload]]")[1:]:
        match = re.search(
            r'^id\s*=\s*"([^"]+)"\s*$',
            block,
            re.M,
        )
        if not match or match.group(1) != workload_id:
            continue

        factors_match = re.search(
            r'^factors\s*=\s*\[(.*?)\]\s*$',
            block,
            re.M,
        )
        trace_match = re.search(
            r'^trace\s*=\s*"([^"]+)"\s*$',
            block,
            re.M,
        )

        if not factors_match or not trace_match:
            raise SystemExit("Bloque workload incompleto")

        return {
            "factors": re.findall(
                r'"([^"]+)"',
                factors_match.group(1),
            ),
            "trace": trace_match.group(1),
        }

    raise SystemExit(f"No se encontró workload {workload_id}")

def sanitize(workload_id, workspace):
    artifacts = workspace / "artifacts"
    if artifacts.exists():
        shutil.rmtree(artifacts)

    if workload_id == "jq-build-check":
        targets = [
            workspace / "source" / "autom4te.cache",
            workspace / "source" / ".deps",
            workspace / "source" / "src" / ".deps",
            workspace / "source" / "vendor" / "oniguruma" / "src" / ".deps",
        ]
        for target in targets:
            if target.exists():
                shutil.rmtree(target)

        for pattern in (
            "source/jq",
            "source/src/*.o",
            "source/vendor/oniguruma/src/*.o",
        ):
            for target in workspace.glob(pattern):
                if target.is_file():
                    target.unlink()

    elif workload_id == "fd-build-completions":
        target = workspace / "source" / "target"
        if target.exists():
            shutil.rmtree(target)

    elif workload_id == "inih-meson-tests":
        target = workspace / "build"
        if target.exists():
            shutil.rmtree(target)

    elif workload_id != "csvkit-realdata":
        raise SystemExit(f"Workload desconocido: {workload_id}")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True)
    parser.add_argument("--workload-id", required=True)
    parser.add_argument("--runtime-source", required=True)
    parser.add_argument("--matrix", required=True)
    parser.add_argument("--variant", choices=["baseline", "treatment"], required=True)
    parser.add_argument("--repetition", type=int, choices=[1, 2], required=True)
    parser.add_argument("--output-root", required=True)
    args = parser.parse_args()

    plan = load_json(args.plan)

    if plan.get("schema_version") != "f9.intervention_plan.v1":
        raise SystemExit("Plan con schema inesperado")

    if plan.get("mechanism") != "environment_override":
        raise SystemExit("F9.8C externo solo admite environment_override")

    if plan.get("resource") not in set(BINDINGS.values()):
        raise SystemExit("Binding ambiental fuera del scope F8")

    if plan.get("confirmations") != 2:
        raise SystemExit("F9.8C espera exactamente dos confirmaciones")

    source = Path(args.runtime_source).resolve()
    if not source.is_dir():
        raise SystemExit("Runtime source no existe")

    output = Path(args.output_root).resolve()
    if output.exists():
        raise SystemExit(f"Output ya existe: {output}")

    output.mkdir(parents=True)
    workspace = output / "workspace"
    shutil.copytree(source, workspace)
    sanitize(args.workload_id, workspace)

    matrix_text = Path(args.matrix).read_text(encoding="utf-8")
    spec = workload_spec(matrix_text, args.workload_id)

    baseline_values = {
        "locale": matrix_value(matrix_text, "locale_baseline"),
        "timezone": matrix_value(matrix_text, "timezone_baseline"),
        "tmpdir": matrix_value(matrix_text, "tmpdir_baseline"),
    }

    env = dict(os.environ)
    env.update({
        "PIP_NO_INDEX": "1",
        "PIP_DISABLE_PIP_VERSION_CHECK": "1",
        "CARGO_NET_OFFLINE": "true",
        "GIT_TERMINAL_PROMPT": "0",
    })

    for factor in spec["factors"]:
        env[BINDINGS[factor]] = baseline_values[factor]

    value_key = (
        "baseline_value"
        if args.variant == "baseline"
        else "treatment_value"
    )
    env[plan["resource"]] = plan[value_key]

    stdout_path = output / "adapter.stdout"
    stderr_path = output / "adapter.stderr"

    timed_out = False

    try:
        proc = subprocess.run(
            ["bash", "adapter/run.sh"],
            cwd=workspace,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=1200,
        )
        exit_code = proc.returncode
        stdout = proc.stdout
        stderr = proc.stderr
    except subprocess.TimeoutExpired as exc:
        timed_out = True
        exit_code = 124
        stdout = exc.stdout or b""
        stderr = exc.stderr or b""

    stdout_path.write_bytes(stdout)
    stderr_path.write_bytes(stderr)

    record = {
        "schema_version": "f9.external_intervention_run.v1",
        "plan_id": plan["plan_id"],
        "dependency_key": plan["dependency_key"],
        "workload_id": args.workload_id,
        "variant": args.variant,
        "repetition": args.repetition,
        "applied_target": plan["resource"],
        "applied_value_sha256": sha256_bytes(
            plan[value_key].encode("utf-8")
        ),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "stdout_sha256": sha256_bytes(stdout),
        "stderr_sha256": sha256_bytes(stderr),
        "workspace_source_unchanged_by_contract": True,
        "treatment_selected_before_execution": True,
        "behavioral_relevance_evaluated": False,
        "external_corpus_execution": True,
    }

    (output / "run.json").write_text(
        json.dumps(record, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

if __name__ == "__main__":
    main()
