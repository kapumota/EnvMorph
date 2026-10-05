#!/usr/bin/env bash
set -euo pipefail
mkdir -p artifacts
BIN="deps/venv/bin"
[[ -x "$BIN/in2csv" ]] || { echo "csvkit no preparado offline" >&2; exit 70; }
"$BIN/in2csv" source/examples/realdata/ne_1033_data.xlsx > artifacts/01_data.csv
"$BIN/csvcut" -c county,item_name,quantity artifacts/01_data.csv > artifacts/02_selected.csv
"$BIN/csvlook" artifacts/02_selected.csv | head -n 20 > artifacts/03_preview.txt
