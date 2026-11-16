#!/usr/bin/env python3
"""M4-G2: planificador tipado prospectivo. Nunca ejecuta workloads."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import sys
import tempfile
from pathlib import Path

SCHEMA = "m4.typed_interventions.v1"
ID_PATTERN = re.compile(r"^[a-z][a-z0-9_]{0,63}$")
ENV_PATTERN = re.compile(r"^[A-Z_][A-Z0-9_]*$")
DENY_ENV = {
    "LD_PRELOAD", "LD_LIBRARY_PATH", "LD_AUDIT", "PATH", "PYTHONPATH",
    "PYTHONHOME", "BASH_ENV", "ENV", "SHELLOPTS", "BASHOPTS",
    "GIT_CONFIG_GLOBAL", "GIT_CONFIG_SYSTEM", "IFS", "HOME",
}
KINDS = {"environment_override", "workspace_file_replace"}


class PlanError(ValueError):
    pass


def fail(message: str) -> None:
    raise PlanError(message)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def file_digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fd:
        for block in iter(lambda: fd.read(131072), b""):
            h.update(block)
    return h.hexdigest()


def canonical_json(obj: object) -> bytes:
    return (json.dumps(obj, ensure_ascii=False, sort_keys=True, indent=2) + "\n").encode("utf-8")


def json_file(path: Path) -> dict:
    value = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(value, dict):
        fail("el archivo JSON debe contener un objeto")
    return value


def relative_safe(value: object, label: str) -> Path:
    if not isinstance(value, str) or not value or "\\" in value or "\x00" in value:
        fail(f"{label}: ruta relativa inválida")
    path = Path(value)
    if path.is_absolute() or any(p in ("", ".", "..") for p in value.split("/")):
        fail(f"{label}: ruta absoluta o recorrido de directorios no permitido")
    return path


def checked_file(root: Path, relative: Path, label: str) -> Path:
    if root.is_symlink() or not root.is_dir():
        fail(f"{label}: raíz inexistente o enlace simbólico")
    current = root
    for part in relative.parts:
        current = current / part
        if current.is_symlink():
            fail(f"{label}: no se admiten enlaces simbólicos en el objetivo")
    if not current.is_file():
        fail(f"{label}: archivo requerido inexistente")
    return current


def schema_check(spec: dict) -> None:
    if set(spec) != {"schema_version", "campaign_id", "workflow_id", "factors"}:
        fail("campos principales inesperados o faltantes")
    if spec["schema_version"] != SCHEMA:
        fail("schema_version no soportado")
    for name in ("campaign_id", "workflow_id"):
        if not isinstance(spec[name], str) or not ID_PATTERN.fullmatch(spec[name]):
            fail(f"{name} inválido")
    factors = spec["factors"]
    if not isinstance(factors, list) or not 1 <= len(factors) <= 6:
        fail("se requieren entre uno y seis factores")
    ids, bindings, targets = set(), set(), set()
    for factor in factors:
        if not isinstance(factor, dict):
            fail("factor no es objeto")
        if not isinstance(factor.get("id"), str) or not ID_PATTERN.fullmatch(factor["id"]):
            fail("id de factor inválido")
        if factor["id"] in ids:
            fail("id duplicado")
        ids.add(factor["id"])
        kind = factor.get("kind")
        if kind not in KINDS:
            fail("clase de intervención no soportada")
        if kind == "environment_override":
            if set(factor) != {"id", "kind", "variable", "baseline", "treatment"}:
                fail("campos de environment_override inválidos")
            variable = factor["variable"]
            if not isinstance(variable, str) or not ENV_PATTERN.fullmatch(variable):
                fail("nombre de variable no seguro")
            if variable in DENY_ENV:
                fail("variable de entorno bloqueada por seguridad")
            if variable in bindings:
                fail("binding ambiental duplicado")
            bindings.add(variable)
            for k in ("baseline", "treatment"):
                if not isinstance(factor[k], str) or len(factor[k]) > 4096 or "\x00" in factor[k]:
                    fail("valor ambiental inválido")
            if factor["baseline"] == factor["treatment"]:
                fail("baseline y treatment deben ser distintos")
        else:
            if set(factor) != {"id", "kind", "target", "baseline_sha256", "fixture", "treatment_sha256"}:
                fail("campos de workspace_file_replace inválidos")
            target = relative_safe(factor["target"], "target").as_posix()
            relative_safe(factor["fixture"], "fixture")
            if target in targets:
                fail("archivo objetivo duplicado")
            targets.add(target)
            for field in ("baseline_sha256", "treatment_sha256"):
                v = factor[field]
                if not isinstance(v, str) or not re.fullmatch(r"[0-9a-f]{64}", v):
                    fail("hash SHA-256 inválido")
            if factor["baseline_sha256"] == factor["treatment_sha256"]:
                fail("los bytes baseline y treatment no pueden coincidir")


def validate(spec: dict, workspace: Path, fixtures: Path) -> dict:
    schema_check(spec)
    workspace = workspace.absolute()
    fixtures = fixtures.absolute()
    if workspace.is_symlink() or not workspace.is_dir():
        fail("workspace no existe o es enlace simbólico")
    if fixtures.is_symlink() or not fixtures.is_dir():
        fail("fixtures no existe o es enlace simbólico")
    evidence = []
    for factor in spec["factors"]:
        entry = {"id": factor["id"], "kind": factor["kind"]}
        if factor["kind"] == "environment_override":
            entry["variable"] = factor["variable"]
            entry["baseline"] = factor["baseline"]
            entry["treatment"] = factor["treatment"]
        else:
            target = checked_file(workspace, relative_safe(factor["target"], "target"), "target")
            fixture = checked_file(fixtures, relative_safe(factor["fixture"], "fixture"), "fixture")
            observed_base = file_digest(target)
            observed_treat = file_digest(fixture)
            if observed_base != factor["baseline_sha256"]:
                fail(f"baseline modificado: {factor['id']}")
            if observed_treat != factor["treatment_sha256"]:
                fail(f"fixture modificado: {factor['id']}")
            entry["target"] = factor["target"]
            entry["fixture"] = factor["fixture"]
            entry["baseline_sha256"] = observed_base
            entry["treatment_sha256"] = observed_treat
        evidence.append(entry)
    return {
        "schema_version": "m4.typed_preflight.v1",
        "campaign_id": spec["campaign_id"],
        "workflow_id": spec["workflow_id"],
        "spec_sha256": digest(canonical_json(spec)),
        "factor_count": len(evidence),
        "factors": evidence,
        "execution_status": "NOT_EXECUTED",
        "causal_decision": "NOT_EVALUATED",
        "scope": "PREPARATION_ONLY_NO_F8_F9_REWRITE",
    }


def ensure_output_location(output: Path, workspace: Path, fixtures: Path) -> None:
    if output.exists():
        fail("la salida ya existe, no se sobreescriben expedientes")
    resolved = output.absolute()
    for protected in (workspace.absolute(), fixtures.absolute()):
        if resolved == protected or resolved.is_relative_to(protected) or protected.is_relative_to(resolved):
            fail("la salida no puede contener ni ser parte del workspace o de fixtures")
    if not output.parent.exists() or output.parent.is_symlink():
        fail("el directorio padre de salida debe existir y no ser enlace simbólico")


def prepare(spec_path: Path, workspace: Path, fixtures: Path, output: Path) -> dict:
    spec = json_file(spec_path)
    preflight = validate(spec, workspace, fixtures)
    ensure_output_location(output, workspace, fixtures)
    output.mkdir(mode=0o700)
    (output / "M4_G2_PREFLIGHT.json").write_bytes(canonical_json(preflight))
    return preflight


def materialize(spec_path: Path, workspace: Path, fixtures: Path, output: Path,
                expected_spec_sha256: str) -> dict:
    spec = json_file(spec_path)
    preflight = validate(spec, workspace, fixtures)
    if preflight["spec_sha256"] != expected_spec_sha256:
        fail("el plan no coincide con el SHA-256 autorizado")
    ensure_output_location(output, workspace, fixtures)
    output.mkdir(mode=0o700)
    arms = {}
    try:
        for arm in ("baseline", "treatment"):
            dest = output / arm
            shutil.copytree(workspace, dest, symlinks=True)
            environment = {}
            for factor in spec["factors"]:
                if factor["kind"] == "environment_override":
                    environment[factor["variable"]] = factor[arm]
                elif arm == "treatment":
                    fixture = checked_file(fixtures, relative_safe(factor["fixture"], "fixture"), "fixture")
                    target = checked_file(dest, relative_safe(factor["target"], "target"), "target")
                    shutil.copyfile(fixture, target)
                    if file_digest(target) != factor["treatment_sha256"]:
                        fail("falló la comprobación posterior de la sustitución")
            # El manifiesto se guarda fuera del workspace copiado.
            arms[arm] = {"workspace": arm, "environment_overrides": environment}
        manifest = dict(preflight)
        manifest["materialized_arms"] = arms
        manifest["execution_status"] = "NOT_EXECUTED"
        (output / "M4_G2_MANIFEST.json").write_bytes(canonical_json(manifest))
        return manifest
    except Exception:
        # Conservar salida parcial para diagnóstico; no reescribir ni ejecutar.
        (output / "M4_G2_INCOMPLETE").write_text("materialización fallida, no ejecutar\n", encoding="utf-8")
        raise


def selftest() -> None:
    with tempfile.TemporaryDirectory(prefix="envmorph-m4-g2-") as directory:
        root = Path(directory)
        workspace = root / "workspace"
        fixture_root = root / "fixtures"
        workspace.mkdir(); fixture_root.mkdir()
        (workspace / "input.txt").write_bytes(b"contenido inicial\n")
        (fixture_root / "candidate.txt").write_bytes(b"contenido tratado\n")
        baseline = file_digest(workspace / "input.txt")
        treatment = file_digest(fixture_root / "candidate.txt")
        spec = {
            "schema_version": SCHEMA,
            "campaign_id": "g2_dev_only",
            "workflow_id": "toy_file_and_locale",
            "factors": [
                {"id": "locale", "kind": "environment_override", "variable": "LC_ALL", "baseline": "C", "treatment": "C.UTF-8"},
                {"id": "input", "kind": "workspace_file_replace", "target": "input.txt", "baseline_sha256": baseline,
                 "fixture": "candidate.txt", "treatment_sha256": treatment},
            ],
        }
        spec_path = root / "spec.json"
        spec_path.write_bytes(canonical_json(spec))
        before = file_digest(workspace / "input.txt")
        pf = prepare(spec_path, workspace, fixture_root, root / "preflight")
        materialize(spec_path, workspace, fixture_root, root / "variants", pf["spec_sha256"])
        assert file_digest(workspace / "input.txt") == before
        assert file_digest(root / "variants/baseline/input.txt") == baseline
        assert file_digest(root / "variants/treatment/input.txt") == treatment
        result = json_file(root / "variants/M4_G2_MANIFEST.json")
        assert result["materialized_arms"]["baseline"]["environment_overrides"]["LC_ALL"] == "C"
        assert result["materialized_arms"]["treatment"]["environment_overrides"]["LC_ALL"] == "C.UTF-8"
        assert result["execution_status"] == "NOT_EXECUTED"
        # Casos adversariales.
        def rejects(obj: dict, ws=workspace, fx=fixture_root):
            try:
                validate(obj, ws, fx)
            except PlanError:
                return
            raise AssertionError("se aceptó un plan adversarial")
        import copy
        candidate = copy.deepcopy(spec); candidate["factors"][0]["variable"] = "LD_PRELOAD"; rejects(candidate)
        candidate = copy.deepcopy(spec); candidate["factors"][1]["target"] = "../escape"; rejects(candidate)
        candidate = copy.deepcopy(spec); candidate["factors"][1]["fixture"] = "/etc/passwd"; rejects(candidate)
        candidate = copy.deepcopy(spec); candidate["factors"][1]["baseline_sha256"] = "0"*64; rejects(candidate)
        candidate = copy.deepcopy(spec); candidate["factors"].append(copy.deepcopy(spec["factors"][0])); rejects(candidate)
        (workspace / "link.txt").symlink_to(root / "outside")
        candidate = copy.deepcopy(spec); candidate["factors"][1]["target"] = "link.txt"; rejects(candidate)
        (workspace / "link.txt").unlink()
        (fixture_root / "candidate.txt").write_bytes(b"alterado\n")
        rejects(spec)
        print("M4_G2_SELFTEST=PASS")
        print("M4_G2_ADVERSARIAL=REJECTED")
        print("M4_G2_REPO_WRITE=NONE")
        print("M4_G2_SCIENTIFIC_EVIDENCE=NONE_TOY_ONLY")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("selftest")
    for action in ("prepare", "materialize"):
        cmd = sub.add_parser(action)
        cmd.add_argument("--spec", type=Path, required=True)
        cmd.add_argument("--workspace", type=Path, required=True)
        cmd.add_argument("--fixtures", type=Path, required=True)
        cmd.add_argument("--output", type=Path, required=True)
        if action == "materialize":
            cmd.add_argument("--expected-spec-sha256", required=True)
            cmd.add_argument("--ack-development-only", action="store_true", required=True)
    args = parser.parse_args()
    try:
        if args.command == "selftest":
            selftest()
        elif args.command == "prepare":
            result = prepare(args.spec, args.workspace, args.fixtures, args.output)
            print("M4_G2_PLAN=PASS")
            print("M4_G2_SPEC_SHA256=" + result["spec_sha256"])
            print("M4_G2_EXECUTIONS=0")
            print("M4_G2_OUTPUT=" + str(args.output.absolute()))
        else:
            if not args.ack_development_only:
                fail("se requiere reconocimiento explícito de materialización de desarrollo")
            result = materialize(args.spec, args.workspace, args.fixtures, args.output,
                                 args.expected_spec_sha256)
            print("M4_G2_MATERIALIZE=PASS")
            print("M4_G2_ARMS=" + ",".join(sorted(result["materialized_arms"])))
            print("M4_G2_EXECUTIONS=0")
        return 0
    except (OSError, ValueError, AssertionError, json.JSONDecodeError) as error:
        print("M4_G2=BLOCKED: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
