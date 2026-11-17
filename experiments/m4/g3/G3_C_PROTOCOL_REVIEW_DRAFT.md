### G3-C: revisión de protocolo prospectivo, NO CONGELADO

Estado: `DRAFT_FOR_REVIEW`. Este texto **no autoriza ejecutar** una campaña confirmatoria. Debe contrastarse primero con `STOP_RULES_DRAFT.md`, actualmente solo en el equipo del investigador, y preservarse su contenido original.

#### Hipótesis y unidad de análisis

- RQ1: proporción de oportunidades de intervención **predeclaradas** que logran ejecución válida y oráculo decidible al permitir `environment_override` y `workspace_file_replace`.
- RQ2: diferencias observables reproducibles obtenidas por EnvMorph frente a un comparador exact-table **con las mismas oportunidades, confirmaciones y presupuesto**.
- RQ3: decisiones directamente verificadas, inferencias condicionadas al modelo H, errores, falsos positivos, abstenciones y costo de F6-v2. Separar niveles de evidencia.
- Unidad de análisis: workflow/proyecto independiente. No contar repeticiones, configuraciones o nodos del grafo como estudios independientes.

#### Campos que requieren congelación ANTES de la primera ejecución

1. **Cohorte:** 3 a 5 workflows upstream con repositorio, commit, licencia, motivo de inclusión/exclusión y criterio de independencia. Registrar candidatos excluidos y fallidos sin sustituirlos por resultados. Asegurar al menos un caso held-out predeclarado o documentar imposibilidad antes de observar etiquetas.
2. **Factores:** especificación M4-G2 completa por workflow, tipos admitidos, rutas de archivo/fixtures y SHA-256, versiones, valores baseline/treatment; máximo 6 factores por especificación M4-G2. Definir combinaciones si se estudian interacciones; M4-G2 por sí solo no es planificador factorial.
3. **Oráculos:** decidir antes qué salidas constituyen la firma observable (stdout, archivos concretos y normalización). Los oráculos de archivos de F3 **no se pueden pasar a F6-v2 como si fueran stdout**. Fijar explícitamente si el caso es compatible con `byte_stdout_vs_baseline_v1`; si no, abstención o extensión validada antes de experimentar.
4. **Holdout:** fijar celdas, separarlas realmente del conjunto de entrenamiento y conservar decisiones y abstenciones. Prohibido escoger una celda tras consultar su respuesta.
5. **Comparación justa:** exact-table y EnvMorph acceden a las mismas oportunidades, oráculos, presupuesto de tool calls, confirmaciones, tiempo y tratamiento de fallos. Registrar también las ventajas ausentes y empates.
6. **Controles:** controles positivos *conocidos* usados solo para verificar integración y controles negativos elegidos por criterios previos; no mezclarlos con el estimador de generalización. F10 y el piloto inih son exploratorios/históricos, no nuevos positivos confirmatorios.
7. **Instrumentación:** observador F9 opcional al camino crítico, pero obligatorio si se formula un claim de observación. Comprobar observer effect y excluir artefactos instrumentales solo por identidad exacta predeclarada. No clasificar resultados sin evidencia trazable.
8. **Seguridad:** ejecución sin red, permisos mínimos, cuotas y tiempos máximos, workspace temporal nuevo, imagen y dependencias con digests; no ejecutar upstream desconocido con el runner F9 sin sandbox.
9. **Presupuesto y STOP:** fijar límite total de configuraciones, ejecuciones y jornadas, tiempo máximo por llamada, reintentos permitidos y criterio de agotamiento. Reconciliar con `STOP_RULES_DRAFT.md` antes de congelar. Ningún aumento tras ver resultados por falta de positivos.
10. **Métricas:** registrar intentadas, válidas, fallidas, timeout, observadas, elegibles, ejecutadas, decidibles, relevantes, falsos positivos, abstenciones, costos, latencia, memoria, cobertura y desempeño held-out. Denominadores explícitos, IC solo cuando diseño lo permita.
11. **Proveniencia:** fijar hashes del manifiesto, plan, herramientas, binarios, imágenes, fixtures y evidencias primarias. Separar corrida original, reproducción mismo host y reproducción tercero en segundo host.
12. **Congelación:** crear commit de protocolo antes de observar etiquetas confirmatorias; ninguna modificación retroactiva. Cerrar tras presupuesto o regla STOP y producir auditoría GO/GO_CONDICIONAL/NO_GO.

#### Condiciones de parada obligatorias

STOP ante fuga de etiquetas held-out; comparador con acceso desigual; oráculo incompatible o cambiado post-hoc; fallo sistemático del aislamiento; incapacidad de documentar procedencia; o agotamiento del presupuesto predeclarado. Conservar negativos y fallos. La ausencia de positivos no permite añadir nuevos casos para obtener un resultado favorable.

#### Decisión de publicación

Un PASS de integración no implica un PASS científico. El objetivo editorial sigue siendo Paper 1 (`docs/PAPER_SCOPE.md`), que exige demostración de derivación de contrato y evaluación held-out decidida cuando sea viable. Si G3 no produce esa evidencia, proceder a reformulación de alcance o NO_GO, no a nuevas fases M5/M6.
