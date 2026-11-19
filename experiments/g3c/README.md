### G3-C v0.3: entrega correctiva PRE-CONGELACIÓN

**No ejecutar tratamientos de los cinco primarios hasta completar RATIFICACIÓN, Docker, F10 inicial y sellado externo.**

Se incluyen exactamente los cinco workflows primarios y el manifiesto original del pool. Se conserva literalmente `evidencia_preflight_sandbox` de v0.1. El paquete anterior sigue disponible como fuente documental. Esta versión **no** integra F9 ni `workspace_file_replace` en la cohorte, y no sostiene mejor desempeño de F6-v2 sobre C2.

#### Validación local segura, sin tratamiento de primarios

```bash
python3 -m unittest discover -s tests -p 'test_g3c*.py' -v
python3 -m py_compile harness/*.py
sh -n docker/*.sh
sha256sum -c MANIFEST.sha256
python3 draw_cohort.py pool_manifest.json /tmp/g3c-sorteo-verificacion.json
python3 harness/g3c_harness.py check-env
```

El test de integración de F6-v2 se salta si no está el binario. Para ejecutarlo se necesita `G3C_F6V2_BIN` apuntando al binario real construido sobre `94a73b4` o la rama ratificada. No usar el PASS de autopruebas como confirmación científica.

#### Reglas de ejecución

0. Mantener `campaign/` y `results/` en rutas nuevas; se prohíbe sobrescribir. Construir y fijar binario F6-v2 e imagen. Ratificar explícitamente el protocolo, los hashes y la cohorte. Sin Docker y F10 verificables: STOP.
1. Preflight de cinco primarios solo con baseline, con `--results` y `--campaign` comunes. Se escribe en `results/preflight/NOMBRE/`. El ledger ya contabiliza estas corridas.
2. Generar freeze solo con `--ratification-record`, `--protocol`, `--pool`, `--workflows`, `--f6v2-bin`, `--image-digest`, `--custodian-public-key` y `--campaign`. Firmar tag y publicar externamente antes de tratamiento.
3. Ejecutar F10 de inicio en la misma imagen, validar y registrar mediante `g3c_bookend.py`. El código no prueba por sí mismo que F10 se ejecutó en esa imagen, esto exige auditoría de imagen y logs.
4. Ejecutar S y NULLA/NULLB una sola vez, con exactamente dos confirmaciones por celda. Todos los controles negativos deben ser STABLE.
5. Ejecutar `g3c_decide.py decide`, `seal`; publicar hashes de freeze, S/N y decisions en OSF/Zenodo, obtener recibo, `authorize-heldout --receipt RECIBO.json --signature RECIBO.sig`. Nunca publicar el held-out antes del sello.
6. Ejecutar H una vez, F10 final y luego `score`. Archivar salidas exitosas y fallidas. La clasificación automática es orientativa y no reemplaza auditoría independiente.

No crear `PROTOCOL_FREEZE.json` antes de ratificar todos los campos y confirmar que el preflight Docker real pasa. El `README.md` original de v0.1 se conserva como pieza histórica, no como instrucción operativa vigente.

#### Esquema mínimo del registro de ratificación

```json
{"estado":"RATIFICADO","protocol_sha256":"<SHA-256 de PROTOCOLO_G3C_v0.3.md>","ratificado_por":"<investigador>","fecha_utc":"<fecha>"}
```

#### Esquema del recibo externo de decisiones

```json
{"schema":"g3c-external-receipt-v1","decisions_sha256":"<sha256>","freeze_sha256":"<sha256>","evidence_sha256":"<sha256 S/N del SEALED.json>","url":"https://osf.io/...","custodian":"<identidad verificable>","attestation_utc":"<fecha externa>"}
```

El validador comprueba los hashes y exige una firma Ed25519 verificable con la clave pública congelada del custodio. La clave privada debe custodiarse fuera del equipo investigador. **No consulta ni verifica automáticamente** la fecha, autenticidad o permanencia del URL externo. Un auditor independiente debe hacerlo. Nunca tratar el recibo local como prueba autosuficiente de preregistro.
