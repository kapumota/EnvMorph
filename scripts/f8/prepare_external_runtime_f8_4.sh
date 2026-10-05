#!/usr/bin/env bash
# F8.4: derivado de scripts/f8/prepare_external_runtime.sh.
# Motivo: un venv mínimo no aporta bdist_wheel. Se congela wheel en el
# wheelhouse antes de construir csvkit. El preparador F8.3 permanece intacto.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUNTIME_ROOT="$ROOT/.envmorph/f8-runtime"
PROV="$ROOT/provenance/f8/runtime"
IDS=(csvkit-realdata jq-build-check fd-build-completions inih-meson-tests)
die(){ printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Falta $1"; }
need git; need sha256sum; need python3; need bash; need find; need sort; need xargs
cd "$ROOT"
bash scripts/f8/materialize_corpus_verified.sh
rm -rf "$RUNTIME_ROOT"
mkdir -p "$RUNTIME_ROOT" "$PROV"
for id in "${IDS[@]}"; do
  rt="$RUNTIME_ROOT/$id"
  mkdir -p "$rt"
  cp -a "workloads/external/$id/source" "$rt/source"
  cp -a "workloads/external/$id/adapter" "$rt/adapter"
done

# csvkit: resolver dependencias una vez y después instalar sin red.
python3 - <<'PY'
import sys
if sys.version_info < (3, 10):
    raise SystemExit("csvkit 2.2.0 requiere Python >= 3.10")
PY
need "python3"
python3 -m venv "$RUNTIME_ROOT/csvkit-realdata/deps/venv"
CSVV="$RUNTIME_ROOT/csvkit-realdata/deps/venv/bin"
mkdir -p "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse"

# csvkit 2.2.0 usa setup.py para el build legacy. El comando bdist_wheel
# no está disponible en un venv mínimo hasta instalar wheel.
# Se descarga wheel al mismo wheelhouse que quedará congelado por SHA-256,
# se instala desde allí y solo después se construye csvkit.
"$CSVV/pip" download   --disable-pip-version-check   --only-binary=:all:   --dest "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse"   wheel

"$CSVV/pip" install   --disable-pip-version-check   --no-index   --find-links "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse"   wheel

"$CSVV/pip" wheel   --disable-pip-version-check   --wheel-dir "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse"   "$RUNTIME_ROOT/csvkit-realdata/source"

"$CSVV/pip" install   --disable-pip-version-check   --no-index   --find-links "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse"   csvkit==2.2.0
"$CSVV/pip" freeze --all | LC_ALL=C sort > "$PROV/csvkit-pip-freeze.txt"
( cd "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum ) > "$PROV/csvkit-wheelhouse.sha256"

# jq: materializar exactamente el gitlink vendor/oniguruma sin modificar el snapshot F8.2.
need autoreconf; need make; need cc
JQ_COMMIT="34f7186b86743a083a589741b6cea95293524108"
tmp="$(mktemp -d)"
git -C "$tmp" init -q
git -C "$tmp" remote add origin https://github.com/jqlang/jq.git
git -C "$tmp" fetch -q --depth 1 origin "$JQ_COMMIT"
onig_commit="$(git -C "$tmp" ls-tree "$JQ_COMMIT" vendor/oniguruma | awk '{print $3}')"
[[ "$onig_commit" =~ ^[0-9a-f]{40}$ ]] || die "No se pudo resolver gitlink oniguruma"
rm -rf "$RUNTIME_ROOT/jq-build-check/source/vendor/oniguruma"
mkdir -p "$RUNTIME_ROOT/jq-build-check/source/vendor/oniguruma"
onigtmp="$(mktemp -d)"
git -C "$onigtmp" init -q
git -C "$onigtmp" remote add origin https://github.com/kkos/oniguruma.git
git -C "$onigtmp" fetch -q --depth 1 origin "$onig_commit"
git -C "$onigtmp" archive --format=tar "$onig_commit" | tar -xf - -C "$RUNTIME_ROOT/jq-build-check/source/vendor/oniguruma"
printf '%s\n' "$onig_commit" > "$PROV/jq-oniguruma.commit"
( cd "$RUNTIME_ROOT/jq-build-check/source/vendor/oniguruma" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum ) > "$PROV/jq-oniguruma.files.sha256"
rm -rf "$tmp" "$onigtmp"

# fd: vendorizar Cargo.lock para ejecución posterior --offline.
need cargo; need rustc
rustc_v="$(rustc --version | awk '{print $2}')"
python3 - "$rustc_v" <<'PY'
import sys
parts=tuple(int(x) for x in sys.argv[1].split('.')[:3])
if parts < (1,90,0):
    raise SystemExit(f"fd 10.5.0 requiere rustc >= 1.90.0, actual {sys.argv[1]}")
PY
mkdir -p "$RUNTIME_ROOT/fd-build-completions/source/.cargo"
( cd "$RUNTIME_ROOT/fd-build-completions/source" && cargo vendor --locked ../vendor > .cargo/config.toml )
printf 'rustc=%s\n' "$(rustc --version)" > "$PROV/fd-toolchain.txt"
printf 'cargo=%s\n' "$(cargo --version)" >> "$PROV/fd-toolchain.txt"
( cd "$RUNTIME_ROOT/fd-build-completions/vendor" && find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum ) > "$PROV/fd-vendor.files.sha256"

# inih: congelar Meson y Ninja en runtime local.
# No se depende de una instalación global del host.
need cc
python3 -m venv "$RUNTIME_ROOT/inih-meson-tests/deps/venv"
INIHV="$RUNTIME_ROOT/inih-meson-tests/deps/venv/bin"
mkdir -p "$RUNTIME_ROOT/inih-meson-tests/deps/wheelhouse"

"$INIHV/pip" download   --disable-pip-version-check   --only-binary=:all:   --dest "$RUNTIME_ROOT/inih-meson-tests/deps/wheelhouse"   "meson==1.12.1"   "ninja==1.13.2"

"$INIHV/pip" install   --disable-pip-version-check   --no-index   --find-links "$RUNTIME_ROOT/inih-meson-tests/deps/wheelhouse"   "meson==1.12.1"   "ninja==1.13.2"

"$INIHV/pip" freeze --all | LC_ALL=C sort > "$PROV/inih-pip-freeze.txt"
(
  cd "$RUNTIME_ROOT/inih-meson-tests/deps/wheelhouse"
  find . -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum
) > "$PROV/inih-wheelhouse.sha256"

# El adapter F8.3 permanece intacto en el repositorio.
# Solo la copia runtime enlaza las herramientas congeladas.
python3 - "$RUNTIME_ROOT/inih-meson-tests/adapter/run.sh" <<'PY_INIH_ADAPTER'
from pathlib import Path
import sys

path = Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
needle = "set -euo pipefail\n"
binding = (
    'set -euo pipefail\n'
    'RUNTIME_WORKDIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"\n'
    'export PATH="$RUNTIME_WORKDIR/deps/venv/bin:$PATH"\n'
)
if binding not in text:
    if needle not in text:
        raise SystemExit("No se encontró set -euo pipefail en adapter inih runtime")
    text = text.replace(needle, binding, 1)
path.write_text(text, encoding="utf-8")
PY_INIH_ADAPTER

{
  printf 'meson=%s\n' "$("$INIHV/meson" --version)"
  printf 'ninja=%s\n' "$("$INIHV/ninja" --version)"
  printf 'cc=%s\n' "$(cc --version | head -n1)"
  printf 'source=python_venv_frozen_by_f8_4\n'
} > "$PROV/inih-toolchain.txt"

sha256sum "$RUNTIME_ROOT/inih-meson-tests/adapter/run.sh" > "$PROV/inih-runtime-adapter.sha256"

# Snapshot de herramientas comunes.
{
  printf 'python3=%s\n' "$(python3 --version 2>&1)"
  printf 'bash=%s\n' "$(bash --version | head -n1)"
  printf 'make=%s\n' "$(make --version | head -n1)"
  printf 'autoreconf=%s\n' "$(autoreconf --version | head -n1)"
} > "$PROV/common-toolchain.txt"

printf 'Runtimes preparados en %s\n' "$RUNTIME_ROOT"
