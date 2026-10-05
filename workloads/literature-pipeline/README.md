### Literature Pipeline como fixture histórico

#### Estado

`source/` es un snapshot histórico congelado heredado de FlowAttest. Se conserva deliberadamente sin modernizar su estilo, mensajes, emojis, placeholders históricos o redacción promocional interna porque esos bytes forman parte de la procedencia experimental.

#### Integridad

La integridad del fixture se verifica mediante:

```text
provenance/LITERATURE_PIPELINE_FILES.sha256
```

Ninguna fase de EnvMorph debe modificar directamente `workloads/literature-pipeline/source/`.

Cuando una incompatibilidad del host exige una adaptación para replay, el harness copia el fixture a un directorio temporal y modifica únicamente esa copia.

#### Interpretación

El contenido editorial dentro de `source/` no representa el estilo documental actual de EnvMorph. La documentación activa del proyecto está en el README raíz y `docs/`.

#### Rol experimental

Este workload es un control interno y de regresión. No cuenta como evidencia externa en F8 ni como workload independiente para el gate editorial de Paper 1.
