### DependencyGraph v1

#### Objetivo

F9.3 transforma eventos neutrales F9.2 en un grafo reproducible de dependencias observadas.

El grafo representa:

```text
process
   |
   +-- observed_dependency --> dependency
                                  |
                                  +-- targets_resource --> resource
```

Cuando el proceso padre y el proceso hijo aparecen en el mismo run también puede registrarse:

```text
process -- parent_of --> process
```

#### Tipos de nodo

`process`

Representa un PID observado dentro de un `run_id`.

Su identidad actual es:

```text
run_id + pid
```

F9.3 no afirma que esta identidad sea suficiente para ejecuciones arbitrariamente largas con reutilización de PID. Esa limitación se conserva explícitamente antes de la validación externa.

`dependency`

Representa una dependencia operacional F9.2 identificada por `dependency_key`.

Una dependencia conserva:

- kind;
- resource_type;
- resource;
- operation.

`resource`

Representa el recurso objetivo sin la operación.

Ejemplos:

```text
path:/etc/localtime
environment_variable:TZ
path:/usr/bin/sort
```

#### Tipos de arista

`observed_dependency`

Une un proceso con una dependencia que produjo uno o más eventos.

La arista agrega:

- occurrence_count;
- event_ids;
- sources;
- results;
- source_sequences.

La agregación no elimina los eventos F9.2.

`targets_resource`

Une una dependencia con exactamente un recurso.

`parent_of`

Une dos procesos solo cuando ambos PIDs aparecen como procesos observados dentro del mismo run y el PPID del hijo coincide con el PID del padre.

No se crea un nodo padre implícito cuando el PPID no tiene eventos propios.

#### Fuentes múltiples

Si ptrace y el interposer observan eventos del mismo PID, F9.3 puede asociarlos al mismo nodo `process`.

Esto no crea un orden total entre las fuentes.

El grafo conserva secuencias separadas por fuente dentro de `observed_dependency`.

#### Semántica

DependencyGraph v1 es un grafo de evidencia observacional.

No es todavía:

- un grafo causal;
- un grafo de proveniencia completa;
- una prueba de relevancia conductual;
- una prueba de que un recurso observado afectó el output.

Esas conclusiones requieren intervenciones posteriores.

#### Identidad de paths

F9.3 consume el recurso normalizado por F9.2 y no realiza una segunda canonicalización.

Por tanto, las limitaciones de symlinks, mounts y namespaces de F9.2 permanecen vigentes.

#### Determinismo

Dados los mismos eventos normalizados, el JSON del grafo debe ser byte-identical.

Nodos y aristas se ordenan por sus identificadores estables.

#### Criterio de F9.3

F9.3 cierra cuando:

- el esquema está versionado;
- el builder es determinista;
- eventos repetidos legítimos se agregan sin perder event IDs;
- múltiples fuentes pueden converger en un proceso sin inventar orden global;
- el árbol padre-hijo se materializa solo con evidencia observada;
- ground truth por fixtures y micro-workload real pasa;
- el corpus F8 no se ejecuta.
