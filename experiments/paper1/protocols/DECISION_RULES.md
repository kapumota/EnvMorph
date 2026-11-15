# P1-EB-01: reglas de decisión fijadas antes de ejecutar resultados

## Clasificación

- **Evaluable**: la versión y el entorno coinciden; la intervención fue aplicada; comando con exit code 0 y stdout capturado; las dos repeticiones del mismo estado coinciden tras la transformación de representación ya fijada.
- **UNRESOLVED**: falta evidencia, falla el comando, la intervención no se aplica, o el oráculo no es estable. No convertir a negativo ni eliminar el caso.
- **SENSITIVE**: tratamiento y baseline reproducibles y diferentes según oracle preregistrado; no basta con cambio del PID, una ruta aleatoria o metadatos no controlados.
- **STABLE**: tratamiento y baseline reproducibles e idénticos según oracle preregistrado. C4 o C6 pueden dar STABLE y se conservará ese resultado.
- **CONTROL_NEGATIVE_FAIL**: un negativo difiere bajo el factor objetivo y no explica la divergencia mediante condiciones predeclaradas; considerar confusores y detener conclusiones causales del caso.
- **CONTRACT_CONTRADICTION**: una decisión `satisfied` o `violated` contradice la referencia del MISMO soporte, entrada y oráculo. No reinterpretar una abstención como acierto.

## Hipótesis y comparación

- H0: F6 no añade utilidad decisoria verificable a una tabla de asignaciones/outcomes con el mismo presupuesto de evidencia.
- H1: el formato y su auditoría evitan decisiones sin soporte o permiten verificaciones trazables que la tabla de igual presupuesto no ofrece. Debe probarse, no suponerse.
- Comparar exact table, ddmin, y si son compatibles reprotest y BugDoc, de manera justa y reproducible. La enumeración 2^2 es una referencia factual.
- Métrica primaria: número de decisiones sin soporte entre los desafíos congelados (requiere congelar también los desafíos antes de la evaluación). **No construir desafíos después de ver la salida**.
- Métricas secundarias: decisiones emitidas / total, errores condicionados a decisión, consultas/repeticiones, mínimos cardinales, casos negativos, costo, tamaño de certificados.

## Separación de datos

- Entradas `heldout_input` del JSON nunca participan de F5 ni del ajuste de controles y deben ejecutarse por primera vez tras generar contratos.
- Son entradas nuevas, no configuraciones ambientales held-out. El contrato F6-v1 no promete extrapolación a ellas.
- El conjunto de 6 casos es calibración/piloto. El gate editorial Q1/Q2 exige aplicaciones/workflows independientes adicionales y una máquina de réplica.

## Parada

- GO técnico: >=3 casos evaluables, al menos dos proyectos, >=1 derivación F5/F6 completa, negativos concordantes y 0 contradicciones decididas.
- GO metodológico: beneficio adicional material y demostrado sobre registro exacto de evidencias y comparadores de igual presupuesto, sin afirmar predicción de configuraciones nuevas.
- KILL de método: solo produce reglas/contratos equivalentes a memorizar tabla, o no mejora la verificabilidad/decisión frente a baseline por el costo invertido.
