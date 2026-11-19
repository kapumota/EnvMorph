#!/usr/bin/env python3
"""G3-C. Decisiones selladas, sellado y puntuacion. Solo biblioteca estandar.

  decide   lee SOLO las celdas S, ejecuta F6-v2 y los comparadores C0 a C2 y escribe decisions.json
  seal     sella decisions.json (hash y fecha) para habilitar la ejecucion de H
  score    puntua las decisiones selladas contra las celdas H ya ejecutadas

C4 (ddmin), C5 (reprotest) y C6 (BugDoc) no estan implementados aqui. Si se
aplican, se corren aparte y se registran como indica el protocolo.
"""
import argparse, datetime, hashlib, json, pathlib, subprocess, sys, shutil
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import g3c_harness as H
import g3c_state as G

THRESH = {"min_evaluables": 3, "min_proyectos": 2, "min_informativos": 2, "min_wf_beneficio": 2}


def canon(obj):
    return json.dumps(obj, indent=1, sort_keys=True, ensure_ascii=False) + "\n"


def f6_manifest(stage, name, cells, base):
    stage.mkdir(parents=True, exist_ok=True)
    (stage / "runs").mkdir(exist_ok=True)
    lines = ["[experiment]", "schema_version = 1", "dimension = 3",
             'oracle = "byte_stdout_vs_baseline_v1"', "confirmations = 2"]
    for cell in sorted(cells):
        good = [r for r in cells[cell] if r["ok"] and r["applied_env_ok"] and r["fixture_ok"]][:2]
        for rep, r in enumerate(good, 1):
            src = pathlib.Path(r["_dir"]) / ("%s_c%d_a%d.stdout" % (cell, r["confirmation"], r["attempt"]))
            rel = "runs/%s-%d.stdout" % (cell, rep)
            shutil.copyfile(src, stage / rel)
            lines += ["", "[[run]]", 'assignment = "%s"' % cell, 'run_id = "%s_%s_r%d"' % (name, cell, rep),
                      'status = "ok"', 'stdout_file = "%s"' % rel,
                      'stdout_sha256 = "%s"' % H.sha_file(stage / rel)]
    data = ("\n".join(lines) + "\n").encode("utf-8")
    (stage / "manifest.toml").write_bytes(data)
    return H.sha_bytes(data)


def load_cells(results, name):
    cells = H.read_cells(results, name)
    for recs in cells.values():
        for r in recs:
            r["_dir"] = str(pathlib.Path(results) / name)
    return cells


def single_factor_cells(fac):
    return [c for c in fac["s_cells"] if c != "000"]


def cmd_decide(a):
    fac = H.load_factors()
    camp = pathlib.Path(a.campaign)
    camp.mkdir(parents=True, exist_ok=True)
    if any(not H.load_spec(w).get("kind")=="campaign" for w in a.workflows):
        # Las autopruebas usan una ruta separada y nunca cuentan como campaña.
        if not all(w.startswith("_selftest_") for w in a.workflows):
            G.fail("no mezclar autopruebas y campaña")
    else:
        f=G.require_freeze(camp,H.ROOT)
        if a.workflows!=f["workflows"]: G.fail("cohorte no coincide con el freeze")
        G.ensure_not_sealed(camp)
        G.ensure_s_only(a.results,a.workflows)
        G.validate_negatives(a.results,a.workflows)
        G.require_f10_start(camp,f)
    binpath = pathlib.Path(a.f6v2_bin)
    if not binpath.is_file():
        H.die("falta el binario F6-v2 " + a.f6v2_bin)
    if not all(w.startswith("_selftest_") for w in a.workflows):
        if H.sha_file(binpath)!=f.get("f6v2_bin_sha256"):
            G.fail("binario F6-v2 distinto del congelado")
    out = {"schema": "g3c-decisions-v1", "f6v2_bin_sha256": H.sha_file(binpath),
           "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
           "workflows": {}}
    for name in a.workflows:
        cells = load_cells(a.results, name)
        leaked = [c for c in fac["h_cells"] if c in cells]
        if leaked:
            H.die("FUGA. Existen celdas held-out ejecutadas antes de sellar en %s: %s" % (name, leaked), 4)
        base_status, base_sha = H.cell_status(cells.get("000", []))
        entry = {"s_status": {}, "h_cells": {}}
        if base_status != "BASELINE":
            entry["workflow_status"] = "UNRESOLVED_BASELINE"
            out["workflows"][name] = entry
            continue
        usable = {"000": cells["000"]}
        entry["s_status"]["000"] = "BASELINE"
        for c in single_factor_cells(fac):
            st, _ = H.cell_status(cells.get(c, []), base_sha)
            entry["s_status"][c] = st
            if st != "UNRESOLVED":
                usable[c] = cells[c]
        stage = camp / "f6v2_stage" / name
        if stage.exists():
            G.fail("etapa F6 ya creada, se prohíbe sobrescribir: "+str(stage))
        msha = f6_manifest(stage, name, usable, base_sha)
        report = stage / "report.json"
        proc = subprocess.run([str(binpath), "evaluate", "--manifest", str(stage / "manifest.toml"),
                               "--expected-manifest-sha256", msha, "--output", str(report)],
                              capture_output=True, text=True, timeout=60)
        f6 = {}
        if proc.returncode == 0 and report.is_file():
            rep = json.loads(report.read_text(encoding="utf-8"))
            f6 = {c["assignment"]: c for c in rep["cells"]}
            entry["f6v2_report_sha256"] = H.sha_file(report)
        else:
            entry["f6v2_error"] = proc.stderr[-300:]
        entry["manifest_sha256"] = msha
        sens = {c: entry["s_status"].get(c) for c in single_factor_cells(fac)}
        for hc in fac["h_cells"]:
            flipped = [i for i, bit in enumerate(hc) if bit == "1"]
            single = ["".join("1" if j == i else "0" for j in range(len(hc))) for i in flipped]
            if any(sens.get(s) == "UNRESOLVED" for s in single):
                c2 = "abstain"
            else:
                c2 = "differs" if any(sens.get(s) == "SENSITIVE" for s in single) else "same"
            f6c = f6.get(hc)
            if f6c and f6c["status"] == "model_supported":
                c3 = {"decision": "differs" if f6c["label"] else "same", "level": "model_supported"}
            else:
                c3 = {"decision": "abstain", "level": (f6c or {}).get("status", "error")}
            entry["h_cells"][hc] = {"C0": "same", "C1": "abstain", "C2": c2, "C3": c3}
        out["workflows"][name] = entry
    text = canon(out)
    G.exclusive_write(camp / "decisions.json", text.encode("utf-8"))
    print("decisions_sha256 =", H.sha_bytes(text.encode("utf-8")))
    return 0


