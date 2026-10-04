### Roadmap técnico y científico

#### Principio

EnvMorph se desarrolla como un único software end-to-end.

Las capacidades se incorporan por fases y cada paper debe utilizar evidencia generada por ese mismo sistema.

#### F0. Baseline técnico

Estado: cerrado.

Incluye migración desde FlowAttest, workload heredado, replay endurecido, gates reproducibles y procedencia estable.

#### F1. Modelo ambiental

Objetivo: representar factores, valores y variantes ambientales de forma explícita.

Primer alcance:

- locale
- timezone
- implementación AWK

Entregable mínimo:

```text
envmorph plan experiment.toml
```

El plan debe ser determinista y todavía no necesita ejecutar la matriz.

#### F2. Exploración end-to-end

Objetivo: ejecutar variantes ambientales controladas sobre un workflow real.

Entregable:

```text
envmorph baseline -- ./pipeline.sh
envmorph explore experiment.toml -- ./pipeline.sh
```

Cada ejecución debe conservar asignación de factores, entorno, herramientas, artefactos, estado de salida y observaciones por etapa.

#### F3. Oráculos de equivalencia

Objetivo: distinguir igualdad de bytes, igualdad estructural y equivalencia semántica configurable.

Primeros oráculos:

- byte
- texto normalizado
- JSON
- CSV

La interfaz debe permitir incorporar comparadores específicos sin acoplarlos al ejecutor.

#### F4. Propagación y absorción

Objetivo: medir cómo una perturbación ambiental atraviesa etapas y dónde deja de ser observable.

Conceptos:

- first divergence
- propagation depth
- blast radius
- absorption boundary

#### F5. Entorno causal mínimo

Objetivo: minimizar el conjunto de factores necesario para reproducir una divergencia.

La primera implementación debe favorecer minimización por subconjuntos y memoization antes que técnicas más complejas.

#### F6. Contratos ambientales

Objetivo: inferir un contrato compacto a partir de ejecuciones observadas y validarlo sobre variantes no usadas durante la inferencia.

Entregables:

```text
envmorph infer
envmorph check envmorph.toml -- ./pipeline.sh
```

F6 constituye el primer MVP científico completo.

#### F7. Portability envelopes

Objetivo: caracterizar regiones del espacio ambiental donde el workflow conserva el comportamiento definido por sus oráculos.

Debe incluir interacciones entre factores y no limitarse a análisis univariado.

#### F8. Corpus externo

Objetivo: evaluar EnvMorph sobre workloads que no hayan sido escritos para el proyecto.

Prioridades:

- workflows Unix reales
- pipelines de build
- ETL
- scientific workflows
- automatización CI
- bugs históricos documentados

F8 convierte el sistema en una plataforma experimental end-to-end con evidencia externa.

#### F9. Hermeticidad conductual

Objetivo: distinguir dependencia ambiental observada de dependencia conductualmente relevante.

La observación transparente mediante ptrace u otras técnicas solo se incorporará si sirve a esta pregunta y después de auditar cualquier reutilización de AgentGuard-FastPath.

#### Definición de software fundamental

EnvMorph se considera end-to-end fundamental cuando integra, al menos:

```text
baseline
plan
explore
equivalence
propagation
causal minimization
contract inference
contract check
portability envelope
external workloads
reproducible experiment bundle
```
