### PR #5: correcciones y estado de integración

PR #5, `https://github.com/kapumota/EnvMorph/pull/5`, fue fusionado en `main`
el 2026-11-16 mediante commit squash
`c53be087309628545737b0d807a669593e4fe6c9`. Los avisos de *no merge*
del expediente original pertenecían a la revisión previa a la fusión.

- A1: validación de directorio de salida canonizado, integrada.
- A2: CI Python 3.11 M3/M4 y pruebas adversariales del binario Rust, integradas.
- A3: distribución del TAR P1-EB-01 documentada, pero **reproducción
  independiente en segundo host DEFERRED**. La fusión no resuelve A3.
- A4: separación de selección por desacuerdo de hipótesis y contraste de
  falsación, integrada. `model_supported` es condicional, no verdad observada.
- A5: árbol regular sin symlinks ni archivos especiales, límites y cotejo
  integral de huellas, integrados. No equivale a sandbox ni elimina TOCTOU.

La rama posterior `research/m4-g3-prospective` contiene resultados **locales**
de ingeniería, no confirmación científica. Ver `docs/M4_G3_STATUS.md`. El
estado editorial Paper 1 sigue NO_GO. Antes de unir la rama G3: inspeccionar
diff, ejecutar CI y evaluar con revisión independiente si corresponde.
