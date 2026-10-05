### DependencyEvent v1

#### Objetivo

F9.2 introduce una representación neutral para desacoplar la evidencia científica de los formatos específicos de los backends de observación.

Las fuentes iniciales son:

- `agentguard-derived-ptrace-observer`;
- `ld_preload_getenv_observer`.

`strace -ff` permanece como referencia de validación, no como fuente primaria de eventos F9.

#### Regla principal

Los eventos crudos se preservan por fuente. La normalización no inventa un orden causal global entre fuentes distintas.

Cada evento normalizado conserva:

- fuente;
- secuencia dentro de la fuente;
- PID y PPID cuando están disponibles;
- clase de dependencia;
- recurso;
- operación;
- resultado;
- errno cuando corresponde.

#### Identidades

`event_id` identifica una observación concreta y se deriva de:

```text
run_id
source
source_sequence
pid
kind
resource_type
resource
operation
result
errno
```

`dependency_key` identifica una dependencia semántica y se deriva de:

```text
kind
resource_type
resource
operation
```

Por diseño, `dependency_key` no incluye PID ni fuente. Esto permite reconocer la misma dependencia cuando una fuente compatible la observa más de una vez.

#### Deduplicación

La normalización aplica dos reglas diferentes.

Primero, un duplicado exacto de captura con la misma fuente, secuencia, PID y contenido semántico se elimina del stream normalizado y se contabiliza.

Segundo, accesos repetidos legítimos con secuencias diferentes se conservan como eventos distintos.

La agregación por `dependency_key` no elimina esos eventos. Solo construye un resumen de ocurrencias, procesos, fuentes y resultados.

#### No equivalencias implícitas

F9.2 no considera equivalentes automáticamente:

```text
getenv("TZ")
open("/etc/localtime")
```

aunque ambos puedan estar relacionados con timezone.

Tampoco considera equivalentes una variable de entorno y un archivo solo porque participen en el mismo fenómeno ambiental.

#### Paths

La normalización de paths es lexical.

No se usa `realpath` y no se resuelven symlinks, mounts o namespaces. Los prefijos locales de workspace pueden sustituirse por una etiqueta lógica declarada para evitar introducir rutas específicas de una máquina en evidencia compacta.

#### Orden

`source_sequence` solo tiene significado dentro de una fuente.

F9.2 no construye un reloj global entre `ptrace` y `LD_PRELOAD`.

La composición causal entre fuentes queda fuera de F9.2.

#### Estados

Para archivos y ejecutables:

```text
success
error
```

Para variables de entorno:

```text
present
missing
```

#### Alcance

F9.2 valida esquema, normalización, deduplicación exacta y agregación de dependencias usando fixtures y micro-workloads.

El corpus F8 no se ejecuta.
