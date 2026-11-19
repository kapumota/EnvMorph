#!/usr/bin/env python3
"""Sorteo determinista de la cohorte G3-C (protocolo v0.2, seccion 3.2).

Uso: python3 draw_cohort.py pool_manifest.json [salida.json]

Reglas.
1. semilla = SHA-256 de los bytes exactos del manifiesto del pool.
2. orden = SHA-256 (semilla_hex + nombre) ascendente.
3. se recorre el orden y se toma el candidato salvo que su familia upstream
   ya alcanzo el limite del manifiesto. Los primeros 5 son primarios y los
   siguientes 3 son reservas, con el mismo limite por familia.
No usa aleatoriedad externa ni resultados de ejecucion.
"""
import hashlib, json, sys

PRIMARIOS, RESERVAS = 5, 3

def main():
    path = sys.argv[1]
    raw = open(path, 'rb').read()
    seed = hashlib.sha256(raw).hexdigest()
    m = json.loads(raw.decode('utf-8'))
    limites = m["limites_por_familia"]
    cands = m["candidatos"]
    orden = sorted(cands, key=lambda c: hashlib.sha256((seed + c["nombre"]).encode('utf-8')).hexdigest())
    tomados, saltados, usados = [], [], {}
    for c in orden:
        fam = c["familia_upstream"]
        # las familias independientes cuentan por separado por nombre
        clave = fam if fam != "independiente" else "independiente:" + c["nombre"]
        lim = limites.get(fam, 1)
        if usados.get(clave, 0) >= lim:
            saltados.append({"nombre": c["nombre"], "motivo": "limite de familia " + fam})
            continue
        usados[clave] = usados.get(clave, 0) + 1
        tomados.append(c["nombre"])
        if len(tomados) == PRIMARIOS + RESERVAS:
            break
    out = {
        "estado": "PROVISIONAL hasta que el pool se congele en commit A",
        "manifiesto": path,
        "manifiesto_sha256": seed,
        "orden_completo": [{"posicion": i + 1, "nombre": c["nombre"],
                            "hash_orden": hashlib.sha256((seed + c["nombre"]).encode('utf-8')).hexdigest(),
                            "familia": c["familia_upstream"]} for i, c in enumerate(orden)],
        "primarios": tomados[:PRIMARIOS],
        "reservas": tomados[PRIMARIOS:PRIMARIOS + RESERVAS],
        "saltados_por_limite": saltados,
    }
    dest = sys.argv[2] if len(sys.argv) > 2 else "COHORT_DRAW.json"
    open(dest, 'w', encoding='utf-8').write(json.dumps(out, indent=1, ensure_ascii=False) + "\n")
    print("semilla", seed)
    print("primarios", out["primarios"])
    print("reservas", out["reservas"])
    print("saltados", [s["nombre"] for s in saltados])

if __name__ == "__main__":
    main()
