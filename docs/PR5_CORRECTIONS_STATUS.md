### PR #5: correcciones de auditoría

- A1: protección canónica del directorio de salida, corrección validada
  localmente. Pendiente comprobar CI remoto.
- A2: nuevo job CI Python 3.11 para M3/M4 y prueba adversarial XOR integrada
  al job Rust. Pendiente comprobar ejecución real en GitHub Actions.
- A3: instrucciones de distribución y reproducibilidad en
  `experiments/m3/retrospective/EVIDENCE_ACCESS.md`. **Pendiente distribuir
  el TAR original o un derivado trazable y reproducir en otro equipo.**
- A4: selección por desacuerdo positivo; búsqueda de falsación del modelo
  separada, sin prometer información ni predicción correcta.
- A5: política conservadora que rechaza symlinks y archivos especiales en
  el árbol completo, exige límites y coteja huellas integrales; no es sandbox
  ni elimina TOCTOU con adversario concurrente.

**No merge** hasta obtener CI actualizado, revisión del diff y aceptación
explícita de la frontera de seguridad. **No GO científico** hasta contar
con evidencia prospectiva externa y comparadores justos.
