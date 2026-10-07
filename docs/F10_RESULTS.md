### F10-MVP: validación de sensibilidad externa

#### Estado

F10-MVP parte de `main` en:

```text
044f31a7179832d4534991c7d51010a356fec94c
```

El freeze científico previo permanece en `v0.1.0`:

```text
29cc7961766f783ee4d642f3acf446fb6f9bdb27
```

F10 no modifica F8, F9, sus manifests ni `CITATION.cff`.

El objetivo es responder una amenaza concreta a la validez: antes de F10,
EnvMorph no había demostrado un positivo externo real end-to-end.

#### Candidatos

La búsqueda acotada usó documentación primaria versionada de GNU coreutils
9.4 y CPython 3.12.3. No constituye una revisión exhaustiva de software Unix.

| Proyecto / versión | Caso y factor | Baseline / treatment | Evidencia | Seleccionado |
|---|---|---|---|---|
| coreutils 9.4 | date, TZ | UTC0 / EST5 | Manual oficial v9.4 | Sí |
| coreutils 9.4 | ls, QUOTING_STYLE | literal / c | Manual oficial v9.4 | Sí |
| CPython 3.12.3 | PYTHONIOENCODING | utf-8 / latin-1 | Documentación oficial v3.12.3 | Sí |
| coreutils 9.4 | sort, LC_ALL | C / de_DE.UTF-8 | Manual oficial v9.4 | No |
| coreutils 9.4 | mktemp, TMPDIR | /tmp/f10-a / /tmp/f10-b | Manual oficial v9.4 | No |
| CPython 3.12.3 | PYTHONHASHSEED | 0 / 1 | Documentación oficial v3.12.3 | No |

Los tres controles seleccionados fueron concretados antes de observar resultados
de EnvMorph. Los otros tres son candidatos potenciales y no forman parte del
denominador experimental.

#### Benchmark congelado

SHA-256 de `experiments/f10/benchmark.json`:

```text
59db7a371740870eda0b50a08a39826e276409cc91b42bcfd48fc7f798a2dff9
```

El congelamiento es local y previo a la primera ejecución F10. No se presenta
como prerregistro externo con sello de tiempo independiente.

#### Controles positivos

| Workload | Factor | Comando | Diferencia esperada |
|---|---|---|---|
| `date-tz` | `TZ` | `/usr/bin/date --date=@0 '+%Y-%m-%d %H:%M:%S %z'` | UTC0 produce `1970-01-01 00:00:00 +0000`, EST5 produce `1969-12-31 19:00:00 -0500` |
| `ls-quoting` | `QUOTING_STYLE` | `/usr/bin/ls -1 --color=never -- 'a b'` | `literal` produce `a b`, `c` produce `"a b"` |
| `python-encoding` | `PYTHONIOENCODING` | `/usr/bin/python3 -S -c 'print(chr(233))'` | UTF-8 produce `c3a90a`, latin-1 produce `e90a` |

El oráculo común es comparación byte de stdout mediante F3, más igualdad con
la salida previamente congelada y dos confirmaciones por variante.

Son funciones ambientales documentadas de software externo, no bugs externos,
ni un corpus representativo de aplicaciones. Dos controles pertenecen a
coreutils.

#### Funnel

| Control | known_positive | observed | normalized | intervenable | eligible | treatment_executed | oracle_decidable | behaviorally_relevant |
|---|---|---|---|---|---|---|---|---|
| `date-tz` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `ls-quoting` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |
| `python-encoding` | PASS | PASS | PASS | PASS | PASS | PASS | PASS | PASS |

El observador registró la variable objetivo en los tres casos. Los eventos se
normalizaron mediante F9.2, los grafos se construyeron con F9.3 y los planes
pasaron el validador F9.4. No se añadieron eventos sintéticos.

Cada control ejecutó dos confirmaciones baseline y dos treatment. Las capturas
usadas por F3 coinciden con los hashes producidos por el runner F9.4.

#### Métricas

| Métrica | Numerador / denominador | Valor |
|---|---|---|
| known_positive_count | 3 | 3 |
| observation_recall | 3/3 | 1.0 |
| normalization_recall | 3/3 | 1.0 |
| intervenability_recall | 3/3 | 1.0 |
| eligibility_recall | 3/3 | 1.0 |
| intervention_success_rate | 3/3 | 1.0 |
| oracle_decidability | 3/3 | 1.0 |
| end_to_end_sensitivity | 3/3 | 1.0 |

