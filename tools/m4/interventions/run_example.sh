#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"
sha256sum -c SHA256SUMS
python3.11 m4_interventions.py selftest
pf="$HOME/envmorph-m4-g2-example-preflight"
out="$HOME/envmorph-m4-g2-example-variants"
python3.11 m4_interventions.py prepare \
  --spec examples/SPEC.json \
  --workspace examples/toy_workspace \
  --fixtures examples/fixtures \
  --output "$pf"
sha="$(python3.11 - "$pf/M4_G2_PREFLIGHT.json" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text())['spec_sha256'])
PY
)"
python3.11 m4_interventions.py materialize \
  --spec examples/SPEC.json \
  --workspace examples/toy_workspace \
  --fixtures examples/fixtures \
  --output "$out" \
  --expected-spec-sha256 "$sha" \
  --ack-development-only
printf 'M4_G2_EXAMPLE=PASS\nM4_G2_SCIENTIFIC_EVIDENCE=NONE_TOY_ONLY\n'
