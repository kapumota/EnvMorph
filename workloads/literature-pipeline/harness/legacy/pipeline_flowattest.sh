#!/bin/bash
# pipeline_flowattest.sh
#
# literature-pipeline v5.0 with every stage wrapped by `flowattest stage`.
# Same stages, same commands and same outputs as pipeline.sh. Differences
#   * every stage is recorded (command, executable hash, inputs, outputs)
#   * the untracked `cp` in stage 4 is an explicit stage
#   * stage 6 lives in scripts/select_core.sh so it is a single command
#   * the automatic `git commit` at the end is NOT performed (side effect
#     outside the pipeline; it would make runs depend on repository state)
#
# Usage
#   ./pipeline_flowattest.sh --elicit F --researchrabbit F --scite F \
#       --output DIR [--top-n N] [--min-year YYYY]
#
# Environment
#   FLOWATTEST_BIN   path to the flowattest binary (default: flowattest)
#   FLOWATTEST_HOME  store location (default: ./.flowattest)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AWK_DIR="$SCRIPT_DIR/scripts"
FA="${FLOWATTEST_BIN:-flowattest}"

ELICIT=""; RESEARCHRABBIT=""; SCITE=""; OUTDIR="output"; TOP_N=20; MIN_YEAR=0

while [[ $# -gt 0 ]]; do
  case $1 in
    --elicit) ELICIT="$2"; shift 2 ;;
    --researchrabbit) RESEARCHRABBIT="$2"; shift 2 ;;
    --scite) SCITE="$2"; shift 2 ;;
    --output) OUTDIR="$2"; shift 2 ;;
    --top-n) TOP_N="$2"; shift 2 ;;
    --min-year) MIN_YEAR="$2"; shift 2 ;;
    -h|--help) sed -n '2,22p' "$0"; exit 0 ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

command -v awk >/dev/null 2>&1 || { echo "awk not installed" >&2; exit 1; }
mkdir -p "$OUTDIR"
OUTDIR="$(cd "$OUTDIR" && pwd)"
abs() { [[ -n "$1" && -e "$1" ]] && echo "$(cd "$(dirname "$1")" && pwd)/$(basename "$1")" || echo "$1"; }
ELICIT="$(abs "$ELICIT")"; RESEARCHRABBIT="$(abs "$RESEARCHRABBIT")"; SCITE="$(abs "$SCITE")"

# Logical names make runs comparable when OUTDIR or the checkout moves.
MAPS=(--map "$OUTDIR=@out" --map "$SCRIPT_DIR=@src")
n=0
for f in "$ELICIT" "$RESEARCHRABBIT" "$SCITE"; do
  [[ -n "$f" && -e "$f" ]] || continue
  d="$(dirname "$f")"
  case " ${seen_dirs:-} " in *" $d "*) continue ;; esac
  seen_dirs="${seen_dirs:-} $d"
  n=$((n + 1))
  MAPS+=(--map "$d=@in$n")
done

RUN_ID="$("$FA" begin "${MAPS[@]}")"
export FLOWATTEST_RUN="$RUN_ID"
fa() { "$FA" stage --quiet "$@"; }

O="$OUTDIR"
echo "FlowAttest $RUN_ID recording to ${FLOWATTEST_HOME:-.flowattest}"

# 1 elicit_clean
if [[ -n "$ELICIT" && -f "$ELICIT" ]]; then
  fa --name elicit_clean --input "$ELICIT" --stdout "$O/stage1_elicit.tsv" \
     -- awk -f "$AWK_DIR/elicit_clean.awk" "$ELICIT"
else
  fa --name elicit_skip --output "$O/stage1_elicit.tsv" -- touch "$O/stage1_elicit.tsv"
fi

# 2 semantic_dedup
if [[ -s "$O/stage1_elicit.tsv" ]]; then
  fa --name semantic_dedup --input "$O/stage1_elicit.tsv" \
     --stdout "$O/stage1_deduped.tsv" --stderr "$O/dedup.log" \
     -- awk -f "$AWK_DIR/semantic_dedup.awk" "$O/stage1_elicit.tsv"
else
  fa --name dedup_skip --output "$O/stage1_deduped.tsv" -- touch "$O/stage1_deduped.tsv"
fi

# 3 merge_sources
if [[ -n "$RESEARCHRABBIT" && -f "$RESEARCHRABBIT" ]]; then
  if [[ -s "$O/stage1_deduped.tsv" ]]; then
    fa --name merge_sources --input "$O/stage1_deduped.tsv" --input "$RESEARCHRABBIT" \
       --stdout "$O/stage2_merged.tsv" \
       -- awk -f "$AWK_DIR/merge_sources.awk" "$O/stage1_deduped.tsv" "$RESEARCHRABBIT"
  else
    fa --name merge_sources --input "$RESEARCHRABBIT" --stdout "$O/stage2_merged.tsv" \
       -- awk -f "$AWK_DIR/merge_sources.awk" /dev/null "$RESEARCHRABBIT"
  fi
else
  fa --name merge_skip --input "$O/stage1_deduped.tsv" --output "$O/stage2_merged.tsv" \
     -- cp "$O/stage1_deduped.tsv" "$O/stage2_merged.tsv"
fi

