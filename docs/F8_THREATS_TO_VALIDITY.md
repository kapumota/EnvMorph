### Amenazas a la validez de F8

#### Validez interna

La selección del corpus se congeló antes de ejecutar EnvMorph y la matriz se congeló antes de la campaña confirmatoria. Esto reduce selección por resultado y reajuste post-hoc. Los adapters permanecen separados de upstream y los resultados negativos se conservaron. Un intento de ejecución fue abortado manualmente antes de la campaña completada y quedó excluido de la evidencia confirmatoria.

#### Validez de constructo

F4 observa diferencias a través de los oráculos F3 declarados. Que un workload sea estable no demuestra ausencia de toda dependencia ambiental, solo ausencia de diferencia observable bajo los factores, valores y oráculos congelados. F5 define causalidad como suficiencia operacional dentro del espacio declarado, no como causalidad física.

#### Validez externa

Solo tres de cuatro workloads seleccionados completaron la cadena F2-F7. Los tres resultaron estables. Esto limita cualquier inferencia sobre prevalencia de sensibilidad ambiental. El corpus tampoco representa todo el ecosistema Unix/Linux y la campaña se ejecutó en una sola máquina.

#### Held-out

Solo se ejecutó un held-out y F6 respondió `out_of_scope`. Por ello no existe una estimación empírica de accuracy predictiva fuera de los valores usados para derivar el contrato. La abstención se conserva y no se cuenta como acierto.

#### Reproducibilidad

Revisiones upstream, licencias, manifests SHA-256, adapters, runtime, matriz y resultados compactos están congelados. La evidencia pesada permanece bajo `.envmorph/f8-evidence`. El intento abortado previo se conserva fuera de la evidencia confirmatoria bajo `.envmorph/f8-aborted-attempts`.

#### Riesgo de conclusión

El principal riesgo sería reinterpretar la ausencia de divergencias como evidencia de portabilidad general. F8 no permite esa conclusión. El resultado correcto es más estrecho: bajo la matriz congelada, los tres workloads evaluables no mostraron diferencias observables.