def cmd_seal(a):
    camp = pathlib.Path(a.campaign)
    dec = camp / "decisions.json"
    if not dec.is_file():
        H.die("no existe decisions.json")
    if (camp / "SEALED.json").exists(): G.fail("sello existente")
    h = H.sha_file(dec)
    dec_obj=G.json_read(dec)
    work=list(dec_obj.get("workflows",{}))
    is_campaign=not all(w.startswith("_selftest_") for w in work)
    evidence_sha = None
    if is_campaign:
        f=G.require_freeze(camp,H.ROOT)
        if work!=f["workflows"]: G.fail("decisiones no coinciden con cohorte congelada")
        G.ensure_s_only(a.results,work)
        G.validate_negatives(a.results,work)
        evidence_sha=G.evidence_snapshot(a.results,work)
    G.exclusive_write(camp / "decisions.sha256",(h + "  decisions.json\n").encode("utf-8"))
    G.exclusive_write(camp / "SEALED.json",canon({"decisions_sha256": h, "evidence_sha256": evidence_sha, "sealed_utc": datetime.datetime.now(
        datetime.timezone.utc).isoformat(timespec="seconds"),
        "nota": "Haga commit de decisions.json, decisions.sha256 y SEALED.json y deposite con sello temporal externo antes de ejecutar H."}).encode("utf-8"))
    print("SEALED decisions_sha256 =", h)
    return 0


def truth_of(status):
    return {"SENSITIVE": "differs", "STABLE": "same"}.get(status)


