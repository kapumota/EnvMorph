### Erratum F9.8C: ensamblado del censo final

#### Incidente

Los dos plans externos F9.8C se ejecutaron bajo el harness pre-treatment congelado.

Ambos produjeron dos confirmaciones F4 `stable` y clasificación `observed_only`.

Después de esas clasificaciones, el agregador F9.7 rechazó el censo final con:

```text
Workload no ejecutable inventa censo o grafo
```

No existió commit de resultados.

#### Causa

El ensamblador inicial copiaba para todos los workloads:

```text
census_frozen_before_treatment = true
census_commit = <commit>
plans_commit = <commit>
```

Eso viola el contrato F9.7 para un workload cuyo `workload_status != completed`.

#### Corrección

Un workload no completado se representa con:

```text
graph_file = null
graph_sha256 = null
census_sha256 = null
census_frozen_before_treatment = false
census_commit = null
plans_commit = null
candidates = []
```

La corrección afecta únicamente la estructura de workloads no completados.

#### Momento de la corrección

Este erratum se congela después de observar los resultados de los dos treatments.

No modifica:

- plans;
- treatments;
- factores;
- valores baseline o treatment;
- oráculos;
- mapping F4/F9.6;
- clasificaciones ya obtenidas;
- candidatos de workloads completados.

Los workloads no se vuelven a ejecutar. Las salidas F4 y bridge se reconstruyen desde la evidencia cruda ya preservada.
