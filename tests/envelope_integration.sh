#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${ENVMORPH_BIN:-$ROOT/target/release/envmorph}"
PASS=0
FAIL=0

check() {
  local label="$1"
  local rc="$2"
  if [[ "$rc" -eq 0 ]]; then
    echo "[PASS] $label"
    PASS=$((PASS + 1))
  else
    echo "[FAIL] $label"
    FAIL=$((FAIL + 1))
  fi
}

run_capture() {
  local __out_var="$1"
  local __rc_var="$2"
  shift 2
  local captured
  local command_rc
  set +e
  captured="$("$@" 2>&1)"
  command_rc=$?
  set -e
  printf -v "$__out_var" '%s' "$captured"
  printf -v "$__rc_var" '%s' "$command_rc"
}

[[ -x "$FA" ]] || { echo "No existe el binario: $FA" >&2; exit 2; }
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/work"
cat > "$TMP/work/workflow.sh" <<'EOF_WORKFLOW'
#!/bin/sh
set -eu
mkdir -p output
if [ "${LC_ALL:-}" = "POSIX" ]; then
  printf 'changed\n' > output/result.txt
else
  printf 'same\n' > output/result.txt
fi
EOF_WORKFLOW
chmod +x "$TMP/work/workflow.sh"

cat > "$TMP/causal.toml" <<'EOF_CAUSAL'
[causal]
name = "envelope-integration"
schema_version = 1
confirmations = 1

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[factor]]
name = "timezone"
baseline = "UTC"
treatment = "America/Lima"
EOF_CAUSAL

cat > "$TMP/trace.tsv" <<'EOF_TRACE'
label	oracle	left_artifact	right_artifact	options
result	byte	workspace/output/result.txt	workspace/output/result.txt	-
EOF_TRACE

run_capture derive_out derive_rc "$FA" derive-contract "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --evidence "$TMP/evidence" \
  --output "$TMP/contract.toml" \
  -- sh workflow.sh
check "F7 consume un contrato real derivado por F6" $([[ "$derive_rc" -eq 0 && -s "$TMP/contract.toml" ]]; echo $?)

contract_before="$(sha256sum "$TMP/contract.toml" | awk '{print $1}')"
run_capture out rc "$FA" build-envelope "$TMP/contract.toml" --output "$TMP/envelope.json"
check "build-envelope produce un sobre persistente" $([[ "$rc" -eq 0 && -s "$TMP/envelope.json" ]]; echo $?)
check "F7 enumera las cuatro celdas del lattice de dos factores" $(grep -Fq '"total_cells":4' "$TMP/envelope.json"; echo $?)
check "F7 conserva clasificación F6 1 satisfied 2 violated 1 out_of_scope" $(grep -Fq '"satisfied_cells":1,"violated_cells":2,"out_of_scope_cells":1' "$TMP/envelope.json"; echo $?)
check "F7 marca el sobre como partial cuando queda evidencia no resuelta" $(grep -Fq '"status":"partial"' "$TMP/envelope.json"; echo $?)
check "F7 calcula cobertura decidida 3 de 4" $(grep -Fq '"coverage":{"decided":3,"total":4}' "$TMP/envelope.json"; echo $?)
check "F7 registra tres fronteras entre celdas vecinas" $(grep -Fq '"boundary_count":3' "$TMP/envelope.json"; echo $?)
check "F7 no genera celdas invalid" $(! grep -Fq '"status":"invalid"' "$TMP/envelope.json"; echo $?)
check "F7 declara explícitamente que el sobre no es portabilidad universal" $(grep -Fq 'finite_observed_contrast_envelope_not_universal_portability' "$TMP/envelope.json"; echo $?)

run_capture json1 rc1 "$FA" build-envelope "$TMP/contract.toml" --output "$TMP/envelope-1.json" --format json
run_capture json2 rc2 "$FA" build-envelope "$TMP/contract.toml" --output "$TMP/envelope-2.json" --format json
check "F7 produce salida JSON determinista" $([[ "$rc1" -eq 0 && "$rc2" -eq 0 && "$json1" == "$json2" ]] && cmp -s "$TMP/envelope-1.json" "$TMP/envelope-2.json"; echo $?)

contract_after="$(sha256sum "$TMP/contract.toml" | awk '{print $1}')"
check "build-envelope no modifica el contrato F6" $([[ "$contract_before" == "$contract_after" ]]; echo $?)

printf 'pasaron %d, fallaron %d\n' "$PASS" "$FAIL"
(( FAIL == 0 ))
