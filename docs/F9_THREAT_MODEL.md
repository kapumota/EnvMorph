### Threat model F9

#### Activos científicos

F9 protege la validez de:

- la identidad del workload;
- la identidad del runtime;
- el orden causal entre observación e intervención;
- los artefactos observables;
- la separación entre dependencia observada y relevancia conductual;
- la evidencia negativa.

#### Amenazas del observador

`observer_effect`

El tracer puede cambiar scheduling, timing, señales, comportamiento de procesos o timeouts.

`coverage_gap`

El backend puede no observar una dependencia real.

`semantic_overreach`

Un acceso observado puede interpretarse erróneamente como dependencia relevante.

`path_aliasing`

Symlinks, cwd, dirfd, mount namespaces y rutas relativas pueden producir identidades ambiguas.

`process_tree_loss`

Fork, clone, vfork, exec y procesos reparentados pueden escapar de una captura incompleta.

`in_memory_dependency`

Variables ambientales y datos cacheados pueden consumirse sin nuevas syscalls.

`vdso_gap`

Algunas fuentes de tiempo pueden resolverse sin syscall tradicional.

`privilege_bias`

Un backend que requiere root, CAP_BPF, ptrace_scope relajado u otra capacidad puede cambiar el entorno experimental o reducir reproducibilidad.

`enforcement_contamination`

Código reutilizado de una herramienta defensiva puede bloquear o reescribir operaciones y alterar el comportamiento que intenta medir.

#### Reglas de mitigación

- modo observador por defecto;
- no bloquear syscalls;
- registrar backend, versión, kernel y privilegios;
- separar event capture de clasificación;
- registrar eventos crudos antes de agregación;
- comprobar baseline con y sin observador para estimar observer effect;
- conservar estados unresolved;
- no imputar dependencia cuando la evidencia de resolución de path es ambigua;
- no declarar cobertura completa;
- no ejecutar intervención confirmatoria sin dependencia observada o control preregistrado.

#### Seguridad operacional

F9 no necesita ejecutar malware ni workloads hostiles.

Los workloads externos deben conservar las reglas de F8 sobre ejecución no destructiva y offline después de preparación, salvo protocolo posterior explícito.

No se relajan controles del host globalmente para facilitar tracing. Cualquier cambio de `ptrace_scope`, capability, namespace o permiso debe quedar registrado y ser reversible.
