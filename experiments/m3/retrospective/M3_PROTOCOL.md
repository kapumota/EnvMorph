# M3: replay retrospectivo con presupuesto controlado

**Estado de la decisión:** preparación de la prueba M3, posterior al conocimiento de P1. El resultado nunca será un ensayo confirmatorio independiente ni una demostración de novedad.

## Evidencia de origen

- Archivo P1-EB-01: SHA-256 `9ba4652dbb6ee098a43c288bda699b7f9e86572383c13126ed397f90d1e13288`.
- Versión de protocolo P1: SHA-256 `da5151445aec76926a9f61fa5ce5cff2591653deaf7d1c834f147bbbf7f36624`.
- Commit original de `main`: `99f89f9a66cf24a6afa59ea8114ee3a6600a4510`.
- Doce pares (C1–C6, primary/negative), cuatro configuraciones por par; dos proyectos upstream, no doce estudios independientes.
- Dos referencias directas por configuración; estas 96 ejecuciones funcionan como oráculo histórico y **no se repiten**.

## Modelo y asignaciones

- Orden de bits: `target_level` seguido de `benign_level`.
- Familia H de F6-v2: constantes 0 y 1 y conjunciones de a lo sumo dos literales firmados.
- Oráculo de etiqueta: `stdout` binario de asignación contra `stdout` baseline 00 del mismo caso/modo.
- Confirmación: dos ejecuciones byte a byte concordantes por asignación; se conserva su procedencia en `M3_RESULTS.csv`.
- **Todas las políticas comienzan en 00** con dos repeticiones, que se imputan al presupuesto.

## Políticas fijadas antes del replay

- `lexicographic`: primera asignación sin consultar.
- `random`: muestreo sin reposición; semillas 0,1,2,3,4 fijadas y derivadas del identificador de workflow; no informar solo la mejor.
- `pairwise`: cobertura de pares bajo desempate lexicográfico. Con **solo dos factores** esta política se degenera a un desempate: no constituye una evaluación informativa de covering arrays.
- `rust_suggested`: usa únicamente `suggested_next_assignment` del reporte Rust anterior y registra fallbacks lexicográficos.
- `greedy_coverage`: expectativa de cobertura condicional bajo una distribución uniforme en H; solo accede a etiquetas consultadas.

## Presupuestos y métricas

- Presupuestos por workflow: 2, 4, 6 y 8 ejecuciones primitivas; 2 por consulta, baseline incluido.
- Una tabla exacta con dos confirmaciones **comparte las mismas consultas para cada política**. Su cobertura directa es exactamente el número de asignaciones adquiridas.
- Métricas: cobertura directa, inferencias condicionadas, errores de inferencia contra la referencia, abstenciones, inconsistencias de H y fallbacks; registrar cada modo y caso.
- Todas las salidas de Rust y los manifiestos verificables se conservan en la carpeta de resultados. Las referencias de procedencia señalan las rutas dentro del TAR original; las copias llevan SHA-256.
- El método propuesto todavía no mide latencia o costo energético de ejecutar workflows reales: el presupuesto cuenta ejecuciones primitivas, no tiempo total o uso de CPU.

## Límites científicos

1. El conjunto P1 ya era conocido cuando se diseñó M3. El experimento solo verifica la ingeniería del comparador y expone posibles errores.
2. Las políticas conocen el orden de factores y la familia H, no las etiquetas futuras. La fuente del oráculo histórico está separada de la función de selección.
3. `evidence_verified` requiere dos salidas, pero no autentica un proceso independiente.
4. `model_supported` es condicional a H. Un modelo incorrecto puede producir falsos positivos o negativos antes de que se detecte la inconsistencia.
5. Sin workflows de 3–6 factores no se puede medir la ventaja real de la selección adaptativa ni interacciones complejas.
6. No confundir 48 celdas con 48 unidades científicas independientes.
7. No hay comparación válida todavía con BugDoc/reprotest/ddmin; no atribuir superioridad.
8. La prueba adversarial XOR de 3 factores es **sintética, solo regresión del prototipo**, no evidencia del artículo.

## Gate posterior

`GO` para la **implementación de M4** solo si el replay reproduce el mismo estado en Rust, documenta presupuestos iguales, conserva sus hashes y no confunde inferencias con decisiones directas. Para un `GO` científico del Paper 1 se necesitan previamente workflows externos, preselección congelada, comparadores competitivos y reproducción independiente.
