#!/bin/bash
# Pruebas de integración de EnvMorph. No compilan el binario, define ENVMORPH_BIN
# o ejecútalas después de `cargo build --release`.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${ENVMORPH_BIN:-$ROOT/target/release/envmorph}"
[[ -x "$FA" ]] || { echo "no se encontró el binario envmorph en $FA" >&2; exit 2; }

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
PASS=0; FAIL=0

ok()   { PASS=$((PASS + 1)); echo "  PASS $1"; }
bad()  { FAIL=$((FAIL + 1)); echo "  FAIL $1"; [[ -n "${2:-}" ]] && echo "$2" | sed 's/^/       /'; }
check() { # check NOMBRE CÓDIGO-DE-CONDICIÓN [SALIDA]
  if [[ "$2" -eq 0 ]]; then ok "$1"; else bad "$1" "${3:-}"; fi
}
contains() { grep -qF -- "$2" <<<"$1"; }

# Fixture: pipeline de prueba de 3 etapas, upper -> sortlines -> count
setup() {
  W="$TMP/$1"; mkdir -p "$W/bin1" "$W/bin2"; cd "$W"
  printf 'pear\napple\nfig\nbanana\n' > input.txt
  printf '#!/bin/sh\nexec sort "$@"\n'    > bin1/mysort
  printf '#!/bin/sh\nexec sort -r "$@"\n' > bin2/mysort
  chmod +x bin1/mysort bin2/mysort
  export ENVMORPH_HOME="$W/.fa"
  unset ENVMORPH_RUN
}

pipeline() { # pipeline NOMBRE_SALIDA [DIR_BIN] [ENTRADA]
  local out="$W/$1" bindir="${2:-bin1}" input="${3:-input.txt}"
  mkdir -p "$out"
  local rid; rid="$("$FA" begin --map "$out=@out" --map "$W=@w" --env LC_ALL)"
  export ENVMORPH_RUN="$rid"
  export PATH="$W/$bindir:$ORIG_PATH"
  "$FA" stage --quiet --name upper --input "$W/$input" --stdout "$out/1.txt" -- sh -c "tr a-z A-Z < '$W/$input'"
  "$FA" stage --quiet --name sortlines --input "$out/1.txt" --stdout "$out/2.txt" -- mysort "$out/1.txt"
  "$FA" stage --quiet --name count --input "$out/2.txt" --stdout "$out/3.txt" -- sh -c 'wc -l < "$1"' sh "$out/2.txt"
  "$FA" end >/dev/null
  unset ENVMORPH_RUN
  echo "$rid"
}
ORIG_PATH="$PATH"

echo "== registro"
setup t1
A="$(pipeline outA)"
B="$(pipeline outB)"
out="$("$FA" inspect "$A")"
check "inspect lista etapas" $(contains "$out" "sortlines"; echo $?) "$out"
n="$("$FA" export "$A" | grep -c '"exit_code"')"
check "registra exactamente 3 etapas ($n)" $([[ "$n" -eq 3 ]]; echo $?)

echo "== ejecuciones idénticas"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "mismas entradas y herramientas: salida 0" $rc "$out"
check "misma ejecución: reporta identidad" $(contains "$out" "salidas idénticas en todas las etapas"; echo $?) "$out"
sha1="$("$FA" export "$A" | grep -m1 '"sha256"' )"
check "campo sha256 presente" $([[ -n "$sha1" ]]; echo $?)

echo "== entrada modificada"
setup t2
A="$(pipeline outA)"
printf 'pear\napple\nfig\nbanana\ncherry\n' > input2.txt
B="$(pipeline outB bin1 input2.txt)"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "entrada modificada: salida 1" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "localizada en la primera etapa" $(contains "$out" "etapa 001 upper"; echo $?) "$out"
check "la causa es data" $(contains "$out" "[data]"; echo $?) "$out"
check "etapa aguas abajo marcada propagated" $(contains "$out" "propagated"; echo $?) "$out"
js="$("$FA" diff "$A" "$B" --json)"
flat="$(echo "$js" | tr -d ' \n')"
check "JSON tiene first_divergence en posición 1" $(contains "$flat" '"first_divergence":{"position":1'; echo $?) "$js"

echo "== herramienta modificada, cambia la salida (isolate)"
setup t3
A="$(pipeline outA bin1)"
B="$(pipeline outB bin2)"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "el cambio de herramienta diverge" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "localizada en sortlines" $(contains "$out" "etapa 002 sortlines"; echo $?) "$out"
check "la causa es toolchain" $(contains "$out" "[toolchain]"; echo $?) "$out"
check "la etapa count es confounded" $(contains "$out" "confounded"; echo $?) "$out"
out="$("$FA" isolate "$A" "$B")"; rc=$?
check "isolate devuelve 1 cuando una etapa es sensitive" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "isolate: sortlines sensitive" $(grep -Eq "002 sortlines +sensitive" <<<"$out"; echo $?) "$out"
check "isolate repite solo etapas cuyo ejecutable difiere" $(contains "$out" "1 de 1 etapa(s) repetidas"; echo $?) "$out"
check "isolate confirma la primera causa" $(contains "$out" "PRIMERA CAUSA CONFIRMADA  etapa 002 sortlines"; echo $?) "$out"