# 4 year_filter (+ the promotion that pipeline.sh does with an untracked cp)
if [[ "$MIN_YEAR" -gt 0 && -s "$O/stage2_merged.tsv" ]]; then
  fa --name year_filter --input "$O/stage2_merged.tsv" --stdout "$O/stage2_filtered.tsv" \
     -- awk -v min_year="$MIN_YEAR" -f "$AWK_DIR/year_filter.awk" "$O/stage2_merged.tsv"
  fa --name promote_filtered --input "$O/stage2_filtered.tsv" --output "$O/stage2_merged.tsv" \
     -- cp "$O/stage2_filtered.tsv" "$O/stage2_merged.tsv"
fi

# 5 scite_enrich
if [[ -n "$SCITE" && -f "$SCITE" && -s "$O/stage2_merged.tsv" ]]; then
  fa --name scite_enrich --input "$SCITE" --input "$O/stage2_merged.tsv" \
     --stdout "$O/stage3_scored.tsv" \
     -- awk -f "$AWK_DIR/scite_enrich.awk" "$SCITE" "$O/stage2_merged.tsv"
else
  fa --name scite_skip --input "$O/stage2_merged.tsv" --output "$O/stage3_scored.tsv" \
     -- cp "$O/stage2_merged.tsv" "$O/stage3_scored.tsv"
fi

# 6 select_core
if [[ -s "$O/stage3_scored.tsv" ]]; then
  fa --name select_core --input "$O/stage3_scored.tsv" --stdout "$O/stage4_core.tsv" \
     -- bash "$AWK_DIR/select_core.sh" "$O/stage3_scored.tsv" "$TOP_N"
else
  fa --name select_skip --output "$O/stage4_core.tsv" -- touch "$O/stage4_core.tsv"
fi

# 7 matrix_generator
[[ -s "$O/stage4_core.tsv" ]] && \
  fa --name matrix_generator --input "$O/stage4_core.tsv" --stdout "$O/RELATED_WORK_MATRIX.md" \
     -- awk -f "$AWK_DIR/matrix_generator.awk" "$O/stage4_core.tsv"

# 8 provenance_graph
[[ -s "$O/stage2_merged.tsv" ]] && \
  fa --name provenance_graph --input "$O/stage2_merged.tsv" --stdout "$O/provenance.dot" \
     -- awk -f "$AWK_DIR/provenance_graph.awk" "$O/stage2_merged.tsv"

# 9 reading_queue
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name reading_queue --input "$O/stage3_scored.tsv" --stdout "$O/READING_QUEUE.md" \
     -- awk -f "$AWK_DIR/reading_queue.awk" "$O/stage3_scored.tsv"

# 10 zettel_generator (writes zettel/ relative to its working directory)
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name zettel_generator --cwd "$O" --input stage3_scored.tsv \
     --output zettel --stdout zettel_index.md \
     -- awk -f "$AWK_DIR/zettel_generator.awk" stage3_scored.tsv

# 11 zombie_detector
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name zombie_detector --input "$O/stage3_scored.tsv" --stdout "$O/ZOMBIE_REPORT.md" \
     -- awk -f "$AWK_DIR/zombie_detector.awk" "$O/stage3_scored.tsv"

# 12 narrative_generator
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name narrative_generator --input "$O/stage3_scored.tsv" --stdout "$O/NARRATIVE.md" \
     -- awk -f "$AWK_DIR/narrative_generator.awk" "$O/stage3_scored.tsv"

# 13 contradiction_map
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name contradiction_map --input "$O/stage3_scored.tsv" --stdout "$O/CONTRADICTION_MAP.md" \
     -- awk -f "$AWK_DIR/contradiction_map.awk" "$O/stage3_scored.tsv"

# 14 serendipity
[[ -s "$O/stage2_merged.tsv" ]] && \
  fa --name serendipity --input "$O/stage2_merged.tsv" --stdout "$O/SERENDIPITY_REPORT.md" \
     -- awk -f "$AWK_DIR/serendipity.awk" "$O/stage2_merged.tsv"

# 15 bias_audit
[[ -s "$O/stage2_merged.tsv" ]] && \
  fa --name bias_audit --input "$O/stage2_merged.tsv" --stdout "$O/BIAS_AUDIT.md" \
     -- awk -f "$AWK_DIR/bias_audit.awk" "$O/stage2_merged.tsv"

# 16 zettel_links (rewrites zettel/ in place)
[[ -d "$O/zettel" && -s "$O/stage3_scored.tsv" ]] && \
  fa --name zettel_links --cwd "$O" --input stage3_scored.tsv --input zettel \
     --output zettel --stdout zettel_links.log --stderr zettel_links.log \
     -- awk -f "$AWK_DIR/zettel_links.awk" stage3_scored.tsv

# 17 latex_export (also writes literature_pipeline.bib as a side effect)
[[ -s "$O/stage3_scored.tsv" ]] && \
  fa --name latex_export --cwd "$O" --input stage3_scored.tsv \
     --output literature_pipeline.bib --stdout RELATED_WORK.tex \
     -- awk -f "$AWK_DIR/latex_export.awk" stage3_scored.tsv

"$FA" end
echo "Done. Outputs in $OUTDIR"
