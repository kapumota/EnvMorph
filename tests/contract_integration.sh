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

contains() { grep -Fq "$2" <<<"$1"; }

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
cat > "$TMP/work/workflow.sh" <<'SCRIPT'
#!/bin/sh
set -eu
mkdir -p output
if [ "${LC_ALL:-}" = "POSIX" ]; then
  printf 'changed\n' > output/result.txt
else
  printf 'same\n' > output/result.txt
fi
SCRIPT
chmod +x "$TMP/work/workflow.sh"

cat > "$TMP/causal.toml" <<'TOML'
[causal]
name = "contract-integration"
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
TOML

cat > "$TMP/trace.tsv" <<'TSV'
label	oracle	left_artifact	right_artifact	options
result	byte	workspace/output/result.txt	workspace/output/result.txt	-
TSV

run_capture derive_out derive_rc "$FA" derive-contract "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --evidence "$TMP/evidence" \
  --output "$TMP/contract.toml" \
  -- ./workflow.sh

check "derive-contract consume F5 y produce contrato" $([[ "$derive_rc" -eq 0 && -s "$TMP/contract.toml" ]]; echo $?)
check "contrato conserva MCE locale" $(grep -Fq 'factors = ["locale"]' "$TMP/contract.toml"; echo $?)
check "derivación conserva mce-result.json" $([[ -s "$TMP/evidence/mce-result.json" ]]; echo $?)

cat > "$TMP/baseline.toml" <<'TOML'
[environment]
name = "baseline"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
[[factor]]
name = "timezone"
value = "UTC"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/baseline.toml"
check "baseline exacto satisface contrato" $([[ "$rc" -eq 0 ]] && contains "$out" 'Estado: satisfied'; echo $?)

cat > "$TMP/mce.toml" <<'TOML'
[environment]
name = "mce"
schema_version = 1
[[factor]]
name = "locale"
value = "POSIX"
[[factor]]
name = "timezone"
value = "UTC"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/mce.toml"
check "MCE exacto viola contrato" $([[ "$rc" -eq 1 ]] && contains "$out" 'Estado: violated'; echo $?)

cat > "$TMP/full.toml" <<'TOML'
[environment]
name = "full"
schema_version = 1
[[factor]]
name = "locale"
value = "POSIX"
[[factor]]
name = "timezone"
value = "America/Lima"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/full.toml"
check "intervención completa confirmada viola contrato" $([[ "$rc" -eq 1 ]] && contains "$out" 'Estado: violated'; echo $?)

cat > "$TMP/partial.toml" <<'TOML'
[environment]
name = "partial"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
[[factor]]
name = "timezone"
value = "America/Lima"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/partial.toml"
check "combinación parcial no establecida queda out_of_scope" $([[ "$rc" -eq 2 ]] && contains "$out" 'Estado: out_of_scope'; echo $?)

cat > "$TMP/unseen.toml" <<'TOML'
[environment]
name = "unseen"
schema_version = 1
[[factor]]
name = "locale"
value = "en_US.UTF-8"
[[factor]]
name = "timezone"
value = "UTC"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/unseen.toml"
check "valor no observado queda out_of_scope" $([[ "$rc" -eq 2 ]] && contains "$out" 'Estado: out_of_scope'; echo $?)

cat > "$TMP/missing.toml" <<'TOML'
[environment]
name = "missing"
schema_version = 1
[[factor]]
name = "locale"
value = "C"
TOML
run_capture out rc "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/missing.toml"
check "factor requerido ausente produce invalid" $([[ "$rc" -eq 3 ]] && contains "$out" 'Estado: invalid'; echo $?)

run_capture json1 rc1 "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/mce.toml" --format json --output "$TMP/eval.json"
run_capture json2 rc2 "$FA" check-contract "$TMP/contract.toml" --environment "$TMP/mce.toml" --format json
check "salida JSON machine-readable se persiste" $([[ "$rc1" -eq 1 && -s "$TMP/eval.json" ]] && grep -Fq '"status":"violated"' "$TMP/eval.json"; echo $?)
check "evaluación JSON es determinista" $([[ "$rc1" -eq 1 && "$rc2" -eq 1 && "$json1" == "$json2" ]]; echo $?)

run_capture toml_out toml_rc "$FA" derive-contract "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --evidence "$TMP/evidence-2" \
  --output "$TMP/contract-2.toml" \
  --format toml \
  -- ./workflow.sh
check "derivación contractual es determinista" $([[ "$toml_rc" -eq 0 ]] && cmp -s "$TMP/contract.toml" "$TMP/contract-2.toml"; echo $?)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
