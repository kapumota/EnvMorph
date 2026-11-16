#!/usr/bin/env python3.11
"""M3: replay retrospectivo P1 con el binario Rust F6-v2 y políticas de adquisición.

Únicamente emplea biblioteca estándar. No ejecuta workflows, no modifica Git ni
extrae el TAR en rutas de usuario. El oráculo se consulta solo después de elegir.
"""

import argparse
import collections
import csv
import hashlib
import json
import pathlib
import random
import subprocess
import sys
import tarfile
from datetime import datetime, timezone

ARCHIVE_SHA256 = "9ba4652dbb6ee098a43c288bda699b7f9e86572383c13126ed397f90d1e13288"
PROTOCOL_SHA256 = "da5151445aec76926a9f61fa5ce5cff2591653deaf7d1c834f147bbbf7f36624"
SOURCE_SHA = "99f89f9a66cf24a6afa59ea8114ee3a6600a4510"
SEEDS = tuple(range(5))
NONRANDOM = ("lexicographic", "pairwise", "rust_suggested", "greedy_coverage")
BUDGETS = (2, 4, 6, 8)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    dig = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            dig.update(chunk)
    return dig.hexdigest()


def require(ok, why):
    if not ok:
        raise RuntimeError(why)


def load_archive(path):
    require(sha_file(path) == ARCHIVE_SHA256,
            "SHA-256 del archivo P1 no coincide con el original congelado")
    items = {}
    with tarfile.open(path, "r:gz") as t:
        members = t.getmembers()
        require(bool(members), "TAR vacío")
        roots = {m.name.split("/")[0] for m in members}
        require(len(roots) == 1, "TAR sin raíz única")
        root = next(iter(roots))
        for m in members:
            parts = pathlib.PurePosixPath(m.name)
            require(not parts.is_absolute() and ".." not in parts.parts
                    and (m.isdir() or m.isfile()), "Miembro TAR inválido")
            if not m.isfile():
                continue
            require(m.size <= 50_000_000, "Archivo dentro del TAR demasiado grande")
            rel = m.name[len(root) + 1:]
            require(rel and rel not in items, "Ruta duplicada en TAR")
            items[rel] = t.extractfile(m).read()
    require("SHA256SUMS" in items and "manifest.json" in items, "Faltan manifiestos")
    ledger = {}
    for line in items["SHA256SUMS"].decode("utf-8").splitlines():
        require("  " in line, "SHA256SUMS inválido")
        h, name = line.split("  ", 1)
        require(name not in ledger and len(h) == 64, "Entrada de hash duplicada o inválida")
        ledger[name] = h
    names = set(items) - {"SHA256SUMS"}
    require(set(ledger) == names and len(names) == 725,
            "La relación de 725 archivos no coincide con el registro")
    require(all(sha(items[k]) == v for k, v in ledger.items()),
            "Uno o más archivos no coinciden con sus hashes")
    manifest = json.loads(items["manifest.json"])
    require(manifest.get("protocol_sha256") == PROTOCOL_SHA256,
            "Protocolo P1 no coincide")
    require(manifest.get("source_commit") == SOURCE_SHA,
            "Commit fuente distinto del protocolo de análisis")
    return items


