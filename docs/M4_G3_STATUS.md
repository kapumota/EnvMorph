### EnvMorph M4-G3: estado de integración y evidencia de ingeniería

Fecha de actualización: 2026-11-16. Base de integración: `main` en `c53be087309628545737b0d807a669593e4fe6c9` (PR #5, fusionado). Trabajo nuevo: rama `research/m4-g3-prospective`.

#### Objetivo y frontera

G3 prepara una última campaña prospectiva acotada antes de la auditoría científica y técnica. **No es una nueva fase abierta de producto.** Los resultados históricos F8, F9, F10 y P1-EB-01 siguen congelados. El piloto existente `inih-meson-tests` comprueba la integración de preparación, ejecución y equivalencia; no se usa como observación confirmatoria ni valida el observador F9 en la nueva ejecución.

#### Componentes reales

| Componente | Implementación | Evidencia G3 | Límite |
|---|---|---|---|
| Preparación tipada M4-G2 | `tools/m4/interventions/m4_interventions.py` | Plan y materialización en copias, preflight PASS | No ejecuta workloads ni es sandbox |
| Piloto Docker aislado | `tools/m4/g3/g3_pilot.py` (copia byte a byte del arnés ejecutado, archivada al actualizar) | 2 baseline + 2 treatment, rc=0 | Script experimental, no motor de producción |
| Oráculo F3 | Binario `target/release/envmorph`, `compare-artifacts --oracle byte` | 4 comparaciones de dos artefactos en dos repeticiones, identical | El oráculo está limitado a los artefactos seleccionados |
| Adaptación F9.6 | `tools/f9/f3-f5-bridge/adapt_oracle_evidence.py` | 4 adaptaciones `observed_only` a nivel F3 | **No** equivale a verificación del observador F9 |
| Observador F9 | `tools/f9/external-observer/`, normalizador y grafo | Sin instrumentación G3 confirmada | No afirmar observación de TMPDIR en G3 |
| Contratos F6-v2 | `src/bin/envmorph-f6v2.rs` | No evaluados en G3 | Requiere `byte_stdout_vs_baseline_v1`, incompatible sin adaptación explícita al oráculo de archivos de inih |

#### Piloto inih-meson-tests

Origen: materialización del runtime histórico `inih-meson-tests` sin los cuatro enlaces simbólicos de `deps/venv`. Intervención: `TMPDIR`, baseline `/tmp`, treatment `/var/tmp`, dos repeticiones por brazo. Docker sin red durante las ejecuciones, privilegios descartados y controles básicos de aislamiento; la inspección de configuración no acredita seguridad perfecta ante código adversarial.

Resultados verificados por el investigador en un único host:

- 4/4 ejecuciones con exit code cero, sin timeout;
- 8/8 artefactos verificados contra SHA-256 registrados;
- artefactos reproducibles dentro de cada brazo;
- artefactos idénticos entre brazos en los oráculos seleccionados;
- 4 comparaciones F3 `identical` y 4 adaptaciones F9.6 `observed_only`;
- **ninguna** conclusión causal sobre ausencia de efectos de TMPDIR en general.

Archivos de evidencia versionados con esta actualización:

- `experiments/m4/g3/PILOT_ENGINEERING_EVIDENCE.json`;
- `experiments/m4/g3/F3_F9_BRIDGE_ENGINEERING_EVIDENCE.json`;
- `experiments/m4/g3/G3_EVIDENCE_INDEX.json`, anclas SHA-256 de los dos expedientes, del script archivado, de las reglas de parada aún provisionales y de la imagen Docker local;
- `experiments/m4/g3/STOP_RULES_DRAFT.md`, **borrador**, no congelado y no modificado por el paquete.

Datos primarios locales no incorporados al repositorio por defecto:

- `~/envmorph-g3-inih-q3hzpaoe/g3_pilot_61rs5xwa`, intento bloqueado por `--mount` inválido, rc 125;
- `~/envmorph-g3-inih-q3hzpaoe/g3_pilot_dlhz9q16`, intento bloqueado por falta de compilador C++, rc 1;
- `~/envmorph-g3-inih-q3hzpaoe/g3_pilot_2p7z08gb`, piloto exitoso;
- `~/envmorph-g3-inih-q3hzpaoe/g3_f3_bridge_sje0i01b`, comparaciones F3 y salidas F9.6.

Los hashes garantizan coincidencia con bytes fijados, no autenticidad de ejecución ni independencia. El repositorio **no** contiene por sí solo todas las evidencias primarias de G3 ni el runtime F8. Para una auditoría completa externa será necesaria una entrega separada de los datos primarios con manifiesto de integridad, las dependencias y un segundo host.

#### Incidencias de ingeniería conservadas

1. Validación inicial de wheel Ninja asumía una ruta interna incorrecta; corregida para reconocer `ninja-1.13.2.data/scripts/ninja` en el wheel verificado.
2. Docker rechazó `--mount type=bind,...,rw`, exit 125; se eliminó el elemento `rw` inválido del arnés.
3. La imagen derivada inicial tenía gcc, no g++; Meson falló en la detección del compilador C++; se creó `envmorph-g3-inih-cpp:local` desde la imagen base. Esta derivación instaló paquetes `g++` desde repositorios Ubuntu con red durante **build**, mientras que la ejecución del piloto quedó offline. La imagen es una etiqueta local, no un digest remoto inmutable; el identificador real se registra en el índice.
4. Tras corrección, 4/4 ejecuciones rc=0; ningún resultado previo se eliminó.

#### Estado de gates

| Gate | Resultado |
|---|---|
| G3-A, preparación tipada y materialización | PASS |
| G3-B, ejecución aislada de piloto externo | PASS de ingeniería |
| G3-B, F3 → F9.6 sobre artefactos primarios | PASS de ingeniería |
| G3-B, observador F9 en la campaña G3 | NOT_VERIFIED |
| G3-B, contrato F6-v2 sobre G3 | NOT_EVALUATED |
| G3-C, campaña prospectiva y comparación equitativa | NOT_STARTED, protocolo sin congelar |
| G3-D, congelación y auditoría independiente | NOT_STARTED |
| Paper 1, envío bajo alcance actual | NO_GO |

#### Punto de decisión inmediato

Antes de nuevas ejecuciones científicas: inspeccionar `STOP_RULES_DRAFT.md`, fijar cohortes, intervenciones, oráculos, holdout, comparador, presupuestos, exclusiones y criterios de parada. Ver `experiments/m4/g3/G3_C_PROTOCOL_REVIEW_DRAFT.md` (plantilla no congelada). No tratar F10 ni el piloto inih como muestras independientes de la campaña confirmatoria. El observador F9 completo es complementario al camino crítico del Paper 1 y no debe convertirse en trabajo indefinido.
