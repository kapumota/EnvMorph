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

F0 todavía no implementa:

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
