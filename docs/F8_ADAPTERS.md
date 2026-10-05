### F8.3 adapters externos

#### Regla

El materializador histórico de F8.2 se conserva byte-exact porque forma parte del freeze. F8.3 registra `scripts/f8/materialize_corpus_verified.sh`, que corrige únicamente la resolución del path del manifest SHA-256 y no modifica snapshots ni revisiones upstream.

Los adapters viven fuera de `source` y no corrigen, normalizan ni alteran el comportamiento upstream. Su única función es materializar comandos upstream en etapas observables para F2-F7.

#### csvkit-realdata

Reproduce el flujo documentado `in2csv -> csvcut -> csvlook` sobre `examples/realdata/ne_1033_data.xlsx`. Conserva CSV convertido, proyección de columnas y preview textual.

#### jq-build-check

Reproduce la ruta de build desde Git mediante `autoreconf`, `configure --with-oniguruma=builtin`, `make` y `make check`. El gitlink de Oniguruma se materializa en una copia runtime, nunca en el snapshot F8.2.

Los logs de build no se usan como observaciones de equivalencia porque contienen detalles de rutas del workspace que podrían introducir diferencias artificiales. Se observan la versión del binario, una transformación sobre el fixture upstream `tests/modules/data.json` y el estado semántico de la suite.

`configure.ac` consulta explícitamente `setlocale`, `strftime`, `localtime` y otras funciones de tiempo. Esto constituye evidencia previa para considerar `locale` y `timezone`, sin afirmar que necesariamente producirán divergencia.

#### fd-build-completions

Ejecuta los tests Cargo y genera completions bash y zsh mediante la opción documentada `--gen-completions`. Cargo.lock se conserva y las crates se vendorizarán antes de la matriz para ejecución offline. El log completo de Cargo no se compara porque puede contener rutas de workspace; se conserva el estado de éxito y los artefactos funcionales.

#### inih-meson-tests

Utiliza el sistema Meson upstream con `tests=true`, compilación y suite de tests contra los baselines incluidos por el proyecto. Como observables se conservan el estado de la suite y la salida real de `unittest_multi`, que el propio `runtest.sh` upstream compara contra `baseline_multi.txt`; no se comparan logs de Meson con rutas específicas del workspace.

#### Factores

F8.3 no activa factores. `locale` y `timezone` ya existen en F2. `tmpdir` continúa como candidato preregistrado para los workloads de build y solo podrá incorporarse en F8.4 con prueba explícita y matriz congelada antes de la ejecución.
