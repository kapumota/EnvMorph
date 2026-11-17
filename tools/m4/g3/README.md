### G3: arnés experimental archivado y alcance

`g3_pilot.py` se copia de `~/Descargas/EnvMorph_G3_Piloto_Offline_v2/g3_pilot.py` durante la actualización documental, **sin reescribir sus bytes**. Es el arnés de piloto con las correcciones `--mount` y compilador C++ aplicado. Su SHA-256 se registra en `experiments/m4/g3/G3_EVIDENCE_INDEX.json`.

El arnés se ejecutó originalmente **fuera** del repositorio, a partir de una copia del runtime congelado F8, ruedas de Meson/Ninja ya disponibles y una imagen derivada de Docker. No hay garantía de que un clone nuevo tenga wheelhouse, source upstream, Docker ni el mismo host. El script se conserva para auditoría de lo realmente ejecutado, **no** como componente estable de biblioteca o interfaz pública.

La imagen `envmorph-g3-inih-cpp:local` se derivó de `envmorph-g3-inih-offline:local` mediante `Dockerfile.cpp`, con instalación `apt-get` de `g++` durante la construcción. Esa etapa **sí usó red**. El piloto se ejecutó con `--network none`; no confundir ambos momentos.

Para revisar los resultados, consultar `docs/M4_G3_STATUS.md` y los expedientes JSON de `experiments/m4/g3/`. Para repetición real, restaurar dependencias fijadas, materiales primarios y fijar una nueva imagen por digest en el equipo que reproduce. El ejecutor original no debe reutilizarse como sandbox general sin auditoría.

No modificar F8–F10 históricos, ni interpretar igualdad de artefactos como ausencia universal de sensibilidad. Los oráculos F3 de archivos no son intercambiables con el `byte_stdout_vs_baseline_v1` exigido por F6-v2.

Verificación local de anclas después de actualizar:

```bash
python3.11 tools/m4/g3/verify_g3_index.py --repo .
```

Esta comprobación solo compara hashes; no sustituye una reproducción independiente ni verifica los datos primarios fuera del repositorio.
