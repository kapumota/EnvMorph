### Roadmap técnico y científico

#### Principio

EnvMorph se desarrolla como un único software end-to-end.

Las capacidades se incorporan por fases y cada paper debe utilizar evidencia generada por ese mismo sistema.

#### F0. Baseline técnico

Estado: cerrado.

Incluye migración desde FlowAttest, workload heredado, replay endurecido, gates reproducibles y procedencia estable.

#### F1. Modelo ambiental

Estado: cerrado en F1.3.

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

Estado: cerrado.

F2 resuelve disponibilidad real de locale, timezone e implementación AWK, ejecuta variantes disponibles en workspaces separados y conserva un bundle reproducible por campaña.

Entregables:

```text
envmorph capabilities EXPERIMENT.toml
envmorph explore EXPERIMENT.toml --output DIR --workdir DIR -- CMD [ARGS...]
```

Una variante no disponible y una ejecución fallida son estados diferentes.

#### F3. Oráculos de equivalencia

Estado: cerrado.

F3 separa explícitamente ejecución, comparación y equivalencia.

Entregables:

```text
ByteOracle
TextOracle
JsonOracle
CsvOracle
envmorph compare-artifacts
```

Resultados comunes:

```text
identical
equivalent
different
missing
error
```

La salida puede ser humana o JSON y puede persistirse con `--output`. Los oráculos operan sobre artefactos ya producidos por F2 y no modifican el ejecutor.


#### F4. Propagación y absorción

Estado: cerrado.

F4 consume resultados de equivalencia F3 y construye trazas ordenadas de diferencias observables.

Entregables:

```text
PropagationAnalyzer
PropagationTrace
PropagationReport
envmorph analyze-propagation
```

Estados por observación:

```text
stable
divergence_start
propagated
absorbed
unresolved
```

Estados globales del trazado:

```text
stable
absorbed
persistent
unresolved
```

La primera divergencia es un inicio observable, no una inferencia causal. `missing` y `error` interrumpen la continuidad. F4 no implementa minimización causal.

#### F5. Entorno causal mínimo

Estado: siguiente fase.

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


#### F5. Minimal Causal Environment

Estado: cerrado.

F5 ejecuta intervenciones ambientales mediante F2, deriva equivalencia con F3, consume firmas de propagación F4 y realiza búsqueda exacta por cardinalidad para encontrar todos los conjuntos mínimos de factores que reproducen la firma objetivo.

La minimalidad se limita al espacio de factores declarado y expresa suficiencia operacional bajo confirmaciones controladas.


#### F6. Environmental Contracts

Estado: cerrado.

F6 deriva y verifica contratos ambientales limitados por evidencia F5.
