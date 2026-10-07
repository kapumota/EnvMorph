#!/usr/bin/env bash
set -euo pipefail

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
IMAGE="envmorph-f10-ubuntu2404:local"

if [[ "$#" -ne 1 ]]; then
  echo "Uso: bash reproduce/f10-docker.sh DIRECTORIO_NUEVO_DE_EVIDENCIA" >&2
  exit 2
fi

OUTPUT="$1"

if [[ "$OUTPUT" != /* ]]; then
  OUTPUT="$(pwd)/$OUTPUT"
fi

if [[ -e "$OUTPUT" ]]; then
  echo "El destino ya existe: $OUTPUT" >&2
  exit 1
fi

PARENT="$(dirname "$OUTPUT")"
NAME="$(basename "$OUTPUT")"

mkdir -p "$PARENT"

for program in docker cargo sha256sum; do
  command -v "$program" >/dev/null || {
    echo "Falta dependencia: $program" >&2
    exit 1
  }
done

docker info >/dev/null 2>&1 || {
  echo "Docker no está disponible" >&2
  exit 1
}

cd "$ROOT"

cargo fmt --check
cargo test --locked
cargo build --release --locked
sha256sum -c provenance/F9_10_FILES.sha256

docker build \
  --pull \
  -f reproduce/Dockerfile.f10 \
  -t "$IMAGE" \
  .

HOST_UID="$(id -u)"
HOST_GID="$(id -g)"

docker run --rm \
  --network none \
  --user "$HOST_UID:$HOST_GID" \
  -e HOME=/tmp \
  -e PYTHONDONTWRITEBYTECODE=1 \
  -v "$ROOT:/repo:ro" \
  -v "$PARENT:/evidence" \
  "$IMAGE" \
  bash -lc "
    set -euo pipefail

    date_v=\"\$(/usr/bin/date --version | head -n1 | awk '{print \$NF}')\"
    ls_v=\"\$(/usr/bin/ls --version | head -n1 | awk '{print \$NF}')\"
    py_v=\"\$(/usr/bin/python3 --version | awk '{print \$2}')\"

    printf 'Container date: %s\n' \"\$date_v\"
    printf 'Container ls: %s\n' \"\$ls_v\"
    printf 'Container python3: %s\n' \"\$py_v\"

    test \"\$date_v\" = '9.4'
    test \"\$ls_v\" = '9.4'
    test \"\$py_v\" = '3.12.3'
    test -f /usr/include/fcntl.h

    cc -shared -fPIC -O2 -Wall -Wextra \
      -o /tmp/envmorph-f10-observer-preflight.so \
      /repo/tools/f9/external-observer/getenv_observer_atomic.c

    test -s /tmp/envmorph-f10-observer-preflight.so

    python3 /repo/reproduce/f10.py --output /evidence/$NAME
    python3 /repo/reproduce/verify_f10.py /evidence/$NAME
    python3 /repo/reproduce/test_f10.py /evidence/$NAME
  "

printf '\nF10 DOCKER PASS\n'
printf 'Evidencia: %s\n' "$OUTPUT"
