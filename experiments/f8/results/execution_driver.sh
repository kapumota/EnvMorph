#!/usr/bin/env bash
set -euo pipefail

ROOT="/home/c-lara/EnvMorph"
BRANCH="fase8/validacion-externa"
EXPECTED_SUBJECT="Preparar runtime y congelar matriz F8"
EXPECTED_PARENT_SHORT="ae16104"
COMMIT_MESSAGE="Ejecutar validación externa de F8"
FA="$ROOT/target/release/envmorph"
RAW_ROOT="$ROOT/.envmorph/f8-evidence"
RESULT_ROOT="$ROOT/experiments/f8/results"
RESULT_MANIFEST="$RESULT_ROOT/results_manifest.json"
RESULT_HASH="$ROOT/provenance/F8_RESULT_FILES.sha256"
REPORT="$ROOT/audits/F8_5_6_EXECUTION_REPORT.txt"
DRIVER_COPY="$RESULT_ROOT/execution_driver.sh"

IDS=(csvkit-realdata jq-build-check fd-build-completions inih-meson-tests)
STEMS=(csvkit jq fd inih)

say(){ printf '\n### %s\n' "$1"; }
die(){ printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Falta el comando requerido: $1"; }

copy_bundle_evidence(){
  local src="$1" dst="$2"
  mkdir -p "$dst"
  [[ -f "$src/experiment.json" ]] && cp "$src/experiment.json" "$dst/experiment.json"
  [[ -f "$src/hashes.sha256" ]] && cp "$src/hashes.sha256" "$dst/hashes.sha256"
  if [[ -d "$src/variants" ]]; then
    local v name out
    for v in "$src"/variants/*; do
      [[ -d "$v" ]] || continue
      name="$(basename "$v")"
      out="$dst/variants/$name"
      mkdir -p "$out"
      for f in manifest.json stdout.txt stderr.txt; do
        [[ -f "$v/$f" ]] && cp "$v/$f" "$out/$f"
      done
      if [[ -d "$v/workspace/artifacts" ]]; then
        mkdir -p "$out/workspace"
        cp -a "$v/workspace/artifacts" "$out/workspace/artifacts"
      fi
    done
  fi
}

pair_from_causal(){
  local causal="$1" bundle="$2"
  python3 - "$causal" "$bundle" <<'PY'
import json,re,sys
from pathlib import Path
causal=Path(sys.argv[1]).read_text(encoding='utf-8')
bundle=Path(sys.argv[2])
blocks=causal.split('[[factor]]')[1:]
factors=[]
for b in blocks:
    n=re.search(r'^name = "([^"]+)"$',b,re.M)
    ba=re.search(r'^baseline = "([^"]+)"$',b,re.M)
    tr=re.search(r'^treatment = "([^"]+)"$',b,re.M)
    if n and ba and tr: factors.append((n.group(1),ba.group(1),tr.group(1)))
if not factors: raise SystemExit(2)
base={n:b for n,b,t in factors}; full={n:t for n,b,t in factors}
found={}
for mf in sorted((bundle/'variants').glob('*/manifest.json')):
    d=json.loads(mf.read_text(encoding='utf-8'))
    a={x['factor']:x['value'] for x in d.get('assignments',[])}
    if a==base: found['baseline']=(mf.parent.name,d.get('status'))
    if a==full: found['full']=(mf.parent.name,d.get('status'))
if 'baseline' not in found or 'full' not in found: raise SystemExit(3)
print(found['baseline'][0],found['baseline'][1],found['full'][0],found['full'][1])
PY
}

pair_from_environment(){
  local causal="$1" envfile="$2" bundle="$3"
  python3 - "$causal" "$envfile" "$bundle" <<'PY'
import json,re,sys
from pathlib import Path
causal=Path(sys.argv[1]).read_text(encoding='utf-8')
envtxt=Path(sys.argv[2]).read_text(encoding='utf-8')
bundle=Path(sys.argv[3])
base={}
for b in causal.split('[[factor]]')[1:]:
    n=re.search(r'^name = "([^"]+)"$',b,re.M); v=re.search(r'^baseline = "([^"]+)"$',b,re.M)
    if n and v: base[n.group(1)]=v.group(1)
target={}
for b in envtxt.split('[[factor]]')[1:]:
    n=re.search(r'^name = "([^"]+)"$',b,re.M); v=re.search(r'^value = "([^"]+)"$',b,re.M)
    if n and v: target[n.group(1)]=v.group(1)
found={}
for mf in sorted((bundle/'variants').glob('*/manifest.json')):
    d=json.loads(mf.read_text(encoding='utf-8'))
    a={x['factor']:x['value'] for x in d.get('assignments',[])}
    if all(a.get(k)==v for k,v in base.items()) and set(a)==set(base):
        found['baseline']=(mf.parent.name,d.get('status'))
    if all(a.get(k)==v for k,v in target.items()):
        # El bundle held-out puede contener factores adicionales conocidos.
        ok=True
        for k,v in base.items():
            if k not in target and a.get(k)!=v: ok=False
        if ok: found['target']=(mf.parent.name,d.get('status'))
if 'baseline' not in found or 'target' not in found: raise SystemExit(3)
print(found['baseline'][0],found['baseline'][1],found['target'][0],found['target'][1])
PY
}

cd "$ROOT" || die "No existe $ROOT"
for c in git sha256sum python3 bash find sort cargo; do need "$c"; done
[[ "$(git rev-parse --show-toplevel)" == "$ROOT" ]] || die "Root Git inesperado"
[[ "$(git branch --show-current)" == "$BRANCH" ]] || die "F8.5 debe ejecutarse en $BRANCH"
[[ -z "$(git status --porcelain)" ]] || die "Working tree debe estar limpio"
[[ "$(git log -1 --format=%s)" == "$EXPECTED_SUBJECT" ]] || die "F8.5 requiere F8.4 cerrado"
[[ "$(git rev-parse --short=7 HEAD)" == "$EXPECTED_PARENT_SHORT" ]] || die "F8.5 requiere exactamente el freeze F8.4 ae16104"
[[ -x "$FA" ]] || die "No existe binario release de EnvMorph"
git check-ignore -q .envmorph || die ".envmorph debe estar ignorado antes de materializar evidencia pesada"

say "Verificar freeze F8.0-F8.4"
for h in \
  provenance/F8_PROTOCOL_FILES.sha256 \
  provenance/F8_SELECTION_FILES.sha256 \
  provenance/F8_CORPUS_FILES.sha256 \
  provenance/F8_ADAPTER_FILES.sha256 \
  provenance/F8_RUNTIME_FILES.sha256 \
  provenance/F8_MATRIX_FILES.sha256; do
  sha256sum -c "$h"
done
bash scripts/f8/materialize_corpus_verified.sh
for id in "${IDS[@]}"; do
  (cd "workloads/external/$id/source" && sha256sum -c "$ROOT/provenance/f8/$id.files.sha256" >/dev/null)
done
git diff --exit-code cad211e -- workloads/literature-pipeline/source
[[ -d .envmorph/f8-runtime ]] || die "Falta runtime F8.4 en .envmorph/f8-runtime"
[[ -x .envmorph/f8-runtime/csvkit-realdata/deps/venv/bin/in2csv ]] || die "csvkit runtime incompleto"
[[ -f .envmorph/f8-runtime/jq-build-check/source/vendor/oniguruma/configure.ac ]] || die "jq runtime sin oniguruma congelado"
[[ -f .envmorph/f8-runtime/fd-build-completions/source/.cargo/config.toml ]] || die "fd runtime sin vendor config"
[[ -x .envmorph/f8-runtime/inih-meson-tests/deps/venv/bin/meson ]] || die "inih runtime sin Meson congelado"
[[ -x .envmorph/f8-runtime/inih-meson-tests/deps/venv/bin/ninja ]] || die "inih runtime sin Ninja congelado"

(
  cd .envmorph/f8-runtime/csvkit-realdata/deps/wheelhouse
  sha256sum -c "$ROOT/provenance/f8/runtime/csvkit-wheelhouse.sha256" >/dev/null
)
(
  cd .envmorph/f8-runtime/jq-build-check/source/vendor/oniguruma
  sha256sum -c "$ROOT/provenance/f8/runtime/jq-oniguruma.files.sha256" >/dev/null
)
(
  cd .envmorph/f8-runtime/fd-build-completions/vendor
  sha256sum -c "$ROOT/provenance/f8/runtime/fd-vendor.files.sha256" >/dev/null
)
(
  cd .envmorph/f8-runtime/inih-meson-tests/deps/wheelhouse
  sha256sum -c "$ROOT/provenance/f8/runtime/inih-wheelhouse.sha256" >/dev/null
)
sha256sum -c provenance/f8/runtime/inih-runtime-adapter.sha256 >/dev/null

export PIP_NO_INDEX=1
export PIP_DISABLE_PIP_VERSION_CHECK=1
export CARGO_NET_OFFLINE=true
export GIT_TERMINAL_PROMPT=0

[[ ! -e "$RESULT_ROOT" ]] || die "Ya existe $RESULT_ROOT"
[[ ! -e "$RESULT_HASH" ]] || die "Ya existe $RESULT_HASH"
[[ ! -e "$REPORT" ]] || die "Ya existe $REPORT"
[[ ! -e "$RAW_ROOT" ]] || die "Ya existe evidencia cruda en $RAW_ROOT, no se sobrescribe"
mkdir -p "$RAW_ROOT" "$RESULT_ROOT"
cp -p "${BASH_SOURCE[0]}" "$DRIVER_COPY"
DRIVER_SHA256="$(sha256sum "$DRIVER_COPY" | awk '{print $1}')"

ORIGINAL_HEAD="$(git rev-parse HEAD)"
COMMITTED=0
cleanup(){
  rc=$?
  if [[ $rc -ne 0 && $COMMITTED -eq 0 ]]; then
    printf '\nERROR: F8.5/F8.6 tuvo un fallo de infraestructura antes del commit.\n' >&2
    printf 'La evidencia cruda se conserva en %s para diagnóstico.\n' "$RAW_ROOT" >&2
    git reset --hard "$ORIGINAL_HEAD" >/dev/null 2>&1 || true
    rm -rf "$RESULT_ROOT"
    rm -f "$RESULT_HASH" "$REPORT"
  fi
  exit "$rc"
}
trap cleanup EXIT

say "Ejecutar campaña confirmatoria F2-F7"
for i in "${!IDS[@]}"; do
  id="${IDS[$i]}"; stem="${STEMS[$i]}"
  spec="$ROOT/experiments/f8/matrix/$stem-inference.toml"
  causal="$ROOT/experiments/f8/matrix/$stem-causal.toml"
  trace="$ROOT/workloads/external/$id/adapter/trace.tsv"
  runtime="$ROOT/.envmorph/f8-runtime/$id"
  raw="$RAW_ROOT/$id"
  result="$RESULT_ROOT/$id"
  mkdir -p "$raw" "$result"
  printf '\n#### %s\n' "$id"

  set +e
  "$FA" explore "$spec" --output "$raw/inference" --workdir "$runtime" -- bash adapter/run.sh >"$raw/explore.log" 2>&1
  explore_rc=$?
  set -e
  printf '%s\n' "$explore_rc" > "$raw/explore.rc"
  [[ -f "$raw/inference/experiment.json" ]] || {
    printf '{"workload":"%s","state":"experiment_error","explore_rc":%s}\n' "$id" "$explore_rc" > "$result/metrics.json"
    cp "$raw/explore.log" "$result/explore.log"
    continue
  }
  copy_bundle_evidence "$raw/inference" "$result/inference"

  set +e
  pair="$(pair_from_causal "$causal" "$raw/inference" 2>/dev/null)"
  pair_rc=$?
  set -e
  if [[ $pair_rc -ne 0 ]]; then
    printf '{"workload":"%s","state":"pair_unresolved","explore_rc":%s}\n' "$id" "$explore_rc" > "$result/metrics.json"
    continue
  fi
  read -r baseline_id baseline_status full_id full_status <<<"$pair"
  if [[ "$baseline_status" != "execution_succeeded" || "$full_status" != "execution_succeeded" ]]; then
    python3 - "$id" "$explore_rc" "$baseline_id" "$baseline_status" "$full_id" "$full_status" "$result/metrics.json" <<'PY'
import json,sys
json.dump({"workload":sys.argv[1],"state":"not_evaluable_f4","explore_rc":int(sys.argv[2]),"baseline_variant":sys.argv[3],"baseline_status":sys.argv[4],"full_variant":sys.argv[5],"full_status":sys.argv[6]},open(sys.argv[7],'w'),indent=2); open(sys.argv[7],'a').write('\n')
PY
    continue
  fi

  set +e
  "$FA" analyze-propagation --trace "$trace" \
    --left-root "$raw/inference/variants/$baseline_id" \
    --right-root "$raw/inference/variants/$full_id" \
    --format json --output "$result/inference/trace.json" >"$raw/f4.log" 2>&1
  f4_rc=$?
  set -e
  if [[ $f4_rc -gt 2 || ! -s "$result/inference/trace.json" ]]; then
    python3 - "$id" "$f4_rc" "$result/metrics.json" <<'PY'
import json,sys
json.dump({"workload":sys.argv[1],"state":"f4_error","f4_rc":int(sys.argv[2])},open(sys.argv[3],'w'),indent=2); open(sys.argv[3],'a').write('\n')
PY
    continue
  fi
  trace_status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["trace_status"])' "$result/inference/trace.json")"
  if [[ "$trace_status" == "unresolved" ]]; then
    python3 - "$id" "$trace_status" "$result/metrics.json" <<'PY'
