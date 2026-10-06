### Erratum F9.8A0: integridad del runtime fd

#### Motivo

El gate inicial F9.8A comparaba todo el directorio `source` del runtime `fd-build-completions` contra el manifest upstream F8.

Eso es incorrecto para un archivo:

```text
source/.cargo/config.toml
```

Durante la preparación runtime F8, ese archivo fue generado deliberadamente por:

```text
cargo vendor --locked ../vendor > .cargo/config.toml
```

Por tanto, la diferencia respecto al snapshot upstream es una transformación runtime preregistrada, no drift del corpus.

#### Evidencia previa a la corrección

El diagnóstico F9.8A mostró que todos los archivos del snapshot `fd` coincidían salvo `.cargo/config.toml`.

Ningún workload externo F9.8A había sido ejecutado cuando se detectó y corrigió este gate.

#### Regla corregida

Para `fd-build-completions`:

1. todos los archivos upstream excepto `.cargo/config.toml` deben coincidir byte-exactamente con el manifest F8;
2. el vendor tree debe coincidir byte-exactamente con `provenance/f8/runtime/fd-vendor.files.sha256`;
3. `.cargo/config.toml` debe enlazar `crates-io` con `vendored-sources`;
4. `vendored-sources` debe usar la ruta relativa `../vendor`;
5. no se acepta una ruta vendor absoluta.

#### Alcance

Este erratum no cambia:

- corpus;
- backend de observación;
- baseline ambiental;
- normalización;
- exclusiones instrumentales;
- reglas de candidatos;
- tratamientos;
- elegibilidad;
- oráculos;
- clasificación.

Es una corrección del gate de integridad local anterior a la observación externa.
