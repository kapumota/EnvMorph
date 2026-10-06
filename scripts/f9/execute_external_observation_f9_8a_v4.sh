#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-/home/c-lara/EnvMorph}"
RAW_ROOT="${2:-.envmorph/f9-8a-evidence}"
PROTOCOL_COMMIT="${3:?Falta commit de protocolo}"

cd "$ROOT"

IDS=(
  csvkit-realdata
  jq-build-check
  fd-build-completions
  inih-meson-tests
)

RUNTIME=".envmorph/f8-runtime"
RECORDS="$RAW_ROOT/records"
TOOLS="$RAW_ROOT/tools"

mkdir -p "$RAW_ROOT" "$RECORDS" "$TOOLS"

need(){
  command -v "$1" >/dev/null 2>&1 || {
    echo "ERROR: falta $1" >&2
    exit 2
  }
}

for c in python3 gcc make sha256sum timeout cp rm locale; do
  need "$c"
done

matrix_value(){
  local key="$1"
  python3 - "$ROOT/experiments/f8/matrix_manifest.toml" "$key" <<'PY'
from pathlib import Path
import re
import sys

text = Path(sys.argv[1]).read_text(encoding="utf-8")
key = re.escape(sys.argv[2])
m = re.search(rf'^{key}\s*=\s*"([^"]*)"\s*$', text, re.M)
if not m:
    raise SystemExit(f"No se encontró {sys.argv[2]} en matrix_manifest")
print(m.group(1))
PY
}

LOCALE_BASE="$(matrix_value locale_baseline)"
TZ_BASE="$(matrix_value timezone_baseline)"
TMP_BASE="$(matrix_value tmpdir_baseline)"

python3 - "$RAW_ROOT/baseline_environment.json" \
  "$LOCALE_BASE" "$TZ_BASE" "$TMP_BASE" "$PROTOCOL_COMMIT" <<'PY'
import json
import sys
from pathlib import Path

Path(sys.argv[1]).write_text(
    json.dumps(
        {
            "schema_version": 1,
            "locale_baseline": sys.argv[2],
            "timezone_baseline": sys.argv[3],
            "tmpdir_baseline": sys.argv[4],
            "source": "experiments/f8/matrix_manifest.toml",
            "observation_protocol_commit": sys.argv[5],
        },
        indent=2,
        sort_keys=True,
    ) + "\n",
    encoding="utf-8",
)
PY

# Verificar runtime preparado, sin reconstruirlo ni descargar dependencias.
[[ -d "$RUNTIME" ]] || {
  echo "ERROR: falta $RUNTIME" >&2
  exit 2
}

[[ -x "$RUNTIME/csvkit-realdata/deps/venv/bin/in2csv" ]] || {
  echo "ERROR: csvkit runtime incompleto" >&2
  exit 2
}
[[ -f "$RUNTIME/jq-build-check/source/vendor/oniguruma/configure.ac" ]] || {
  echo "ERROR: jq runtime incompleto" >&2
  exit 2
}
[[ -f "$RUNTIME/fd-build-completions/source/.cargo/config.toml" ]] || {
  echo "ERROR: fd runtime incompleto" >&2
  exit 2
}
[[ -x "$RUNTIME/inih-meson-tests/deps/venv/bin/meson" ]] || {
  echo "ERROR: inih Meson runtime incompleto" >&2
  exit 2
}
[[ -x "$RUNTIME/inih-meson-tests/deps/venv/bin/ninja" ]] || {
  echo "ERROR: inih Ninja runtime incompleto" >&2
  exit 2
}

(
  cd "$RUNTIME/csvkit-realdata/deps/wheelhouse"
  sha256sum -c "$ROOT/provenance/f8/runtime/csvkit-wheelhouse.sha256" >/dev/null
)
(
  cd "$RUNTIME/jq-build-check/source/vendor/oniguruma"
  sha256sum -c "$ROOT/provenance/f8/runtime/jq-oniguruma.files.sha256" >/dev/null
)
(
  cd "$RUNTIME/fd-build-completions/vendor"
  sha256sum -c "$ROOT/provenance/f8/runtime/fd-vendor.files.sha256" >/dev/null
)
(
  cd "$RUNTIME/inih-meson-tests/deps/wheelhouse"
  sha256sum -c "$ROOT/provenance/f8/runtime/inih-wheelhouse.sha256" >/dev/null
)
sha256sum -c provenance/f8/runtime/inih-runtime-adapter.sha256 >/dev/null

# csvkit, jq e inih conservan snapshots source byte-exactos dentro del runtime.
for id in csvkit-realdata jq-build-check inih-meson-tests; do
  (
    cd "$RUNTIME/$id/source"
    sha256sum -c "$ROOT/provenance/f8/$id.files.sha256" >/dev/null
  )
done

