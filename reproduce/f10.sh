#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
if [ "$#" -ne 1 ]; then
  echo "Uso: bash reproduce/f10.sh DIRECTORIO_NUEVO_DE_EVIDENCIA" >&2
  exit 2
fi
exec python3 "$ROOT/reproduce/f10.py" --output "$1"