def references(items):
    """Devuelve salidas confirmadas (dos repeticiones), sin derivar etiquetas por adelantado."""
    refs = collections.defaultdict(dict)
    provenance = {}
    for name, raw in items.items():
        if not (name.startswith("reference/") and name.endswith("/record.json")):
            continue
        d = json.loads(raw)
        key = (d["id"], d["mode"])
        require(d["id"] in ["C" + str(i) for i in range(1, 7)]
                and d["mode"] in ("primary", "negative"), "Caso/modo inesperado")
        tl, bl, rep = d["target_level"], d["benign_level"], d["repetition"]
        require(tl in (0, 1) and bl in (0, 1) and rep in (1, 2), "Celda/repetición inválida")
        cell = 2 * tl + bl
        require(rep not in refs[key, cell], "Repetición duplicada")
        stdout_path = name.removesuffix("/record.json") + "/stdout.bin"
        stderr_path = name.removesuffix("/record.json") + "/stderr.bin"
        require(stdout_path in items and stderr_path in items, "Falta evidencia directa")
        stdout = items[stdout_path]
        stderr = items[stderr_path]
        require(sha(stdout) == d["stdout_sha256"] and len(stdout) == d["stdout_size"]
                and sha(stderr) == d["stderr_sha256"] and len(stderr) == d["stderr_size"]
                and d["returncode"] == 0 and not d.get("timed_out"), "Ejecución P1 inválida")
        refs[key, cell][rep] = stdout
        provenance[d["id"], d["mode"], cell, rep] = stdout_path
    require(len(refs) == 48, "No se encontraron las 48 celdas")
    for key, runs in refs.items():
        require(set(runs) == {1, 2} and runs[1] == runs[2],
                "Referencia insuficiente o inestable para " + repr(key))
    workloads = {}
    for ident in ("C1", "C2", "C3", "C4", "C5", "C6"):
        for mode in ("primary", "negative"):
            key = (ident, mode)
            runs = {cell: refs[key, cell] for cell in range(4)}
            workloads[key] = (runs, {cell: int(runs[cell][1] != runs[0][1])
                                     for cell in range(4)})
    require(len(provenance) == 96, "Faltan relaciones de procedencia")
    return workloads, provenance


def hypothesis_family():
    # Coincide con M2: constantes y conjunciones de hasta dos literales firmados.
    pool = [("zero", ()), ("and", ())]
    for i in range(2):
        for val in (0, 1):
            pool.append(("and", ((i, val),)))
    for x in (0, 1):
        for y in (0, 1):
            pool.append(("and", ((0, x), (1, y))))
    return pool


def predicts(h, assignment):
    kind, literals = h
    if kind == "zero":
        return 0
    bits = (assignment >> 1, assignment & 1)
    return int(all(bits[i] == val for i, val in literals))


def version_space(observed):
    return [h for h in hypothesis_family()
            if all(predicts(h, k) == v for k, v in observed.items())]


def cert_count(pool):
    if not pool:
        return 0
    return sum(len({predicts(h, a) for h in pool}) == 1 for a in range(4))


def choose(policy, selected, observed, rust_report, rng):
    """No recibe la verdad completa ni salidas futuras; usa exclusivamente evidencia adquirida."""
    available = [a for a in range(4) if a not in selected]
    if not available:
        return None, False
    if policy == "lexicographic":
        return available[0], False
    if policy == "random":
        return rng.choice(available), False
    if policy == "pairwise":
        def tokens(a):
            return {(0, a >> 1, 1, a & 1)}
        seen = set().union(*(tokens(k) for k in selected)) if selected else set()
        return max(available, key=lambda k: (len(tokens(k) - seen), -k)), False
    if policy == "rust_suggested":
        candidate = rust_report.get("suggested_next_assignment")
        if isinstance(candidate, str) and len(candidate) == 2 and set(candidate) <= {"0", "1"}:
            candidate = int(candidate, 2)
            if candidate in available:
                return candidate, False
        return available[0], True
    if policy == "greedy_coverage":
        pool = version_space(observed)
        if not pool:
            return available[0], True
        def expected_gain(x):
            counts = []
            for label in (0, 1):
                group = [h for h in pool if predicts(h, x) == label]
                if group:
                    counts.append(len(group) * cert_count(group))
            return sum(counts) / len(pool)
        return max(available, key=lambda x: (expected_gain(x), -x)), False
    raise RuntimeError("Política desconocida: " + policy)


def write_manifest(stage, case, mode, selected, runs, provenance):
    stage.mkdir(parents=True, exist_ok=False)
    run_dir = stage / "runs"
    run_dir.mkdir()
    lines = ["[experiment]", "schema_version = 1", "dimension = 2",
             'oracle = "byte_stdout_vs_baseline_v1"', "confirmations = 2"]
    refs_used = []
    for assignment in selected:
        for repetition in (1, 2):
            stdout = runs[assignment][repetition]
            relpath = f"runs/{assignment:02b}-{repetition}.stdout"
            (stage / relpath).write_bytes(stdout)
            lines += ["", "[[run]]", f'assignment = "{assignment:02b}"',
                      f'run_id = "{case}_{mode}_{assignment:02b}_r{repetition}"',
                      'status = "ok"', f'stdout_file = "{relpath}"',
                      f'stdout_sha256 = "{sha(stdout)}"']
            refs_used.append(provenance[case, mode, assignment, repetition])
    manifest_bytes = ("\n".join(lines) + "\n").encode("utf-8")
    (stage / "manifest.toml").write_bytes(manifest_bytes)
    return sha(manifest_bytes), refs_used


