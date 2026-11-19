### EnvMorph, protocolo G3-C v0.3

Fecha: 17 de noviembre de 2026. Base: `main` en `94a73b41be0b8c62d391e07f119d152ba00e1fc0`.
Estado: **PROPUESTA TÉCNICA PARA RATIFICACIÓN, NO CONGELADA, NO AUTORIZA CAMPAÑA**.
Autoría de revisión: asistida por IA; la ratificación y ejecución corresponden al investigador.

### 1. Enmienda científica esencial

Se sustituye el criterio A de v0.2 que exigía que F6-v2 acertase en H cuando C2 fallase. Es **matemáticamente imposible** para S={000,100,010,001}, H={110,101,011,111}, y la familia actual de F6-v2 (constantes y conjunciones firmadas con un máximo de dos literales). La comprobación exhaustiva reproduce los ocho patrones de sensibilidad de los tres singletons; cuando F6-v2 decide una celda H, C2 hace la misma predicción. En particular:

- Sin singleton sensible: F6-v2 abstiene en todo H.
- Con exactamente uno: F6-v2 solo decide STABLE para el par que no incluye el factor sensible, y C2 coincide.
- Con dos o tres: la familia de hipótesis de F6-v2 es inconsistente, no decide H.

El ejercicio no evalúa superioridad predictiva de F6-v2 sobre C2 ni superioridad del método sobre una exact table. No se modifican F6-v2 ni C2, ni se agregan workflows. El resultado prospectivo será un **estudio de alcance empírico y límites del artefacto**, con métricas de cobertura selectiva, errores, abstenciones, estabilidad y validez del oráculo. Si se desea reivindicar mejora de método, se necesita otra pregunta de investigación y otro protocolo independiente, no una modificación post-hoc de G3-C.

### 2. Unidad, población y selección

Se mantienen los cinco primarios `findutils`, `miller`, `unzip`, `exiftool`, `patch`; reservas, en orden, `less`, `tree`, `moreutils`. Fuente de pool: Ubuntu 24.04 noble y noble-updates (consulta declarada del 16/11/2026). `pool_manifest.json` debe conservar exactamente el SHA-256 `743c7d2f2d326d6d1c366172542df31d24424270819297f3fc0b2cbc9805d5bc`. `draw_cohort.py` reproduce el orden original, y el nuevo commit A debe documentar que **el sorteo provisional fue conocido antes de congelar el pool**. No describir la muestra como sorteo ciego, preregistrado o estadísticamente representativo. Cualquier cambio de pool posterior sería una selección informada y se rechaza en esta campaña. Las tres reservas se preservan documentalmente, **pero no se activan mediante el ejecutor v0.3**. Si falla un primario, se declara el fallo y se aplica la regla STOP correspondiente; no se sustituye por otro workflow sin un cambio de protocolo prospectivo anterior a S, que aquí no está previsto.

Los cinco workflows utilizan paquetes de distribución fijados por versión y digest del `.deb`, no nuevos commits upstream. La unidad inferencial es el workflow/proyecto, n máximo 5, no las celdas ni las confirmaciones. Las fichas propuestas no son resultados de tratamiento.

### 3. Catálogo fijado

- Factores en este orden: TZ (`UTC`, `America/Lima`); LC_ALL (`C`, `C.UTF-8`); TMPDIR (dos directorios creados por corrida), todos exclusivamente aplicados al hijo de herramienta.
- Retículo de ocho celdas, S exactamente `000`, `100`, `010`, `001`; H exactamente `110`, `101`, `011`, `111`. Cero normalización. Oráculo: stdout byte a byte de la herramienta o de un envoltorio determinista congelado.
- Controles negativos obligatorios: NULLA/NULLB, misma celda 000 salvo `ENVMORPH_NULL_FACTOR`, dos confirmaciones por cada uno; ambos deben concordar con 000. N2 solo si existe documentación primaria y código precongelado específico; no se añade después.
- Se mantienen únicamente intervenciones `environment_override` implementadas por el harness. `workspace_file_replace` no se ejecuta en este retículo, y se excluye del denominador y del claim experimental de G3-C. M4-G2 se conserva como capacidad de preparación histórica, no acreditada como ejecución en la cohorte.
- La evidencia de aplicación disponible consiste en el entorno pasado a `execve` y un proceso de sondeo independiente `/usr/bin/env -0`; **no demuestra la variable leída internamente por cada herramienta**. Sin observador F9 integrado, no hay claim de observación de dependencias ni de recall.

