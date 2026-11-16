# EnvMorph M3: replay retrospectivo con binario Rust F6-v2

## Uso

El paquete se ejecuta desde un Ubuntu donde M2 haya superado `M2_LOCAL_VALIDATION=PASS`. **No instalar M2 otra vez ni modificar Rust.**

```bash
cd ~/Descargas
unzip -o EnvMorph_M3_Replay.zip -d ~/Descargas
cd ~/Descargas/EnvMorph_M3_Replay
sha256sum -c SHA256SUMS

bash ./preflight.sh ~/EnvMorph ~/envmorph-p1-eb01-20261115T235104Z.tar.gz
python3.11 ./m3_replay.py --selftest

cd ~/EnvMorph
cargo build --release --locked --bin envmorph-f6v2

set -o pipefail
python3.11 ~/Descargas/EnvMorph_M3_Replay/m3_replay.py \
  --archive ~/envmorph-p1-eb01-20261115T235104Z.tar.gz \
  --rust-binary ~/EnvMorph/target/release/envmorph-f6v2 \
  --output ~/envmorph-m3-retrospectivo-20261116 \
  2>&1 | tee ~/Descargas/envmorph_m3_replay.log
rc=${PIPESTATUS[0]}
echo "CODIGO_M3=$rc"

python3.11 ~/Descargas/EnvMorph_M3_Replay/m3_stress.py \
  --rust-binary ~/EnvMorph/target/release/envmorph-f6v2 \
  --output ~/envmorph-m3-xor-stress-20261116

git status --short
```

La ejecución real usa F6-v2 de Rust para cada checkpoint; no se simulan sus resultados. El archivo P1 se verifica íntegro y se consulta solo como oráculo de replay, nunca se altera ni se ejecutan programas Unix. La carpeta de resultados **no puede existir previamente**: para un reintento diagnóstico utilice un sufijo distinto, conservando el intento original.

## Resultados esperados del arnés

- `M3_PREFLIGHT=PASS`
- `M3_HARNESS_SELFTEST=PASS`
- `M3_P1_ARCHIVE_VERIFIED=725`
- `M3_RUST_REPORTS=432`
- `M3_REPLAY=PASS`
- `CODIGO_M3=0`
- `M3_OOD_EARLY_WRONG=OBSERVED`
- `M3_OOD_INCONSISTENCY=DETECTED`
- `M3_STRESS=PASS`

`M3_REPLAY=PASS` significa **solo ejecución técnica** del replay retrospectivo. `M3_STRESS=PASS` significa que el prototipo muestra correctamente su limitación ante un modelo incorrecto. Ninguno permite afirmar aceptación Q1/Q2.

## Archivos que compartir para evaluar M3

- `~/envmorph-m3-retrospectivo-20261116/M3_METADATA.json`
- `~/envmorph-m3-retrospectivo-20261116/M3_AGGREGATES.json`
- `~/envmorph-m3-retrospectivo-20261116/M3_RESUMEN.md`
- `~/envmorph-m3-xor-stress-20261116/M3_STRESS.json`

No abra PR ni commit hasta revisar el resultado. El script original no modifica Git; sólo las compilaciones ya autorizadas escriben en `target/`.
