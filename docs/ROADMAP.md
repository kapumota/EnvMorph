### Roadmap de EnvMorph

#### Principio de cierre

Una fase cerrada significa que su implementación, semántica y gates internos están completos. No implica por sí sola validez externa. F8 es el primer gate explícito de generalización empírica.

#### F0. Baseline técnico y documental

Estado: cerrado.

Congela procedencia, fixture canónico, replay y baseline técnico sin redefinir el workload heredado.

#### F1. Modelo ambiental y planificación

Estado: cerrado.

Modela factores, especificaciones TOML y planificación determinista de variantes.

#### F2. Ejecución ambiental

Estado: cerrado.

Resuelve capacidades reales, ejecuta variantes en workspaces separados y distingue indisponibilidad de fallo de ejecución.

#### F3. Equivalencia de artefactos

Estado: cerrado.

Separa ejecución de equivalencia y aporta oráculos explícitos para byte, texto, JSON y CSV.

#### F4. Propagación y absorción

Estado: cerrado.

Localiza la primera divergencia observable y clasifica continuidad, propagación y absorción sin atribuir causalidad física.

#### F5. Minimal Causal Environment

Estado: cerrado.

Realiza búsqueda exacta por cardinalidad dentro del conjunto declarado y reporta todos los conjuntos mínimos que reproducen la firma F4. La interpretación es suficiencia operacional delimitada.

#### F6. Environmental Contracts

Estado: cerrado.

Convierte evidencia F1 a F5 en contratos versionables con estados `satisfied`, `violated`, `out_of_scope` e `invalid`, sin extrapolar a valores no observados.

#### F7. Portability Envelopes

Estado: cerrado.

Enumera el producto cartesiano baseline/treatment de F6, registra cobertura decidida y fronteras entre celdas vecinas. Un envelope completo solo es exhaustivo dentro del contraste finito declarado.

#### F8. External Corpus and Validation

Estado: cerrado.

Debe validar F2 a F7 sobre entre tres y cinco workloads externos, independientes de EnvMorph, seleccionados mediante `docs/F8_PROTOCOL.md` antes de observar resultados. F8 debe conservar casos positivos, negativos, indisponibles y fuera de alcance.

La extensión de factores y oráculos solo se permite cuando una necesidad del corpus seleccionado la justifique y quede registrada antes de ejecutar la matriz experimental afectada.


Validación externa: corpus seleccionado y congelado antes de ejecutar, adapters separados de upstream, matriz preregistrada, evidencia F2-F7 materializada, resultados negativos preservados y held-out evaluado sin reajuste post-hoc.

#### F9. Behavioral Hermeticity

Estado: extensión posterior, fuera del camino crítico de Paper 1.

Puede estudiar dependencias ambientales no declaradas mediante observación de sistema, potencialmente con tracing de llamadas al sistema. No debe comenzar hasta cerrar F8 y realizar una revisión específica frente a trabajo de hermeticidad y localización causal basada en tracing.

#### Relación provisional con publicaciones

Paper 1 se apoya principalmente en F2 a F6 y requiere F8 como validación externa. F7 puede aparecer como análisis complementario, pero no debe ser condición para defender el claim central de contratos ambientales.

Paper 2 puede estudiar envelopes, interacciones y cobertura en espacios ambientales más ricos.

Paper 3 puede convertirse en un estudio empírico de sensibilidad ambiental si F8 produce un corpus suficientemente diverso y reproducible.

Paper 4 queda condicionado a que F9 demuestre novedad separable respecto de tracing, hermeticidad de builds y localización causal existente.
