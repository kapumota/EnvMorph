### F9.8B elegibilidad externa

#### Regla

Las 12,717 dependencias del censo F9.8A permanecen en el ledger. F9.8B no busca divergencias y no agrega factores.

Solo una observación `environment/getenv` cuyo recurso sea exactamente el binding de un factor F8 congelado para el mismo workload puede ser `eligible`:

```text
locale   -> LC_ALL
timezone -> TZ
tmpdir   -> TMPDIR
```

Otras variables observadas quedan `out_of_scope` porque intervenirlas ampliaría el espacio experimental después de observar el corpus.

Los archivos `@WORKSPACE@/...` también quedan `out_of_scope`: F9.4 implementa el mecanismo, pero F8 no preregistró valores de contenido. Los archivos del host y ejecutables quedan `not_intervenable`.

Un factor exacto pasa a `unavailable` si sus valores baseline o treatment ya no están disponibles en el host.

Cada dependencia `eligible` obtiene un único `InterventionPlan v1`, dos confirmaciones por variante y el trace F8 congelado como referencia de oráculo.

F9.8B no ejecuta treatments y no asigna clasificación conductual.
