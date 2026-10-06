### EnvMorph

[![CI](https://github.com/kapumota/EnvMorph/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kapumota/EnvMorph/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/kapumota/EnvMorph)](https://github.com/kapumota/EnvMorph/releases)
[![License](https://img.shields.io/github/license/kapumota/EnvMorph)](LICENSE)

EnvMorph es un prototipo de investigación en Rust para estudiar sensibilidad ambiental, portabilidad y hermeticidad conductual en workflows Unix. Su pregunta central es: dado un workflow y un conjunto explícito de contrastes ambientales, qué cambios alteran el comportamiento observable, dónde aparece una divergencia, cómo se propaga o se absorbe y qué evidencia permite formular contratos ambientales delimitados.

#### Estado actual

F0 a F9 están cerradas.

La versión `v0.1.0` es la primera versión científica pública y corresponde al congelamiento científico F9.10:

```text
29cc796 Cerrar científicamente F9 y congelar release
```

F9.9 cerró la auditoría final de reproducibilidad. F9.10 no modificó resultados científicos, planes, factores, oráculos ni treatments externos.

#### Cadena implementada

La cadena activa separa responsabilidades:

- `plan`, `capabilities` y `explore` modelan, resuelven y ejecutan variantes ambientales.
- `compare-artifacts` aplica oráculos explícitos de byte, texto, JSON y CSV.
- `analyze-propagation` clasifica aparición, propagación y absorción de diferencias observables.
- `minimize-environment` busca por cardinalidad todos los conjuntos ambientales mínimos que reproducen la firma observable configurada.
- `derive-contract` y `check-contract` producen y evalúan contratos ambientales delimitados por evidencia.
- `build-envelope` enumera el lattice finito baseline/treatment de un contrato y registra cobertura decidida y fronteras entre celdas vecinas.
- F9 añade observación de dependencias, normalización de eventos, grafos de dependencias observadas, intervenciones controladas y clasificación conductual.

La semántica distingue ejecución, comparación, equivalencia, propagación, suficiencia operacional, contrato, envelope, observación y relevancia conductual. Un resultado `unavailable`, `missing`, `error`, `execution_failed`, `unresolved` u `out_of_scope` no se transforma silenciosamente en evidencia negativa.

#### F8: validación externa

F8 evaluó la cadena F2-F7 sobre un corpus externo seleccionado y congelado antes de observar resultados.

El cierre de F8 fue científicamente negativo:

- 4 workloads seleccionados,
- 3 de 4 workloads evaluables a través de F2-F7,
- 3 workloads estables,
- 0 diferencias observables,
- 1 held-out intentado,
- 0 predicciones held-out decididas,
- 1 abstención `out_of_scope`,
- sin sustitución de workloads por resultado,
- sin reajuste post-hoc.

F8 no demuestra prevalencia de sensibilidad ambiental ni generalización predictiva held-out. El resultado conserva explícitamente evidencia negativa y estados no evaluables.

Los detalles están en:

- `docs/F8_RESULTS.md`
- `docs/F8_THREATS_TO_VALIDITY.md`
- `audits/F8_FINAL_REPORT.txt`
- `provenance/F8_FINAL_FILES.sha256`

#### F9: hermeticidad conductual

F9 extiende la infraestructura con observación de dependencias e intervenciones controladas sobre dependencias observadas.

El resultado externo congelado en F9.10 es:

```text
observed_candidates = 12717
eligible_candidates = 2
behaviorally_relevant = 0
observed_only = 2
unresolved = 0
execution_failed = 0
observer_failure = 0
decidable_candidates = 2
decidability_among_observed = 0.00015726979633561374
relevance_among_decidable = 0.0
```

Las dos dependencias externas elegibles fueron clasificadas como `observed_only` bajo las intervenciones prerregistradas y los observables F4 congelados.

Este resultado está limitado al corpus externo, los valores de factores y los observables congelados. No implica que las restantes dependencias observadas sean conductualmente irrelevantes.

Los detalles están en:

- `docs/F9_SCIENTIFIC_CLOSURE.md`
- `docs/F9_REPRODUCIBILITY_AUDIT.md`
- `docs/F9_THREATS_TO_VALIDITY.md`
- `experiments/f9/external_intervention/metrics_summary.json`
- `provenance/F9_10_FILES.sha256`

#### Alcance científico

EnvMorph no afirma:

- observación completa de dependencias,
- causalidad física,
- prevalencia externa,
- portabilidad universal,
- hermeticidad universal,
- una métrica escalar de brecha de hermeticidad conductual,
- un orden total entre fuentes de observación.

La búsqueda causal de F5 es exacta únicamente dentro del conjunto de factores declarado. Los contratos F6 y envelopes F7 conservan el alcance de los contrastes observados.

Variar entornos, comparar artefactos, observar dependencias y minimizar cambios no se presentan por sí solos como contribuciones novedosas. El posicionamiento se centra en la composición de trazado de divergencia y absorción por etapa, conjuntos ambientales mínimos, contratos delimitados por evidencia, envelopes finitos y clasificación conductual mediante intervenciones controladas.

#### Fixture histórico

`workloads/literature-pipeline/source/` es un snapshot histórico inmutable. Su contenido, estilo y documentación interna se preservan por procedencia y están cubiertos por `provenance/LITERATURE_PIPELINE_FILES.sha256`.

Las adaptaciones de compatibilidad se realizan sobre copias temporales, nunca sobre el fixture canónico.

Algunos manifests, auditorías, fixtures legacy y drivers científicos congelados conservan rutas absolutas del host experimental original. Son artefactos históricos y no representan requisitos de ejecución del repositorio público.

#### Construcción y pruebas

Requisitos mínimos:

- Rust estable,
- Cargo,
- entorno Unix para los experimentos específicos que lo requieran.

Gate público básico:

```bash
cargo fmt --check
cargo test --locked
cargo build --release --locked
sha256sum -c provenance/F9_10_FILES.sha256
```

El workflow de GitHub Actions ejecuta el mismo gate sobre `ubuntu-latest`.

Los warnings de código no utilizado no cambian el resultado de los tests ni del build y se conservan en `v0.1.0` como deuda de ingeniería posterior al congelamiento científico.

#### Reproducibilidad y evidencia

El repositorio público incluye protocolos, resultados versionados, auditorías y manifests SHA-256.

La evidencia cruda de adquisición almacenada bajo `.envmorph` no forma parte del repositorio Git. Los hashes disponibles de esa evidencia se conservan como anclas de procedencia donde corresponde.

Después de la publicación de `v0.1.0`, el repositorio fue clonado desde GitHub en un directorio limpio y volvió a superar formato, tests, build release y verificación de `provenance/F9_10_FILES.sha256`.

#### Citación

El repositorio incluye `CITATION.cff`.

Si utiliza EnvMorph en trabajo académico, cite el software mediante la información publicada por GitHub en la sección `Cite this repository`.

#### Desarrollo posterior a v0.1.0

`v0.1.0` conserva el estado científico congelado de F9.10.

Los cambios posteriores en `main` se consideran release engineering, documentación o trabajo nuevo. No reescriben retrospectivamente resultados de F8 o F9.

Nuevos experimentos o claims científicos deben introducir un protocolo nuevo, una nueva fase o una nueva versión claramente separada del freeze `v0.1.0`.