import json,sys
json.dump({"workload":sys.argv[1],"state":"f4_unresolved","trace_status":sys.argv[2]},open(sys.argv[3],'w'),indent=2); open(sys.argv[3],'a').write('\n')
PY
    continue
  fi

  mkdir -p "$raw/causal" "$result/causal"
  set +e
  "$FA" derive-contract "$causal" --trace "$trace" --workdir "$runtime" \
    --evidence "$raw/causal/evidence" --output "$result/causal/contract.toml" \
    --format toml -- bash adapter/run.sh >"$raw/derive-contract.log" 2>&1
  contract_rc=$?
  set -e
  [[ -f "$raw/causal/evidence/mce-result.json" ]] && cp "$raw/causal/evidence/mce-result.json" "$result/causal/mce-result.json"
  if [[ $contract_rc -ne 0 || ! -s "$result/causal/contract.toml" || ! -s "$result/causal/mce-result.json" ]]; then
    python3 - "$id" "$contract_rc" "$trace_status" "$result/metrics.json" <<'PY'
import json,sys
json.dump({"workload":sys.argv[1],"state":"f5_f6_error","contract_rc":int(sys.argv[2]),"trace_status":sys.argv[3]},open(sys.argv[4],'w'),indent=2); open(sys.argv[4],'a').write('\n')
PY
    continue
  fi

  set +e
  "$FA" build-envelope "$result/causal/contract.toml" --output "$result/causal/envelope.json" --format json >"$raw/envelope.log" 2>&1
  envelope_rc=$?
  set -e
  if [[ $envelope_rc -ne 0 || ! -s "$result/causal/envelope.json" ]]; then
    python3 - "$id" "$envelope_rc" "$trace_status" "$result/metrics.json" <<'PY'
