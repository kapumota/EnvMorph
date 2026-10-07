### EnvMorph

[![CI](https://github.com/kapumota/EnvMorph/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kapumota/EnvMorph/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/kapumota/EnvMorph)](https://github.com/kapumota/EnvMorph/releases)
[![License](https://img.shields.io/github/license/kapumota/EnvMorph)](LICENSE)

EnvMorph es un prototipo de investigación para estudiar sensibilidad ambiental,
portabilidad y hermeticidad conductual en workflows Unix mediante contrastes
ambientales explícitos y evidencia reproducible.

#### Estado actual

F0 a F10 están cerradas como fases experimentales.

La versión `v0.1.0` conserva el congelamiento científico F9.10:

```text
29cc796 Cerrar científicamente F9 y congelar release
```

F10 es trabajo científico posterior a `v0.1.0` y no modifica ni reinterpreta
los resultados congelados de F8 o F9.

#### Arquitectura del artefacto

EnvMorph es actualmente un artefacto híbrido:

```text
Rust core F0-F7
+
Python/C experimental harness F9
+
Python/shell reproduction harness F10
```

El CLI Rust no expone todavía la cadena completa F9/F10. La frontera exacta
está documentada en `docs/ARTIFACT_BOUNDARY.md`.

#### F8: validación externa result-blind

F8 seleccionó cuatro workloads externos antes de observar resultados EnvMorph.

Resultado:

- 4 workloads seleccionados;
- 3 de 4 evaluables a través de F2-F7;
- 3 workloads estables;
- 0 diferencias observables;
- 1 held-out intentado;
- 0 predicciones held-out decididas;
- 1 abstención `out_of_scope`;
- sin sustitución de workloads por resultado;
- sin retuning post-hoc.

F8 es un resultado científicamente negativo. No demuestra ausencia universal
de sensibilidad ni capacidad predictiva held-out.

Detalles:

- `docs/F8_RESULTS.md`
- `docs/F8_THREATS_TO_VALIDITY.md`
- `audits/F8_FINAL_REPORT.txt`

#### F9: observación e intervención

F9 combina un backend ptrace para operaciones de archivo y ejecución con una
fuente complementaria basada en interposición dinámica de `getenv`.

Resultado externo congelado:

```text
observed_candidates = 12717
eligible_candidates = 2
behaviorally_relevant = 0
observed_only = 2
decidable_candidates = 2
```

La baja elegibilidad no implica que las dependencias restantes sean
conductualmente irrelevantes.

La observación no es completa. En particular, el hook `getenv` no cubre todas
las formas de acceso a variables ambientales, y el backend ptrace observa
recursos del sistema, no necesariamente la variable ambiental que originó una
decisión dentro de libc.

Detalles:

- `docs/F9_SCIENTIFIC_CLOSURE.md`
- `docs/F9_REPRODUCIBILITY_AUDIT.md`
- `docs/F9_THREATS_TO_VALIDITY.md`
- `docs/ARTIFACT_BOUNDARY.md`

#### F10: validación de sensibilidad externa

F10-MVP introduce tres controles externos `known-positive` seleccionados antes
de observar el resultado de EnvMorph:

- GNU coreutils 9.4, `date`, factor `TZ`;
- GNU coreutils 9.4, `ls`, factor `QUOTING_STYLE`;
- CPython 3.12.3, factor `PYTHONIOENCODING`.

Los tres completaron el funnel:

```text
known_positive = 3
observed = 3
normalized = 3
intervenable = 3
eligible = 3
oracle_decidable = 3
behaviorally_relevant = 3
end_to_end_sensitivity = 1.0
```

Este 3/3 es una validación de sensibilidad sobre controles positivos, no una
estimación general de recall. F10-MVP no contiene controles negativos y no
estima especificidad.

F10 incorpora reproducción nativa y una ruta Docker basada en Ubuntu 24.04.

Detalles:

- `docs/F10_PROTOCOL.md`
- `docs/F10_RESULTS.md`
- `experiments/f10/benchmark.json`
- `experiments/f10/results.json`
- `reproduce/README.md`

#### Requisitos de Rust

El crate declara:

```text
rust-version = 1.85
```

El lockfile usa formato v4 y la dependencia `toml 1.1.6` declara Rust 1.85 como
mínimo.

#### Reproducibilidad

Gate público básico:

```bash
cargo fmt --check
cargo test --locked
cargo build --release --locked
sha256sum -c provenance/F9_10_FILES.sha256
```

Reproducción F10:

```bash
bash reproduce/f10-docker.sh /tmp/envmorph-f10-run
```

La evidencia cruda histórica de F8/F9 no está íntegramente incluida en el
repositorio público. Los artefactos congelados conservan hashes y procedencia,
pero no se afirma regeneración completa de F8/F9 desde un clone público.

#### Alcance científico

EnvMorph no afirma:

- observación completa de dependencias;
- causalidad física;
- recall poblacional de 1.0;
- especificidad demostrada por F10;
- prevalencia externa;
- representatividad del software Unix;
- portabilidad universal;
- hermeticidad universal;
- validación held-out positiva de contratos externos.

#### Estado editorial

El repositorio puede usarse para comenzar la redacción de papers.

El Paper 1 definido actualmente en `docs/PAPER_SCOPE.md` no está todavía listo
para envío porque su gate held-out no tiene una predicción decidida y F10 no es
una validación held-out de un contrato F6.

El estado exacto y el trabajo mínimo restante están en:

- `docs/PAPER_READINESS.md`

#### Citación

El repositorio incluye `CITATION.cff`.

Si utiliza EnvMorph en trabajo académico, cite el software mediante la
información publicada por GitHub en la sección `Cite this repository`.