### 4. Orden de operaciones y bloqueo

**Precongelación**: comprobación de paquetes y de locales, test de fixtures; preflight de dos baseline sin tratamiento en ruta independiente. Los preflight cuentan en el presupuesto global. No usar el preflight para seleccionar por sensibilidad. Los archivos baseline de preflight nunca sustituyen a S/000.

**Congelación**: commit A del pool exacto, commit B del sorteo documentado, ratificación explícita, commit C con protocolo, especificaciones, harness, imagen por digest, binarios F6-v2 por SHA y fixtures. Marcar `PROTOCOLO_G3C_v0.3.md` como RATIFICADO solo mediante un registro por separado que identifique su SHA. Generar `PROTOCOL_FREEZE.json` nuevo con `g3c_freeze.py`, firmar tag y publicar hash más sello de tiempo externo ANTES de S. Registrar huellas de archivos y dependencias. La función de freeze por sí sola no acredita congelación externa.

**Bookend F10 inicial**: ejecutar F10 auténtico sobre tres controles congelados, dentro de la misma imagen declarada, verificar evidencia mediante `verify_f10.py` y `test_f10.py`, conservar el registro con SHA; un 3/3 de otra imagen no sustituye esta condición. El código de registro no valida por sí solo qué imagen produjo la evidencia: auditoría Docker obligatoria.

**Etapa S y N**: cinco workflows por cuatro celdas S y dos celdas N, dos confirmaciones cada una, con registros nuevos, fallos conservados. Si falta cualquiera de las 12 confirmaciones por workflow, no sellar. Toda diferencia de N1 detiene la campaña hasta dictamen de defecto o confusor; no es un resultado estable. Se impide ajustar el método tras observar S.

**Decisión**: C0 predice SAME; C1 exact-table abstiene en todo H; C2 predice OR de singletons sensibles; C3 es F6-v2 exactamente congelado. Generar todas las decisiones H sin etiquetas H, salvar decisiones una sola vez, guardar hashes de evidencia S/N y de decisiones, publicar el recibo externo de ese contenido con fecha y custodio verificables. La autorización para H requiere un recibo con hashes de freeze, decisiones y evidencia, firmado con Ed25519 por un custodio distinto, cuyo PEM público se congela con el protocolo; no basta un `SEALED.json` local. Los archivos locales siguen siendo modificables por su propietario; la defensa contra modificación retrospectiva depende de la firma externa y del depósito cotejado independientemente.

**Etapa H**: solo tras recibo externo, cuatro celdas H por workflow, dos confirmaciones, ninguna selección a partir de resultados. No se reintenta sin expediente documentado y autorización independiente; la versión 0.3 bloquea reintentos automáticos. Los fallos permanecen UNRESOLVED, nunca STABLE.

**Bookend F10 final**: tres controles repetidos y verificados en la misma imagen. Tras cerrar H o una parada temprana, calcular métricas y archivar evidencia completa. Revisión por tercero y segundo host forman parte del gate editorial, no de la ejecución original.

### 5. Presupuesto estricto y parada

