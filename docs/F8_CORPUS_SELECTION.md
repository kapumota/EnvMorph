### F8.1 Selección auditable del corpus externo

#### Estado

Selección congelada antes de ejecutar EnvMorph sobre los workloads externos.

La selección sigue `docs/F8_PROTOCOL.md` versión 1. El objetivo primario es cuatro workloads. Ningún workload se eligió por una divergencia observada, por un bug ambiental conocido ni por un resultado previo de EnvMorph.

Los factores mencionados en este documento son únicamente hipótesis técnicas previas. No quedan activados por esta selección.

#### Criterios aplicados

Cada workload seleccionado satisface los criterios preregistrados:

- código fuente público con revisión fijable
- licencia compatible con reproducción académica
- ejecución Unix o Linux no interactiva
- workflow de build, ETL, transformación de archivos o utilidades Unix
- posibilidad de ejecución offline después de preparar dependencias
- artefactos observables
- costo compatible con repeticiones locales
- preparación documentable sin cambiar la semántica upstream
- procedencia independiente de EnvMorph

No se ha ejecutado EnvMorph para decidir la inclusión.

#### W1 csvkit-realdata

Proyecto de origen: `wireservice/csvkit`.

Procedencia: `https://github.com/wireservice/csvkit`.

Versión congelada para F8: tag `2.2.0`.

Commit: `bdd6e7e25b52cb52073c1a19dd0210cd85894871`.

Licencia: MIT, archivo upstream `COPYING`.

Lenguaje y herramientas: Python, csvkit, shell POSIX para encadenamiento y redirección.

Tipo: ETL y transformación tabular.

Entrada upstream principal: `examples/realdata/ne_1033_data.xlsx`.

Preparación reproducible propuesta:

```text
python3 -m venv .venv
. .venv/bin/activate
python -m pip install .
```

Ejecución reproducible propuesta:

```text
mkdir -p out
in2csv examples/realdata/ne_1033_data.xlsx > out/data.csv
csvcut -c county,item_name,quantity out/data.csv > out/selected.csv
csvlook out/selected.csv > out/selected.txt
```

Outputs observables:

- `out/data.csv`
- `out/selected.csv`
- `out/selected.txt`
- stdout, stderr y código de salida de cada etapa

Motivo de inclusión: pipeline ETL real documentado por upstream, con varias etapas distinguibles y artefactos CSV y texto.

Criterios satisfechos: fuente pública, versión fijable, MIT, Unix no interactivo, ETL, ejecución offline tras preparar dependencias, artefactos intermedios y finales, costo local bajo, procedencia independiente.

Posibles factores previos, no activados: `locale`, relación `LC_ALL` y `LANG`, `path_tool_resolution`.

Oráculos existentes previsibles: CSV y texto.

Viabilidad local: alta, condicionada a preparar y congelar previamente las dependencias Python.

#### W2 jq-build-check

Proyecto de origen: `jqlang/jq`.

Procedencia: `https://github.com/jqlang/jq`.

Versión congelada para F8: tag `jq-1.8.2`.

Commit: `34f7186b86743a083a589741b6cea95293524108`.

Licencia: MIT para jq, con avisos de terceros preservados en `COPYING`.

Lenguaje y herramientas: C, Autotools, Make, shell.

Tipo: build y validación de una utilidad de transformación JSON.

Preparación reproducible propuesta para checkout Git:

```text
git submodule update --init
autoreconf -i
./configure --with-oniguruma=builtin
```

Ejecución reproducible propuesta:

```text
mkdir -p out
make -j2 > out/build.stdout 2> out/build.stderr
make check > out/check.stdout 2> out/check.stderr
cp jq out/jq
```

Outputs observables:

- `out/jq`
- `out/build.stdout`
- `out/build.stderr`
- `out/check.stdout`
- `out/check.stderr`
- códigos de salida de build y tests

Motivo de inclusión: build Unix convencional de una herramienta real de transformación JSON, con build y tests upstream y varios observables.

Criterios satisfechos: fuente pública, commit fijable, licencia compatible, Unix no interactivo, build reproducible, ejecución offline después de preparar toolchain y submódulo, artefactos observables, costo local razonable, procedencia independiente.

Posibles factores previos, no activados: `shell_implementation`, `path_tool_resolution`, `TMPDIR`, `umask`, `locale`.

Oráculos existentes previsibles: byte y texto. JSON queda disponible para etapas posteriores solo si se fija previamente una transformación concreta.

Viabilidad local: alta, condicionada a disponer de Autotools, Make, compilador C y materializar el submódulo fijado por upstream.

#### W3 fd-build-completions

Proyecto de origen: `sharkdp/fd`.

Procedencia: `https://github.com/sharkdp/fd`.

Versión congelada para F8: tag `v10.5.0`.

Commit: `4f81778774463bf414a184cbe6d5219ad2229646`.

