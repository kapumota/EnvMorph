#!/bin/bash
# Mutation experiment on benchmarks/literature-pipeline.
#
# For every mutation M, run the baseline pipeline and the mutated pipeline,
# run `flowattest diff`, and compare the localized stage with the stage that
# the mutation is expected to change first. Results go to results/rq2_localization.csv.
#
# Usage  experiments/run_mutations.sh [REPEATS]   (default 3)

set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${FLOWATTEST_BIN:-$ROOT/target/release/flowattest}"
WL="$ROOT/benchmarks/literature-pipeline"
REPEATS="${1:-3}"
RES="$ROOT/results"; mkdir -p "$RES"
WORK="$(mktemp -d)"; trap 'rm -rf "$WORK"' EXIT
export FLOWATTEST_HOME="$WORK/store"

BASE_ARGS=(--top-n 20 --min-year 2020)
# The workload stamps notes with systime(); pin it (reproducible-builds convention)
# except in case M13 where wall-clock nondeterminism is the injected fault.
export SOURCE_DATE_EPOCH=1700000000
ORIG_PATH="$PATH"

# --- workload copies -------------------------------------------------------
fresh_workload() { # fresh_workload DIR  (copy of the benchmark, own scripts/ and examples/)
  rm -rf "$1"; mkdir -p "$1"; cp -r "$WL/scripts" "$WL/examples" "$WL/pipeline_flowattest.sh" "$1/"
}

run_pipeline() { # run_pipeline WORKDIR OUTDIR [EXTRA pipeline args...]  -> prints run id
  local wd="$1" out="$2"; shift 2
  FLOWATTEST_BIN="$FA" bash "$wd/pipeline_flowattest.sh" \
    --elicit "$wd/examples/sample_elicit.csv" --researchrabbit "$wd/examples/sample_researchrabbit.csv" \
    --scite "$wd/examples/sample_scite.csv" --output "$out" "${BASE_ARGS[@]}" "$@" >"$out.log" 2>&1
  cat "$FLOWATTEST_HOME/CURRENT"
}

# --- mutations: name | expected first stage | family | description ---------
# Each mutation is a function mutate_<id> WORKDIR that edits the copy in place
# and may export PATH / env for the mutated run. Baseline uses the pristine copy.
declare -A EXPECT FAMILY DESC
add() { EXPECT[$1]="$2"; FAMILY[$1]="$3"; DESC[$1]="$4"; }

add M01 elicit_clean       DATA  "append a paper row to the Elicit CSV"
add M02 -                  CTRL  "CRLF line endings in the Elicit CSV (control, expected no observable effect)"
add M03 elicit_clean       DATA  "quoted comma inside a title"
add M04 merge_sources      DATA  "alter a ResearchRabbit row"
add M05 scite_enrich       DATA  "alter the supporting count of a post-2020 paper in the Scite export"
add M06 merge_sources      DATA  "drop a ResearchRabbit row"
add M07 semantic_dedup     CODE  "lower the dedup similarity threshold from 0.85 to 0.10"
add M08 matrix_generator   CODE  "edit the matrix script header text"
add M09 reading_queue      CODE  "edit the reading queue script"
add M10 bias_audit         CODE  "edit the bias audit script"
add M11 merge_sources      ENV   "gawk instead of mawk (executable swap)"
add M12 -                  CTRL  "LC_ALL=C for the mutated run (control, expected no observable effect)"
add M13 zettel_generator   NOND  "wall clock: SOURCE_DATE_EPOCH unset, runs 1.1 s apart (systime in notes)"

mutate_M01() { printf '"Extra Paper One","Some Author et al.","10.1000/extra.1",2023,"An added abstract."\n' >> "$1/examples/sample_elicit.csv"; }
mutate_M02() { sed -i 's/$/\r/' "$1/examples/sample_elicit.csv"; }
mutate_M03() { sed -i '2s/"Attention Is All You Need"/"Attention, Is All You Need"/' "$1/examples/sample_elicit.csv"; }
mutate_M04() { sed -i '2s/[0-9][0-9][0-9][0-9]\(,\|$\)/1999\1/' "$1/examples/sample_researchrabbit.csv"; }
mutate_M05() { sed -i '6s/,198,/,12,/' "$1/examples/sample_scite.csv"; }
mutate_M06() { sed -i '2d' "$1/examples/sample_researchrabbit.csv"; }
mutate_M07() { sed -i 's/THRESHOLD = 0.85/THRESHOLD = 0.10/' "$1/scripts/semantic_dedup.awk"; }
mutate_M08() { printf '\nBEGIN { print "<!-- mutated -->" }\n' >> "$1/scripts/matrix_generator.awk"; }
mutate_M09() { printf '\nBEGIN { print "<!-- mutated -->" }\n' >> "$1/scripts/reading_queue.awk"; }
mutate_M10() { printf '\nBEGIN { print "<!-- mutated -->" }\n' >> "$1/scripts/bias_audit.awk"; }
mutate_M11() { :; }
mutate_M12() { :; }
mutate_M13() { :; }

