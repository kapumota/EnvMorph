#!/usr/bin/env python3
"""Piloto M4-G3, ingeniería exploratoria sobre inih, sin modificar F8-F10/P1.

CLI en inglés; comentarios, mensajes de diagnóstico y documentación en español.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import uuid
import zipfile
from datetime import datetime, timezone
from pathlib import Path
from pathlib import PurePosixPath

IMAGE_BASE = "envmorph-p1-ubuntu2404-locales:local"
IMAGE_DERIVED = "envmorph-g3-inih-cpp:local"
EXPECTED_BRANCH = "research/m4-g3-prospective"
EXPECTED_FACTOR = {"baseline": "/tmp", "treatment": "/var/tmp"}
WHEEL_VERSIONS = {"meson": "1.12.1", "ninja": "1.13.2"}


def ninja_binary_entry(archive: zipfile.ZipFile) -> str:
    """Ubicar el ejecutable ELF x86-64 sin suponer una ruta fija del wheel."""
    candidates = []
    elf_files = []
    for member in archive.infolist():
        if member.is_dir():
            continue
        if member.file_size < 64 or member.file_size > 40 * 1024 * 1024:
            continue
        with archive.open(member) as stream:
            head = stream.read(64)
        # ELF64 little-endian x86-64, e_machine 62.
        if not (head[:4] == b"\x7fELF" and head[4:6] == b"\x02\x01"
                and int.from_bytes(head[18:20], "little") == 62):
            continue
        elf_files.append(member.filename)
        if PurePosixPath(member.filename).name in {"ninja", "ninja.bin"}:
            candidates.append(member.filename)
    if len(candidates) == 1:
        return candidates[0]
    # Para wheels que han reestructurado el árbol, sólo aceptar un único ELF.
    if not candidates and len(elf_files) == 1:
        return elf_files[0]
    raise RuntimeError(
        "Wheel Ninja sin ejecutable ELF x86_64 inequívoco; "
        f"candidatos={candidates!r}; archivos_ELF={elf_files!r}"
    )

INSTALLER = r'''#!/usr/bin/env python3
"""Instalación offline, solo de paquetes wheel previamente congelados."""
import os
import re
import shutil
import stat
import zipfile
from pathlib import Path

wheels = Path("/opt/wheels")
root = Path("/opt/envmorph-g3-py")
root.mkdir(parents=True, exist_ok=True)
files = sorted(wheels.glob("*.whl"))
assert len(files) == 2, "se esperaban exactamente dos wheels"


def locate_ninja(archive):
    candidates = []
    elf_files = []
    for member in archive.infolist():
        if member.is_dir() or member.file_size < 64 or member.file_size > 40 * 1024 * 1024:
            continue
        with archive.open(member) as stream:
            head = stream.read(64)
        if not (head[:4] == b"\x7fELF" and head[4:6] == b"\x02\x01"
                and int.from_bytes(head[18:20], "little") == 62):
            continue
        elf_files.append(member.filename)
        if Path(member.filename).name in ("ninja", "ninja.bin"):
            candidates.append(member.filename)
    if len(candidates) == 1:
        return candidates[0]
    if not candidates and len(elf_files) == 1:
        return elf_files[0]
    raise RuntimeError("Ninja sin ELF x86_64 inequívoco; archivos=" + repr(elf_files))

ninja_member = None
for wheel in files:
    with zipfile.ZipFile(wheel) as archive:
        if wheel.name.startswith("ninja-"):
            ninja_member = locate_ninja(archive)
            print("G3_NINJA_WHEEL_ENTRY=" + ninja_member)
        for member in archive.infolist():
            path = member.filename
            parts = Path(path).parts
            assert parts and not path.startswith("/") and "\\" not in path and ".." not in parts, path
            mode = (member.external_attr >> 16) & 0xFFFF
            assert not stat.S_ISLNK(mode), path
            dest = root.joinpath(*parts)
            if member.is_dir():
                dest.mkdir(parents=True, exist_ok=True)
            else:
                dest.parent.mkdir(parents=True, exist_ok=True)
                with archive.open(member) as src, dest.open("wb") as out:
                    shutil.copyfileobj(src, out)

meson = root / "mesonbuild/mesonmain.py"
assert ninja_member is not None, "no se identificó el ejecutable ninja"
ninja = root / ninja_member
assert meson.is_file(), "no existe mesonbuild/mesonmain.py"
assert ninja.is_file(), "no existe el ejecutable Ninja seleccionado"

launcher = Path("/usr/local/bin/meson")
launcher.write_text(
    '#!/usr/bin/python3\n'
    'import sys\n'
    'sys.path.insert(0, "/opt/envmorph-g3-py")\n'
    'from mesonbuild.mesonmain import main\n'
    'if __name__ == "__main__":\n'
    '    raise SystemExit(main())\n'
)
launcher.chmod(0o755)
shutil.copy2(ninja, "/usr/local/bin/ninja")
Path("/usr/local/bin/ninja").chmod(0o755)
print("G3_OFFLINE_TOOLS_INSTALLED=PASS")
'''

DOCKERFILE = '''FROM envmorph-p1-ubuntu2404-locales:local
COPY wheels/ /opt/wheels/
COPY install_offline.py /usr/local/sbin/install_offline.py
RUN python3 /usr/local/sbin/install_offline.py \\
    && /usr/local/bin/meson --version \\
    && /usr/local/bin/ninja --version \\
    && rm -rf /opt/wheels /usr/local/sbin/install_offline.py
'''


def digest(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(128 * 1024), b""):
            h.update(block)
    return h.hexdigest()


def run(args: list[str], *, output: Path | None = None, timeout: int = 180) -> subprocess.CompletedProcess:
    result = subprocess.run(args, capture_output=True, timeout=timeout, check=False)
    if output is not None:
        output.mkdir(parents=True, exist_ok=True)
        (output / "stdout.bin").write_bytes(result.stdout)
        (output / "stderr.bin").write_bytes(result.stderr)
    return result


def require_command(command: str) -> None:
    if shutil.which(command) is None:
        raise RuntimeError(f"Falta comando requerido: {command}")


def require_env(repo: Path, stage: Path) -> dict:
    if not repo.is_dir() or not (repo / ".git").exists():
        raise RuntimeError("Ejecutar desde la raíz del repositorio EnvMorph")
    branch = subprocess.check_output(["git", "branch", "--show-current"], cwd=repo, text=True).strip()
    if branch != EXPECTED_BRANCH:
        raise RuntimeError(f"Rama incorrecta: {branch}")
    if not stage.is_dir() or stage.is_symlink():
        raise RuntimeError("La carpeta de preparación no existe o es un enlace")
    if stage.resolve().is_relative_to(repo.resolve()):
        raise RuntimeError("La salida del piloto debe estar fuera del repositorio")
    manifest_path = stage / "variants/M4_G2_MANIFEST.json"
    spec_path = stage / "PILOT_SPEC_DRAFT.json"
    if not manifest_path.is_file() or not spec_path.is_file():
        raise RuntimeError("No están el manifiesto o el plan materializado")
    if (stage / "variants/M4_G2_INCOMPLETE").exists():
        raise RuntimeError("Materialización incompleta")
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    spec = json.loads(spec_path.read_text(encoding="utf-8"))
    if manifest.get("execution_status") != "NOT_EXECUTED":
        raise RuntimeError("El manifiesto no corresponde a una preparación no ejecutada")
    if manifest.get("workflow_id") != "inih_meson_tests":
        raise RuntimeError("El workflow no coincide con el piloto")
    if len(spec.get("factors", [])) != 1 or spec["factors"][0].get("variable") != "TMPDIR":
        raise RuntimeError("Este piloto admite únicamente TMPDIR")
    for arm, value in EXPECTED_FACTOR.items():
        if manifest["materialized_arms"][arm]["environment_overrides"].get("TMPDIR") != value:
            raise RuntimeError(f"Variante {arm} alterada")
        workspace = stage / "variants" / arm
        if not (workspace / "source").is_dir() or not (workspace / "adapter/run.sh").is_file():
            raise RuntimeError(f"Workspace de {arm} incompleto")
    return manifest


def validate_wheels(repo: Path) -> dict[str, Path]:
    wheelhouse = repo / ".envmorph/f8-runtime/inih-meson-tests/deps/wheelhouse"
    provenance = repo / "provenance/f8/runtime/inih-wheelhouse.sha256"
    if not wheelhouse.is_dir():
        raise RuntimeError("Falta el wheelhouse de F8.4; no reconstruir F8")
    if not provenance.is_file():
        raise RuntimeError("Falta procedencia SHA-256 del wheelhouse; no crear la imagen")
    selected: dict[str, Path] = {}
    for package, version in WHEEL_VERSIONS.items():
        candidates = list(wheelhouse.glob(f"{package}-{version}-*.whl"))
        if len(candidates) != 1:
            raise RuntimeError(f"Se esperaba exactamente un wheel de {package}=={version}; hay {len(candidates)}")
        selected[package] = candidates[0]
    # Comprobar todos los registros del manifiesto de F8, no solo los elegidos.
    for raw in provenance.read_text(encoding="utf-8").splitlines():
        if not raw.strip():
            continue
        match = re.fullmatch(r"([0-9a-f]{64})\s+\*?(.+)", raw)
        if not match:
            raise RuntimeError("Manifiesto SHA-256 de F8 con formato desconocido")
        expected, rel = match.groups()
        file = wheelhouse / rel
        if not file.is_file() or digest(file) != expected:
            raise RuntimeError(f"No coincide el SHA-256 congelado de {rel}")
    for package, wheel in selected.items():
        with zipfile.ZipFile(wheel) as archive:
            if archive.testzip() is not None:
                raise RuntimeError(f"Wheel corrupto: {wheel.name}")
            names = set(archive.namelist())
            if package == "meson":
                if "mesonbuild/mesonmain.py" not in names:
                    raise RuntimeError("Wheel de Meson sin mesonbuild/mesonmain.py")
            else:
                selected_ninja = ninja_binary_entry(archive)
                print(f"G3_NINJA_WHEEL_ENTRY={selected_ninja}")
        print(f"G3_WHEEL_{package.upper()}={wheel.name}")
        print(f"G3_SHA256_{package.upper()}={digest(wheel)}")
    print("G3_F8_WHEEL_PROVENANCE=PASS")
    return selected


def build_image(repo: Path, stage: Path) -> None:
    require_command("docker")
    for args in (["docker", "image", "inspect", IMAGE_BASE],):
        check = run(list(args), timeout=30)
        if check.returncode != 0:
            raise RuntimeError(f"No existe la imagen base local: {IMAGE_BASE}")
    wheels = validate_wheels(repo)
    buildroot = stage / "g3_image_context"
    if buildroot.exists():
        raise RuntimeError("Ya existe el contexto de compilación; no sobrescribir")
    (buildroot / "wheels").mkdir(parents=True)
    (buildroot / "Dockerfile").write_text(DOCKERFILE, encoding="utf-8")
    (buildroot / "install_offline.py").write_text(INSTALLER, encoding="utf-8")
    for wheel in wheels.values():
        shutil.copy2(wheel, buildroot / "wheels" / wheel.name)
    result = run([
        "docker", "build", "--pull=false", "--network=none", "--progress=plain",
        "-t", IMAGE_DERIVED, "-f", str(buildroot / "Dockerfile"), str(buildroot)
    ], output=stage / "g3_image_build_log", timeout=240)
    if result.returncode != 0:
        raise RuntimeError("La creación offline falló; consultar g3_image_build_log/stderr.bin y stdout.bin")
    print("G3_DOCKER_OFFLINE_BUILD=PASS")
    print("G3_IMAGE=" + IMAGE_DERIVED)


def tool_smoke() -> None:
    cmd = [
        "docker", "run", "--rm", "--pull=never", "--network=none",
        "--cap-drop=ALL", "--security-opt=no-new-privileges", "--read-only",
        "--pids-limit=64", "--memory=256m", "--cpus=1", "--user=65534:65534",
        "--tmpfs=/tmp:rw,nosuid,nodev,size=32m", "--entrypoint=/bin/sh",
        IMAGE_DERIVED, "-c", "meson --version && ninja --version && cc --version | head -n 1 && g++ --version | head -n 1"
    ]
    result = run(cmd, timeout=60)
    if result.returncode != 0:
        raise RuntimeError("Smoke test restringido de la imagen fallido: " + result.stderr.decode(errors="replace")[-1000:])
    print("G3_DOCKER_RESTRICTED_TOOLS=PASS")
    print(result.stdout.decode(errors="replace").strip())


def launch_arm(stage: Path, trial: Path, arm: str, repetition: int) -> dict:
    src = stage / "variants" / arm
    work = trial / f"{arm}_rep{repetition}" / "workspace"
    work.parent.mkdir(parents=True, exist_ok=False)
    # Fuente materializada no se modifica; cada repetición parte de una copia fresca.
    shutil.copytree(src, work, symlinks=False)
    out = work.parent
    container_name = "envmorph-g3-" + uuid.uuid4().hex[:16]
    cmd = [
        "docker", "run", "--rm", "--pull=never", "--name", container_name,
        "--network=none", "--cap-drop=ALL", "--security-opt=no-new-privileges",
        "--read-only", "--pids-limit=128", "--memory=1g", "--cpus=2",
        "--user", f"{os.getuid()}:{os.getgid()}",
        "--tmpfs=/tmp:rw,nosuid,nodev,size=128m",
        "--tmpfs=/var/tmp:rw,nosuid,nodev,size=128m",
        "--mount", f"type=bind,src={work},dst=/work",
        "--workdir=/work", "--env", "HOME=/tmp", "--env", "LC_ALL=C",
        "--env", "TZ=UTC", "--env", f"TMPDIR={EXPECTED_FACTOR[arm]}",
        "--entrypoint=/bin/bash", IMAGE_DERIVED, "adapter/run.sh"
    ]
    timed_out = False
    try:
        result = run(cmd, output=out, timeout=300)
        returncode, stdout, stderr = result.returncode, result.stdout, result.stderr
    except subprocess.TimeoutExpired as error:
        timed_out = True
        returncode, stdout, stderr = 124, error.stdout or b"", error.stderr or b""
        (out / "stdout.bin").write_bytes(stdout)
        (out / "stderr.bin").write_bytes(stderr)
    finally:
        if timed_out:
            subprocess.run(["docker", "rm", "-f", container_name], capture_output=True, timeout=30, check=False)
    observed = {}
    for rel in ("artifacts/01_test_status.txt", "artifacts/02_unittest_multi.txt"):
        path = work / rel
        observed[rel] = digest(path) if path.is_file() else None
    record = {
        "schema_version": "g3.engineering_pilot.v1", "arm": arm, "rep": repetition,
        "tmpdir": EXPECTED_FACTOR[arm], "return_code": returncode,
        "timed_out": timed_out,
        "stdout_sha256": hashlib.sha256(stdout).hexdigest(),
        "stderr_sha256": hashlib.sha256(stderr).hexdigest(),
        "artifact_sha256": observed,
        "scope": "DEVELOPMENT_PILOT_NOT_SCIENTIFIC_VALIDATION"
    }
    (out / "run.json").write_text(json.dumps(record, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(f"G3_RUN_{arm.upper()}_REP{repetition}_EXIT={returncode}")
    return record


def perform_pilot(stage: Path) -> None:
    tool_smoke()
    trial = Path(tempfile.mkdtemp(prefix="g3_pilot_", dir=stage))
    records = []
    for arm in ("baseline", "treatment"):
        for rep in (1, 2):
            records.append(launch_arm(stage, trial, arm, rep))
    two_repetitions_agree = {}
    for arm in EXPECTED_FACTOR:
        x, y = [r for r in records if r["arm"] == arm]
        two_repetitions_agree[arm] = (
            x["return_code"] == y["return_code"] == 0 and
            all(v is not None for v in x["artifact_sha256"].values()) and
            x["artifact_sha256"] == y["artifact_sha256"]
        )
    summary = {
        "schema_version": "g3.engineering_pilot_summary.v1",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "records": records,
        "within_arm_reproducible": two_repetitions_agree,
        "cross_arm_same_oracle_artifacts": (
            records[0]["artifact_sha256"] == records[2]["artifact_sha256"]
            if all(two_repetitions_agree.values()) else None
        ),
        "scientific_confirmatory": False,
        "f9_observer_integrated": False,
        "f6_contract_integrated": False,
        "a3_independent_host_reproduction": "DEFERRED"
    }
    (trial / "G3_PILOT_SUMMARY.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8"
    )
    print("G3_PILOT_OUTPUT=" + str(trial))
    if all(two_repetitions_agree.values()):
        print("G3_ENGINEERING_PILOT=PASS")
        print("G3_CROSS_ARM_SAME_ARTIFACTS=" + str(summary["cross_arm_same_oracle_artifacts"]).upper())
    else:
        print("G3_ENGINEERING_PILOT=INCOMPLETE")
        raise RuntimeError("Fallo o inestabilidad en el piloto, se conservaron todos los registros")
    print("G3_SCIENTIFIC_CONFIRMATORY=NO")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("preflight", "build", "run"))
    parser.add_argument("--stage", required=True, type=Path)
    args = parser.parse_args()
    repo, stage = Path.cwd(), args.stage.expanduser().absolute()
    try:
        require_command("git")
        require_command("python3.11")
        require_env(repo, stage)
        if args.action == "preflight":
            require_command("docker")
            validate_wheels(repo)
            print("G3_OFFLINE_IMAGE_PREFLIGHT=PASS")
        elif args.action == "build":
            build_image(repo, stage)
            tool_smoke()
        else:
            require_command("docker")
            if run(["docker", "image", "inspect", IMAGE_DERIVED], timeout=30).returncode != 0:
                raise RuntimeError("Primero ejecutar build para crear la imagen derivada")
            perform_pilot(stage)
        print("G3_REPO_WRITES=NONE")
        return 0
    except (OSError, ValueError, KeyError, AssertionError, RuntimeError, subprocess.CalledProcessError,
            subprocess.TimeoutExpired, zipfile.BadZipFile) as error:
        print("G3=BLOCKED: " + str(error), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
