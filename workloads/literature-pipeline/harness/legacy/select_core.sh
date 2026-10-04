#!/bin/bash
# select_core.sh FILE N  (stage 6 of pipeline.sh as a standalone command)
# Header line, then the top-N rows by column 8 (numeric, descending).
set -euo pipefail
head -n1 "$1"
tail -n +2 "$1" | sort -t$'\t' -k8,8nr | head -n "$2"
