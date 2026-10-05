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

mkdir -p "$TMP/left" "$TMP/right"
printf 'same\n' > "$TMP/left/a.txt"
printf 'same\n' > "$TMP/right/a.txt"
printf 'left\n' > "$TMP/left/b.txt"
printf 'right\n' > "$TMP/right/b.txt"
printf 'left-2\n' > "$TMP/left/c.txt"
printf 'right-2\n' > "$TMP/right/c.txt"
printf 'same-end\n' > "$TMP/left/d.txt"
printf 'same-end\n' > "$TMP/right/d.txt"

cat > "$TMP/absorbed.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
stage-a	byte	a.txt	a.txt	-
stage-b	byte	b.txt	b.txt	-
stage-c	byte	c.txt	c.txt	-
stage-d	byte	d.txt	d.txt	-
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/absorbed.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right"
check "F4 detecta divergencia, propagación y absorción" $(
  [[ "$rc" -eq 0 ]] &&
  contains "$out" "Estado del trazado: absorbed" &&
  contains "$out" "Primera divergencia observada: stage-b" &&
  contains "$out" "Fronteras de absorción: stage-d" &&
  contains "$out" "Propagación: divergence_start" &&
  contains "$out" "Propagación: propagated" &&
  contains "$out" "Propagación: absorbed"
  echo $?
)

cat > "$TMP/stable.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
stage-a	byte	a.txt	a.txt	-
stage-d	byte	d.txt	d.txt	-
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/stable.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right"
check "F4 distingue un trazado estable" $(
  [[ "$rc" -eq 0 ]] && contains "$out" "Estado del trazado: stable"
  echo $?
)

cat > "$TMP/persistent.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
stage-b	byte	b.txt	b.txt	-
stage-c	byte	c.txt	c.txt	-
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/persistent.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json
check "F4 distingue una diferencia persistente" $(
  [[ "$rc" -eq 0 ]] && contains "$out" '"trace_status":"persistent"'
  echo $?
)

cat > "$TMP/unresolved.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
stage-b	byte	b.txt	b.txt	-
gap	byte	missing.txt	missing.txt	-
stage-c	byte	c.txt	c.txt	-
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/unresolved.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json
check "missing rompe continuidad y produce unresolved" $(
  [[ "$rc" -eq 2 ]] &&
  contains "$out" '"trace_status":"unresolved"' &&
  contains "$out" '"propagation":"unresolved"'
  echo $?
)

printf '{"a":}\n' > "$TMP/left/bad.json"
printf '{"a":1}\n' > "$TMP/right/good.json"
cat > "$TMP/parse-error.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
json-step	json	bad.json	good.json	-
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/parse-error.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json
check "parse error F3 se conserva como unresolved F4" $(
  [[ "$rc" -eq 2 ]] &&
  contains "$out" '"equivalence":"error"' &&
  contains "$out" '"propagation":"unresolved"'
  echo $?
)

printf 'a\r\n' > "$TMP/left/text.txt"
printf 'a\n' > "$TMP/right/text.txt"
cat > "$TMP/text-options.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
text-step	text	text.txt	text.txt	normalize_line_endings
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/text-options.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json
check "F4 reutiliza normalizaciones explícitas de TextOracle" $(
  [[ "$rc" -eq 0 ]] &&
  contains "$out" '"equivalence":"equivalent"' &&
  contains "$out" '"trace_status":"stable"'
  echo $?
)

cat > "$TMP/bad-options.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
bad	json	a.txt	a.txt	normalize_line_endings
TSV

run_capture out rc "$FA" analyze-propagation \
  --trace "$TMP/bad-options.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right"
check "opciones de texto no contaminan otros oráculos" $(
  [[ "$rc" -ne 0 ]] && contains "$out" "solo son válidas para el oráculo text"
  echo $?
)

MACHINE="$TMP/trace.json"
run_capture json_out rc "$FA" analyze-propagation \
  --trace "$TMP/absorbed.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json --output "$MACHINE"
check "salida F4 machine-readable se guarda" $(
  [[ "$rc" -eq 0 && -s "$MACHINE" ]] &&
  python3 - "$MACHINE" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

required = {
    "trace_status",
    "first_observed_divergence",
    "absorption_boundaries",
    "unresolved_count",
    "observations",
}

ok = (
    required <= data.keys()
    and data["trace_status"] == "absorbed"
    and data["first_observed_divergence"] == "stage-b"
    and data["absorption_boundaries"] == ["stage-d"]
    and len(data["observations"]) == 4
)
raise SystemExit(0 if ok else 1)
PY
  echo $?
)

run_capture json_out_2 rc2 "$FA" analyze-propagation \
  --trace "$TMP/absorbed.tsv" \
  --left-root "$TMP/left" --right-root "$TMP/right" \
  --format json
check "salida F4 machine-readable es determinista" $(
  [[ "$rc2" -eq 0 && "$json_out" == "$json_out_2" ]]
  echo $?
)

mkdir -p "$TMP/work"
cat > "$TMP/work/pipeline.sh" <<'PIPE'
#!/usr/bin/env bash
set -eu
printf '%s\n' "${LC_ALL:-}" > stage1.txt
printf 'prefijo:%s\n' "${LC_ALL:-}" > stage2.txt
printf 'resultado-estable\n' > stage3.txt
PIPE
chmod +x "$TMP/work/pipeline.sh"

cat > "$TMP/bundle.toml" <<'TOML'
[experiment]
name = "f4-bundle"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]
TOML

BUNDLE="$TMP/bundle"
run_capture explore_out explore_rc "$FA" explore "$TMP/bundle.toml" \
  --output "$BUNDLE" --workdir "$TMP/work" -- ./pipeline.sh
check "F2 produce bundle real para F4" $(
  [[ "$explore_rc" -eq 0 &&
     -f "$BUNDLE/variants/variant-0001/workspace/stage1.txt" &&
     -f "$BUNDLE/variants/variant-0002/workspace/stage3.txt" ]]
  echo $?
)

cat > "$TMP/bundle-trace.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
stage1	byte	workspace/stage1.txt	workspace/stage1.txt	-
stage2	byte	workspace/stage2.txt	workspace/stage2.txt	-
stage3	byte	workspace/stage3.txt	workspace/stage3.txt	-
TSV

run_capture bundle_trace bundle_rc "$FA" analyze-propagation \
  --trace "$TMP/bundle-trace.tsv" \
  --left-root "$BUNDLE/variants/variant-0001" \
  --right-root "$BUNDLE/variants/variant-0002" \
  --format json
check "F4 analiza propagación sobre artefactos reales de bundles F2" $(
  [[ "$bundle_rc" -eq 0 ]] &&
  contains "$bundle_trace" '"trace_status":"absorbed"' &&
  contains "$bundle_trace" '"first_observed_divergence":"stage1"' &&
  contains "$bundle_trace" '"absorption_boundaries":["stage3"]'
  echo $?
)

check "F4 no afirma causalidad en la salida" $(
  ! grep -Eqi 'causó|causado por|causalmente' <<<"$bundle_trace"
  echo $?
)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
