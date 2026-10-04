### Baseline F0 de EnvMorph

#### Estado

F0 está cerrado como baseline técnico verificable.

#### Gates cerrados

- 25 tests unitarios Rust aprobados
- 37 checks de integración aprobados
- 19 tests del `literature-pipeline` aprobados
- `cargo fmt --check` aprobado
- build release aprobado
- sintaxis Bash aprobada
- identidad activa de EnvMorph verificada
- workload canónico preservado byte a byte
- secret scan de alta confianza aprobado
- auditoría de archivos grandes y symlinks aprobada

#### Hardening de replay

F0 rechaza rutas lógicas absolutas no mapeadas y traversal con componentes `..` durante la materialización de replay.

Esta protección no equivale a un sandbox de sistema operativo.

#### Evidencia

F0 demuestra estabilidad técnica del baseline.

No declara resultados científicos sobre portabilidad, causalidad ambiental, contratos o hermeticidad conductual.