import json,sys
json.dump({"workload":sys.argv[1],"state":"f7_error","envelope_rc":int(sys.argv[2]),"trace_status":sys.argv[3]},open(sys.argv[4],'w'),indent=2); open(sys.argv[4],'a').write('\n')
PY
    continue
  fi

  heldout_spec="$ROOT/experiments/f8/matrix/$stem-heldout.toml"
  heldout_env="$ROOT/experiments/f8/matrix/$stem-heldout-environment.toml"
  heldout_attempted=false
  heldout_contract_status="not_applicable"
  heldout_trace_status="not_applicable"
  heldout_correctness="not_applicable"
  if [[ -f "$heldout_spec" && -f "$heldout_env" ]]; then
    heldout_attempted=true
    set +e
    "$FA" explore "$heldout_spec" --output "$raw/heldout" --workdir "$runtime" -- bash adapter/run.sh >"$raw/heldout.log" 2>&1
    heldout_rc=$?
    set -e
    if [[ -f "$raw/heldout/experiment.json" ]]; then
      copy_bundle_evidence "$raw/heldout" "$result/heldout"
      set +e
      hpair="$(pair_from_environment "$causal" "$heldout_env" "$raw/heldout" 2>/dev/null)"
      hpair_rc=$?
      set -e
      if [[ $hpair_rc -eq 0 ]]; then
        read -r hbase hbase_status htarget htarget_status <<<"$hpair"
        if [[ "$hbase_status" == "execution_succeeded" && "$htarget_status" == "execution_succeeded" ]]; then
          set +e
          "$FA" analyze-propagation --trace "$trace" \
            --left-root "$raw/heldout/variants/$hbase" \
            --right-root "$raw/heldout/variants/$htarget" \
            --format json --output "$result/heldout/trace.json" >"$raw/heldout-f4.log" 2>&1
          heldout_f4_rc=$?
          set -e
          if [[ $heldout_f4_rc -le 2 && -s "$result/heldout/trace.json" ]]; then
            heldout_trace_status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["trace_status"])' "$result/heldout/trace.json")"
          else
            heldout_trace_status="error"
          fi
        else
          heldout_trace_status="execution_unavailable_or_failed"
        fi
      else
        heldout_trace_status="pair_unresolved"
      fi
    else
      heldout_trace_status="experiment_error"
    fi

    set +e
    "$FA" check-contract "$result/causal/contract.toml" --environment "$heldout_env" \
      --format json --output "$result/heldout/contract-evaluation.json" >"$raw/heldout-contract.log" 2>&1
    check_rc=$?
    set -e
    if [[ $check_rc -le 2 && -s "$result/heldout/contract-evaluation.json" ]]; then
      heldout_contract_status="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["status"])' "$result/heldout/contract-evaluation.json")"
      if [[ "$heldout_contract_status" == "out_of_scope" ]]; then
        heldout_correctness="abstained"
      elif [[ "$heldout_trace_status" == "stable" && "$heldout_contract_status" == "satisfied" ]]; then
        heldout_correctness="correct"
      elif [[ "$heldout_trace_status" != "stable" && "$heldout_trace_status" != "error" && "$heldout_contract_status" == "violated" ]]; then
        heldout_correctness="correct"
      else
        heldout_correctness="incorrect"
      fi
    else
      heldout_contract_status="invalid_or_error"
      heldout_correctness="not_evaluable"
    fi
  fi

  python3 - "$id" "$explore_rc" "$baseline_id" "$full_id" "$trace_status" \
    "$heldout_attempted" "$heldout_contract_status" "$heldout_trace_status" "$heldout_correctness" \
    "$result/inference/experiment.json" "$result/causal/mce-result.json" "$result/causal/envelope.json" "$result/metrics.json" <<'PY'