echo "== diferencia absorbed"
setup t4
printf '#!/bin/sh\nexec sort "$@"\n# padding\n' > bin2/mysort
A="$(pipeline outA bin1)"
B="$(pipeline outB bin2)"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "wrapper diferente, misma salida: salida 0" $rc "$out"
check "reportada como absorbed" $(contains "$out" "absorbed"; echo $?) "$out"

echo "== diferencia de entorno"
setup t5
A="$(LC_ALL=C pipeline outA)"
B="$(LC_ALL=en_US.UTF-8 pipeline outB)"
out="$("$FA" diff "$A" "$B")"
check "diferencia de entorno listada" $(contains "$out" "LC_ALL"; echo $?) "$out"

echo "== no determinismo"
setup t6
mk() { local out="$W/$1"; mkdir -p "$out"; local rid; rid="$("$FA" begin --map "$out=@out")"; export ENVMORPH_RUN="$rid"
  "$FA" stage --quiet --name stable --input "$W/input.txt" --stdout "$out/a" -- cat "$W/input.txt"
  "$FA" stage --quiet --name noisy  --input "$out/a" --stdout "$out/b" -- sh -c 'echo $$; cat "$1"' sh "$out/a"
  "$FA" end >/dev/null; unset ENVMORPH_RUN; echo "$rid"; }
A="$(mk oA)"; B="$(mk oB)"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "la etapa no determinista diverge" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "localizada en noisy" $(contains "$out" "etapa 002 noisy"; echo $?) "$out"
check "la causa es unexplained" $(contains "$out" "[unexplained]"; echo $?) "$out"

echo "== indicio estático de dependencia del reloj"
setup t6b
printf 'BEGIN { print "stamp", systime() }\n' > stamp.awk
mk2() { local out="$W/$1"; mkdir -p "$out"; local rid; rid="$("$FA" begin --map "$out=@out" --map "$W=@w")"; export ENVMORPH_RUN="$rid"
  "$FA" stage --quiet --name stamp --stdout "$out/s.txt" -- awk -f "$W/stamp.awk" /dev/null
  "$FA" end >/dev/null; unset ENVMORPH_RUN; echo "$rid"; }
A="$(mk2 cA)"; sleep 1.1; B="$(mk2 cB)"
out="$("$FA" diff "$A" "$B")"; rc=$?
check "la etapa dependiente del reloj diverge" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "la causa es unexplained" $(contains "$out" "[unexplained]"; echo $?) "$out"
check "el indicio estático identifica systime" $(contains "$out" "usa tiempo de reloj de pared (awk systime)"; echo $?) "$out"
check "el indicio estático indica archivo y línea" $(grep -Eq "@w/stamp.awk:1 usa" <<<"$out"; echo $?) "$out"

echo "== etapa fallida y salida faltante"
setup t7
export ENVMORPH_RUN="$("$FA" begin --map "$W=@w")"
"$FA" stage --quiet --name boom --input "$W/input.txt" --output "$W/never.txt" -- sh -c 'exit 3'
rc=$?
check "stage devuelve el código de salida envuelto" $([[ $rc -eq 3 ]]; echo $?)
"$FA" end >/dev/null; id="$(cat "$ENVMORPH_HOME/CURRENT")"; unset ENVMORPH_RUN
out="$("$FA" export "$id")"
check "código de salida registrado" $(contains "$out" '"exit_code": 3'; echo $?) "$out"
check "salida faltante registrada como missing" $(contains "$out" '"kind": "missing"'; echo $?) "$out"
"$FA" stage --name x --input a -- true >/dev/null 2>&1; rc=$?
check "stage sin ejecución activa produce error" $([[ $rc -eq 2 ]]; echo $?)

echo "== verificación"
setup t8
A="$(pipeline outA)"
out="$("$FA" verify "$A")"; rc=$?
check "verify pasa sobre ejecución intacta" $rc "$out"
echo tampered >> "$W/outA/3.txt"
out="$("$FA" verify "$A")"; rc=$?
check "verify detecta salida modificada" $([[ $rc -eq 1 ]]; echo $?) "$out"
check "verify identifica el archivo" $(contains "$out" "MODIFICADO"; echo $?) "$out"

echo "== grafo"
setup t9
A="$(pipeline outA bin1)"; B="$(pipeline outB bin2)"
dot="$("$FA" graph "$A")"
check "el grafo es un digraph" $(contains "$dot" "digraph"; echo $?)
dot="$("$FA" graph "$A" --diff "$B")"
check "el grafo diff resalta la primera divergencia" $(contains "$dot" "penwidth=3"; echo $?)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ $FAIL -eq 0 ]]
