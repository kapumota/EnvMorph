# G3-C, harness y workflows de la cohorte primaria (borrador v0.1)

Complementa el Protocolo G3-C v0.2 y el documento Cohorte G3-C. Todo es PROVISIONAL hasta el commit A (pool) y el commit C (congelamiento).

## Contenido

- factors.json             catalogo de factores y celdas. TZ = UTC y America/Lima, LC_ALL = C y C.UTF-8, TMPDIR = dos directorios.
- workflows/<nombre>/      spec.json (pasos, oraculo, digest del fixture, paquete y .deb fijados) y make_fixture.sh.
  Primarios: findutils, miller, unzip, exiftool, patch.
  _selftest_*: workflows ficticios para probar el harness. Nunca entran a la campana.
- harness/g3c_harness.py   aplica factores solo al proceso de la herramienta, ejecuta pasos auxiliares con entorno fijo, registra evidencia por ejecucion y aplica guardias.
- harness/g3c_decide.py    decide (F6-v2, C0, C1, C2), seal y score.
- harness/g3c_freeze.py    genera PROTOCOL_FREEZE.json.
- tests/test_g3c.py        pruebas del harness (5 pruebas, incluye un extremo a extremo con F6-v2 sobre workflows ficticios).
- docker/                  plantillas SIN PROBAR (no habia Docker donde se preparo).
- evidencia_preflight_sandbox/   registros de la unica ejecucion real que se hizo, solo celda baseline.

## Requisitos

Ubuntu 24.04 con los paquetes fijados (version y SHA-256 del .deb en cada spec.json), tzdata 2026e-0ubuntu0.24.04.1, Python 3.12 y el binario envmorph-f6v2 construido desde el commit de protocolo.

## Secuencia (la guardia obliga a respetarla)

1. Verificar el entorno, sin ejecutar herramientas.
   python3 harness/g3c_harness.py check-install
   python3 harness/g3c_harness.py check-env --verbose
2. Preflight (SOLO baseline, dos ejecuciones, para cada primario).
   python3 harness/g3c_harness.py preflight --workflow findutils --results results
3. Ratificar parametros, completar fichas, hacer commit A, B y C. Generar el congelamiento.
   python3 harness/g3c_freeze.py --protocol PROTOCOLO.docx --pool pool_manifest.json --workflows findutils miller unzip exiftool patch
   Firmar tag y depositar con sello temporal externo.
4. Etapa 1. Ejecutar celdas S y controles nulos, dos confirmaciones cada una.
   for cell in 000 100 010 001 NULLA NULLB; do python3 harness/g3c_harness.py run-cell --workflow findutils --cell $cell --results results; done
5. Etapas 2 y 3. Decidir y sellar. Despues del sellado, hacer commit y depositar con sello temporal.
   python3 harness/g3c_decide.py decide --workflows findutils miller unzip exiftool patch --f6v2-bin target/release/envmorph-f6v2 --results results
   python3 harness/g3c_decide.py seal
6. Etapa 4. Ejecutar celdas H.
   for cell in 110 101 011 111; do python3 harness/g3c_harness.py run-cell --workflow findutils --cell $cell --results results; done
7. Etapa 5. Puntuar y resumir.
   python3 harness/g3c_decide.py score
   python3 harness/g3c_harness.py summarize --workflow findutils --results results

## Lo verificado en el entorno de preparacion (Ubuntu 24.04, Python 3.12.3)

- check-install PASS. Versiones de paquetes y tzdata coinciden con lo fijado.
- check-env PASS en las 10 celdas (8 del reticulo y 2 nulas). No se ejecuto ninguna herramienta.
- preflight PASS para los cinco primarios con dos ejecuciones baseline identicas, fixture con digest fijado y reconstruccion identica.
- 5 pruebas del harness OK. Verifican entorno por celda, guardias (sin congelamiento, H sin sello, fuga de H) y un flujo extremo a extremo sobre workflows ficticios.
- No se ejecuto ninguna celda de tratamiento de los cinco primarios. Los efectos de TZ, LC_ALL y TMPDIR sobre las salidas reales siguen sin observarse, como exige el protocolo.

## No verificado o pendiente

- Docker. Las plantillas de docker/ no se probaron. La imagen debe fijarse por digest.
- La evidencia de aplicacion de factores es por construccion identica (env -0 con el mismo diccionario). No lee /proc/PID/environ del proceso real de la herramienta.
- Los comparadores C4 (ddmin), C5 (reprotest) y C6 (BugDoc) no estan implementados. Se corren aparte y se registran.
- Los comandos de miller, exiftool y patch tienen hipotesis de efecto no verificadas. Miller no deberia reaccionar a LC_ALL ni TMPDIR, lo que sirve como observacion de especificidad.
- Las reservas less, tree y moreutils no tienen workflow implementado. less casi no cambia sin terminal y ts depende del reloj.
- La regla de salida outcome_hint de score es una pista automatica. La clasificacion final requiere juicio humano.

## Enmiendas respecto al protocolo v0.2 que debe ratificar

- V2-6. Niveles de TZ como nombres IANA (UTC y America/Lima) en lugar de UTC0 y EST5. Las cadenas POSIX como EST5 pueden no interpretarse en programas que leen TZ como nombre de zoneinfo. Se fija tzdata.
- V2-7. Las decisiones se generan y sellan con g3c_decide.py. El harness impide ejecutar H sin sello y aborta si detecta H ejecutada antes de decidir.
