### Reproducir F10-MVP

#### Alcance

F10 reproduce exactamente tres controles known-positive definidos en
`experiments/f10/benchmark.json`.

El benchmark está congelado por SHA-256:

```text
59db7a371740870eda0b50a08a39826e276409cc91b42bcfd48fc7f798a2dff9
```

F10 no reejecuta ni reescribe F8 o F9.

#### Baseline

F10 parte del baseline:

```text
044f31a7179832d4534991c7d51010a356fec94c
```

El freeze científico previo permanece en:

```text
v0.1.0 -> 29cc7961766f783ee4d642f3acf446fb6f9bdb27
```

#### Ruta nativa

Requisitos validados:

- Linux x86_64
- coreutils 9.4
- CPython 3.12.3 en `/usr/bin/python3`
- `cc`
- SHA-256
- Rust/Cargo con rustfmt

Preparación y validación:

```bash
bash reproduce/bootstrap.sh
cargo fmt --check
cargo test --locked --offline
cargo build --release --locked --offline
sha256sum -c provenance/F9_10_FILES.sha256

bash reproduce/f10.sh /tmp/envmorph-f10-run-1
python3 reproduce/verify_f10.py /tmp/envmorph-f10-run-1
python3 reproduce/test_f10.py /tmp/envmorph-f10-run-1
```

Usar un directorio nuevo para cada ejecución.

`reproduce/bootstrap.sh` puede acceder a red únicamente para `cargo fetch`.
Los controles F10 no necesitan red.

#### Ruta Docker recomendada para hosts con versiones distintas

Si el host no dispone de coreutils 9.4 o CPython 3.12.3, ejecutar:

```bash
bash reproduce/f10-docker.sh /tmp/envmorph-f10-run-docker
```

Esta ruta:

1. verifica formato, tests, build release y freeze F9.10 en el host;
2. construye `reproduce/Dockerfile.f10`;
3. comprueba dentro del contenedor coreutils 9.4 y CPython 3.12.3;
4. compila el observador F9 dentro del contenedor;
5. ejecuta F10 con `--network none`;
6. verifica evidencia y agregados.

La construcción de la imagen puede usar red para instalar paquetes. La
ejecución experimental se realiza sin red.

El repositorio se monta read-only dentro del contenedor.

#### Evidencia

Cada ejecución conserva:

- `environment.json`
- raw `getenv`
- eventos normalizados
- grafo
- plan
- elegibilidad
- resultados baseline y treatment
- stdout y stderr
- comparaciones F3
- evidencia adaptada por F9.6
- `results.json`
- `SHA256SUMS`

`verify_f10.py` recalcula métricas desde la evidencia y valida los estados PASS.

`test_f10.py` comprueba evidencia original, denominador vacío, raw adulterado,
adulteración semántica con hashes recalculados y agregados adulterados.

#### Niveles de reproducción

1. Fuentes: baseline y `Cargo.lock` fijados.
2. Ejecución: mismos tres controles, versiones y factores.
3. Evidencia semántica: mismos estados y diferencias observables.
4. Agregados: mismas métricas descriptivas.

No se exige igualdad byte a byte entre ejecuciones para PID, rutas temporales
u otros metadatos no científicos.

#### Límites

Los controles usan GNU coreutils y CPython y no representan una población de
software Unix.

Dos de tres controles pertenecen al mismo proyecto.

Las ejecuciones realizadas durante F10 no constituyen reproducción
independiente por un segundo equipo.

F8 y F9 históricos no se reconstruyen completamente mediante esta ruta.