# fd es el único caso donde la preparación runtime F8 modifica deliberadamente
# source/.cargo/config.toml mediante:
# cargo vendor --locked ../vendor > .cargo/config.toml
# El resto del snapshot upstream debe permanecer byte-exacto.
fd_manifest="$(mktemp)"
grep -vE '^[0-9a-f]{64}[[:space:]]+\./\.cargo/config\.toml$' \
  "$ROOT/provenance/f8/fd-build-completions.files.sha256" \
  > "$fd_manifest"

(
  cd "$RUNTIME/fd-build-completions/source"
  sha256sum -c "$fd_manifest" >/dev/null
)
rm -f "$fd_manifest"

# El binding runtime se valida semánticamente y el contenido vendor ya fue
# validado byte-exacto arriba mediante fd-vendor.files.sha256.
python3 - "$RUNTIME/fd-build-completions/source/.cargo/config.toml" <<'PY_FD_CONFIG'
from pathlib import Path
import re
import sys

path = Path(sys.argv[1])
if not path.is_file():
    raise SystemExit("Falta fd source/.cargo/config.toml runtime")

text = path.read_text(encoding="utf-8")

checks = [
    re.search(r'(?m)^\[source\.crates-io\]\s*$', text),
    re.search(
        r'(?m)^\s*replace-with\s*=\s*"vendored-sources"\s*$',
        text,
    ),
    re.search(r'(?m)^\[source\.vendored-sources\]\s*$', text),
    re.search(
        r'(?m)^\s*directory\s*=\s*"\.\./vendor"\s*$',
        text,
    ),
]

if not all(checks):
    raise SystemExit(
        "fd .cargo/config.toml no conserva el binding offline esperado"
    )

if re.search(r'(?m)^\s*directory\s*=\s*"/', text):
    raise SystemExit(
        "fd .cargo/config.toml contiene una ruta vendor absoluta"
    )
PY_FD_CONFIG

# El backend sigue siendo el observer-only derivado de AgentGuard.
# La única corrección es el transporte JSON byte-safe de rutas.
gcc -std=c11 -O2 -Wall -Wextra -Wpedantic -Werror   -o "$TOOLS/f9-observer"   tools/f9/external-observer/observer_bytesafe.c

gcc -std=c11 -O2 -Wall -Wextra -Wpedantic -Werror -fPIC -shared \
  -o "$TOOLS/libf9-getenv-observer-atomic.so" \
  tools/f9/external-observer/getenv_observer_atomic.c

OBSERVER="$ROOT/$TOOLS/f9-observer"
INTERPOSER="$ROOT/$TOOLS/libf9-getenv-observer-atomic.so"

sanitize_workspace(){
  local id="$1"
  local ws="$2"

  rm -rf "$ws/artifacts"

  case "$id" in
    jq-build-check)
      rm -rf \
        "$ws/source/autom4te.cache" \
        "$ws/source/.deps" \
        "$ws/source/src/.deps" \
        "$ws/source/vendor/oniguruma/src/.deps"
      rm -f \
        "$ws/source/jq" \
        "$ws/source/src/"*.o \
        "$ws/source/vendor/oniguruma/src/"*.o \
        2>/dev/null || true
      ;;
    fd-build-completions)
      rm -rf "$ws/source/target"
      ;;
    inih-meson-tests)
      rm -rf "$ws/build"
      ;;
    csvkit-realdata)
      ;;
    *)
      echo "ERROR: workload desconocido $id" >&2
      return 2
      ;;
  esac
}

baseline_env_json(){
  local id="$1"
  local output="$2"

  python3 - "$id" "$output" "$LOCALE_BASE" "$TZ_BASE" "$TMP_BASE" <<'PY'
import json
import sys
from pathlib import Path

wid = sys.argv[1]
values = {
    "locale": sys.argv[3],
    "timezone": sys.argv[4],
    "tmpdir": sys.argv[5],
}

factors = {
    "csvkit-realdata": ["locale"],
    "jq-build-check": ["locale", "timezone"],
    "fd-build-completions": ["tmpdir"],
    "inih-meson-tests": ["tmpdir"],
}[wid]

bindings = {
    "locale": "LC_ALL",
    "timezone": "TZ",
    "tmpdir": "TMPDIR",
}

env = {
    bindings[factor]: values[factor]
    for factor in factors
}

Path(sys.argv[2]).write_text(
    json.dumps(
        {
            "factors": factors,
            "environment": env,
        },
        indent=2,
        sort_keys=True,
    ) + "\n",
    encoding="utf-8",
)
PY
}

