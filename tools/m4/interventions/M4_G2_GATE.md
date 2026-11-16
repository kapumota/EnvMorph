# EnvMorph M4-G2: alcance y criterios de aceptación

## Motivo

F9 histórico: 12 717 candidatos observados, 8 807 `out_of_scope`, 3 908 `not_intervenable`, 2 elegibles y 0 relevantes entre los 2 decidibles. La elegibilidad histórica estaba acotada por el protocolo F8 (LC_ALL, TZ y TMPDIR). Es **inválido** reinterpretar los 12 715 restantes como irrelevantes.

M4-G1B identificó una explicación concreta de la salida 141 de csvkit: cambio controlado de `head` por `sed`, código 141→0 e identidad de tres artefactos. Los resultados diagnósticos no alteran F8/F9. Para jq, `cleanup`, `autoreconf`, `configure`, `make`, `make check` y artefactos finalizaron correctamente en condiciones nuevas; el timeout histórico de 1200 s conserva causa desconocida.

## Hipótesis de investigación futura

RQ1. ¿Qué proporción de **oportunidades predeclaradas** pasa a intervención válida y oráculo decidible al soportar reemplazo tipado de archivos del workspace además de variables de entorno?

RQ2. ¿Qué cobertura adicional de divergencias reproducibles aporta frente a un comparador que puede realizar las mismas intervenciones con igual presupuesto?

RQ3. ¿Cómo se comportan las decisiones, errores y abstenciones de F6-v2 cuando hay combinaciones con tres a seis factores? `model_supported` se evalúa separadamente y no equivale a verdad observada.

## Qué demuestra este paquete

- Parseo/validación de un plan estructurado con los dos tipos descritos.
- Rechazo determinista de rutas peligrosas, hashes obsoletos, variables bloqueadas, duplicados y especificaciones inconsistentes.
- Generación de expediente de preparación (`M4_G2_PREFLIGHT.json`) sin ejecutar nada.
- Materialización opcional de baseline/treatment en copias distintas, conservando intacto el workspace fuente y verificando los hashes de los archivos modificados.
- Pruebas adversariales sintéticas realizadas en Python 3.11.

## Qué no demuestra

- Aumento de elegibilidad externa o efectos conductuales.
- Comportamiento de herramientas existentes como reprotest, BugDoc o ReproZip.
- Validez predictiva o garantías de seguridad del sistema.
- Robustez de la observación `getenv` para accesos directos a `environ`, vDSO, mmap, etc.
- Contribución Q1/Q2, todavía pendiente de evidencia externa comparativa.

## Gate posterior M4-G3

1. Inventariar clases de dependencia observada y oportunidades de intervención por proyecto; no contabilizar nodos individuales como estudios independientes.
2. Seleccionar proyectos externos por criterios anteriores a los tratamientos, con licencia, commit upstream, dependencias y comandos registrados.
3. Congelar los recursos, variantes, oráculos y condiciones de parada; preservar los no elegibles y fallidos.
4. Materializar los tratamientos y correrlos con un ejecutor aislado bajo presupuesto idéntico, al menos dos confirmaciones por brazo.
5. Comparar contra baselines que disponen de las mismas oportunidades y oráculos, no una tabla artificialmente debilitada.
6. Medir por separado recuperación de ejecución, elegibilidad, ejecución válida, decisión, relevancia, falsos positivos, abstenciones, tiempo y costo.

El paquete es un prototipo exploratorio G2, no un congelamiento científico G3.
