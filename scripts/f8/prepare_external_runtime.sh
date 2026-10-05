#!/usr/bin/env bash
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
"$CSVV/pip" wheel --disable-pip-version-check --wheel-dir "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse" "$RUNTIME_ROOT/csvkit-realdata/source"
"$CSVV/pip" install --disable-pip-version-check --no-index --find-links "$RUNTIME_ROOT/csvkit-realdata/deps/wheelhouse" csvkit==2.2.0
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

# inih: solo toolchain local, no descarga requerida.
need meson; need ninja; need cc
{
  printf 'meson=%s\n' "$(meson --version)"
  printf 'ninja=%s\n' "$(ninja --version)"
  printf 'cc=%s\n' "$(cc --version | head -n1)"
} > "$PROV/inih-toolchain.txt"

# Snapshot de herramientas comunes.
{
  printf 'python3=%s\n' "$(python3 --version 2>&1)"
  printf 'bash=%s\n' "$(bash --version | head -n1)"
  printf 'make=%s\n' "$(make --version | head -n1)"
  printf 'autoreconf=%s\n' "$(autoreconf --version | head -n1)"
} > "$PROV/common-toolchain.txt"

printf 'Runtimes preparados en %s\n' "$RUNTIME_ROOT"