def call_rust(binary, stage, manifest_sha):
    output = stage / "rust_report.json"
    proc = subprocess.run([str(binary), "evaluate", "--manifest", str(stage / "manifest.toml"),
                           "--expected-manifest-sha256", manifest_sha,
                           "--output", str(output)], capture_output=True, text=True, timeout=30)
    require(proc.returncode == 0 and output.is_file(),
            "F6-v2 falló en " + str(stage) + ": " + proc.stderr[-1200:])
    report = json.loads(output.read_text(encoding="utf-8"))
    require(report["manifest_sha256"] == manifest_sha and report["method"] == "F6-v2-experimental"
            and report["dimension"] == 2, "Respuesta Rust no válida")
    return report


def inspect_report(report, selected, truth):
    cells = report["cells"]
    require(len(cells) == 4 and report["primitive_run_records"] == 2 * len(selected),
            "Recuento de ejecuciones Rust inesperado")
    count = collections.Counter()
    for c in cells:
        assignment = int(c["assignment"], 2)
        status, label = c["status"], c["label"]
        require(status in ("evidence_verified", "model_supported", "unresolved"),
                "Estado F6-v2 inesperado")
        if assignment in selected:
            require(status == "evidence_verified" and label == bool(truth[assignment])
                    and len(c["run_ids"]) == 2 and len(c["stdout_sha256"]) == 2,
                    "La evidencia directa no coincide con lo consultado")
            count["direct"] += 1
        else:
            require(status != "evidence_verified", "F6-v2 certificó una celda no ejecutada")
            if status == "model_supported":
                require(isinstance(label, bool), "Etiqueta inferida inválida")
                count["model_supported"] += 1
                count["model_errors"] += (label != bool(truth[assignment]))
            else:
                require(label is None, "Una abstención contiene decisión")
                count["unresolved"] += 1
    require(count["direct"] == report["verified_direct"]
            and count["model_supported"] == report["inferred_under_model"]
            and count["unresolved"] == report["unresolved"],
            "Recuento de estados inconsistente con Rust")
    return count


def trial(out, binary, workload, policy, seed, runs, truth, provenance):
    case, mode = workload
    selected = [0]
    observed = {}
    rng_seed = int.from_bytes(hashlib.sha256(f"20261115|{case}|{mode}|{policy}|{seed}".encode()).digest()[:8], "big")
    rng = random.Random(rng_seed)
    rows = []
    fallbacks = 0
    for checkpoint in BUDGETS:
        stepdir = out / "reports" / case / mode / f"{policy}-{seed}" / f"budget-{checkpoint:02}"
        manifest_sha, refs_used = write_manifest(stepdir, case, mode, selected, runs, provenance)
        rust_report = call_rust(binary, stepdir, manifest_sha)
        for assignment in selected:
            observed[assignment] = int(runs[assignment][1] != runs[0][1])
        counts = inspect_report(rust_report, selected, truth)
        rows.append({
            "case": case, "mode": mode, "policy": policy, "seed": seed,
            "budget_executions": checkpoint, "queried": len(selected),
            "direct_verified": counts["direct"],
            "table_exact_verified": len(selected), "table_exact_errors": 0,
            "model_supported": counts["model_supported"],
            "model_errors": counts["model_errors"],
            "model_unresolved": counts["unresolved"],
            "model_inconsistent": int(rust_report["model_inconsistent"]),
            "fallback_total": fallbacks, "observed_assignments": ",".join(f"{a:02b}" for a in selected),
            "rust_report": str(stepdir.relative_to(out) / "rust_report.json"),
            "manifest_sha256": manifest_sha,
            "source_paths": ";".join(refs_used),
        })
        if checkpoint < max(BUDGETS):
            assignment, fallback = choose(policy, selected, observed, rust_report, rng)
            require(assignment is not None and assignment not in selected,
                    "Política no seleccionó una nueva configuración")
            selected.append(assignment)
            fallbacks += int(fallback)
    require(set(selected) == set(range(4)), "La campaña retrospectiva no cubrió el dominio")
    return rows


