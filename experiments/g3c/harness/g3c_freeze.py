#!/usr/bin/env python3
"""Genera campaign/PROTOCOL_FREEZE.json con los hashes que la guardia exige.

Uso: python3 harness/g3c_freeze.py --protocol PROTOCOLO.docx --pool pool_manifest.json --workflows findutils miller unzip exiftool patch

Ejecutelo solo despues de ratificar parametros y completar las fichas. Luego
haga commit, firme un tag y deposite el archivo con sello temporal externo.
"""
import argparse, datetime, json, pathlib, sys, shutil
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import g3c_harness as H
import g3c_state as G


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--protocol", required=True)
    ap.add_argument("--pool", required=True)
    ap.add_argument("--workflows", nargs="+", required=True)
    ap.add_argument("--campaign", default=str(H.ROOT / "campaign"))
    ap.add_argument("--image-digest",required=True)
    ap.add_argument("--f6v2-bin",required=True)
    ap.add_argument("--ratification-record",required=True)
    ap.add_argument("--custodian-public-key",required=True)
    a = ap.parse_args()
    root = H.ROOT
    if a.workflows != list(G.WORKFLOWS):
        G.fail("solo se acepta cohorte predeclarada en orden fijado")
    if H.sha_file(a.pool)!="743c7d2f2d326d6d1c366172542df31d24424270819297f3fc0b2cbc9805d5bc":
        G.fail("pool distinto del provisional, requiere enmienda informada")
    if not pathlib.Path(a.protocol).is_file() or 'v0.3' not in pathlib.Path(a.protocol).name:
        G.fail("usar el protocolo G3-C v0.3 ratificado")
    if pathlib.Path(a.campaign,"PROTOCOL_FREEZE.json").exists():
        G.fail("congelamiento ya existente")
    import re
    if not re.fullmatch(r"sha256:[0-9a-f]{64}",a.image_digest):
        G.fail("digest de imagen inválido")
    rat = G.json_read(a.ratification_record)
    if rat.get("estado")!="RATIFICADO" or rat.get("protocol_sha256")!=H.sha_file(a.protocol):
        G.fail("ratificación ausente o distinta al protocolo")
    if not pathlib.Path(a.f6v2_bin).is_file():G.fail("binario F6-v2 inexistente")
    if not pathlib.Path(a.custodian_public_key).is_file():
        G.fail("falta clave pública del custodio externo")
    checked_files=["harness/g3c_freeze.py","harness/g3c_bookend.py",
        "docker/Dockerfile.g3c","docker/run_g3c.sh","docker/verify_debs.sh",
        "docker/verify_debs.py","docker/dependency_debs.json",
        "draw_cohort.py"]
    for w in a.workflows:
        spec=H.load_spec(w)
        if spec.get("fixture_script"):
            checked_files.append("workflows/"+w+"/"+spec["fixture_script"])
    aux_paths={}
    for w in a.workflows:
        spec=H.load_spec(w)
        identity=H.tool_identity(spec)
        if identity.get("package_version")!=spec["version"] or not identity.get("binary_sha256"):
            G.fail("binario/paquete no está instalado como se declaró: "+w)
        for step in spec["steps"]:
            if step["role"]=="aux":
                cmd=step["argv"][0]
                path=shutil.which(cmd,path="/usr/bin:/bin")
                if not path:G.fail("auxiliar no instalado: "+cmd)
                aux_paths[cmd]=H.sha_file(path)
    out = {
        "file_sha256":{item:H.sha_file(root/item) for item in checked_files},
        "tool_binary_sha256":{w:H.sha_file(H.load_spec(w)["tool_binary"]) for w in a.workflows},
        "aux_binary_sha256":aux_paths,
        "schema": "g3c-freeze-v3",
        "workflows":a.workflows,
        "image_digest":a.image_digest,
        "f6v2_bin_sha256":H.sha_file(a.f6v2_bin),
        "ratification_sha256":H.sha_file(a.ratification_record),
        "custodian_public_key_sha256":H.sha_file(a.custodian_public_key),
        "budget_max": G.BUDGET, "bookend_reserve":G.BOOKEND_RESERVED,
        "created_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="seconds"),
        "protocol_sha256": H.sha_file(a.protocol),
        "pool_manifest_sha256": H.sha_file(a.pool),
        "harness_sha256": H.sha_file(root / "harness" / "g3c_harness.py"),
        "state_sha256": H.sha_file(root / "harness" / "g3c_state.py"),
        "decide_sha256": H.sha_file(root / "harness" / "g3c_decide.py"),
        "factors_sha256": H.sha_file(root / "factors.json"),
        "spec_sha256": {w: H.sha_file(root / "workflows" / w / "spec.json") for w in a.workflows},
        "fixture_tree_sha256": {w: H.load_spec(w).get("fixture_tree_sha256") for w in a.workflows},
    }
    for w, d in out["fixture_tree_sha256"].items():
        if d in (None, "PENDIENTE") and H.load_spec(w).get("fixture_script"):
            H.die("fixture sin digest fijado en " + w)
    pathlib.Path(a.campaign).mkdir(parents=True, exist_ok=True)
    G.exclusive_write(pathlib.Path(a.campaign) / "CUSTODIAN_PUBLIC_KEY.pem",pathlib.Path(a.custodian_public_key).read_bytes())
    G.exclusive_write(pathlib.Path(a.campaign) / "PROTOCOL_FREEZE.json",G.canonical(out))
    print("PROTOCOL_FREEZE.json escrito")


if __name__ == "__main__":
    main()
