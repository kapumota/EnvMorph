### Baseline F0 de EnvMorph

#### Estado

La Fase 0 establece el primer baseline técnico verificable de EnvMorph a partir del núcleo heredado de FlowAttest y del workload literature-pipeline.

Este baseline no implementa todavía environmental metamorphic testing, portability envelopes, environmental contracts, equivalence oracles ni minimal causal environment.

#### Gates

El cierre local de F0 verificó:

- 25 tests unitarios Rust aprobados
- 37 checks de integración aprobados
- 19 tests del literature-pipeline aprobados
- build release aprobado
- formato Rust aprobado
- sintaxis Bash aprobada
- hardening de replay para rechazar rutas absolutas no mapeadas
- hardening de replay para rechazar traversal con `..`
- identidad activa de EnvMorph verificada
- workload canónico conservado byte a byte
- secret scan de alta confianza aprobado
- auditoría de archivos grandes y symlinks aprobada

#### Procedencia

Los hashes de los ZIP de origen se conservan en `SOURCE_ARTIFACTS.sha256`.

Los hashes del núcleo precursor se conservan en `FLOWATTEST_CORE_FILES.sha256`.

Los hashes del workload canónico se conservan en `LITERATURE_PIPELINE_FILES.sha256`.

#### Alcance científico

Los resultados experimentales históricos de FlowAttest se preservan únicamente como material heredado. No constituyen evidencia científica de EnvMorph.

Toda evidencia futura de EnvMorph deberá provenir de experimentos controlados y reproducibles ejecutados sobre la nueva infraestructura.

#### Limitaciones del baseline

El baseline F0 conserva comparación basada principalmente en igualdad de contenido y todavía no dispone de equivalencia estructural o semántica general.

El replay endurecido evita materializar rutas absolutas no mapeadas y traversal de directorios, pero no constituye todavía un sandbox de sistema operativo.

La instrumentación heredada sigue siendo explícita. El descubrimiento transparente de dependencias ambientales queda fuera de F0.