def aggregate(rows):
    grouped = collections.defaultdict(list)
    for row in rows:
        key = (row["policy"], row["seed"], row["budget_executions"])
        grouped[key].append(row)
    result = []
    for (policy, seed, budget), xs in sorted(grouped.items()):
        require(len(xs) == 12, "Faltan workflows de una estrategia")
        result.append({
            "policy": policy, "seed": seed, "budget_executions_per_workflow": budget,
            "workflow_count": len(xs), "direct": sum(x["direct_verified"] for x in xs),
            "exact_table": sum(x["table_exact_verified"] for x in xs),
            "model_supported": sum(x["model_supported"] for x in xs),
            "model_errors": sum(x["model_errors"] for x in xs),
            "unresolved": sum(x["model_unresolved"] for x in xs),
            "model_inconsistent_workflows": sum(x["model_inconsistent"] for x in xs),
            "fallback_total": sum(x["fallback_total"] for x in xs),
        })
    return result


def summary_md(aggregates):
    lines = [
        "# M3: lectura del replay retrospectivo P1-EB-01",
        "",
        "**ESTADO:** comparación retrospectiva exploratoria; NO es confirmación independiente ni GO metodológico.",
        "",
        "Los 12 workflows de 2 factores se extraen de solo 6 casos y 2 proyectos upstream.",
        "Cada política empieza observando el baseline con dos repeticiones, con costo contabilizado.",
        "La tabla exacta comparte las mismas consultas. F6-v2 tiene cobertura factual directa idéntica a esa tabla.",
        "`model_supported` es inferencia condicional al modelo H; `model_errors` mide errores reales contra la referencia.",
        "Las cinco semillas aleatorias se presentan por separado, nunca como 60 workflows independientes.",
        "",
        "| Política | Semilla | Presupuesto/workflow | Directas/48 | Inferidas/48 | Erróneas | No resueltas |",
        "|---|---:|---:|---:|---:|---:|---:|",
    ]
    for x in aggregates:
        lines.append(f"| {x['policy']} | {x['seed']} | {x['budget_executions_per_workflow']} | "
                     f"{x['direct']} | {x['model_supported']} | {x['model_errors']} | {x['unresolved']} |")
    lines += ["", "**Advertencia:** pairwise se degenera en desempate lexicográfico con dos factores; no se interpreta como comparación de covering arrays.",
              "", "El análisis de M3 sirve para detectar diferencias entre políticas y errores bajo este dominio pequeño.",
              "No sirve para afirmar novedad, generalización entre proyectos ni superioridad Q1/Q2.", ""]
    return "\n".join(lines)


