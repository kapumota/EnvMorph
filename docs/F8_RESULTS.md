### Validación externa F8

#### Resultado principal

F8 cerró con un resultado científicamente negativo bajo la matriz congelada. Tres de los cuatro workloads seleccionados completaron la cadena F2-F7 y, en los tres casos evaluables, no se observó una diferencia conductual bajo los contrastes ambientales declarados. El cuarto workload se conserva como no completamente evaluable y no fue sustituido.

El único held-out ejecutado terminó en abstención `out_of_scope`. Por tanto, F8 no demuestra sensibilidad ambiental positiva ni capacidad de extrapolación de F6 a valores ambientales no observados.

#### Resumen por workload

| Workload | Estado | F4 | F5 | MCE | F7 | Held-out |
| --- | --- | --- | --- | ---: | --- | --- |
| `csvkit-realdata` | `not_evaluable_f4` | `n/a` | `n/a` | n/a | `n/a` | `no` |
| `fd-build-completions` | `full_chain_evaluable` | `stable` | `no_observable_effect` | n/a | `complete` | `no` |
| `inih-meson-tests` | `full_chain_evaluable` | `stable` | `no_observable_effect` | n/a | `complete` | `no` |
| `jq-build-check` | `full_chain_evaluable` | `stable` | `no_observable_effect` | n/a | `partial` | `out_of_scope / abstained` |

#### Agregado

Workloads con cadena F2-F7 evaluable: **3/4**.
Workloads estables: **3**.
Workloads con diferencia observable: **0**.
Workloads no completamente evaluables: `['csvkit-realdata']`.
Distribución F4: `{'not_evaluable': 1, 'stable': 3}`.
Distribución F5: `{'no_observable_effect': 3, 'not_evaluable': 1}`.
Distribución F7: `{'complete': 2, 'not_evaluable': 1, 'partial': 1}`.
Held-out intentados: **1**, decididos por F6: **0**, abstenciones `out_of_scope`: **1**.

No se reporta accuracy held-out porque no hubo predicciones decididas. La abstención `out_of_scope` se conserva como resultado y no se reinterpreta como acierto.

#### Interpretación

El resultado apoya la robustez metodológica de la infraestructura para ejecutar, conservar y auditar resultados externos negativos, pero no aporta evidencia de que los factores congelados produzcan sensibilidad observable en estos tres workloads. Tampoco justifica ampliar factores de manera post-hoc dentro de F8.

#### Desviación antes de la interpretación agregada

un workload seleccionado no completó la cadena F2-F7 y se conserva sin sustitución. El corpus seleccionado permanece en cuatro workloads y el caso no evaluable queda materializado en la evidencia.

#### Límites de interpretación

Los resultados solo cubren los contrastes finitos congelados antes de la campaña. No demuestran portabilidad universal, ausencia universal de dependencia ambiental, causalidad física ni una ley de escalabilidad. Un resultado estable significa únicamente que los oráculos declarados no detectaron diferencias bajo los contrastes ejecutados.
