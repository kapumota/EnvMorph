### Procedencia de EnvMorph

EnvMorph parte de dos artefactos locales verificados por SHA-256.

#### FlowAttest

FlowAttest se conserva como precursor técnico. Su núcleo Rust aporta captura de ejecuciones, almacenamiento, comparación, replay, grafos e informes.

No se consideran sus experimentos anteriores como evidencia científica de EnvMorph.

#### literature-pipeline

El literature-pipeline independiente se conserva como snapshot canónico del primer workload heredado. El snapshot no se modifica dentro de `workloads/literature-pipeline/source`.

La instrumentación creada anteriormente para FlowAttest se conserva aparte en `workloads/literature-pipeline/harness/legacy`.

#### Renombres mecánicos del núcleo

- `env.rs` pasa a `environment.rs`
- `stage.rs` pasa a `runner.rs`
- `diff.rs` pasa a `compare.rs`
- `sha256.rs` pasa a `hash.rs`

Estos renombres no introducen funcionalidad científica nueva.
