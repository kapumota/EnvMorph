### EnvMorph

EnvMorph es una herramienta experimental en Rust para estudiar bajo qué variaciones del entorno un workflow Unix conserva su comportamiento, cuáles lo alteran y qué condiciones ambientales mínimas son necesarias para reproducirlo.

#### Estado

La Fase 0 está cerrada como baseline técnico verificable.

El núcleo heredado de FlowAttest conserva registro de ejecuciones, manifests, snapshots, comparación, replay, grafos e informes. El `literature-pipeline` se conserva como primer workload heredado y como corpus inicial para experimentos controlados.

El baseline F0 tiene:

- 25 tests unitarios Rust aprobados
- 37 checks de integración aprobados
- 19 tests del `literature-pipeline` aprobados
- replay endurecido contra rutas absolutas no mapeadas y traversal con `..`
- workload canónico preservado byte a byte

F1 incorpora un modelo ambiental tipado, especificaciones experimentales TOML versionadas y `envmorph plan` para construir matrices deterministas de variantes. F1 todavía no ejecuta esas variantes.

#### Objetivo end-to-end

El producto final debe cubrir una cadena completa:

```text
baseline
  |
  v
plan de variantes ambientales
  |
  v
ejecuciones controladas
  |
  v
oráculos de equivalencia
  |
  v
propagación y absorción
  |
  v
minimización causal
  |
  v
inferencia de contrato
  |
  v
portability envelope
  |
  v
validación con envmorph check
```

#### Alcance actual

Después de F1, todavía no se implementan:

- environmental metamorphic testing
- portability envelopes
- environmental contracts
- equivalencia estructural o semántica general
- minimal causal environment
- descubrimiento transparente de dependencias ambientales

Estas capacidades se incorporarán por fases y deberán producir evidencia reproducible antes de sostener conclusiones científicas.

#### Principio de trabajo

EnvMorph no se plantea como otro sistema genérico de provenance ni como un workflow manager.

La investigación se centra en dependencias ambientales conductualmente relevantes, interacciones entre factores, propagación, absorción y condiciones mínimas de portabilidad.

Los resultados experimentales anteriores de FlowAttest se preservan como material histórico, pero no se presentan como evidencia científica de EnvMorph.

#### Roadmap

El plan técnico y científico está en `docs/ROADMAP.md`.

Las líneas de publicación y sus gates de evidencia están en `docs/PUBLICATIONS.md`.

#### F2.1: capacidades ambientales

F2.1 incorpora resolución de capacidades ambientales sin ejecutar el workflow.

```text
envmorph capabilities experiments/f1/literature-pipeline-plan.toml
```

La resolución distingue una capacidad disponible de una no disponible. Una herramienta, locale o zona horaria ausente no se interpreta como fallo conductual del workflow.

#### F2: ejecución ambiental end-to-end

F2 está cerrado como cadena de ejecución ambiental end-to-end.

```text
envmorph explore EXPERIMENT.toml --output DIR --workdir DIR -- CMD [ARGS...]
```

`explore` planifica variantes, resuelve capacidades, ejecuta únicamente las variantes disponibles y conserva un bundle por campaña.

Una capacidad ausente se registra como `unavailable`. Una ejecución iniciada que termina con código distinto de cero se registra como `execution_failed`. Estas categorías no se mezclan.

El bundle contiene `experiment.json`, un directorio por variante, `manifest.json`, `stdout.txt`, `stderr.txt`, una copia de trabajo y `hashes.sha256`.

F2 todavía no decide si dos ejecuciones son equivalentes. Esa responsabilidad comienza en F3.

#### F3: equivalencia de artefactos

F3 está cerrado como capa de equivalencia separada de la ejecución.

```text
envmorph compare-artifacts --oracle byte --left FILE --right FILE
envmorph compare-artifacts --oracle text --left FILE --right FILE --normalize-line-endings
envmorph compare-artifacts --oracle json --left FILE --right FILE --format json
envmorph compare-artifacts --oracle csv --left FILE --right FILE --output comparison.json --format json
```

La ejecución F2 produce artefactos. F3 los compara mediante un oráculo explícito. El ejecutor no decide equivalencia.

Los resultados comunes son `identical`, `equivalent`, `different`, `missing` y `error`.

`ByteOracle` exige identidad byte a byte. `TextOracle` solo aplica normalizaciones solicitadas explícitamente. `JsonOracle` ignora el orden de claves de objetos, conserva el orden de arrays y mantiene todos los campos significativos. `CsvOracle` compara filas, columnas y celdas, con orden de filas significativo por defecto.

La salida humana y la salida JSON registran oráculo, artefacto izquierdo, artefacto derecho, resultado, razón y metadatos relevantes.
