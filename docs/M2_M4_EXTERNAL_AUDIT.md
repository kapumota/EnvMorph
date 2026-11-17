> **Nota de actualización, 2026-11-16:** este documento conserva el
> estado histórico de revisión fechado el 2026-11-15. PR #5 ya fue fusionado
> en `main` (`c53be08`); M4-G3 cerró un piloto de ingeniería y el puente
> F3 → F9.6, **sin** validación prospectiva ni reproducción independiente.
> Estado actual y frontera: `docs/M4_G3_STATUS.md`; auditoría pendiente:
> `audits/M4_G3_AUDIT_CHECKLIST.md`. No reinterpretar los textos históricos
> de esta página como estado vigente de ramas o PR.

### EnvMorph: expediente de auditoría externa de código y evidencia, M2–M4

Fecha de referencia: 2026-11-15. Estado: **candidato a revisión externa, NO listo para publicar el Paper 1**.

#### Identidad y límites

- Repositorio público: https://github.com/kapumota/EnvMorph
- Commit base de `main` verificado: `99f89f9a66cf24a6afa59ea8114ee3a6600a4510`.
- Rama local de origen: `research/p1-eb01-execution`, con `Cargo.toml` modificado y `src/bin/envmorph-f6v2.rs` no rastreado.
- La rama de revisión se debe abrir como **draft PR**. Sin merge automático, sin alterar F8–F10 ni P1-EB-01.
- «Independiente» significa revisión por alguien ajeno a la implementación y, para reproducción empírica, ejecutar sobre otro host. Un pull request público por sí solo no prueba independencia.

#### Evidencia clasificada sin confundir sus naturalezas

| Fase | Naturaleza de datos | Observaciones comprobables | ¿Prueba utilidad externa general? |
|---|---|---|---|
| F8 | Cuatro proyectos/workflows upstream seleccionados y congelados | 3 cadenas evaluables, 1 no evaluable, 0 divergencias en los 3 contrastes evaluables; held-out abstiene | No |
| F9 | Censo externo congelado | 12 717 candidatos observados en dos workflows completados; 8 807 fuera de alcance, 3 908 no intervenibles, 2 elegibles/decidibles; 0 relevantes entre ellos | No; 12 715 no fueron descartados como irrelevantes |
| F10 | Controles positivos **conocidos de antemano** | 3/3 sensibilidad end-to-end en controles seleccionados, no estimación poblacional de recall | No |
| P1-EB-01 | 96 ejecuciones reales de herramientas existentes en 6 casos controlados, dos proyectos upstream | 48 configuraciones de dos factores, 30/48 decisiones F6-v1; tabla exacta con dos confirmaciones también 30/48 | No; casos diseñados y muy acotados |
| M2 | Implementación experimental Rust con pruebas de desarrollo | 116 pruebas históricas + 15 del binario, adversariales pasan en host local | No; pruebas de programación, no validación externa |
| M3 | Replay **retrospectivo** de P1, no campaña nueva | 432 invocaciones Rust, igualdad de cobertura directa con tabla exacta, sin superioridad de `rust_suggested` | No; 12 pares provienen de solo 6 casos y 2 proyectos |
| M4-G1B | Diagnóstico de ingeniería sobre aplicaciones reales, **no** tratamiento ambiental | csvkit: 141 a 0 al reemplazar `head` por `sed` y 3 artefactos idénticos; jq: etapas finalizan en prueba prospectiva, origen del 124 histórico sin establecer | No |
| M4-G2 | Planificador Python y ejemplos **sintéticos** | Dos tipos de intervención validados, baseline/treatment preparados; 0 workflows ejecutados | No |

#### Afirmaciones que NO deben figurar en un abstract