GAWK_SHIM=""
if command -v gawk >/dev/null 2>&1; then
  GAWK_SHIM="$WORK/shim"; mkdir -p "$GAWK_SHIM"; ln -sf "$(command -v gawk)" "$GAWK_SHIM/awk"
elif [[ -x /tmp/shim-gawk/awk ]]; then
  GAWK_SHIM=/tmp/shim-gawk
fi

CSV="$RES/rq2_localization.csv"
echo "mutation,family,repeat,observable,expected_stage,localized_stage,correct,stages_total,candidates,root_causes,confounded,propagated,absorbed" > "$CSV"

mut_ids=$(printf '%s\n' "${!EXPECT[@]}" | sort)
for id in $mut_ids; do
  [[ "$id" == "M11" && -z "$GAWK_SHIM" ]] && { echo "skip $id (no gawk)"; continue; }
  for r in $(seq 1 "$REPEATS"); do
    BASE="$WORK/base-$id-$r"; MUT="$WORK/mut-$id-$r"
    fresh_workload "$BASE"; fresh_workload "$MUT"
    export PATH="$ORIG_PATH"; unset LC_ALL
    [[ "$id" == "M13" ]] && unset SOURCE_DATE_EPOCH || export SOURCE_DATE_EPOCH=1700000000
    A="$(run_pipeline "$BASE" "$WORK/outA-$id-$r")"
    [[ "$id" == "M13" ]] && sleep 1.1
    "mutate_$id" "$MUT"
    case "$id" in
      M11) export PATH="$GAWK_SHIM:$ORIG_PATH" ;;
      M12) export LC_ALL=C ;;
    esac
    B="$(run_pipeline "$MUT" "$WORK/outB-$id-$r")"
    export PATH="$ORIG_PATH"; unset LC_ALL; export SOURCE_DATE_EPOCH=1700000000
    js="$("$FA" diff "$A" "$B" --json | tr -d ' \n')"
    stages=$(grep -o '"stage_count":[0-9]*' <<<"$js" | cut -d: -f2)
    roots=$(grep -o '"root_count":[0-9]*' <<<"$js" | cut -d: -f2)
    conf=$(grep -o '"confounded_count":[0-9]*' <<<"$js" | cut -d: -f2)
    prop=$(grep -o '"propagated_count":[0-9]*' <<<"$js" | cut -d: -f2)
    abs=$(grep -o '"absorbed_count":[0-9]*' <<<"$js" | cut -d: -f2)
    loc=$(grep -o '"first_divergence":{"position":[0-9]*,"name":"[^"]*"' <<<"$js" | sed 's/.*"name":"//; s/"$//')
    [[ -z "$loc" ]] && loc="none"
    # Independent oracle: byte comparison of the two output trees (no provenance logic).
    obs=0; diff -rq "$WORK/outA-$id-$r" "$WORK/outB-$id-$r" >/dev/null 2>&1 || obs=1
    exp="${EXPECT[$id]}"
    if [[ "$obs" -eq 0 ]]; then exp="none"; fi
    if [[ "$obs" -eq 0 && "$exp" != "none" ]]; then echo "warning $id has no observable effect"; fi
    if [[ "${EXPECT[$id]}" != "-" && "$obs" -eq 0 ]]; then echo "warning: $id declared effective but outputs are identical" >&2; fi
    ok=0; [[ "$loc" == "$exp" ]] && ok=1
    cand=$(( ${roots:-0} + ${conf:-0} ))
    echo "$id,${FAMILY[$id]},$r,$obs,$exp,$loc,$ok,${stages:-0},$cand,${roots:-0},${conf:-0},${prop:-0},${abs:-0}" >> "$CSV"
  done
  printf '%s %-5s %s\n' "$id" "${FAMILY[$id]}" "${DESC[$id]}"
done

echo; echo "Results written to $CSV"
awk -F, 'NR>1 && $4==1 { n++; c+=$7; cand+=$9; tot+=$8; by[$2]++; byc[$2]+=$7 }
  NR>1 && $4==0 { m++; fp+=($6!="none") }
  END { printf "effective mutations     %d runs, localization accuracy %d/%d = %.1f%%\n", n, c, n, (n?100*c/n:0);
        printf "mean candidate stages   %.2f of %.1f total\n", (n?cand/n:0), (n?tot/n:0);
        for (f in by) printf "  %-5s %d/%d\n", f, byc[f], by[f];
        printf "controls (no effect)    %d runs, false divergences %d\n", m, fp }' "$CSV"