Licencia: `MIT OR Apache-2.0`, archivos upstream `LICENSE-MIT` y `LICENSE-APACHE`.

Lenguaje y herramientas: Rust 2024, Cargo, shell para orquestación.

Requisito upstream relevante: Rust `1.90.0`.

Tipo: build, tests y generación de artefactos de completado de shell.

Preparación reproducible propuesta:

```text
cargo fetch --locked
```

Ejecución reproducible propuesta después de congelar dependencias:

```text
mkdir -p out
cargo build --locked --offline > out/build.stdout 2> out/build.stderr
cargo test --locked --offline > out/test.stdout 2> out/test.stderr
target/debug/fd --gen-completions bash > out/fd.bash
target/debug/fd --gen-completions zsh > out/_fd
target/debug/fd --gen-completions fish > out/fd.fish
```

Outputs observables:

- `out/fd.bash`
- `out/_fd`
- `out/fd.fish`
- `out/build.stdout`
- `out/build.stderr`
- `out/test.stdout`
- `out/test.stderr`
- binario `target/debug/fd`
- códigos de salida

Motivo de inclusión: workflow Rust real con lockfile, build y tests reproducibles y generación determinista de varios artefactos textuales.

Criterios satisfechos: fuente pública, revisión fijable, licencia permisiva, Unix no interactivo, build y transformación de archivos, posibilidad offline tras `cargo fetch`, artefactos intermedios y finales, costo compatible con repetición local, procedencia independiente.

Posibles factores previos, no activados: `path_tool_resolution`, `TMPDIR`, `umask`, `shell_implementation`.

Oráculos existentes previsibles: byte y texto.

Viabilidad local: alta si existe Rust 1.90.0 o superior y las dependencias Cargo se preparan antes de la ejecución offline.

#### W4 inih-meson-tests

Proyecto de origen: `benhoyt/inih`.

Procedencia: `https://github.com/benhoyt/inih`.

Versión congelada para F8: tag `r62`.

Commit: `26254ee9de7681f8825433415443e7116ff24b98`.

Licencia: BSD-3-Clause, archivo upstream `LICENSE.txt`.

Lenguaje y herramientas: C, C++, Meson, Ninja o backend compatible.

Tipo: build y tests de parser INI.

Preparación reproducible propuesta:

```text
meson setup build -Dtests=true -Dwith_INIReader=true
```

Ejecución reproducible propuesta:

```text
mkdir -p out
meson compile -C build > out/build.stdout 2> out/build.stderr
meson test -C build --print-errorlogs > out/test.stdout 2> out/test.stderr
```

Outputs observables:

- bibliotecas y ejecutables bajo `build`
- `out/build.stdout`
- `out/build.stderr`
- `out/test.stdout`
- `out/test.stderr`
- códigos de salida
- baselines de tests upstream cuando sean consumidos por el runner

Motivo de inclusión: build C/C++ pequeño y repetible con tests y baselines upstream, útil para validar la cadena F2 a F7 sin seleccionar por sensibilidad esperada.

Criterios satisfechos: fuente pública, revisión fijable, BSD-3-Clause, Unix no interactivo, build, offline tras preparar toolchain, artefactos observables, costo local bajo, procedencia independiente.

Posibles factores previos, no activados: `path_tool_resolution`, `TMPDIR`, `umask`, `shell_implementation`.

Oráculos existentes previsibles: byte y texto.

Viabilidad local: muy alta si Meson, Ninja y compiladores C/C++ están disponibles.

#### Decisión de selección

Corpus F8 primario:

```text
csvkit-realdata
jq-build-check
fd-build-completions
inih-meson-tests
```

Número de workloads: 4.

No existe desviación respecto del tamaño objetivo preregistrado.

Ninguno puede eliminarse posteriormente porque produzca cero divergencias, `identical`, `equivalent`, `unavailable`, `execution_failed`, `unresolved`, `out_of_scope`, un envelope parcial o una predicción held-out incorrecta.

#### Estado de factores y oráculos

La selección no activa factores nuevos.

La selección no añade oráculos nuevos.

Antes de la matriz experimental se debe congelar el corpus, materializar provenance, preparar adapters mínimos y justificar la matriz workload por factores por oráculos a partir de evidencia del corpus ya seleccionado.

#### Leads no incorporados al corpus

Durante discovery se revisaron otros proyectos potenciales. No se registran como workloads excluidos porque no llegaron a ser candidatos formalmente admitidos bajo el procedimiento F8.1. Esta distinción evita inventar exclusiones retrospectivas.

#### Gate F8.1

F8.1 queda cerrada cuando este documento, `experiments/f8/corpus_selection.toml` y sus hashes se encuentran en un commit de `fase8/validacion-externa`, sin ejecución de EnvMorph sobre los cuatro workloads.
