#!/usr/bin/env python3.11
"""Prueba adversarial XOR-3 para F6-v2: sólo datos artificiales, jamás evidencia del paper."""
import argparse
import hashlib
import json
import pathlib
import subprocess
import sys


def require(ok, msg):
    if not ok:
        raise RuntimeError(msg)


def create_manifest(root, assignments):
    run_dir = root / "runs"
    run_dir.mkdir(parents=True, exist_ok=True)
    lines = ["[experiment]", "schema_version = 1", "dimension = 3",
             'oracle = "byte_stdout_vs_baseline_v1"', "confirmations = 2"]
    for a in assignments:
        truth = (a.bit_count() % 2) == 1
        content = b"different\n" if truth else b"same\n"
        for r in (1, 2):
            name = f"runs/{a:03b}-{r}.stdout"
            (root / name).write_bytes(content)
            lines += ["", "[[run]]", f'assignment = "{a:03b}"',
                      f'run_id = "xor_{a:03b}_r{r}"', 'status = "ok"',
                      f'stdout_file = "{name}"',
                      f'stdout_sha256 = "{hashlib.sha256(content).hexdigest()}"']
    manifest = root / "manifest.toml"
    manifest.write_text("\n".join(lines) + "\n", encoding="utf-8")
    return hashlib.sha256(manifest.read_bytes()).hexdigest()


def evaluate(binary, root, subset):
    root.mkdir(parents=True, exist_ok=False)
    digest = create_manifest(root, subset)
    out = root / "rust_report.json"
    args = [str(binary), "evaluate", "--manifest", str(root / "manifest.toml"),
            "--expected-manifest-sha256", digest, "--output", str(out)]
    completed = subprocess.run(args, text=True, capture_output=True, timeout=30)
    require(completed.returncode == 0 and out.is_file(),
            "Rust rechazó la prueba XOR: " + completed.stderr)
    return json.loads(out.read_text(encoding="utf-8"))


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--rust-binary", required=True, type=pathlib.Path)
    ap.add_argument("--output", required=True, type=pathlib.Path)
    opts = ap.parse_args()
    require(not opts.output.exists(), "Salida existente: no sobrescribir")
    require(opts.rust_binary.is_file(), "Falta el binario Rust")
    opts.output.mkdir(parents=True)
    early = evaluate(opts.rust_binary, opts.output / "early", [0, 1])
    disputed = [c for c in early["cells"] if c["assignment"] == "010"]
    require(len(disputed) == 1 and disputed[0]["status"] == "model_supported"
            and disputed[0]["label"] is False,
            "La prueba no produjo la inferencia condicional falsa esperada")
    require(not early["model_inconsistent"], "El modelo falló antes de la inferencia")
    full = evaluate(opts.rust_binary, opts.output / "full", list(range(8)))
    require(full["model_inconsistent"] and full["inferred_under_model"] == 0,
            "El modelo no detectó el XOR incompatible tras observar todo")
    data = {
        "status": "SYNTHETIC_ADVERSARIAL_TEST_NOT_PAPER_EVIDENCE",
        "early_query_count": 2, "early_wrong_conditional_prediction": "010 -> false (real XOR -> true)",
        "early_model_inconsistent": False,
        "final_model_inconsistent": True, "full_observation_count": 8,
        "warning": "La inconsistencia posterior NO revoca automáticamente una inferencia falsa ya emitida."
    }
    (opts.output / "M3_STRESS.json").write_text(json.dumps(data, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print("M3_OOD_EARLY_WRONG=OBSERVED")
    print("M3_OOD_INCONSISTENCY=DETECTED")
    print("M3_STRESS=PASS")
    print("M3_SCIENTIFIC_EVIDENCE=NONE_SYNTHETIC_ADVERSARIAL_ONLY")


if __name__ == "__main__":
    try:
        main()
    except Exception as exc:
        print("M3_STRESS_BLOCKED=" + str(exc), file=sys.stderr)
        sys.exit(2)
