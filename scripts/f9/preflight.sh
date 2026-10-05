#!/usr/bin/env bash
set -euo pipefail

ROOT="/home/c-lara/EnvMorph"
cd "$ROOT"

[[ "$(git branch --show-current)" == "fase9/hermeticidad-conductual" ]] || {
  echo "ERROR: rama F9 inesperada" >&2
  exit 1
}

for h in \
  provenance/F8_PROTOCOL_FILES.sha256 \
  provenance/F8_SELECTION_FILES.sha256 \
  provenance/F8_CORPUS_FILES.sha256 \
  provenance/F8_ADAPTER_FILES.sha256 \
  provenance/F8_RUNTIME_FILES.sha256 \
  provenance/F8_MATRIX_FILES.sha256 \
  provenance/F8_RESULT_FILES.sha256 \
  provenance/F8_FINAL_FILES.sha256; do
  sha256sum -c "$h" >/dev/null
done

sha256sum -c provenance/F9_0_PROTOCOL_FILES.sha256 >/dev/null

if find src -maxdepth 1 -type f \
  \( -name '*observer*' -o -name '*ptrace*' -o -name '*ebpf*' \) \
  | grep -q .; then
  echo "ERROR: F9.0 detectó backend F9 prematuro en src/" >&2
  exit 1
fi

echo "F9.0 preflight: PASS"
