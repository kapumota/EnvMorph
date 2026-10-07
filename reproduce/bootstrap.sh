#!/usr/bin/env bash
set -euo pipefail
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$ROOT"
for program in cargo cc python3 sha256sum; do
  command -v "$program" >/dev/null || { echo "Falta dependencia: $program" >&2; exit 1; }
done
# La descarga de crates ocurre solo aquí, antes del experimento.
cargo fetch --locked
cargo build --release --locked --offline
sha256sum -c provenance/F9_10_FILES.sha256
