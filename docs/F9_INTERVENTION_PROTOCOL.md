### Protocolo de intervenciones controladas F9

#### Objetivo

F9.4 define cuándo una dependencia observada puede convertirse en una intervención experimental válida.

La regla central es:

```text
observed_dependency
        |
        v
eligibility check
        |
        v
controlled intervention
```

F9.4 no clasifica todavía una dependencia como `behaviorally_relevant` ni como `observed_only`.

#### Precondición de observación

Una intervención confirmatoria solo es elegible si su `dependency_key` ya existe como nodo `dependency` en un grafo F9.3 del mismo `source_run_id`.

No se permite inventar una dependencia durante la fase de tratamiento.

#### Selección ciega al resultado

Todo plan debe declarar:

```text
result_blind_selection = true
```

y no puede contener campos como:

```text
expected_result
expected_behavior
expected_difference
```

La selección del tratamiento se realiza antes de observar el resultado conductual de esa intervención.

#### Confirmaciones

Toda intervención confirmatoria requiere:

```text
confirmations >= 2
```

F9.4 prueba esta regla sobre micro-workloads.

F9.5 decidirá cómo convertir confirmaciones y oráculos en estados conductuales.

#### Mecanismos implementados

##### environment_override

Aplicable a:

```text
kind = environment
resource_type = environment_variable
operation = getenv
```

El runner crea una copia aislada del workspace y modifica únicamente la variable objetivo en el entorno del proceso hijo.

##### isolated_text_file_replacement

Aplicable a:

```text
kind = file
resource_type = path
```

La ruta debe estar bajo la etiqueta lógica:

```text
@WORKSPACE@/
```

El runner crea una copia temporal del workspace y reemplaza únicamente el contenido UTF-8 del archivo objetivo.

No se modifican archivos del host.

#### Mecanismos no implementados en F9.4

F9.4 no implementa todavía:

- sustitución de ejecutables;
- intervención sobre archivos del sistema;
- red;
- clock o vDSO;
- mount namespaces;
- symlinks como mecanismo experimental;
- binarios arbitrarios;
- cambios globales de locale o timezone fuera del entorno aislado.

Esos casos deben quedar como `out_of_scope` o requerir un protocolo posterior antes de ejecutarse.

#### Baseline y treatment

`baseline_value` y `treatment_value` deben ser diferentes.

El runner materializa exactamente uno de los dos valores por ejecución.

F9.4 valida que el mecanismo aplique el valor solicitado y que dos repeticiones del mismo variant produzcan una aplicación reproducible.

Esto es ground truth del mecanismo, no evidencia de relevancia conductual.

#### Oracle

Todo plan contiene `oracle_ref`.

F9.4 solo exige que la referencia exista como declaración no vacía.

F9.5 y F9.6 definirán la evaluación conductual y su integración con los oráculos F3.

#### Aislamiento

Los mecanismos F9.4 usan:

```text
workspace_copy
```

Cada ejecución obtiene una copia temporal independiente.

Los argumentos `@WORKSPACE@` y `@WORKSPACE@/...` se materializan dentro de esa copia temporal. Las rutas con `..` se rechazan.

El runner no modifica el workspace fuente.

#### Elegibilidad de dependencias externas

Antes de F9.8 debe congelarse un manifiesto por workload con todas las dependencias candidatas y su estado:

```text
eligible
unavailable
out_of_scope
not_intervenable
```

La decisión debe realizarse antes de ejecutar treatments externos.

No se permite seleccionar solo dependencias que posteriormente produzcan divergencia.

#### Estado científico

Una intervención válida demuestra únicamente que:

1. la dependencia fue observada;
2. el mecanismo es compatible;
3. baseline y treatment están preregistrados;
4. la ejecución puede aislarse;
5. el tratamiento puede aplicarse reproduciblemente.

No demuestra todavía causalidad ni relevancia conductual.

#### Siguiente gate

F9.5 aplicará el protocolo sobre micro-workloads con ground truth para obtener por primera vez:

```text
behaviorally_relevant
observed_only
unresolved
```

sin ejecutar todavía el corpus F8.