- «EnvMorph predice portabilidad ambiental en general».
- «F6-v2 supera a tablas exactas o a BugDoc, reprotest, ddmin u otros métodos».
- «El censo F9 demuestra que 12 715 dependencias no influyen en el comportamiento».
- «M4-G2 ha mejorado la elegibilidad o ha ejecutado intervenciones externas».
- «3/3 controles F10 demuestra recall general o especificidad».
- «432 reportes M3 o 48 celdas son 432/48 estudios independientes».
- «El uso de SHA-256 prueba autenticidad o independencia de ejecuciones».

#### Observaciones específicas para el revisor técnico

1. **Rust F6-v2**: familia de hipótesis H pequeña (constantes y conjunciones de hasta dos literales), oráculo byte-stdout, dos repeticiones por configuración. Distinguir evidencia directa de inferencia condicional; buscar predicción falsa bajo mala especificación de H.
2. **Integridad**: los digests verifican bytes respecto a un hash fijado; verificar modelo de amenaza, fijación del digest y posibles rutas relativas o referencias no autenticadas.
3. **M4-G2**: `environment_override` prepara un manifiesto y **todavía no aplica variables a procesos**; `workspace_file_replace` modifica una copia, no el origen. No hay ejecución ni aislamiento de kernel.
4. **Seguridad**: revisar validación de paths y enlaces simbólicos en **todo** el árbol copiado, TOCTOU, archivos especiales, límites del tamaño de copia, denegación de servicio, variables peligrosas no incluidas en la lista de bloqueo y `copytree(..., symlinks=True)`; no es un sandbox seguro para ejecutar upstream desconocido.
5. **Reproducibilidad**: el corpus P1 original tiene SHA-256 `9ba4652dbb6ee098a43c288bda699b7f9e86572383c13126ed397f90d1e13288`. La rama incluye arnés y agregados M3, **no** los bytes originales de las 96 ejecuciones por defecto. Para reproducir M3, el revisor necesita acceso autorizado al archivo TAR y comprobar el hash.
6. **Inferencia**: comprobar que las 96 ejecuciones son referencias del mismo host y que las políticas no leen etiquetas futuras. M3 es retrospectivo y no está cegado al diseño previo.

#### Gate de auditoría externa (no redacción editorial)

- **GO integración de código experimental**: tests reproducibles, ausencia de regresión, separación de F6-v1, sin afectación F8–F10, límites y amenazas documentados; revisor inspecciona diff.
- **NO GO para el claim científico** hasta que una campaña prospectiva real y preseleccionada muestre resultados frente a comparadores equivalentes, incluyendo casos negativos, fallidos y abstenciones; reproducción por un tercero.
- **STOP** si el revisor detecta leak de resultados futuros, un oráculo no comparable, referencias manipulables sin control, ruptura de aislamiento, o un claim no respaldado.

#### Preguntas que la revisión debe contestar

1. ¿Qué proporción del código nuevo se ha ejecutado sobre aplicaciones upstream no diseñadas como demos?
2. ¿Cuáles entradas y tratamientos están declarados **antes** de sus resultados?
3. ¿Se conoce y reproduce la causa de cada fallo? Especialmente jq histórico no resuelto.
4. ¿Es posible producir una decisión nueva que una tabla de igual presupuesto y mismo oráculo no obtendría? ¿Con qué costo/garantías?
5. ¿Puede el modelo H producir falsos positivos/negativos antes de detectar contradicción? Documentar contraejemplo y tasa.
6. ¿Qué dependencia observada adicional se volvió intervenible por la nueva clase `workspace_file_replace`? Por ahora: **ninguna demostrada empíricamente**.
7. ¿Puede un tercero repetir todo sin los archivos locales del investigador? Por ahora: **no totalmente**.

#### Recomendación

Publicar código y documentación en una rama auditada ahora. No fusionar con `main` como método consolidado ni presentar el prototipo como evidencia científica. Priorizar después el **primer experimento prospectivo end-to-end sobre un workflow externo real**, con plan, oráculo y controles congelados, antes de expandir a 12 workflows o producir más prototipos.