write_record_failure(){
  local id="$1"
  local run_id="$2"
  local status="$3"
  local observer_effect="$4"
  local reason="$5"
  local env_json="$6"
  local record="$RECORDS/$id.json"

  python3 - "$record" "$id" "$run_id" "$status" \
    "$observer_effect" "$reason" "$env_json" <<'PY'
import json
import sys
from pathlib import Path

env = json.loads(Path(sys.argv[7]).read_text(encoding="utf-8"))

data = {
    "workload_id": sys.argv[2],
    "run_id": sys.argv[3],
    "workload_status": sys.argv[4],
    "observer_effect": sys.argv[5],
    "failure_reason": sys.argv[6],
    "baseline_environment": env,
    "f4_trace_status": None,
    "graph_file": None,
    "graph_sha256": None,
    "census_sha256": None,
    "candidate_count": 0,
    "instrumentation_events_removed": 0,
}

Path(sys.argv[1]).write_text(
    json.dumps(data, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY
}

for id in "${IDS[@]}"; do
  echo
  echo "### F9.8A observar $id"

  run_id="f9-8a-$id"
  wr="$RAW_ROOT/$id"
  base_parent="$wr/baseline"
  obs_parent="$wr/observed"
  base_ws="$base_parent/workspace"
  obs_ws="$obs_parent/workspace"
  env_json="$wr/baseline_env.json"

  mkdir -p "$base_parent" "$obs_parent"
  baseline_env_json "$id" "$env_json"

  cp -a "$RUNTIME/$id" "$base_ws"
  cp -a "$RUNTIME/$id" "$obs_ws"

  sanitize_workspace "$id" "$base_ws"
  sanitize_workspace "$id" "$obs_ws"

  common_env=(
    "PIP_NO_INDEX=1"
    "PIP_DISABLE_PIP_VERSION_CHECK=1"
    "CARGO_NET_OFFLINE=true"
    "GIT_TERMINAL_PROMPT=0"
  )

  case "$id" in
    csvkit-realdata)
      workload_env=("LC_ALL=$LOCALE_BASE")
      ;;
    jq-build-check)
      workload_env=("LC_ALL=$LOCALE_BASE" "TZ=$TZ_BASE")
      ;;
    fd-build-completions|inih-meson-tests)
      workload_env=("TMPDIR=$TMP_BASE")
      ;;
  esac

  set +e
  (
    cd "$base_ws"
    timeout 1200 env \
      "${common_env[@]}" \
      "${workload_env[@]}" \
      bash adapter/run.sh
  ) > "$wr/baseline.stdout" 2> "$wr/baseline.stderr"
  base_rc=$?
  set -e
  printf '%s\n' "$base_rc" > "$wr/baseline.rc"

  if [[ "$base_rc" -ne 0 ]]; then
    write_record_failure \
      "$id" "$run_id" "execution_failed" "not_evaluated" \
      "unobserved_baseline_rc_$base_rc" "$env_json"
    rm -rf "$base_parent" "$obs_parent"
    continue
  fi

  ptrace_log="$ROOT/$wr/ptrace.jsonl"
  getenv_log="$ROOT/$wr/getenv.jsonl"

  : > "$wr/getenv.jsonl"

  set +e
  (
    cd "$obs_ws"
    timeout 1200 env \
      "${common_env[@]}" \
      "${workload_env[@]}" \
      "$OBSERVER" \
      --output "$ptrace_log" \
      -- \
      bash -c '
        export LD_PRELOAD="$1"
        export ENVMORPH_F9_INTERPOSE_LOG="$2"
        exec bash adapter/run.sh
      ' _ "$INTERPOSER" "$getenv_log"
  ) > "$wr/observed.stdout" 2> "$wr/observed.stderr"
  obs_rc=$?
  set -e
  printf '%s\n' "$obs_rc" > "$wr/observed.rc"

  if [[ "$obs_rc" -ne 0 ]]; then
    write_record_failure \
      "$id" "$run_id" "observer_failure" "unresolved" \
      "observed_execution_rc_$obs_rc" "$env_json"
    rm -rf "$base_parent" "$obs_parent"
    continue
  fi

  trace="$ROOT/workloads/external/$id/adapter/trace.tsv"
  f4_json="$wr/observer_effect_f4.json"

  set +e
  "$ROOT/target/release/envmorph" analyze-propagation \
    --trace "$trace" \
    --left-root "$base_parent" \
    --right-root "$obs_parent" \
    --format json \
    --output "$f4_json" \
    > "$wr/observer_effect.stdout" \
    2> "$wr/observer_effect.stderr"
  f4_rc=$?
  set -e
  printf '%s\n' "$f4_rc" > "$wr/observer_effect.rc"

  [[ -s "$f4_json" ]] || {
    write_record_failure \
      "$id" "$run_id" "observer_failure" "unresolved" \
      "observer_effect_f4_missing_output" "$env_json"
    rm -rf "$base_parent" "$obs_parent"
    continue
  }

  observer_effect="$(
    python3 - "$f4_json" <<'PY'
