### Protocolo F8: External Corpus and Validation

#### Estado del protocolo

Versión 1. Este protocolo se congela antes de seleccionar definitivamente y ejecutar el corpus F8. Sus hashes se registran en `provenance/F8_PROTOCOL_FILES.sha256`.

Una modificación posterior debe declararse como enmienda con fecha, motivo y workloads afectados. Los resultados previos a una enmienda no pueden reclasificarse retrospectivamente como confirmatorios.

#### Objetivo

Evaluar si la cadena F2 a F7 identifica y caracteriza sensibilidad ambiental en workflows Unix que no fueron construidos para EnvMorph.

F8 no tiene como objetivo maximizar el número de divergencias encontradas. Un resultado estable, no soportado, indisponible o fuera de alcance es evidencia válida y debe conservarse.

#### Preguntas experimentales

- RQ1: con qué frecuencia los contrastes ambientales declarados producen diferencias observables reproducibles en workloads externos.
- RQ2: cuando aparece una diferencia, en qué etapa surge y si se propaga o se absorbe.
- RQ3: qué conjuntos ambientales mínimos reproducen la firma observada dentro del espacio declarado.
- RQ4: hasta qué punto los contratos inferidos predicen correctamente variantes held-out no usadas para derivarlos.
- RQ5: qué cobertura decidida y regiones `out_of_scope` producen los envelopes finitos.

#### Tamaño y composición del corpus

Objetivo primario: cuatro workloads externos.

Rango permitido: entre tres y cinco workloads. Utilizar tres o cinco requiere justificar la desviación en el reporte F8 antes de interpretar resultados agregados.

Ningún workload puede haber sido escrito específicamente para EnvMorph ni por el autor de EnvMorph. El fixture `literature-pipeline` se mantiene solo como control interno histórico y no cuenta como workload externo.

#### Criterios de inclusión

Un candidato puede entrar al corpus si cumple todos los criterios siguientes:

- código fuente público y referencia de versión o commit fijable
- licencia que permita reproducción académica
- workflow ejecutable en un sistema Unix o Linux sin interacción gráfica
- comportamiento basado en shell, build, ETL, CI, transformación de archivos o utilidades Unix
- posibilidad de ejecutar offline después de preparar dependencias o fixtures
- al menos un artefacto final observable y, cuando sea viable, dos o más artefactos intermedios
- tiempo y recursos compatibles con repetición local
- procedimiento de preparación documentable sin modificar la semántica del proyecto
- procedencia independiente de EnvMorph

#### Criterios de exclusión

Se excluye un candidato si ocurre cualquiera de los siguientes casos antes de observar resultados EnvMorph:

- requiere servicios remotos no congelables durante cada ejecución
- requiere credenciales, datos privados o secretos
- necesita privilegios destructivos o modifica el host de manera no aislable
- no permite fijar una revisión reproducible
- su licencia impide el uso experimental previsto
- no produce artefactos observables adecuados para los oráculos disponibles o un perfil declarativo justificable
- el costo de ejecución impide las confirmaciones mínimas

Una vez aceptado un workload, no se elimina por producir cero divergencias.

#### Procedimiento de selección

Antes de ejecutar EnvMorph sobre un candidato se registra: repositorio, commit o versión, licencia, lenguaje, tipo de workflow, comando de preparación, comando de ejecución, artefactos observados y razones de inclusión o exclusión.

La selección no puede usar como criterio que el workload tenga un bug ambiental conocido, salvo que se declare previamente como caso de control positivo. El corpus principal debe contener workloads seleccionados por criterios estructurales, no por resultado esperado.

#### Factores candidatos

La lista siguiente es un registro de candidatos, no una lista de features obligatorias:

- `locale`
- relación `LC_ALL` y `LANG`
- `timezone`
- implementación de `awk`
- implementación de shell, por ejemplo `dash` frente a `bash` cuando el workflow lo permita
- resolución de herramientas mediante `PATH`
- `umask`
- `TMPDIR`
- orden de archivos o entradas de directorio
- implementación de `sed`
- implementación de `sort`
- implementación de `grep`
- implementación de `date`

Un factor se activa solo si el workload lo usa o existe una hipótesis técnica previa verificable. No se implementan factores para aumentar artificialmente dimensionalidad.

#### Regla para extender factores

Si un workload seleccionado requiere un factor todavía no soportado, se debe registrar antes de ejecutar la matriz afectada:

