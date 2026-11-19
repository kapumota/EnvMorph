#!/usr/bin/env bash
# Precongelacion: SOLO construye imagen y ejecuta chequeos sin tratamientos.
# No ejecuta S/H, ni 'preflight' ni F10 confirmatorio.
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"
G3C="$ROOT/experiments/g3c"
for cmd in docker python3 sha256sum; do
  command -v "$cmd" >/dev/null || { echo "STOP: falta $cmd" >&2; exit 3; }
done
if [[ -e "$G3C/campaign/PROTOCOL_FREEZE.json" || -e "$G3C/campaign/decisions.json" ]]; then
  echo 'STOP: la campaña ya tiene artefactos de freeze o decisiones' >&2
  exit 3
fi
if ! docker info >/dev/null 2>&1; then
  echo 'STOP: Docker Engine no disponible' >&2
  exit 3
fi
(cd "$G3C" && sha256sum -c MANIFEST.sha256 >/dev/null && \
  python3 docker/preparar_debs.py validar)
for f in target/release/envmorph target/release/envmorph-f6v2; do
  [[ -x "$f" ]] || { echo "STOP: falta $f, construir cargo build --release --locked" >&2; exit 3; }
done

# No se borra una sesion antigua: salida unica, fuera del arbol de Git.
DATE="$(date -u +%Y%m%dT%H%M%SZ)"
LOG_BASE="${G3C_PREPARACION_LOG_DIR:-$HOME/EnvMorph-G3C-preparacion}"
mkdir -p "$LOG_BASE"
LOG_DIR="$(mktemp -d "$LOG_BASE/ejecucion-${DATE}-XXXXXX")"
BASE_TAG="envmorph-f10-precongelacion:${DATE}"
G3C_TAG="envmorph-g3c-precongelacion:${DATE}"
echo "LOGS=$LOG_DIR"
echo 'Reutilizando la imagen F10 contra la que se fijo el cierre transitivo de Perl...'
BASE_ID="$(python3 - "$G3C/docker/dependency_debs.json" <<'PYBASE'
import json,sys
print(json.load(open(sys.argv[1],encoding='utf-8')).get('base_f10_image_id',''))
PYBASE
)"
[[ "$BASE_ID" =~ ^sha256:[a-f0-9]{64}$ ]] || {
  echo 'STOP: cierre de dependencias Perl sin base F10 fijada (ejecutar cerrar_dependencias_perl.py)' >&2
  exit 3
}
[[ "$(docker image inspect --format '{{.Id}}' "$BASE_ID" 2>/dev/null || true)" == "$BASE_ID" ]] || {
  echo 'STOP: la imagen base F10 fijada no existe en este Docker; NO reconstruir otra silenciosamente' >&2
  exit 3
}
docker image tag "$BASE_ID" "$BASE_TAG"
printf 'BASE_REUTILIZADA=%s\n' "$BASE_ID" | tee "$LOG_DIR/01-f10-build.log"
printf '%s\n' "$BASE_ID" > "$LOG_DIR/02-f10-image-id.txt"

echo 'Construyendo G3-C con la base local identificada por contenido...'
docker build -f experiments/g3c/docker/Dockerfile.g3c \
  --build-arg "BASE_IMAGE=$BASE_TAG" -t "$G3C_TAG" . 2>&1 | tee "$LOG_DIR/03-g3c-build.log"
AFTER_BASE_ID="$(docker image inspect --format '{{.Id}}' "$BASE_TAG")"
[[ "$AFTER_BASE_ID" == "$BASE_ID" ]] || { echo 'STOP: etiqueta base modificada durante build' >&2; exit 3; }
G3C_ID="$(docker image inspect --format '{{.Id}}' "$G3C_TAG")"
[[ "$G3C_ID" =~ ^sha256:[a-f0-9]{64}$ ]] || { echo 'STOP: ID G3C invalido' >&2; exit 3; }
printf '%s\n' "$G3C_ID" > "$LOG_DIR/04-g3c-image-id.txt"

# Los chequeos NO ejecutan herramientas de los cinco primarios.
FLAGS=(--rm --network none --read-only --tmpfs /tmp:rw,exec,nosuid,nodev,size=512m
       --cap-drop ALL --security-opt no-new-privileges
       --pids-limit 256 --memory 2g --cpus 2 --user 10001:10001)

echo 'Verificando imagen: paquetes fijados y ambiente...'
docker run "${FLAGS[@]}" "$G3C_ID" harness/g3c_harness.py check-install 2>&1 | tee "$LOG_DIR/05-check-install.log"
docker run "${FLAGS[@]}" "$G3C_ID" harness/g3c_harness.py check-env 2>&1 | tee "$LOG_DIR/06-check-env.log"

echo 'Verificando pruebas G3-C dentro de la imagen...'
docker run "${FLAGS[@]}" -e G3C_F6V2_BIN=/repo/target/release/envmorph-f6v2 \
  "$G3C_ID" -m unittest discover -s tests -p 'test_g3c*.py' -v \
  2>&1 | tee "$LOG_DIR/07-test-g3c.log"

(
  cd "$LOG_DIR"
  sha256sum 0*.log 0*.txt > SHA256SUMS_PREPARACION
)
cat <<EOF
G3C_DOCKER_PRECONGELACION=PASS
IMAGEN_BASE_ID=$BASE_ID
IMAGEN_G3C_ID=$G3C_ID
LOGS=$LOG_DIR
NOTA=Imagen local identificada por sha256 (ID), NO digest de manifiesto OCI publicado.
PENDIENTE=F10 real, permisos de volúmenes, dependencias en segundo host y custodia externa.
EOF