def cmd_score(a):
    fac = H.load_factors()
    camp = pathlib.Path(a.campaign)
    dec = json.loads((camp / "decisions.json").read_text(encoding="utf-8"))
    sealed = json.loads((camp / "SEALED.json").read_text(encoding="utf-8"))
    if sealed["decisions_sha256"] != H.sha_file(camp / "decisions.json"):
        H.die("decisions.json no coincide con el sello", 3)
    names=list(dec["workflows"])
    if not all(w.startswith("_selftest_") for w in names):
        f=G.require_freeze(camp,H.ROOT)
        G.require_heldout_authorization(camp)
        if names!=f["workflows"]: G.fail("cohorte distinta del freeze")
        if sealed.get("evidence_sha256")!=G.evidence_snapshot(a.results,names):
            G.fail("evidencia S/N cambió después de sellar")
        G.audit_present_results(a.results,names)
        for name in names:
            entry=dec['workflows'][name]
            stage=camp/'f6v2_stage'/name/'report.json'
            if entry.get('f6v2_report_sha256'):
                if not stage.is_file() or G.digest(stage)!=entry['f6v2_report_sha256']:
                    G.fail('reporte F6-v2 distinto al sellado: '+name)
        # H puede estar incompleto por STOP. Se mantiene UNRESOLVED, no se inventa resultado.
    comps = ["C0", "C1", "C2", "C3"]
    rows, glob = {}, {c: {"correct": 0, "wrong": 0, "abstain": 0} for c in comps}
    proyectos_eval, n_eval, n_inf, neg_fail = set(), 0, 0, 0
    f6_wrong, wf_benefit, wf_benefit_proj = 0, 0, set()
    h_decidibles = 0
    h_unresolved = 0
    dominated_by_c2, f6_better_than_c2 = False, False
    for name, entry in dec["workflows"].items():
        spec = H.load_spec(name)
        cells = H.read_cells(a.results, name)
        bst, bsha = H.cell_status(cells.get("000", []))
        row = {"baseline": bst, "informativo": False, "celdas": {}, "controles_negativos": {}}
        if bst != "BASELINE" or "h_cells" not in entry or not entry["h_cells"]:
            rows[name] = row
            continue
        row["informativo"] = any(v == "SENSITIVE" for k, v in entry["s_status"].items() if k != "000")
        for nc in fac["null_cells"]:
            if nc in cells:
                st, _ = H.cell_status(cells[nc], bsha)
                row["controles_negativos"][nc] = st
        if any(v != "STABLE" for v in row["controles_negativos"].values()):
            neg_fail += 1
        scored = 0
        f6_extra = 0
        for hc, decs in entry["h_cells"].items():
            st, _ = H.cell_status(cells.get(hc, []), bsha)
            t = truth_of(st)
            r = {"verdad": t or "UNRESOLVED"}
            if t is not None:
                scored += 1
                h_decidibles += 1
                for c in comps:
                    d = decs[c]["decision"] if isinstance(decs[c], dict) else decs[c]
                    k = "abstain" if d == "abstain" else ("correct" if d == t else "wrong")
                    glob[c][k] += 1
                    r[c] = k
                if r["C3"] == "wrong":
                    f6_wrong += 1
                if r["C3"] == "correct":
                    f6_extra += 1
                    if r["C2"] != "correct":
                        f6_better_than_c2 = True
                if r["C3"] == "wrong" and r["C2"] == "correct":
                    dominated_by_c2 = True
            if t is None:
                h_unresolved += 1
            row["celdas"][hc] = r
        if scored > 0:
            n_eval += 1
            proyectos_eval.add(spec.get("project", name))
            n_inf += 1 if row["informativo"] else 0
            if f6_extra > 0:
                wf_benefit += 1
                wf_benefit_proj.add(spec.get("project", name))
        rows[name] = row
    # El hallazgo matemático impide alegar superioridad sobre C2.
    # A significa únicamente evidencia selectiva no vacía, no victoria de método.
    if (neg_fail >= 1 or f6_wrong > 0 or n_eval < THRESH["min_evaluables"]
        or h_unresolved > 0 or (not all(w.startswith("_selftest_") for w in dec["workflows"])
            and not (camp/"F10_END.json").is_file())):
        hint = "D_NO_GO_METODO_REQUIERE_AUDITORIA"
    elif n_inf < THRESH["min_informativos"]:
        hint = "C_NULO_EMPIRICO"
    elif (wf_benefit >= THRESH["min_wf_beneficio"]
          and len(wf_benefit_proj) >= THRESH["min_proyectos"]
          and glob['C3']['wrong'] == 0 and h_unresolved == 0):
        hint = "A_EVIDENCIA_SELECTIVA_NO_VACIA_SIN_SUPERIORIDAD"
    else:
        hint = "B_ESTUDIO_EMPIRICO_SIN_SUPERIORIDAD_DE_METODO"
    res = {"workflows": rows, "global": glob, "evaluables": n_eval, "proyectos_evaluables": sorted(proyectos_eval),
           "informativos": n_inf, "f6v2_decisiones_erroneas": f6_wrong, "workflows_con_controles_negativos_fallidos": neg_fail,
           "workflows_con_decisiones_C3_correctas": wf_benefit, "outcome_hint": hint,
           "heldout_predeclaradas":len(dec["workflows"])*len(fac["h_cells"]),
           "heldout_decidibles":h_decidibles,
           "heldout_unresolved":h_unresolved,
           "C3_decididas":glob['C3']['correct']+glob['C3']['wrong'],
           "C3_abstenciones":glob['C3']['abstain'],
           "F10_inicio_registrado":(camp/"F10_START.json").is_file(),
           "F10_final_registrado":(camp/"F10_END.json").is_file(),
           "nota": "Pista automatica. La clasificacion final y la condicion D requieren juicio humano segun el protocolo."}
    text = canon(res)
    G.exclusive_write(a.out,text.encode("utf-8"))
    print(text)
    return 0


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sp = ap.add_subparsers(dest="cmd", required=True)
    root = H.ROOT
    def common(p):
        p.add_argument("--results", default=str(root / "results"))
        p.add_argument("--campaign", default=str(root / "campaign"))
    p = sp.add_parser("decide"); common(p); p.add_argument("--workflows", nargs="+", required=True)
    p.add_argument("--f6v2-bin", required=True); p.set_defaults(f=cmd_decide)
    p = sp.add_parser("seal"); common(p); p.set_defaults(f=cmd_seal)
    p = sp.add_parser("score"); common(p); p.add_argument("--out", default=str(root / "campaign" / "score.json")); p.set_defaults(f=cmd_score)
    p = sp.add_parser("authorize-heldout"); common(p); p.add_argument("--receipt",required=True); p.add_argument("--signature",required=True); p.set_defaults(f=lambda a: G.record_receipt(a.campaign,a.receipt,a.signature))
    a = ap.parse_args(argv)
    try:
        return a.f(a) or 0
    except ValueError as e:
        H.die(str(e),3)


if __name__ == "__main__":
    sys.exit(main())