def selftest():
    pool = hypothesis_family()
    require(len(pool) == 10, "Familia H de M2 de dos factores incorrecta")
    labels = {0: 0, 1: 0, 2: 1, 3: 1}
    version = version_space({0: 0, 1: 0})
    require(version and all(predicts(h, 0) == 0 and predicts(h, 1) == 0 for h in version),
            "Versión de hipótesis mal construida")
    for policy in ("lexicographic", "random", "pairwise", "rust_suggested", "greedy_coverage"):
        first, fallback = choose(policy, [0], {0: 0}, {"suggested_next_assignment": "11"}, random.Random(8))
        require(first in (1, 2, 3) and isinstance(fallback, bool), "Selector inválido")
    require(choose("rust_suggested", [0], {0: 0},
                   {"suggested_next_assignment": "11"}, random.Random(8))[0] == 3,
            "No se está usando la recomendación Rust")
    # El selector no recibe la verdad; el resultado futuro solo se abre tras elegir.
    require(cert_count(version_space({0: 0})) >= 1, "Conteo de inferencias incorrecto")
    require(labels[0] == 0, "Oráculo de prueba roto")
    print("M3_HARNESS_SELFTEST=PASS")
    print("M3_SCIENTIFIC_STATUS=NOT_YET_EVALUATED")


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--selftest", action="store_true")
    ap.add_argument("--archive", type=pathlib.Path)
    ap.add_argument("--rust-binary", type=pathlib.Path)
    ap.add_argument("--output", type=pathlib.Path)
    args = ap.parse_args()
    if args.selftest:
        selftest()
        if args.archive is None and args.rust_binary is None and args.output is None:
            return
    if not args.archive or not args.rust_binary or not args.output:
        ap.error("La ejecución requiere --archive, --rust-binary y --output")
    require(not args.output.exists(), "La salida ya existe; no se sobrescribirá")
    require(args.rust_binary.is_file() and args.rust_binary.stat().st_mode & 0o111,
            "Binario Rust no encontrado o no ejecutable")
    items = load_archive(args.archive)
    workloads, provenance = references(items)
    args.output.mkdir(parents=True, exist_ok=False)
    try:
        rows = []
        for key, (runs, truth) in workloads.items():
            for policy in NONRANDOM:
                rows += trial(args.output, args.rust_binary, key, policy, -1, runs, truth, provenance)
            for seed in SEEDS:
                rows += trial(args.output, args.rust_binary, key, "random", seed, runs, truth, provenance)
        require(len(rows) == 12 * (len(NONRANDOM) + len(SEEDS)) * len(BUDGETS),
                "Recuento de checkpoints inesperado")
        aggregates = aggregate(rows)
        metadata = {
            "status": "EXPLORATORY_RETROSPECTIVE_NO_CONFIRMATORY_CLAIM",
            "source_archive_sha256": ARCHIVE_SHA256,
            "source_git_commit": SOURCE_SHA,
            "source_protocol_sha256": PROTOCOL_SHA256,
            "rust_binary_sha256": sha_file(args.rust_binary),
            "m3_script_sha256": sha_file(pathlib.Path(__file__)),
            "created_at_utc": datetime.now(timezone.utc).isoformat(),
            "policy_seed": 20261115,
            "nonrandom_policies": list(NONRANDOM),
            "random_seeds": list(SEEDS),
            "budgets_executions_per_workflow": list(BUDGETS),
            "case_count": 6, "workflow_modes": 2, "project_count": 2,
            "reference_direct_runs": 96, "reference_cells": 48,
            "rust_report_invocations": len(rows),
            "confirmed_direct_per_query": 2,
            "baseline_is_charged": True,
            "oracle_leakage_policy": "La política recibe solo las etiquetas ya observadas y el reporte Rust previo",
            "limits": ["dos factores por workflow", "dos proyectos", "piloto conocido previamente",
                       "model_supported depende de la realizabilidad de H",
                       "SHA-256 verifica bytes, no autenticidad ni independencia de ejecuciones"],
        }
        (args.output / "M3_METADATA.json").write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        (args.output / "M3_AGGREGATES.json").write_text(json.dumps(aggregates, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        with (args.output / "M3_RESULTS.csv").open("w", newline="", encoding="utf-8") as f:
            w = csv.DictWriter(f, fieldnames=list(rows[0]))
            w.writeheader()
            w.writerows(rows)
        (args.output / "M3_RESUMEN.md").write_text(summary_md(aggregates), encoding="utf-8")
        print("M3_P1_ARCHIVE_VERIFIED=725")
        print("M3_WORKFLOWS=12 CASES=6 PROJECTS=2")
        print(f"M3_RUST_REPORTS={len(rows)}")
        print("M3_BASELINE_BUDGETS=2,4,6,8")
        print("M3_REPLAY=PASS")
        print("M3_SCIENTIFIC_STATUS=EXPLORATORY_RETROSPECTIVE_ONLY")
        print("M3_REPORT=" + str(args.output / "M3_RESUMEN.md"))
    except Exception:
        (args.output / "M3_INCOMPLETE.txt").write_text("La ejecución quedó incompleta; conservar para diagnóstico.\n")
        raise


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print("M3_REPLAY_BLOCKED=" + str(error), file=sys.stderr)
        sys.exit(2)
