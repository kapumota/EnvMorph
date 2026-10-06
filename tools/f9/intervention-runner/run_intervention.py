#!/usr/bin/env python3
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

def sha256_bytes(data):
    return hashlib.sha256(data).hexdigest()

def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))

def logical_to_workspace(resource, workspace):
    prefix = "@WORKSPACE@/"
    if not resource.startswith(prefix):
        raise SystemExit("Ruta fuera de @WORKSPACE@")
    relative = resource[len(prefix):]
    if not relative or relative.startswith("/") or ".." in Path(relative).parts:
        raise SystemExit("Ruta lógica insegura")
    return workspace / relative

def replace_workspace_token(values, workspace):
    rendered = []
    prefix = "@WORKSPACE@/"

    for value in values:
        if value == "@WORKSPACE@":
            rendered.append(str(workspace))
        elif value.startswith(prefix):
            relative = value[len(prefix):]
            if not relative or relative.startswith("/") or ".." in Path(relative).parts:
                raise SystemExit("Ruta lógica insegura en argumento")
            rendered.append(str(workspace / relative))
        else:
            rendered.append(value)

    return rendered

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--plan", required=True)
    parser.add_argument("--variant", choices=["baseline", "treatment"], required=True)
    parser.add_argument("--workspace-source", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    if not args.command or args.command[0] != "--":
        raise SystemExit("Se requiere -- antes del comando")

    command = args.command[1:]
    if not command:
        raise SystemExit("Comando vacío")

    plan = load_json(args.plan)
    value_key = (
        "baseline_value"
        if args.variant == "baseline"
        else "treatment_value"
    )
    value = plan[value_key]

    source = Path(args.workspace_source).resolve()
    if not source.is_dir():
        raise SystemExit("workspace-source no es directorio")

    with tempfile.TemporaryDirectory(prefix="envmorph-f9-intervention-") as td:
        workspace = Path(td) / "workspace"
        shutil.copytree(source, workspace)

        env = dict(os.environ)
        applied_target = plan["resource"]

        if plan["mechanism"] == "environment_override":
            env[plan["resource"]] = value

        elif plan["mechanism"] == "isolated_text_file_replacement":
            target = logical_to_workspace(plan["resource"], workspace)
            if not target.is_file():
                raise SystemExit("Archivo objetivo no existe en workspace aislado")
            target.write_text(value, encoding="utf-8")

        else:
            raise SystemExit("Mecanismo no soportado")

        rendered = replace_workspace_token(command, workspace)

        proc = subprocess.run(
            rendered,
            cwd=workspace,
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
            timeout=20,
        )

        result = {
            "schema_version": 1,
            "phase": "F9.4",
            "plan_id": plan["plan_id"],
            "variant": args.variant,
            "mechanism": plan["mechanism"],
            "applied_target": applied_target,
            "applied_value_sha256": sha256_bytes(value.encode("utf-8")),
            "exit_code": proc.returncode,
            "stdout_sha256": sha256_bytes(proc.stdout),
            "stderr_sha256": sha256_bytes(proc.stderr),
            "workspace_source_unchanged_by_contract": True,
            "behavioral_relevance_evaluated": False,
            "f8_corpus_executed": False,
        }

        Path(args.output).write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )

        if proc.returncode != 0:
            raise SystemExit(
                f"Comando de ground truth falló con rc={proc.returncode}"
            )

if __name__ == "__main__":
    main()