import json
import sys

x = json.load(open(sys.argv[1], encoding="utf-8"))
status = x.get("trace_status")

if status == "stable":
    print("preserved")
elif status in ("absorbed", "persistent"):
    print("altered")
elif status == "unresolved":
    print("unresolved")
else:
    raise SystemExit(f"trace_status F4 desconocido: {status}")
PY
  )"

  f4_status="$(
    python3 - "$f4_json" <<'PY'
import json
import sys
print(json.load(open(sys.argv[1], encoding="utf-8"))["trace_status"])
PY
  )"

  normalized="$wr/normalized.jsonl"
  normalize_metrics="$wr/normalize_metrics.json"

  python3 tools/f9/event-normalizer/normalize_events.py \
    --ptrace "$wr/ptrace.jsonl" \
    --getenv "$wr/getenv.jsonl" \
    --run-id "$run_id" \
    --output "$normalized" \
    --dependencies "$wr/dependencies_before_filter.json" \
    --metrics "$normalize_metrics" \
    --path-prefix "$ROOT/$obs_ws" \
    --path-label "@WORKSPACE@"

  filtered="$wr/filtered_events.jsonl"
  filter_metrics="$wr/filter_metrics.json"

  python3 tools/f9/external-observer/filter_instrumentation.py \
    --input "$normalized" \
    --output "$filtered" \
    --metrics "$filter_metrics" \
    --exclude-path "$INTERPOSER" \
    --exclude-path "$getenv_log" \
    --exclude-env "LD_PRELOAD" \
    --exclude-env "ENVMORPH_F9_INTERPOSE_LOG"

  if [[ ! -s "$filtered" ]]; then
    write_record_failure \
      "$id" "$run_id" "observer_failure" "unresolved" \
      "no_events_after_instrumentation_filter" "$env_json"
    rm -rf "$base_parent" "$obs_parent"
    continue
  fi

  graph="$wr/graph.json"
  graph_metrics="$wr/graph_metrics.json"

  python3 tools/f9/dependency-graph/build_graph.py \
    --events "$filtered" \
    --output "$graph" \
    --metrics "$graph_metrics"

  graph_sha="$(sha256sum "$graph" | awk '{print $1}')"

  python3 - "$graph" "$id" "$run_id" "$graph_sha" \
    "$observer_effect" "$f4_status" "$env_json" "$filter_metrics" \
    "$RECORDS/$id.json" "$ROOT/$graph" <<'PY'
import hashlib
import json
import sys
from pathlib import Path

graph = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
wid = sys.argv[2]
run_id = sys.argv[3]
graph_sha = sys.argv[4]
observer_effect = sys.argv[5]
f4_status = sys.argv[6]
baseline_environment = json.loads(
    Path(sys.argv[7]).read_text(encoding="utf-8")
)
filter_metrics = json.loads(
    Path(sys.argv[8]).read_text(encoding="utf-8")
)
output = Path(sys.argv[9])
graph_path_abs = Path(sys.argv[10]).resolve()
repo = Path("/home/c-lara/EnvMorph").resolve()

try:
    graph_rel = str(graph_path_abs.relative_to(repo))
except ValueError:
    raise SystemExit("Grafo fuera del repositorio")

keys = sorted(
    node["dependency_key"]
    for node in graph["nodes"]
    if node.get("node_type") == "dependency"
)

payload = json.dumps(
    {
        "run_id": run_id,
        "workload_id": wid,
        "dependency_keys": keys,
    },
    sort_keys=True,
    separators=(",", ":"),
    ensure_ascii=False,
).encode("utf-8")

census_sha = hashlib.sha256(payload).hexdigest()

record = {
    "workload_id": wid,
    "run_id": run_id,
    "workload_status": "completed",
    "observer_effect": observer_effect,
    "failure_reason": None,
    "baseline_environment": baseline_environment,
    "f4_trace_status": f4_status,
    "graph_file": graph_rel,
    "graph_sha256": graph_sha,
    "census_sha256": census_sha,
    "candidate_count": len(keys),
    "instrumentation_events_removed":
        filter_metrics["removed_events"],
}

output.write_text(
    json.dumps(record, indent=2, sort_keys=True) + "\n",
    encoding="utf-8",
)
PY

  # Los workspaces son copias de ejecución, no evidencia compacta.
  rm -rf "$base_parent" "$obs_parent"
done

# El binario y la biblioteca son materializaciones reproducibles, no evidencia.
rm -rf "$TOOLS"

# Evidencia cruda y grafos quedan localmente bajo .envmorph.
(
  cd "$RAW_ROOT"
  find . -type f ! -name RAW.sha256 -print0 |
    LC_ALL=C sort -z |
    xargs -0 -r sha256sum > RAW.sha256
)

exit 0