Tope global de 180 invocaciones, incluyendo intentos fallidos y todas las confirmaciones. El ledger registra una reserva atómica antes de cada par de confirmaciones, y bloquea repeticiones. Se reservan **66 invocaciones de los comandos objetivo** para dos bookends F10 (11 por cada uno de los tres casos y por bookend, al contar 2 ground-truth, 1 observado, 4 runner y 4 capturas). Quedan como máximo **114 ejecuciones de celdas G3-C**. La cohorte completa consume 5 × (2 preflight + 8 S + 4 N + 8 H) = 110, y el total planificado es 176, dejando un margen de 4. Los auxiliares de preprocesamiento, compilación, hashes y comparación se registran como costos operativos, no como corridas de las herramientas objetivo. Esta definición debe congelarse y auditarse; el harness no invoca internamente F10 y el número de ejecuciones reales debe cotejarse con sus registros. No aumentar 180 ni la cohorte tras ver etiquetas. Máximo de 15 minutos por llamada, máximo de 10 jornadas (seguimiento humano); en las fichas actuales el timeout operativo de algunas herramientas es 120 segundos, más estricto. Reintento automático: 0 en v0.3. Una desviación de infraestructura exige expediente aparte y no borra ejecuciones.

STOP por fuga H, modificación de protocolo, SHA incorrecto, imagen no inmovilizada, oráculo no estable, controles N fallidos, comparadores no equivalentes, falta de recibo externo, evidencia manipulada, ejecución sin aislamiento exigido, exceso de presupuesto o fin de ventana. Una parada temprana produce registro e informe; no se agregan muestras.

### 6. Estadística y decisiones científicas

Para cada workflow, reportar S sensibles, H total=4, H decidibles, errores y aciertos de cada comparador, abstenciones, controles negativos, intentos, tiempo y memoria si hay mediciones fiables. RQ1 se restringe a la factibilidad de los tres factores ambientales ejecutados; no atribuir valor causal a oportunidades `workspace_file_replace` no ejecutadas. Métrica principal RQ3: contradicciones de C3 entre sus decisiones emitidas, presentada siempre con cobertura C3 decididas/H decidibles y con denominador total H predeclarado. **Cero errores con cero decisiones es vacuidad, no éxito.** La comparación con C2 es una curva de cobertura-riesgo; si C3 se abstiene donde C2 se equivoca, es un intercambio de abstención por riesgo, no una predicción superior ni un resultado independiente.

- A, evidencia selectiva no vacía: al menos 3 workflows evaluables, 2 informativos de al menos 2 proyectos, C3 emite al menos una decisión correcta en 2 workflows de 2 proyectos, ninguna contradicción emitida y todos los controles negativos pasan. Solo respalda un claim acotado de exactitud selectiva y abstención, condicionado a auditoría externa.
- B, resultado empírico acotado: campaña evaluable que no alcanza A; preservar abstenciones y ausencia de superioridad.
- C, nulo: menos de 2 workflows informativos o todos estables. Cerrar, no alterar cohortes.
- D, STOP/NO_GO: integridad o controles inválidos, held-out contaminado, modelo contradictorio en decisiones emitidas, fallos de aislamiento, presupuesto agotado sin evaluabilidad. No reivindicar método; clasificar desviaciones y reportar negativos.

No calcular p valores ni IC entre cinco workflows seleccionados por conveniencia documental; mostrar conteos y resultados por unidad, sin generalización poblacional.

### 7. Alcance editorial y auditoría

Paper 1 sigue NO GO. Si A se cumple más custodia externa y segundo host, se puede estudiar un texto empírico de alcance limitado; ni A ni tests de integración autorizan un paper de método superior. Si B/C, reformular como estudio de límites y resultado negativo, evaluando su interés editorial real; si D o evidencia no auditable, cerrar sin envío. No abrir fases M5/M6 ni repetir campaña para obtener otro patrón. Auditar por separado ingeniería, seguridad, validez científica, reproducción y preparación editorial. No modificar F8, F9, F10 ni P1-EB-01.

### 8. Limitaciones obligatorias antes del congelamiento

Es imprescindible verificar en Ubuntu Docker real: construcción fail-closed con `.deb` y dependencias fijadas, versiones tzdata/locales, imagen por digest, escritura bajo UID 10001, ejecución de F10 y F6-v2 en la misma imagen, aislamiento, presupuesto y sellos externos. Este paquete corrige código y prueba guardias locales, pero **no demuestra ejecución Docker de la campaña**, independización de custodia, ni reproducción en segundo host. El manejo del estado por propietario del host no es almacenamiento físicamente inmutable.
