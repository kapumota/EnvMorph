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

out1="$($FA capabilities "$SPEC")"
out2="$($FA capabilities "$SPEC")"

check "capabilities acepta la especificación F1" $([[ -n "$out1" ]]; echo $?)
check "capabilities reporta seis capacidades" $(contains "$out1" "Capacidades: 6"; echo $?)
check "locale C está disponible" $(grep -Eq '^locale=C  disponible([[:space:]]|$)' <<<"$out1"; echo $?)
check "timezone UTC está disponible" $(grep -Eq '^timezone=UTC  disponible([[:space:]]|$)' <<<"$out1"; echo $?)
check "mawk queda clasificado" $(grep -Eq '^awk_implementation=mawk  (disponible|no_disponible)([[:space:]]|$)' <<<"$out1"; echo $?)
check "gawk queda clasificado" $(grep -Eq '^awk_implementation=gawk  (disponible|no_disponible)([[:space:]]|$)' <<<"$out1"; echo $?)
check "dos inspecciones consecutivas son idénticas" $(cmp -s <(printf '%s' "$out1") <(printf '%s' "$out2"); echo $?)

echo
echo "pasaron $PASS, fallaron $FAIL"
[[ "$FAIL" -eq 0 ]]
