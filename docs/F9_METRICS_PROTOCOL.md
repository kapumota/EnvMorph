### Protocolo de métricas F9.7

#### Objetivo y congelación

Este protocolo se congela antes de ejecutar otra vez el corpus externo F8. Un resultado negativo, la abstención y los fallos de observación son evidencia, no motivos para sustituir workloads, factores u oráculos.

El universo externo contiene exactamente cuatro workloads ya seleccionados en F8: `csvkit-realdata`, `jq-build-check`, `fd-build-completions` e `inih-meson-tests`.

#### Unidad de análisis y censo

La unidad primaria es la clave `dependency_key` de F9.2/F9.3 dentro de un workload. Todas las dependencias de nodos `dependency` del grafo F9.3 deben aparecer una vez en el censo, incluso si no se pueden intervenir. El agregador exige igualdad de conjuntos entre grafo y censo; no permite seleccionar solo dependencias con diferencia observable.

El censo y los planes deben congelarse antes de ejecutar treatments. F9.8 deberá registrar dos commits de checkpoint anteriores a los tratamientos, uno para el censo y otro para los planes. Un hash registrado por sí solo no demuestra anterioridad temporal. F9.9 comprobará la genealogía Git y la procedencia.

Se permite como máximo un plan confirmatorio por dependencia y workload. Si hay tratamientos exploratorios adicionales, deben tener un registro separado y no entrar en las métricas confirmatorias congeladas.

#### Regla de admisibilidad

Una clasificación decidible exige una dependencia previamente observada, `InterventionPlan v1` válido por el validador F9.4, plan y evidencia con identidad SHA-256 verificable, al menos dos confirmaciones baseline y dos treatment, comparación sustentada por salida auténtica del bridge F9.6 y ausencia de efecto del observador que invalide la comparación.

La equivalencia F3 y los estados de F4/F5 son relativos al oráculo. La validez de la intervención no se deduce de un `different` o `minimal` aislado. La clasificación se enlaza a la misma `dependency_key` y al plan precongelado, sin afirmar causalidad física.

#### Estados que nunca se colapsan

`behaviorally_relevant`: diferencia reproducible bajo el oráculo declarado.

`observed_only`: intervención válida y equivalencia reproducible bajo el oráculo declarado, no ausencia general de dependencia.

`unresolved`: evidencia insuficiente, inestabilidad o ambigüedad.

`unavailable`, `out_of_scope`, `observer_failure` y `execution_failed` se mantienen por separado como estados operacionales. El agregador distingue `unavailable_candidates` y `out_of_scope_candidates` (elegibilidad) de `unavailable_interventions` y `out_of_scope_interventions` (intervenciones iniciadas). `not_intervenable` es una elegibilidad, no una demostración de equivalencia.

Todo intento que termina sin decisión requiere una evidencia con hash y referencia al plan: clasificación F9.5 `unresolved` o registro explícito `f9.intervention_exception.v1` con estado y motivo. Los errores no desaparecen del censo.

`observer_effect = altered` prohíbe una clasificación confirmada. Ausencia de eventos no demuestra hermeticidad.

#### Métricas primarias

Por workload y en total:

- `observed_candidates`: nodos `dependency` observados en grafos disponibles.
- `eligible_candidates`, `not_intervenable_candidates`, `unavailable_candidates`, `out_of_scope_candidates`: categorías de elegibilidad declaradas antes de tratamientos.
- `behaviorally_relevant`, `observed_only`, `unresolved`, `execution_failed`, `observer_failure`: resultados confirmatorios y fallos separados.
- `decidable_candidates = behaviorally_relevant + observed_only`.

Fracciones descriptivas, sin intervalos inferenciales ni extrapolación:

```text
decidability_among_observed = decidable_candidates / observed_candidates
relevance_among_decidable = behaviorally_relevant / decidable_candidates
```

Cuando el denominador es cero la proporción es `null`, nunca 0. El denominador de relevancia excluye explícitamente `unresolved` y fallos, pero sus conteos se muestran siempre. No se define un escalar Behavioral Hermeticity Gap.

#### Integridad de evidencia

El agregador lee grafos, planes y salidas de F9.6 bajo una raíz de evidencia explícita. Rechaza rutas fuera de esa raíz, hashes incorrectos, planes incompatibles, confirmaciones insuficientes, cambios de identidad o inconsistencias con el mapeo F3/F4/F5. Los JSON de salida son deterministas ante un mismo conjunto de datos, independientemente de la ordenación del censo.

La completitud de observación es desconocida. Se puede medir la cobertura *del protocolo* sobre las dependencias observadas, pero no la fracción de todas las dependencias reales del sistema.

#### Riesgos y límites

No se mezclan micro-workloads sintéticos con los cuatro workloads externos. Los casos sintéticos prueban el agregador, no cuentan como validación externa. Los resultados externos pueden ser enteramente negativos o no decidibles y siguen siendo válidos.

F9.7 no ejecuta el corpus F8 ni aplica tratamientos. F9.8 requiere un nuevo checkpoint del censo y los planes antes de ejecutar intervenciones, respetando las restricciones originales de F8.
