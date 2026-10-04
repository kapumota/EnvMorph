### Arquitectura inicial

#### Núcleo

El núcleo Rust contiene el modelo de ejecución, captura ambiental, almacenamiento, hashing, comparación, replay, grafos e informes.

#### Workloads

Los workloads se mantienen separados del producto. `literature-pipeline/source` conserva el snapshot canónico y `literature-pipeline/harness` contiene instrumentación externa.

#### Experimentos heredados

Los experimentos de FlowAttest se preservan únicamente como material histórico y no forman parte de la evidencia científica de EnvMorph.
