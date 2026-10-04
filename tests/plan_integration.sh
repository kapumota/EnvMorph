#!/usr/bin/env bash
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${ENVMORPH_BIN:-$ROOT/target/release/envmorph}"
SPEC="$ROOT/experiments/f1/literature-pipeline-plan.toml"
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

out1="$($FA plan "$SPEC")"
out2="$($FA plan "$SPEC")"

check "plan acepta la especificación F1" $([[ -n "$out1" ]]; echo $?)
check "plan reporta tres factores" $(contains "$out1" "Factores: 3"; echo $?)
check "plan reporta ocho variantes" $(contains "$out1" "Variantes: 8"; echo $?)
check "primera variante es determinista" $(contains "$out1" "variant-0001  locale=C  timezone=UTC  awk_implementation=mawk"; echo $?)
check "última variante es determinista" $(contains "$out1" "variant-0008  locale=en_US.UTF-8  timezone=America/Lima  awk_implementation=gawk"; echo $?)
check "dos planificaciones consecutivas son idénticas" $(cmp -s <(printf '%s' "$out1") <(printf '%s' "$out2"); echo $?)

set +e
"$FA" plan >/dev/null 2>&1
rc=$?
set -e
check "plan sin archivo devuelve error de uso" $([[ "$rc" -eq 2 ]]; echo $?)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
