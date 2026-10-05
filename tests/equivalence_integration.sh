#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${ENVMORPH_BIN:-$ROOT/target/release/envmorph}"
PASS=0
FAIL=0

check() {
  local name="$1"
  local rc="$2"

  if [[ "$rc" -eq 0 ]]; then
    echo "  PASS $name"
    PASS=$((PASS + 1))
  else
    echo "  FAIL $name"
    FAIL=$((FAIL + 1))
  fi
}

run_capture() {
  local __out_var="$1"
  local __rc_var="$2"
  shift 2

  local captured
  local command_rc

  if captured="$("$@" 2>&1)"; then
    command_rc=0
  else
    command_rc=$?
  fi

  printf -v "$__out_var" '%s' "$captured"
  printf -v "$__rc_var" '%s' "$command_rc"
}

contains() {
  grep -Fq "$2" <<<"$1"
}

[[ -x "$FA" ]] || {
  echo "No existe el binario ejecutable: $FA" >&2
  exit 2
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

printf 'abc\n' > "$TMP/byte-a"
printf 'abc\n' > "$TMP/byte-b"
printf 'abd\n' > "$TMP/byte-c"

run_capture out rc "$FA" compare-artifacts \
  --oracle byte --left "$TMP/byte-a" --right "$TMP/byte-b"
check "ByteOracle detecta identidad" $([[ "$rc" -eq 0 ]] && contains "$out" "Resultado: identical"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle byte --left "$TMP/byte-a" --right "$TMP/byte-c"
check "ByteOracle detecta diferencia" $([[ "$rc" -eq 1 ]] && contains "$out" "Resultado: different"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle byte --left "$TMP/no-existe" --right "$TMP/byte-b"
check "artefacto ausente produce missing" $([[ "$rc" -eq 2 ]] && contains "$out" "Resultado: missing"; echo $?)

printf 'a\r\nb\r\n' > "$TMP/text-crlf"
printf 'a\nb\n' > "$TMP/text-lf"

run_capture out rc "$FA" compare-artifacts \
  --oracle text --left "$TMP/text-crlf" --right "$TMP/text-lf"
check "TextOracle es conservador por defecto" $([[ "$rc" -eq 1 ]] && contains "$out" "Resultado: different"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle text --left "$TMP/text-crlf" --right "$TMP/text-lf" \
  --normalize-line-endings
check "TextOracle normaliza line endings solo por configuración" $([[ "$rc" -eq 0 ]] && contains "$out" "Resultado: equivalent"; echo $?)

printf 'a  \r\nb\t\r\n' > "$TMP/text-spaces"
printf 'a\nb' > "$TMP/text-clean"

run_capture out rc "$FA" compare-artifacts \
  --oracle text --left "$TMP/text-spaces" --right "$TMP/text-clean" \
  --normalize-line-endings --trim-trailing-whitespace --ignore-final-newline
check "TextOracle combina normalizaciones explícitas" $([[ "$rc" -eq 0 ]] && contains "$out" "Resultado: equivalent"; echo $?)

printf '{"a":1,"b":2}\n' > "$TMP/json-a"
printf '{ "b": 2, "a": 1 }\n' > "$TMP/json-b"
printf '[1,2]\n' > "$TMP/json-array-a"
printf '[2,1]\n' > "$TMP/json-array-b"
printf '{"a":}\n' > "$TMP/json-bad"

run_capture out rc "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-a" --right "$TMP/json-b"
check "JsonOracle ignora orden de claves" $([[ "$rc" -eq 0 ]] && contains "$out" "Resultado: equivalent"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-array-a" --right "$TMP/json-array-b"
check "JsonOracle conserva orden de arrays" $([[ "$rc" -eq 1 ]] && contains "$out" "Resultado: different"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-bad" --right "$TMP/json-a"
check "JsonOracle expone parse error" $([[ "$rc" -eq 2 ]] && contains "$out" "Resultado: error"; echo $?)

printf 'a,"b"\r\n' > "$TMP/csv-a"
printf 'a,b\n' > "$TMP/csv-b"
printf 'x,1\ny,2\n' > "$TMP/csv-order-a"
printf 'y,2\nx,1\n' > "$TMP/csv-order-b"
printf '"a\n' > "$TMP/csv-bad"

run_capture out rc "$FA" compare-artifacts \
  --oracle csv --left "$TMP/csv-a" --right "$TMP/csv-b"
check "CsvOracle compara estructura tabular" $([[ "$rc" -eq 0 ]] && contains "$out" "Resultado: equivalent"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle csv --left "$TMP/csv-order-a" --right "$TMP/csv-order-b"
check "CsvOracle mantiene orden de filas significativo" $([[ "$rc" -eq 1 ]] && contains "$out" "Resultado: different"; echo $?)

run_capture out rc "$FA" compare-artifacts \
  --oracle csv --left "$TMP/csv-bad" --right "$TMP/csv-b"
check "CsvOracle expone parse error" $([[ "$rc" -eq 2 ]] && contains "$out" "Resultado: error"; echo $?)

MACHINE="$TMP/result.json"
run_capture json_out rc "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-a" --right "$TMP/json-b" \
  --format json --output "$MACHINE"
check "salida machine-readable se guarda" $(
  [[ "$rc" -eq 0 && -s "$MACHINE" ]] &&
  python3 - "$MACHINE" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

required = {
    "oracle",
    "left_artifact",
    "right_artifact",
    "result",
    "reason",
    "metadata",
}
raise SystemExit(0 if required <= data.keys() and data["result"] == "equivalent" else 1)
PY
  echo $?
)

run_capture json_out_2 rc2 "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-a" --right "$TMP/json-b" \
  --format json
check "salida machine-readable es determinista" $(
  [[ "$rc2" -eq 0 && "$json_out" == "$json_out_2" ]]
  echo $?
)

mkdir -p "$TMP/work"
cat > "$TMP/work/emit.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
printf 'salida-bundle-estable\n'
SCRIPT
chmod +x "$TMP/work/emit.sh"

cat > "$TMP/bundle.toml" <<'TOML'
[experiment]
name = "f3-bundle"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]
TOML

BUNDLE="$TMP/bundle"
run_capture explore_out explore_rc "$FA" explore "$TMP/bundle.toml" \
  --output "$BUNDLE" --workdir "$TMP/work" -- ./emit.sh
check "F2 produce bundle real para F3" $(
  [[ "$explore_rc" -eq 0 &&
     -f "$BUNDLE/variants/variant-0001/stdout.txt" &&
     -f "$BUNDLE/variants/variant-0002/stdout.txt" ]]
  echo $?
)

run_capture bundle_compare bundle_rc "$FA" compare-artifacts \
  --oracle byte \
  --left "$BUNDLE/variants/variant-0001/stdout.txt" \
  --right "$BUNDLE/variants/variant-0002/stdout.txt" \
  --format json
check "F3 compara artefactos concretos de dos variantes F2" $(
  [[ "$bundle_rc" -eq 0 ]] &&
  contains "$bundle_compare" '"result":"identical"'
  echo $?
)

run_capture bad_option bad_rc "$FA" compare-artifacts \
  --oracle json --left "$TMP/json-a" --right "$TMP/json-b" \
  --normalize-line-endings
check "normalizaciones de texto no contaminan otros oráculos" $(
  [[ "$bad_rc" -ne 0 ]] &&
  contains "$bad_option" "solo son válidas con --oracle text"
  echo $?
)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