import json,sys
id_,explore_rc,base,full,trace,hattempt,hcontract,htrace,hcorrect,expf,mcef,envf,out=sys.argv[1:]
exp=json.load(open(expf)); mce=json.load(open(mcef)); env=json.load(open(envf))
d={
 "workload":id_,"state":"full_chain_evaluable","explore_rc":int(explore_rc),
 "planned":exp.get("planned"),"succeeded":exp.get("succeeded"),"unavailable":exp.get("unavailable"),"failed":exp.get("failed"),
 "availability_rate":(exp.get("succeeded",0)/exp.get("planned",1)) if exp.get("planned") else None,
 "baseline_variant":base,"full_variant":full,"trace_status":trace,
 "first_observed_divergence":json.load(open(sys.argv[10].replace('experiment.json','trace.json'))).get('first_observed_divergence') if False else None,
 "causal_status":mce.get("status"),"confirmations":mce.get("confirmations"),"minimal_cardinality":mce.get("minimal_cardinality"),
 "minimal_sets":mce.get("minimal_sets",[]),"essential_factors":mce.get("essential_factors",[]),
 "target_signature":mce.get("target_signature"),
 "envelope_status":env.get("status"),"envelope_decided":env.get("decided_cells"),"envelope_total":env.get("total_cells"),
 "envelope_out_of_scope":env.get("out_of_scope_cells"),"boundary_count":env.get("boundary_count"),
 "heldout_attempted":hattempt.lower()=="true","heldout_contract_status":hcontract,"heldout_trace_status":htrace,"heldout_correctness":hcorrect,
}
tracefile=__import__('pathlib').Path(out).parent/'inference'/'trace.json'
if tracefile.exists():
    tr=json.load(open(tracefile)); d['first_observed_divergence']=tr.get('first_observed_divergence'); d['absorption_boundaries']=tr.get('absorption_boundaries',[])
