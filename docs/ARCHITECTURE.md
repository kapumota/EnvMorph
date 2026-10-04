### Arquitectura de EnvMorph

#### Baseline F0

El núcleo Rust actual contiene:

- modelo de ejecuciones y etapas
- captura ambiental
- almacenamiento y snapshots
- hashing
- comparación entre ejecuciones
- replay
- grafos
- reportes

Esta infraestructura es el baseline técnico. Todavía no constituye el motor completo de environmental metamorphic testing.

#### Separación de responsabilidades

El producto se mantiene separado de sus workloads y del material histórico.

```text
src/
    núcleo activo de EnvMorph

workloads/
    workloads experimentales y harness externos

experiments/legacy-flowattest/
    material histórico del precursor técnico

provenance/
    procedencia estable y hashes del baseline
```

`workloads/literature-pipeline/source` es un snapshot canónico y no debe modificarse para instrumentarlo.

#### Arquitectura objetivo end-to-end

La evolución prevista es:

```text
Workflow
   |
   v
Baseline Runner
   |
   v
Environmental Model
   |
   v
Variant Planner
   |
   v
Controlled Executor
   |
   v
Equivalence Oracles
   |
   v
Behavior Comparison
   |
   +------------------+
   |                  |
   v                  v
Propagation       Absorption
   |                  |
   +--------+---------+
            |
            v
     Causal Minimizer
            |
            v
     Contract Inference
            |
            v
    Portability Envelope
            |
            v
       Contract Check
```

#### Regla arquitectónica

Cada capacidad científica debe integrarse en esa cadena end-to-end.

No se incorporarán módulos aislados únicamente para sostener un paper. Los papers deberán consumir evidencia producida por el sistema reproducible.
