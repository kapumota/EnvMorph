### F10-MVP: protocolo mínimo

Selección previa a ejecutar EnvMorph. Benchmark SHA-256: `59db7a371740870eda0b50a08a39826e276409cc91b42bcfd48fc7f798a2dff9`.
El archivo benchmark.json congela seis candidatos potenciales y exactamente tres seleccionados.
Evidencia consultada: manual coreutils v9.4 y documentación CPython v3.12.3.
La documentación corresponde a funciones ambientales del software real. Son controles de sensibilidad documentada, no bugs ni aplicaciones representativas.

#### Diseño

Ejecutar date-tz, ls-quoting y python-encoding, sin sustituciones. Los factores son TZ,
QUOTING_STYLE y PYTHONIOENCODING. Los dos últimos son overrides ambientales ya
admitidos por F9.4, no extensiones del motor. Se prefieren a sort y mktemp por
no requerir locales adicionales ni normalización de aleatoriedad. Se incluyen
solo dos proyectos, la diversidad sigue siendo limitada.

Versiones de referencia: coreutils 9.4 y CPython 3.12.3. Se registran versiones,
hashes binarios y paquetes de distribución; no se afirma identidad de fuentes
upstream con un binario de Ubuntu. Si la versión difiere, abortar antes del experimento.

Entorno controlado: PATH=/usr/bin:/bin, LC_ALL=C, HOME en workspace temporal,
TZ=UTC0, PYTHONNOUSERSITE=1. Crear archivo vacío a b en workspace nuevo.
El comando Python únicamente proporciona entrada Unicode, no implementa getenv,
la selección de codificación ni el oráculo del intérprete.

#### Funnel y oráculo

Por caso: known_positive, observed, normalized, intervenable, eligible,
treatment_executed, oracle_decidable, behaviorally_relevant.
Estados PASS, FAIL, UNRESOLVED, NOT_APPLICABLE. No usar NOT_APPLICABLE para
ocultar una etapa bloqueada, las etapas dependientes bloqueadas son UNRESOLVED.
Observación mediante getenv_observer_atomic.c de F9 sin modificarlo. El alcance
es getenv dinámicamente interponible, no lecturas internas de libc ni todos los accesos.
No inventar eventos. Normalizar con F9.2, construir grafo con F9.3 y validar plan
con F9.4. Un override posible no demuestra elegibilidad sin observación real.

Dos repeticiones baseline y dos treatment mediante runner F9.4. Guardar evidencia
de aplicación y hashes. Capturar además stdout y stderr en ejecuciones aisladas
con entorno equivalente, comprobar hashes contra runner y salidas exactas congeladas.
F3 compare-artifacts --oracle byte evalúa cada par, puente F9.6 interpreta sus
resultados. Deben coincidir ambas confirmaciones y hashes dentro de cada variante.
Un resultado inesperado se conserva y queda unresolved/invalid_ground_truth,
no se cambia el oráculo. La salida esperada no se añade al plan F9, queda en F10.

#### Métricas y gate

Denominador principal tres controles conocidos. Recalls por etapa sobre tres.
Intervention_success_rate: treatments con rc=0 y aplicación confirmada sobre
casos con tratamiento intentado. Oracle_decidability: casos decidibles sobre
casos con tratamiento intentado. Si denominador cero, null, nunca cero inventado.
GO si al menos un positivo llega al final. Con cero, REPAIR solo con fallo
concreto reparable, repetir los mismos tres; en otro caso STOP. No ampliar corpus.

#### Alcance histórico y reproducción

No modificar F8/F9 ni sus manifests. F9 obtuvo 0/2 positivos entre decidibles,
no una estimación poblacional de baja relevancia. F10 es un benchmark conocido,
no prevalencia ni evaluación general de hermeticidad.
Distinguir reproducción de fuentes, ejecución, evidencia semántica y agregados.
PID y rutas temporales no tienen identidad byte a byte esperada. Guardar raw y
SHA-256 por ejecución; comparar semánticamente estados, salidas y métricas.
No ejecutar red durante los controles. Bootstrap puede descargar dependencias.
