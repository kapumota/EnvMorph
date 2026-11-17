### M4-G3: preparación de auditoría integral e independiente

Estado: lista de verificación **previa**. No es un dictamen ni una auditoría ya completada.

#### 1. Código y ramas

- [ ] Confirmar base `main` c53be087309628545737b0d807a669593e4fe6c9, PR #5 fusionado; registrar HEAD final de rama G3 y PR propio.
- [ ] Revisar diff y cobertura de `tools/m4/interventions/m4_interventions.py` y copia archivada de `tools/m4/g3/g3_pilot.py`.
- [ ] Comprobar que el script versionado coincide byte a byte con el script local utilizado y que se conserva su SHA-256; no afirmar que fue inmutable durante intentos anteriores.
- [ ] `cargo fmt --check`, `cargo test --locked`, `cargo build --release --locked`, pruebas Python y CI remota en el mismo commit.
- [ ] Confirmar que no hay modificaciones a F8–F10/P1 congelados ni artefactos históricos.

#### 2. Datos y reproducción

- [ ] Verificar índices SHA-256 y evidencia resumida G3 en repositorio.
- [ ] Empaquetar por separado evidencia primaria G3, intentos fallidos y exitosos, registros de herramienta, resultados F3 y F9.6, wheelhouse, recetas de imágenes y hashes.
- [ ] Verificar que un revisor pueda obtener artefactos sin acceso a rutas privadas del host; declarar cualquier material que falte.
- [ ] Reproducir desde un clone limpio en segundo host; la reproducción por el propio desarrollador en un contenedor no basta para declararla independiente.
- [ ] Reproducir P1-EB-01 con TAR de hash `9ba4652dbb6ee098a43c288bda699b7f9e86572383c13126ed397f90d1e13288`; A3 sigue DEFERRED hasta verificación externa.

#### 3. Seguridad

- [ ] Auditar restricciones Docker efectivas (red, root, capacidades, montaje writable, cuotas, tmpfs, uid, kernel y seccomp).
- [ ] Revisar problemas TOCTOU, symlinks, archivos especiales, normalización de paths, límites de recursos y mutabilidad de imágenes locales.
- [ ] Distinguir build de imagen con `apt-get` por red de ejecución experimental sin red.
- [ ] Evaluar fuga de información del host y comportamiento ante fallo.

#### 4. Validez científica

- [ ] Mantener separados F8 negativo, F9 elegibilidad limitada, F10 known-positive, P1, M3 retrospectivo y G3 piloto.
- [ ] Revisar prerregistro auténtico, selección de workflows e independencia de unidades.
- [ ] Comprobar equivalencia de presupuestos, oportunidades, oráculos y confirmaciones frente al comparador exact-table.
- [ ] Auditar holdout y ausencia de entrenamiento o selección sobre etiquetas futuras.
- [ ] Validar trazas F3/F4/F5/F6 sin convertir `model_supported` en evidencia observada.
- [ ] Calcular fallos, indecidibles, abstenciones y límites estadísticos usando denominadores correctos.
- [ ] Evaluar comparación con trabajos previos sin afirmar superioridad no demostrada.

#### 5. Dictamen final y STOP

Emitir tres dictámenes separados: reproducibilidad técnica, validez científica y madurez editorial. Cada uno GO/GO_CONDICIONAL/NO_GO con evidencia y riesgos. Distinguir aceptación de código de envío Paper 1. Si la campaña termina sin divergencia útil y held-out evaluable, no ampliar el trabajo indefinidamente; considerar reformulación del claim o cierre.