- evidencia de que el workflow depende del factor
- baseline y treatment o conjunto finito de valores
- mecanismo de capability detection
- semántica de `unavailable`
- cambios de código necesarios
- pruebas nuevas

La extensión se versiona y se asocia a los workloads que la motivaron.

#### Oráculos y perfiles

F8 comienza con los oráculos F3 existentes: byte, texto, JSON y CSV.

Si un workload requiere tolerancia o normalización adicional, se prefiere un perfil declarativo y versionado antes de ejecutar sus contrastes. Candidatos permitidos incluyen:

- tolerancia numérica explícita
- rutas JSON ignoradas declarativamente
- colecciones JSON declaradas como no ordenadas
- columnas CSV ignoradas de forma explícita
- filas CSV no ordenadas cuando el dominio lo justifique
- normalización de timestamps o campos volátiles identificados previamente

No existe un oráculo semántico implícito. Toda normalización debe quedar en la evidencia y formar parte del hash o versión del perfil.

Una normalización creada después de observar una divergencia se registra como análisis exploratorio y no puede convertir retrospectivamente esa ejecución en evidencia confirmatoria.

#### Artefactos observados

Para cada workload se registran, cuando existan:

- código de salida
- stdout
- stderr
- artefacto final principal
- artefactos intermedios que representen etapas semánticamente distinguibles
- manifest de ejecución F2
- versión de herramientas resueltas
- resultados F3, F4, F5, F6 y F7 aplicables

#### Repetición y estabilidad

Cada contraste utilizado para inferencia causal debe cumplir al menos dos confirmaciones consistentes. Si el baseline o la intervención completa no son estables entre confirmaciones, el caso se clasifica como no resuelto para inferencia confirmatoria y se conserva como resultado negativo o exploratorio.

#### Inferencia y validación held-out

Cuando un factor ofrezca más de dos valores plausibles o existan combinaciones adicionales, se separan configuraciones de inferencia y validación antes de derivar el contrato.

El contrato F6 se deriva solo de la porción de inferencia. Las variantes held-out se evalúan después y se reportan sin reajustar el contrato.

Si un workload solo admite contrastes binarios, el reporte debe declarar que no existe validación held-out fuerte para ese factor y no presentar esa ausencia como éxito predictivo.

#### Resultados negativos

F8 conserva y publica:

- `identical` y `equivalent`
- `different`
- `unavailable`
- `execution_failed`
- `missing`
- `error`
- `unresolved`
- `out_of_scope`
- contratos que no predicen held-out
- envelopes parciales
- workloads sin sensibilidad ambiental observada

No se sustituyen workloads para mejorar las tasas de éxito después de comenzar la ejecución confirmatoria.

#### Métricas mínimas

Por workload:

- número de factores candidatos y activados
- número de configuraciones ejecutadas
- tasa de ejecuciones disponibles
- diferencias observables reproducibles
- primera divergencia y fronteras de absorción cuando existan
- cardinalidad y número de MCEs
- factores esenciales
- precisión de predicción held-out cuando sea aplicable
- celdas `satisfied`, `violated` y `out_of_scope`
- cobertura decidida del envelope

Agregadas:

- distribución de resultados positivos y negativos
- frecuencia de factores activados
- distribución de cardinalidades MCE
- proporción de envelopes completos y parciales
- porcentaje de predicciones held-out correctas sobre casos evaluables

No se agregan porcentajes cuando el denominador sea ambiguo o mezcle casos no evaluables.

#### Política de modificación del software

F8 puede extender EnvMorph únicamente cuando un workload ya seleccionado lo requiera. Cada extensión debe conservar los gates F0 a F7 y añadir pruebas que demuestren la necesidad concreta.

No se incorporan features independientes del corpus durante F8.

#### Criterio de cierre F8

F8 puede cerrarse cuando:

- el corpus final contiene entre tres y cinco workloads externos válidos
- se conserva la trazabilidad de inclusión y exclusión
- todos los resultados, incluidos negativos, están materializados
- las extensiones de factores u oráculos están justificadas por el corpus
- existe al menos una evaluación held-out cuando el dominio seleccionado lo permita
- los claims se ajustan a la evidencia obtenida
- F0 a F7 continúan pasando
- el fixture histórico permanece intacto

F8 puede cerrar con resultados científicos negativos. No requiere que todos los workloads exhiban divergencias.
