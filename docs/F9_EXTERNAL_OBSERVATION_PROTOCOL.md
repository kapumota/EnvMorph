### Protocolo de observación externa F9.8A

#### Separación temporal

F9.8A usa dos commits.

El primer commit congela este protocolo y las herramientas de censo.

Solo después se ejecutan los cuatro workloads externos.

El segundo commit congela el censo observado.

Esto permite demostrar que las reglas de adquisición y exclusión preceden a los resultados externos.

#### Corpus

Se reutilizan exactamente los cuatro workloads congelados en F8:

```text
csvkit-realdata
jq-build-check
fd-build-completions
inih-meson-tests
```

No se sustituyen workloads por sus resultados.

#### Entorno de referencia

La observación usa los valores baseline ya congelados en:

```text
experiments/f8/matrix_manifest.toml
```

Los bindings son los mismos de F2:

```text
locale   -> LC_ALL
timezone -> TZ
tmpdir   -> TMPDIR
```

F9.8A no selecciona nuevos tratamientos.

#### Dos ejecuciones por workload

Cada workload se ejecuta sobre dos copias independientes del runtime F8 preparado:

```text
unobserved baseline
observed baseline
```

La segunda ejecución usa:

```text
agentguard-derived-ptrace-observer
ld_preload_getenv_observer
```

No se ejecuta treatment.

#### Efecto del observador

Los artefactos producidos por las dos ejecuciones se comparan con el trace y los oráculos congelados en F8 mediante F4.

Mapeo:

```text
stable               -> preserved
absorbed/persistent  -> altered
unresolved           -> unresolved
```

`altered` no se oculta ni provoca sustitución del workload.

#### Exclusiones instrumentales

La instrumentación introduce recursos que no pertenecen al workload.

Antes del grafo se excluyen solamente por identidad exacta:

```text
ruta exacta de la biblioteca LD_PRELOAD
ruta exacta del log getenv
variable LD_PRELOAD
variable ENVMORPH_F9_INTERPOSE_LOG
```

No se usan regex de contenido, resultados F8 ni comportamiento observado para excluir dependencias.

Todas las demás dependencias normalizadas permanecen en el grafo.

#### Censo

Para cada workload completado, el censo contiene todos los nodos `dependency` del grafo F9.3 posterior a las exclusiones instrumentales exactas.

F9.8A no asigna:

```text
eligible
not_intervenable
out_of_scope
unavailable
```

Esa decisión pertenece a F9.8B.

Tampoco crea planes, treatments o clasificaciones conductuales.

#### Workloads fallidos

Los estados se conservan:

```text
execution_failed
observer_failure
completed
```

Un workload fallido no genera candidatos ficticios.

#### Evidencia

Logs, eventos y grafos completos quedan bajo:

```text
.envmorph/f9-8a-evidence
```

El commit del censo conserva hashes, conteos y la lista completa de dependencias observadas.

#### No claims

F9.8A no afirma:

- observación completa;
- causalidad;
- relevancia conductual;
- prevalencia externa;
- orden total entre fuentes.

#### Siguiente gate

F9.8B asignará elegibilidad y congelará un plan o una abstención para cada dependencia censada antes de ejecutar cualquier treatment.
