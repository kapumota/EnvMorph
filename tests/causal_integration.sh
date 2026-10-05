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

mkdir -p "$TMP/work"
cat > "$TMP/work/workflow.sh" <<'WORKFLOW'
#!/usr/bin/env bash
set -eu

if [[ "${LC_ALL:-}" == "POSIX" && "${TZ:-}" == "America/Lima" ]]; then
  printf 'changed\n' > result.txt
else
  printf 'same\n' > result.txt
fi
WORKFLOW
chmod +x "$TMP/work/workflow.sh"

cat > "$TMP/causal.toml" <<'TOML'
[causal]
name = "interaction-f5"
schema_version = 1
confirmations = 2

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
result	byte	workspace/result.txt	workspace/result.txt	-
TSV

run_capture out rc "$FA" minimize-environment "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --output "$TMP/mce-human" \
  -- ./workflow.sh
check "F5 encuentra interacción mínima de dos factores" $(
  [[ "$rc" -eq 0 ]] &&
  contains "$out" "Estado MCE: minimal" &&
  contains "$out" "Cardinalidad mínima: 2" &&
  contains "$out" "MCE 1: locale, timezone"
  echo $?
)

check "F5 usa dos confirmaciones controladas" $(
  contains "$out" "Confirmaciones: 2"
  echo $?
)

check "F5 conserva la firma observable F4" $(
  contains "$out" "Firma objetivo F4: persistent" &&
  contains "$out" "Primera divergencia objetivo: result"
  echo $?
)

check "F5 persiste resultado JSON machine-readable" $(
  [[ -s "$TMP/mce-human/mce-result.json" ]] &&
  grep -Fq '"minimal_cardinality":2' "$TMP/mce-human/mce-result.json" &&
  grep -Fq '"causality_claim":"operational_sufficiency_not_physical_proof"' "$TMP/mce-human/mce-result.json"
  echo $?
)

check "F5 conserva bundles F2 como evidencia de intervenciones" $(
  find "$TMP/mce-human" -path '*/variants/variant-0001/manifest.json' -type f -print -quit | grep -q .
  echo $?
)

run_capture json1 rc1 "$FA" minimize-environment "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --output "$TMP/mce-json-1" \
  --format json \
  -- ./workflow.sh
run_capture json2 rc2 "$FA" minimize-environment "$TMP/causal.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --output "$TMP/mce-json-2" \
  --format json \
  -- ./workflow.sh
check "salida causal JSON es determinista" $(
  [[ "$rc1" -eq 0 && "$rc2" -eq 0 ]] &&
  cmp -s <(printf '%s' "$json1") <(printf '%s' "$json2")
  echo $?
)

cat > "$TMP/work/no-effect.sh" <<'WORKFLOW'
#!/usr/bin/env bash
set -eu
printf 'same\n' > result.txt
WORKFLOW
chmod +x "$TMP/work/no-effect.sh"

cat > "$TMP/no-effect.toml" <<'TOML'
[causal]
name = "no-effect"
schema_version = 1
confirmations = 2

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"
TOML

run_capture no_effect rc_no_effect "$FA" minimize-environment "$TMP/no-effect.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --output "$TMP/mce-no-effect" \
  --format json \
  -- ./no-effect.sh
check "intervención sin efecto observable no produce MCE" $(
  [[ "$rc_no_effect" -eq 1 ]] &&
  contains "$no_effect" '"status":"no_observable_effect"' &&
  contains "$no_effect" '"minimal_cardinality":null'
  echo $?
)

cat > "$TMP/invalid.toml" <<'TOML'
[causal]
name = "invalid"
schema_version = 1

[[factor]]
name = "locale"
baseline = "C"
treatment = "C"
TOML

run_capture invalid rc_invalid "$FA" minimize-environment "$TMP/invalid.toml" \
  --trace "$TMP/trace.tsv" \
  --workdir "$TMP/work" \
  --output "$TMP/mce-invalid" \
  -- ./workflow.sh
check "F5 rechaza baseline y treatment idénticos" $(
  [[ "$rc_invalid" -ne 0 ]]
  echo $?
)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