Estas métricas son descriptivas exclusivamente para los tres controles
known-positive. Las confirmaciones no se cuentan como observaciones
independientes. No se realiza inferencia poblacional.

F9 conserva 0/2 positivos externos entre candidatos decidibles. F10 no
reinterpreta ese resultado ni estima prevalencia externa.

#### Archivos F10

F10 añade:

- `docs/F10_PROTOCOL.md`
- `docs/F10_RESULTS.md`
- `experiments/f10/benchmark.json`
- `experiments/f10/results.json`
- `reproduce/README.md`
- `reproduce/bootstrap.sh`
- `reproduce/f10.sh`
- `reproduce/f10.py`
- `reproduce/verify_f10.py`
- `reproduce/test_f10.py`
- `reproduce/Dockerfile.f10`
- `reproduce/f10-docker.sh`

No modifica ni elimina archivos históricos.

#### Validación

La entrega original produjo dos ejecuciones F10 con el mismo entorno y ambas
dieron 3/3 controles `behaviorally_relevant`.

Antes del commit, los archivos fueron incorporados sobre el baseline público y
se ejecutó una tercera reproducción desde la máquina del desarrollador mediante
un contenedor Ubuntu 24.04 aislado. Esa reproducción volvió a producir 3/3.

El gate completo también verificó:

```text
cargo fmt --check                                      PASS
cargo test --locked                                    PASS, 108 tests
cargo build --release --locked                         PASS
sha256sum -c provenance/F9_10_FILES.sha256            PASS
verify_f10.py sobre evidencia original run-1           PASS
verify_f10.py sobre evidencia original run-2           PASS
test_f10.py                                            PASS, 5 tests
reproducción F10 en contenedor Ubuntu 24.04            PASS, 3/3
```

La reproducción en contenedor no constituye una réplica independiente en una
segunda máquina. Sí demuestra que el resultado F10 puede regenerarse desde la
máquina del desarrollador sin cambiar las versiones ambientales congeladas.

#### Reproducibilidad

La ruta nativa requiere coreutils 9.4 y CPython 3.12.3 en `/usr/bin`.

Para hosts con versiones diferentes se incluye una ruta Docker reproducible:

```bash
bash reproduce/f10-docker.sh /tmp/envmorph-f10-run
```

El script construye el binario Rust en el host, crea un entorno Ubuntu 24.04
con coreutils 9.4, CPython 3.12.3 y los headers de compilación necesarios, y
ejecuta los controles sin red.

Los niveles de reproducción se distinguen como:

1. fuentes y baseline;
2. ejecución;
3. evidencia semántica;
4. agregados.

No se exige identidad byte a byte para PID o rutas temporales.

La reconstrucción histórica completa de F8 y F9 permanece fuera del alcance de
F10-MVP.

#### Respuesta a la auditoría

| Observación | Estado | Resultado F10 |
|---|---|---|
| Positivo externo real | RESOLVED | 3/3 controles documentados alcanzan `behaviorally_relevant` |
| Embudo F9 12717 -> 2 | PARTIALLY_RESOLVED | F10 demuestra el funnel completo en tres objetivos known-positive, pero no reevalúa los 12,717 candidatos históricos |
| Selección del corpus | PARTIALLY_RESOLVED | F10 separa un benchmark known-positive de F8 result-blind, pero usa solo dos proyectos |
| Reproducibilidad end-to-end | PARTIALLY_RESOLVED | F10 puede regenerarse con ruta nativa o Docker; F8/F9 históricos no se reconstruyen completamente |
| Proceso frente a evidencia | PARTIALLY_RESOLVED | Se obtiene evidencia positiva externa sin modificar el motor ni ampliar el benchmark |

#### Decisión

```text
GO
```

Los tres controles seleccionados llegaron a `behaviorally_relevant`. F10-MVP
termina en este punto.

Este resultado demuestra sensibilidad end-to-end en los tres controles
documentados. No demuestra prevalencia, representatividad del software Unix,
hermeticidad universal ni madurez científica general.
