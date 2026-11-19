#!/usr/bin/env python3
"""Harness G3-C de EnvMorph (borrador v0.1). Solo biblioteca estandar.

Aplica los factores al proceso de la herramienta evaluada, ejecuta los pasos
auxiliares con entorno fijo, registra evidencia por ejecucion y hace cumplir el
orden del protocolo (congelamiento antes de S, sellado antes de H).

Subcomandos
  check-install   verifica paquetes, locales y zonas (sin ejecutar la herramienta)
  pin-fixture     construye el fixture y calcula su digest de arbol
  check-env       verifica la aplicacion de factores sin ejecutar la herramienta
  preflight       ejecuta SOLO la celda baseline dos veces
  run-cell        ejecuta una celda con confirmaciones (requiere guardias)
  summarize       resume las celdas ya ejecutadas
"""
import argparse, datetime, hashlib, json, os, pathlib, platform, shutil, stat as S, subprocess, sys, time, tempfile
import g3c_state as G

ROOT = pathlib.Path(__file__).resolve().parent.parent
MAX_STDOUT = 1_048_576


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(p):
    h = hashlib.sha256()
    with open(p, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()


def die(msg, code=2):
    print("G3C_ERROR=" + msg, file=sys.stderr)
    sys.exit(code)


def load_json(p):
    return json.loads(pathlib.Path(p).read_text(encoding="utf-8"))


def load_factors(root=ROOT):
    return load_json(root / "factors.json")


def load_spec(name, root=ROOT):
    p = root / "workflows" / name / "spec.json"
    if not p.is_file():
        die("workflow inexistente " + name)
    return load_json(p)


def parse_cell(cell, fac):
    """Devuelve (bits, null) para una celda 'NNN' o 'NULLA'/'NULLB'."""
    if cell in fac["null_cells"]:
        return "000", ("a" if cell == "NULLA" else "b")
    n = len(fac["factors"])
    if len(cell) != n or any(c not in "01" for c in cell):
        die("celda invalida " + cell)
    return cell, None


def subst(v, ws):
    return v.replace("{ws}", str(ws))


def build_tool_env(fac, cell, ws):
    bits, null = parse_cell(cell, fac)
    env = {k: subst(v, ws) for k, v in fac["tool_base_env"].items()}
    for bit, f in zip(bits, fac["factors"]):
        env[f["env"]] = subst(f["levels"][int(bit)], ws)
    if null is not None:
        nf = fac["null_factor"]
        env[nf["env"]] = nf["levels"][0 if null == "a" else 1]
    return env


def build_aux_env(fac, ws):
    return {k: subst(v, ws) for k, v in fac["aux_env"].items()}


def tree_digest(root):
    """Digest canonico del arbol del fixture (nombres en bytes, modo, tamano, mtime de archivos, contenido)."""
    root = os.fsencode(str(root))
    lines = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames.sort()
        for name in sorted(dirnames + filenames):
            full = os.path.join(dirpath, name)
            rel = os.path.relpath(full, root)
            st = os.lstat(full)
            if S.S_ISLNK(st.st_mode):
                die("el fixture contiene un enlace simbolico")
            if S.S_ISDIR(st.st_mode):
                lines.append("d %o %s" % (S.S_IMODE(st.st_mode), rel.hex()))
            elif S.S_ISREG(st.st_mode):
                lines.append("f %o %d %d %s %s" % (S.S_IMODE(st.st_mode), st.st_size, int(st.st_mtime),
                                                   sha_file(full), rel.hex()))
            else:
                die("el fixture contiene un archivo especial")
    return sha_bytes(("\n".join(lines) + "\n").encode("ascii"))


def prepare_workspace(spec, fac, work_root, root=ROOT):
    ws = pathlib.Path(work_root).resolve() / "ws"
    if ws.exists():
        shutil.rmtree(ws)
    for sub in ("home", "tmp/a", "tmp/b", "fx"):
        (ws / sub).mkdir(parents=True)
    os.umask(0o022)
    script = spec.get("fixture_script")
    digest = None
    if script:
        sp = root / "workflows" / spec["name"] / script
        if not sp.is_file():
            die("falta el script de fixture " + str(sp))
        env = build_aux_env(fac, ws)
        proc = subprocess.run(["/bin/sh", str(sp), str(ws / "fx")], env=env, cwd=ws, capture_output=True, timeout=120)
        if proc.returncode != 0:
            die("fallo la construccion del fixture: " + proc.stderr.decode("utf-8", "replace")[-400:])
        digest = tree_digest(ws / "fx")
    return ws, digest


def probe_env(env):
    """Evidencia de aplicacion. Lanza /usr/bin/env -0 con el mismo diccionario y compara."""
    proc = subprocess.run(["/usr/bin/env", "-0"], env=env, capture_output=True, timeout=10)
    seen = {}
    for item in proc.stdout.split(b"\0"):
        if item:
            k, _, v = item.partition(b"=")
            seen[k.decode()] = v.decode()
    return seen == env, sha_bytes(proc.stdout), seen


def resolve(argv0, env):
    if os.path.isabs(argv0):
        return argv0
    found = shutil.which(argv0, path=env.get("PATH", "/usr/bin:/bin"))
    if not found:
        die("no se encuentra el ejecutable " + argv0)
    return found


def run_pipeline(spec, fac, cell, ws):
    tool_env = build_tool_env(fac, cell, ws)
    aux_env = build_aux_env(fac, ws)
    applied, probe_sha, _ = probe_env(tool_env)
    outs, steps = {}, []
    ok = True
    for st in spec["steps"]:
        env = tool_env if st["role"] == "tool" else aux_env
        argv = [subst(a, ws) for a in st["argv"]]
        exe = resolve(argv[0], env)
        stdin = None
        if st.get("stdin_from"):
            stdin = outs[st["stdin_from"]]
        elif st.get("stdin_file"):
            stdin = (ws / st["stdin_file"]).read_bytes()
        t0 = time.monotonic()
        try:
            proc = subprocess.run([exe] + argv[1:], input=stdin, env=env, cwd=ws, capture_output=True,
                                  timeout=spec.get("timeout_s", 300))
            rc, out, err, to = proc.returncode, proc.stdout, proc.stderr, False
        except subprocess.TimeoutExpired as e:
            rc, out, err, to = None, (e.stdout or b""), (e.stderr or b""), True
        dur = time.monotonic() - t0
        if len(out) > MAX_STDOUT:
            ok = False
        allowed = spec.get("tool_success_exit", [0]) if st["role"] == "tool" else [0]
        good = (not to) and (rc in allowed)
        ok = ok and good
        outs[st["id"]] = out
        steps.append({"id": st["id"], "role": st["role"], "exe": exe, "argv": argv, "rc": rc, "timeout": to,
                      "duration_s": round(dur, 4), "stdout_sha256": sha_bytes(out), "stdout_size": len(out),
                      "stderr_sha256": sha_bytes(err), "stderr_size": len(err),
                      "stderr_tail": err[-300:].decode("utf-8", "replace")})
        if not good:
            break
    oracle = outs.get(spec["oracle_step"], b"")
    return {"applied": applied, "env_probe_sha256": probe_sha, "tool_env": tool_env, "steps": steps,
            "ok": ok and spec["oracle_step"] in outs, "oracle": oracle}


def tool_identity(spec):
    if spec.get("kind") == "selftest":
        return {"kind": "selftest"}
    b = spec["tool_binary"]
    ident = {"binary": b, "binary_sha256": sha_file(b) if os.path.isfile(b) else None}
    q = subprocess.run(["dpkg-query", "-W", "-f=${Version}", spec["package"]], capture_output=True, text=True)
    ident["package_version"] = q.stdout.strip() or None
    ident["package_version_ok"] = (ident["package_version"] == spec["version"])
    return ident


def host_info():
    return {"uname": " ".join(platform.uname()), "python": platform.python_version()}


def write_run(results, spec, cell, confirmation, attempt, run, fixture_digest, fixture_ok, stage):
    d = pathlib.Path(results) / spec["name"]
    d.mkdir(parents=True, exist_ok=True)
    stem = "%s_c%d_a%d" % (cell, confirmation, attempt)
    G.exclusive_write(d / (stem + ".stdout"),run["oracle"])
    rec = {
        "schema": "g3c-run-v1", "workflow": spec["name"], "cell": cell, "stage": stage,
        "confirmation": confirmation, "attempt": attempt,
        "utc": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "applied_env_ok": run["applied"], "env_probe_sha256": run["env_probe_sha256"], "tool_env": run["tool_env"],
        "ok": run["ok"], "oracle_stdout_sha256": sha_bytes(run["oracle"]), "oracle_size": len(run["oracle"]),
        "steps": run["steps"], "fixture_tree_sha256": fixture_digest, "fixture_ok": fixture_ok,
        "tool": tool_identity(spec), "host": host_info(),
        "spec_sha256": sha_file(ROOT / "workflows" / spec["name"] / "spec.json"),
        "harness_sha256": sha_file(pathlib.Path(__file__)),
    }
    G.exclusive_write(d / (stem + ".json"),(json.dumps(rec, indent=1, sort_keys=True) + "\n").encode("utf-8"))
    return rec


def execute(spec, fac, cell, confirmations, results, work_root, stage, attempt=1):
    recs = []
    if confirmations != 2:
        G.fail("no se permite alterar las dos confirmaciones")
    for k in range(1, confirmations + 1):
        with tempfile.TemporaryDirectory(prefix="g3c-",dir=work_root if pathlib.Path(work_root).is_dir() else None) as fresh:
            ws, digest = prepare_workspace(spec, fac, fresh)
            fixture_ok = (spec.get("fixture_script") is None) or (digest == spec.get("fixture_tree_sha256"))
            run = run_pipeline(spec, fac, cell, ws)
            if not fixture_ok:
                run["ok"] = False
            recs.append(write_run(results, spec, cell, k, attempt, run, digest, fixture_ok, stage))
    return recs


def campaign_guard(spec, fac, cell, campaign, results=None):
    if spec.get("kind") == "selftest":
        return
    f = G.require_freeze(campaign, ROOT, spec["name"])
    if cell in fac["h_cells"]:
        G.require_f10_start(campaign,f)
        G.require_heldout_authorization(campaign)
        if results is not None:
            f=G.require_freeze(campaign,ROOT,spec["name"])
            seal=G.json_read(pathlib.Path(campaign)/"SEALED.json")
            if G.evidence_snapshot(results,f["workflows"])!=seal.get("evidence_sha256"):
                G.fail("evidencia S/N cambió después de sellar")
    else:
        G.ensure_not_sealed(campaign)
        G.require_f10_start(campaign,f)


def workflow_names(include_selftest=False):
    return sorted(p.name for p in (ROOT / "workflows").iterdir()
                  if p.is_dir() and (include_selftest or not p.name.startswith("_")))


def cmd_check_install(a):
    fac = load_factors()
    names = [a.workflow] if a.workflow else workflow_names()
    bad = 0
    loc = [x.lower().replace("-", "") for x in subprocess.run(["locale", "-a"], capture_output=True, text=True).stdout.split()]
    for lv in fac["factors"][1]["levels"]:
        have = lv == "C" or lv.lower().replace("-", "") in loc
        print("locale %s %s" % (lv, "OK" if have else "FALTA"))
        bad += (not have)
    for lv in fac["factors"][0]["levels"]:
        ok = lv == "UTC" or os.path.isfile("/usr/share/zoneinfo/" + lv)
        print("zona %s %s" % (lv, "OK" if ok else "FALTA"))
        bad += (not ok)
    tz = subprocess.run(["dpkg-query", "-W", "-f=${Version}", "tzdata"], capture_output=True, text=True).stdout.strip()
    pin = fac["dependencias_fijadas"]["tzdata"]
    print("tzdata %s (fijada %s) %s" % (tz, pin, "OK" if tz == pin else "DIFIERE"))
    bad += (tz != pin)
    for n in names:
        s = load_spec(n)
        i = tool_identity(s)
        ok = i.get("package_version_ok") and i.get("binary_sha256")
        print("%s paquete=%s fijada=%s %s" % (n, i.get("package_version"), s["version"], "OK" if ok else "FALLA"))
        bad += (not ok)
    print("G3C_INSTALL=" + ("PASS" if bad == 0 else "FAIL"))
    return 0 if bad == 0 else 1


def cmd_pin_fixture(a):
    fac = load_factors()
    spec = load_spec(a.workflow)
    if not spec.get("fixture_script"):
        print(a.workflow, "sin fixture")
        return 0
    with tempfile.TemporaryDirectory(prefix="g3c-pin-") as t:
        ws, d1 = prepare_workspace(spec, fac, t)
        ws, d2 = prepare_workspace(spec, fac, t)
    print(a.workflow, "fixture_tree_sha256 =", d1, "(reconstruccion identica)" if d1 == d2 else "(NO DETERMINISTA)")
    if d1 != d2:
        return 1
    if a.write:
        p = ROOT / "workflows" / a.workflow / "spec.json"
        s = load_json(p)
        s["fixture_tree_sha256"] = d1
        p.write_text(json.dumps(s, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return 0


def all_cells(fac):
    return fac["s_cells"] + fac["h_cells"] + fac["null_cells"]


def cmd_check_env(a):
    fac = load_factors()
    bad = 0
    with tempfile.TemporaryDirectory(prefix="g3c-env-") as t:
        ws = pathlib.Path(t) / "ws"
        for sub in ("home", "tmp/a", "tmp/b"):
            (ws / sub).mkdir(parents=True)
        for cell in all_cells(fac):
            env = build_tool_env(fac, cell, ws)
            applied, sha, seen = probe_env(env)
            ok = applied and pathlib.Path(env["TMPDIR"]).is_dir()
            bad += (not ok)
            if a.verbose:
                print(cell, "OK" if ok else "FALLA", {k: env[k] for k in ("TZ", "LC_ALL", "TMPDIR")})
    print("celdas verificadas =", len(all_cells(fac)))
    print("G3C_CHECK_ENV=" + ("PASS" if bad == 0 else "FAIL"))
    return 0 if bad == 0 else 1


def cmd_preflight(a):
    fac = load_factors()
    spec = load_spec(a.workflow)
    for k in (1,2):
        if (pathlib.Path(a.results)/"preflight"/spec["name"]/(f"000_c{k}_a1.json")).exists():
            G.fail("preflight existente, no sobrescribir")
    if spec.get("kind") != "selftest":
        G.reserve(a.campaign,spec["name"],"000","preflight",2)
    recs = execute(spec, fac, "000", 2, pathlib.Path(a.results)/"preflight", a.work_root, "preflight")
    stable = len({r["oracle_stdout_sha256"] for r in recs}) == 1
    ok = all(r["ok"] and r["applied_env_ok"] and r["fixture_ok"] for r in recs) and stable
    why = []
    if not all(r["ok"] for r in recs):
        why.append("ejecucion fallida")
    if not all(r["fixture_ok"] for r in recs):
        why.append("digest de fixture distinto del fijado")
    if not stable:
        why.append("baseline inestable")
    print("%s PREFLIGHT=%s %s" % (a.workflow, "PASS" if ok else "FAIL", "; ".join(why)))
    return 0 if ok else 1


def cmd_run_cell(a):
    fac = load_factors()
    spec = load_spec(a.workflow)
    if a.cell not in all_cells(fac):
        die("celda fuera del reticulo " + a.cell)
    campaign_guard(spec, fac, a.cell, a.campaign, a.results)
    stage = "H" if a.cell in fac["h_cells"] else ("N" if a.cell in fac["null_cells"] else "S")
    if spec.get("kind") != "selftest":
        G.reserve(a.campaign,spec["name"],a.cell,stage,a.confirmations,a.attempt)
    for k in (1,2):
        if (pathlib.Path(a.results)/spec["name"]/(f"{a.cell}_c{k}_a{a.attempt}.json")).exists():
            G.fail("resultados existentes, no sobrescribir")
    recs = execute(spec, fac, a.cell, a.confirmations, a.results, a.work_root, stage, a.attempt)
    for r in recs:
        print("%s %s c%d ok=%s applied=%s sha=%s" % (a.workflow, a.cell, r["confirmation"], r["ok"],
                                                   r["applied_env_ok"], r["oracle_stdout_sha256"][:12]))
    return 0 if all(r["ok"] for r in recs) else 1


def read_cells(results, name):
    d = pathlib.Path(results) / name
    cells = {}
    if d.is_dir():
        for p in sorted(d.glob("*.json")):
            r = load_json(p)
            cells.setdefault(r["cell"], []).append(r)
    return cells


def cell_status(recs, base_sha=None):
    good = [r for r in recs if r["ok"] and r["applied_env_ok"] and r["fixture_ok"]]
    if len(good) < 2:
        return "UNRESOLVED", None
    shas = {r["oracle_stdout_sha256"] for r in good}
    if len(shas) != 1:
        return "UNRESOLVED", None
    sha = next(iter(shas))
    if base_sha is None:
        return "BASELINE", sha
    return ("SENSITIVE" if sha != base_sha else "STABLE"), sha


def cmd_summarize(a):
    fac = load_factors()
    cells = read_cells(a.results, a.workflow)
    base = cell_status(cells.get("000", []))
    print("workflow", a.workflow, "baseline", base[0])
    for c in all_cells(fac):
        if c in cells and c != "000":
            st, sha = cell_status(cells[c], base[1] if base[0] == "BASELINE" else None)
            print(" ", c, st, (sha or "")[:12])
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)

    def common(p, wf_required=True):
        p.add_argument("--workflow", required=wf_required)
        p.add_argument("--work-root", default="/tmp/g3c-work")
        p.add_argument("--results", default=str(ROOT / "results"))
        p.add_argument("--campaign", default=str(ROOT / "campaign"))
    p = sp.add_parser("check-install"); common(p, False); p.set_defaults(f=cmd_check_install)
    p = sp.add_parser("pin-fixture"); common(p); p.add_argument("--write", action="store_true"); p.set_defaults(f=cmd_pin_fixture)
    p = sp.add_parser("check-env"); common(p, False); p.add_argument("--verbose", action="store_true"); p.set_defaults(f=cmd_check_env)
    p = sp.add_parser("preflight"); common(p); p.set_defaults(f=cmd_preflight)
    p = sp.add_parser("run-cell"); common(p); p.add_argument("--cell", required=True)
    p.add_argument("--confirmations", type=int, default=2); p.add_argument("--attempt", type=int, default=1)
    p.set_defaults(f=cmd_run_cell)
    p = sp.add_parser("summarize"); common(p); p.set_defaults(f=cmd_summarize)
    a = ap.parse_args(argv)
    try:
        return a.f(a)
    except ValueError as exc:
        die(str(exc),3)


if __name__ == "__main__":
    sys.exit(main())