json.dump(d,open(out,'w'),indent=2,sort_keys=True); open(out,'a').write('\n')
PY
  printf '%s: cadena F2-F7 materializada\n' "$id"
done

say "Construir agregado F8.5-F8.6"
python3 - "$RESULT_ROOT" "$RESULT_MANIFEST" <<'PY'
import json,sys
from pathlib import Path
root=Path(sys.argv[1]); out=Path(sys.argv[2])
items=[]
for p in sorted(root.glob('*/metrics.json')):
    items.append(json.loads(p.read_text()))
selected=['csvkit-realdata','jq-build-check','fd-build-completions','inih-meson-tests']
by={x.get('workload'):x for x in items}
missing=[x for x in selected if x not in by]
if missing: raise SystemExit(f'Faltan métricas de workloads: {missing}')
full=[x for x in items if x.get('state')=='full_chain_evaluable']
positive=[x for x in full if x.get('trace_status') not in (None,'stable')]
negative=[x for x in full if x.get('trace_status')=='stable']
held=[x for x in full if x.get('heldout_attempted')]
decided=[x for x in held if x.get('heldout_correctness') in ('correct','incorrect')]
correct=[x for x in decided if x.get('heldout_correctness')=='correct']
agg={
 'schema_version':1,'phase':'F8.5-F8.6','selected_workloads':4,'workloads':items,
 'full_chain_evaluable':len(full),'positive_observable_difference':len(positive),'no_observable_effect':len(negative),
 'heldout_attempted':len(held),'heldout_decided':len(decided),'heldout_correct':len(correct),
 'heldout_accuracy':(len(correct)/len(decided)) if decided else None,
 'heldout_abstentions':sum(1 for x in held if x.get('heldout_correctness')=='abstained'),
 'negative_results_retained':True,'posthoc_retuning_performed':False
}
out.write_text(json.dumps(agg,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print(json.dumps({k:agg[k] for k in ('full_chain_evaluable','positive_observable_difference','no_observable_effect','heldout_attempted','heldout_decided','heldout_abstentions')},sort_keys=True))
PY

cat > "$REPORT" <<EOF_REPORT
F8.5-F8.6 EXTERNAL VALIDATION EXECUTION REPORT

Rama: $BRANCH
Parent: $(git rev-parse --short HEAD)
Freeze F8.4 esperado: ae16104
Driver de campaña: experiments/f8/results/execution_driver.sh
Driver SHA-256: $DRIVER_SHA256
Política de ejecución offline: PIP_NO_INDEX=1, CARGO_NET_OFFLINE=true
Workloads seleccionados: 4
Ejecución EnvMorph posterior al freeze: sí
Matriz reajustada después de observar resultados: no
Resultados negativos descartados: no
Held-out ejecutado cuando estaba preregistrado: sí

Resumen machine-readable:
$(cat "$RESULT_MANIFEST")

Evidencia pesada local:
$RAW_ROOT

Evidencia compacta versionable:
$RESULT_ROOT

Siguiente gate: F8.7-F8.10, agregado, amenazas, posicionamiento y cierre condicionado a evidencia suficiente.
EOF_REPORT

find experiments/f8/results -type f -print0 | LC_ALL=C sort -z | xargs -0 -r sha256sum > "$RESULT_HASH"
sha256sum "$REPORT" >> "$RESULT_HASH"
sha256sum -c "$RESULT_HASH"

for h in provenance/F8_PROTOCOL_FILES.sha256 provenance/F8_SELECTION_FILES.sha256 provenance/F8_CORPUS_FILES.sha256 provenance/F8_ADAPTER_FILES.sha256 provenance/F8_RUNTIME_FILES.sha256 provenance/F8_MATRIX_FILES.sha256; do sha256sum -c "$h" >/dev/null; done
for id in "${IDS[@]}"; do (cd "workloads/external/$id/source" && sha256sum -c "$ROOT/provenance/f8/$id.files.sha256" >/dev/null); done
git diff --exit-code cad211e -- workloads/literature-pipeline/source
AUTHORED_RESULT_FILES=(
  "$RESULT_MANIFEST"
  "$DRIVER_COPY"
  "$REPORT"
  "$RESULT_HASH"
)
while IFS= read -r p; do
  AUTHORED_RESULT_FILES+=("$p")
done < <(find "$RESULT_ROOT" -name metrics.json -type f -print | LC_ALL=C sort)
git diff --check -- "${AUTHORED_RESULT_FILES[@]}"

say "Crear commit F8.5-F8.6"
git add experiments/f8/results provenance/F8_RESULT_FILES.sha256 audits/F8_5_6_EXECUTION_REPORT.txt
git diff --cached --quiet && die "No hay cambios staged"
git commit -m "$COMMIT_MESSAGE"
COMMITTED=1
[[ -z "$(git status --porcelain)" ]] || die "Working tree no quedó limpio"

printf '\nF8.5-F8.6 PASS\n'
printf 'Rama: %s\n' "$(git branch --show-current)"
printf 'HEAD: %s\n' "$(git rev-parse --short HEAD)"
printf 'Commit: %s\n' "$(git log -1 --oneline)"
python3 - "$RESULT_MANIFEST" <<'PY'
import json,sys
x=json.load(open(sys.argv[1]))
print(f"Full chain evaluable: {x['full_chain_evaluable']}/4")
print(f"Diferencia observable: {x['positive_observable_difference']}")
print(f"Sin efecto observable: {x['no_observable_effect']}")
print(f"Held-out intentados: {x['heldout_attempted']}")
print(f"Held-out abstenciones: {x['heldout_abstentions']}")
PY
printf 'Working tree: CLEAN\n'
printf 'Siguiente gate: F8.7-F8.10 cierre condicionado.\n'
printf 'No se hizo push, tag ni release.\n'
