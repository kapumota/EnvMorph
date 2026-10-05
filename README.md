### EnvMorph

EnvMorph es un prototipo de investigación en Rust para estudiar sensibilidad ambiental en workflows Unix heredados. Su pregunta central es: dado un workflow y un conjunto explícito de contrastes ambientales, qué cambios alteran el comportamiento observable, dónde aparece una divergencia, cómo se propaga o se absorbe y qué evidencia mínima permite formular un contrato ambiental delimitado.

#### Estado actual

F0 a F7 están cerradas. F8, External Corpus and Validation, es la siguiente fase y es bloqueante para sostener empíricamente el Paper 1. F9, Behavioral Hermeticity, queda fuera del camino crítico del Paper 1 y se trata como una extensión posterior condicionada a una revisión específica de novedad y viabilidad.

#### Cadena implementada

La cadena activa separa responsabilidades:

- `plan`, `capabilities` y `explore` modelan, resuelven y ejecutan variantes ambientales.
- `compare-artifacts` aplica oráculos explícitos de byte, texto, JSON y CSV.
- `analyze-propagation` clasifica aparición, propagación y absorción de diferencias observables.
- `minimize-environment` busca por cardinalidad todos los conjuntos ambientales mínimos que reproducen la firma observable configurada.
- `derive-contract` y `check-contract` producen y evalúan contratos ambientales delimitados por evidencia.
- `build-envelope` enumera el lattice finito baseline/treatment de un contrato y registra cobertura decidida y fronteras entre celdas vecinas.

La semántica distingue ejecución, comparación, equivalencia, propagación, suficiencia operacional, contrato y envelope. Un resultado `unavailable`, `missing`, `error`, `execution_failed` u `out_of_scope` no se transforma silenciosamente en evidencia negativa.

#### Alcance científico

EnvMorph no afirma causalidad física, portabilidad universal ni hermeticidad global. La búsqueda causal de F5 es exacta solo dentro del conjunto de factores declarado. Los contratos F6 y envelopes F7 conservan el alcance de los contrastes observados.

Variar entornos, comparar artefactos y minimizar cambios no se presentan por sí solos como contribuciones novedosas. El posicionamiento actual se centra en la composición de trazado de divergencia y absorción por etapa, conjuntos ambientales mínimos, contratos delimitados por evidencia y envelopes finitos, sujetos a validación externa en F8.

#### Limitación experimental actual

Hasta F7, el único workload canónico es `literature-pipeline`, heredado de FlowAttest y mantenido como fixture histórico. Por ello, F1 a F7 constituyen validación interna del mecanismo, no evidencia suficiente de generalización. F8 debe evaluar entre tres y cinco workloads externos seleccionados mediante un protocolo fijado antes de ejecutar experimentos.

#### Protocolo pre-F8

El protocolo congelado está en:

- `docs/F8_PROTOCOL.md`
- `experiments/f8/protocol.toml`
- `docs/PRIOR_ART.md`
- `docs/PAPER_SCOPE.md`
- `provenance/F8_PROTOCOL_FILES.sha256`

Cualquier cambio posterior a estos archivos debe registrarse como enmienda de protocolo y no puede reescribir retrospectivamente resultados ya observados.

#### Fixture histórico

`workloads/literature-pipeline/source/` es un snapshot histórico inmutable. Su contenido, estilo y documentación interna se preservan por procedencia y están cubiertos por `provenance/LITERATURE_PIPELINE_FILES.sha256`. No representan el estilo editorial actual de EnvMorph.

Las adaptaciones de compatibilidad para replay se realizan sobre copias temporales, nunca sobre el fixture canónico.

#### Construcción y pruebas

```text
cargo test
cargo build --release
```

Los gates de fase verifican además integración, workloads y hashes de procedencia. El cierre de una fase no equivale a validación externa de sus claims.

#### Próximo objetivo

F8 debe intentar falsar, no confirmar por construcción, los claims de F1 a F7 mediante workloads externos, resultados positivos y negativos, factores activados por necesidad del corpus y validación held-out de contratos.

#### F8: validación externa

F8 valida externamente la cadena F2-F7 sobre un corpus congelado antes de la ejecución. Tres de cuatro workloads completaron la cadena y los tres resultaron estables bajo los contrastes congelados. El único held-out produjo una abstención `out_of_scope`. La evidencia conserva resultados negativos y estados no evaluables sin sustituir workloads ni reajustar la matriz. Los detalles están en `docs/F8_RESULTS.md` y `audits/F8_FINAL_REPORT.txt`.
