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

contains() {
  grep -Fq "$2" <<<"$1"
}

[[ -x "$FA" ]] || {
  echo "No existe el binario ejecutable: $FA" >&2
  exit 2
}

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/work"
cat > "$TMP/work/workflow.sh" <<'SCRIPT'
#!/usr/bin/env bash
set -eu
printf 'LC_ALL=%s\n' "${LC_ALL:-}"
printf 'TZ=%s\n' "${TZ:-}"
printf 'salida-controlada\n'
printf 'stderr-controlado\n' >&2
SCRIPT
chmod +x "$TMP/work/workflow.sh"

cat > "$TMP/success.toml" <<'TOML'
[experiment]
name = "explore-success"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "POSIX"]
TOML

SUCCESS_BUNDLE="$TMP/bundle-success"
set +e
success_out="$(
  "$FA" explore "$TMP/success.toml" \
    --output "$SUCCESS_BUNDLE" \
    --workdir "$TMP/work" \
    -- ./workflow.sh 2>&1
)"
success_rc=$?
set -e

check "explore completa una campaña disponible" $([[ "$success_rc" -eq 0 ]]; echo $?)
check "explore planifica dos variantes" $(contains "$success_out" "Variantes planificadas: 2"; echo $?)
check "explore ejecuta correctamente dos variantes" $(contains "$success_out" "Ejecutadas correctamente: 2"; echo $?)
check "bundle contiene experiment.json" $([[ -f "$SUCCESS_BUNDLE/experiment.json" ]]; echo $?)
check "bundle contiene manifest de variant-0001" $([[ -f "$SUCCESS_BUNDLE/variants/variant-0001/manifest.json" ]]; echo $?)
check "bundle captura stdout" $(grep -Fq "salida-controlada" "$SUCCESS_BUNDLE/variants/variant-0001/stdout.txt"; echo $?)
check "bundle captura stderr" $(grep -Fq "stderr-controlado" "$SUCCESS_BUNDLE/variants/variant-0001/stderr.txt"; echo $?)
check "bundle contiene hashes sha256" $([[ -s "$SUCCESS_BUNDLE/hashes.sha256" ]]; echo $?)
check "workspace excluye .git" $([[ ! -e "$SUCCESS_BUNDLE/variants/variant-0001/workspace/.git" ]]; echo $?)

cat > "$TMP/work/should-not-run.sh" <<'SCRIPT'
#!/usr/bin/env bash
touch should-not-exist
SCRIPT
chmod +x "$TMP/work/should-not-run.sh"

cat > "$TMP/unavailable.toml" <<'TOML'
[experiment]
name = "explore-unavailable"
schema_version = 1

[[factor]]
name = "awk_implementation"
values = ["envmorph-awk-that-does-not-exist-a", "envmorph-awk-that-does-not-exist-b"]
TOML

UNAVAILABLE_BUNDLE="$TMP/bundle-unavailable"
set +e
unavailable_out="$(
  "$FA" explore "$TMP/unavailable.toml" \
    --output "$UNAVAILABLE_BUNDLE" \
    --workdir "$TMP/work" \
    -- ./should-not-run.sh 2>&1
)"
unavailable_rc=$?
set -e

check "campaña con capacidades ausentes completa sin error conductual" $([[ "$unavailable_rc" -eq 0 ]]; echo $?)
check "explore distingue dos variantes no disponibles" $(contains "$unavailable_out" "No disponibles: 2"; echo $?)
check "variantes no disponibles no ejecutan el workflow" $([[ ! -e "$UNAVAILABLE_BUNDLE/variants/variant-0001/workspace/should-not-exist" ]]; echo $?)
check "manifest registra unavailable" $(grep -Fq '"status":"unavailable"' "$UNAVAILABLE_BUNDLE/variants/variant-0001/manifest.json"; echo $?)

cat > "$TMP/work/fail.sh" <<'SCRIPT'
#!/usr/bin/env bash
exit 9
SCRIPT
chmod +x "$TMP/work/fail.sh"

FAILED_BUNDLE="$TMP/bundle-failed"
set +e
failed_out="$(
  "$FA" explore "$TMP/success.toml" \
    --output "$FAILED_BUNDLE" \
    --workdir "$TMP/work" \
    -- ./fail.sh 2>&1
)"
failed_rc=$?
set -e

check "campaña con ejecución fallida devuelve estado no cero" $([[ "$failed_rc" -eq 1 ]]; echo $?)
check "explore registra dos ejecuciones fallidas" $(contains "$failed_out" "Fallidas: 2"; echo $?)
check "manifest preserva exit code" $(grep -Fq '"exit_code":9' "$FAILED_BUNDLE/variants/variant-0001/manifest.json"; echo $?)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
