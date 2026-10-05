### Extensión ambiental F8: tmpdir

#### Necesidad

El factor `tmpdir` estaba preregistrado como candidato y se activa solo después del freeze del corpus. La hipótesis técnica previa a la ejecución es que los workflows de build y test pueden usar almacenamiento temporal, directamente o a través de sus toolchains. En `fd` la revisión congelada declara `tempfile` como dependencia de desarrollo, y el workload `inih` ejecuta Meson, compilación y tests.

La activación no presupone que `tmpdir` produzca una divergencia. Un resultado `no_observable_effect` es válido.

#### Semántica F2

`tmpdir` se resuelve como una ruta absoluta existente que debe ser un directorio. Cuando está disponible, F2 establece `TMPDIR` para la variante. Una ruta relativa, inexistente o que no sea directorio se clasifica como capacidad no disponible.

#### Contraste congelado

Baseline: `/tmp`

Treatment: `/var/tmp`

Los valores se fijan antes de ejecutar los workloads externos.

#### Cambio de software

La extensión modifica únicamente la resolución de capacidades de F2 para reconocer `tmpdir`. No añade un oráculo, no modifica F3-F7 y no altera upstream.

#### Verificación

`tests/f8_tmpdir_integration.sh` comprueba detección disponible/no disponible y que dos variantes reciben valores distintos de `TMPDIR`. Los tests heredados se conservan.
