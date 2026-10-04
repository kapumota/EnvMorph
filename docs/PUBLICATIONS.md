### Plan de publicaciones

#### Regla

No se escribirá un paper antes de que exista evidencia reproducible suficiente para sus preguntas de investigación.

Los papers no definen módulos aislados. Cada trabajo consume capacidades ya integradas en EnvMorph.

#### Paper 1

Título de trabajo:

`Environmental Metamorphic Contracts for Legacy Unix Workflows`

Gate mínimo:

- F1 a F6 cerradas
- contrato inferido desde variantes controladas
- validación sobre variantes no usadas durante la inferencia
- al menos un workload externo además del `literature-pipeline`
- protocolo experimental reproducible

Contribución principal: inferencia y validación de contratos ambientales conductualmente relevantes.

#### Paper 2

Título de trabajo:

`Mining Portability Envelopes for Unix Workflows`

Gate mínimo:

- F7 cerrada
- múltiples factores
- interacciones entre factores
- minimización causal
- varios workloads externos
- evaluación del costo de exploración

Contribución principal: minería de regiones de portabilidad y sensibilidad de interacción.

#### Paper 3

Título de trabajo:

`How Portable Are Real-World Unix Workflows?`

Gate mínimo:

- F8 cerrada
- corpus externo definido antes del análisis final
- protocolo de selección documentado
- resultados agregados sobre sensibilidad, interacciones y absorción
- amenazas a la validez explícitas

Contribución principal: estudio empírico de portabilidad ambiental en workflows reales.

#### Paper 4

Título de trabajo:

`Not Every Hidden Dependency Matters: Measuring the Behavioral Hermeticity Gap in Real-World Workflows`

Gate mínimo:

- F9 cerrada
- observación transparente de dependencias
- intervenciones ambientales controladas
- separación entre dependencia observada y dependencia causalmente relevante
- workloads externos suficientes para evitar conclusiones basadas en casos propios

Contribución principal: medición de la brecha entre dependencias observadas y dependencias que afectan el comportamiento.

#### Trabajo sobre absorción

`Change Propagation and Absorption Boundaries in Legacy Workflows` se mantiene como línea candidata.

Solo se separará como paper independiente si F4 produce evidencia suficiente para una contribución propia. En caso contrario, propagación y absorción formarán parte de Paper 2 o Paper 3.
