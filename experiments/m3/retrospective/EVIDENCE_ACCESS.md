# M3: acceso y verificación del expediente P1-EB-01

## Estado de reproducibilidad

La rama PR #5 contiene código y agregados, no las 96 salidas originales ni todos los
informes intermedios de Rust. **A3 sigue pendiente** hasta que se publique un
paquete inspeccionado, con descarga accesible a revisores independientes.

El TAR histórico no debe alterarse para ocultar rutas, fechas o resultados: si
requiere redacción de datos sensibles, publicar un derivado distinto con su
propia huella, manifiesto de diferencias y limitaciones, sin afirmar equivalencia
byte a byte con el TAR congelado.

- SHA-256 del TAR original: `9ba4652dbb6ee098a43c288bda699b7f9e86572383c13126ed397f90d1e13288`.
- SHA-256 del protocolo: `da5151445aec76926a9f61fa5ce5cff2591653deaf7d1c834f147bbbf7f36624`.
- Código fuente de la campaña: `99f89f9a66cf24a6afa59ea8114ee3a6600a4510`.
- 96 ejecuciones de referencia, 48 celdas con dos confirmaciones, 725 archivos
  internos verificados; dos proyectos upstream (no 48 estudios independientes).

## Pasos antes de distribuir el TAR

1. Inspeccionar manualmente nombres de archivo, stdout, stderr, comandos,
   metadatos y rutas de trabajo. Buscar contraseñas, tokens, datos personales,
   identificadores internos, rutas privadas o archivos de terceros no redistribuibles.
2. Comprobar la identidad exacta del archivo original:

   ```bash
   sha256sum ~/envmorph-p1-eb01-20261115T235104Z.tar.gz
   ```

3. Elegir un canal de distribución sostenible y accesible para terceros
   (por ejemplo, un release de GitHub o un repositorio de artefactos con DOI).
   Registrar URL real, versión, autor, licencia de los datos y checksum.
   **No inventar ni incluir enlaces hasta completar la publicación.**
4. Verificar mediante descarga desde otro equipo, sin utilizar las rutas locales
   del desarrollador, y ejecutar:

   ```bash
   cargo build --release --locked --bin envmorph-f6v2
   python3.11 experiments/m3/retrospective/m3_replay.py \
     --archive /ruta/del/TAR-original.tar.gz \
     --rust-binary target/release/envmorph-f6v2 \
     --output /ruta/nueva/replay-independiente
   ```

5. Conservar `M3_METADATA.json`, `M3_AGGREGATES.json`, `M3_RESULTS.csv`,
   `M3_RESUMEN.md` y todos los reportes Rust generados; compararlos con los
   resultados versionados, distinguiendo posibles diferencias de rutas/tiempos.

## Criterio de cierre A3

`A3_REPRODUCIBLE=PASS` requiere URL de descarga accesible, huella correcta,
verificación completa de 725 archivos y reproducción en otro host.
Un documento de procedimiento o un PASS en la máquina original **no cierra A3**.
