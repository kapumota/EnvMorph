#!/bin/bash
# literature-pipeline v5.0
# Pipeline completo: Elicit -> Semantic Dedup -> Year Filter -> ResearchRabbit ->
#   Scite -> Core Selection -> Matrix -> Provenance -> Reading Queue -> Zettelkasten ->
#   Zombie Detector -> Narrative -> Contradiction Map -> Serendipity ->
#   Bias Audit -> Zettel Links -> LaTeX Export
#
# Uso:
#   ./pipeline.sh --elicit data/elicit.csv --researchrabbit data/rr.csv \
#                 --scite data/scite.csv --output out/ --top-n 20 --min-year 2020

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AWK_DIR="$SCRIPT_DIR/scripts"

# Defaults
ELICIT=""
RESEARCHRABBIT=""
SCITE=""
OUTDIR="output"
TOP_N=20
MIN_YEAR=0

# Parsear argumentos
while [[ $# -gt 0 ]]; do
  case $1 in
    --elicit) ELICIT="$2"; shift 2 ;;
    --researchrabbit) RESEARCHRABBIT="$2"; shift 2 ;;
    --scite) SCITE="$2"; shift 2 ;;
    --output) OUTDIR="$2"; shift 2 ;;
    --top-n) TOP_N="$2"; shift 2 ;;
    --min-year) MIN_YEAR="$2"; shift 2 ;;
    -h|--help)
      echo "Uso: $0 --elicit FILE --researchrabbit FILE --scite FILE [--output DIR] [--top-n N] [--min-year YYYY]"
      echo ""
      echo "Genera 12 artefactos:"
      echo "  1. RELATED_WORK_MATRIX.md    Tabla Markdown de papers core"
      echo "  2. READING_QUEUE.md          Cola de lectura topologica"
      echo "  3. provenance.dot            Grafo de procedencia (Graphviz)"
      echo "  4. zettel/                   Notas atomicas para Obsidian"
      echo "  5. ZOMBIE_REPORT.md          Deteccion de papers sobrevalorados"
      echo "  6. NARRATIVE.md              Borrador de seccion Related Work"
      echo "  7. CONTRADICTION_MAP.md      Mapa de disputas academicas"
      echo "  8. SERENDIPITY_REPORT.md     Score de sorpresa interdisciplinaria"
      echo "  9. BIAS_AUDIT.md             Auditoria de sesgo temporal/fuente"
      echo "  10. zettel/ (con links)      Zettels con links bidireccionales reales"
      echo "  11. RELATED_WORK.tex         Documento LaTeX con bibliografia"
      echo "  12. literature_pipeline.bib  Archivo BibTeX con entradas"
      exit 0
      ;;
    *) echo "Opcion desconocida: $1"; exit 1 ;;
  esac
done

# Validar requisitos
command -v awk >/dev/null 2>&1 || { echo "Error: awk no instalado"; exit 1; }

# Crear directorio de salida
mkdir -p "$OUTDIR"

echo "========================================"
echo "  Literature Pipeline v5.0"
echo "  12 artefactos novedosos"
echo "========================================"
if [[ "$MIN_YEAR" -gt 0 ]]; then
  echo "  Filtro: papers desde $MIN_YEAR"
fi
echo ""

# ============================================================
# STAGE 1: Elicit clean
# ============================================================
if [[ -n "$ELICIT" && -f "$ELICIT" ]]; then
  echo "[1/15] Cleaning Elicit export..."
  awk -f "$AWK_DIR/elicit_clean.awk" "$ELICIT" > "$OUTDIR/stage1_elicit.tsv"
  echo "      -> $(($(wc -l < "$OUTDIR/stage1_elicit.tsv") - 1)) papers"
else
  echo "[1/15] Skipping Elicit (no file provided)"
  touch "$OUTDIR/stage1_elicit.tsv"
fi

# ============================================================
# STAGE 2: Semantic deduplication
# ============================================================
if [[ -s "$OUTDIR/stage1_elicit.tsv" ]]; then
  echo "[2/15] Semantic deduplication..."
  awk -f "$AWK_DIR/semantic_dedup.awk" "$OUTDIR/stage1_elicit.tsv" > "$OUTDIR/stage1_deduped.tsv" 2>"$OUTDIR/dedup.log"
  dedup_count=$(grep -oP '\d+' "$OUTDIR/dedup.log" | head -1 || echo "0")
  echo "      -> $(($(wc -l < "$OUTDIR/stage1_deduped.tsv") - 1)) unique papers (merged $dedup_count duplicates)"
else
  echo "[2/15] Skipping deduplication (no Elicit data)"
  touch "$OUTDIR/stage1_deduped.tsv"
fi

# ============================================================
# STAGE 3: Merge with ResearchRabbit
# ============================================================
if [[ -n "$RESEARCHRABBIT" && -f "$RESEARCHRABBIT" ]]; then
  echo "[3/15] Merging with ResearchRabbit..."
  if [[ -s "$OUTDIR/stage1_deduped.tsv" ]]; then
    awk -f "$AWK_DIR/merge_sources.awk" "$OUTDIR/stage1_deduped.tsv" "$RESEARCHRABBIT" > "$OUTDIR/stage2_merged.tsv"
  else
    awk -f "$AWK_DIR/merge_sources.awk" /dev/null "$RESEARCHRABBIT" > "$OUTDIR/stage2_merged.tsv"
  fi
  echo "      -> $(($(wc -l < "$OUTDIR/stage2_merged.tsv") - 1)) unique papers"
else
  echo "[3/15] Skipping ResearchRabbit (no file provided)"
  cp "$OUTDIR/stage1_deduped.tsv" "$OUTDIR/stage2_merged.tsv" 2>/dev/null || true
fi

# ============================================================
# STAGE 4: Year filter (NEW v5.0)
# ============================================================
if [[ "$MIN_YEAR" -gt 0 && -s "$OUTDIR/stage2_merged.tsv" ]]; then
  echo "[4/15] Filtering papers >= $MIN_YEAR..."
  awk -v min_year="$MIN_YEAR" -f "$AWK_DIR/year_filter.awk" "$OUTDIR/stage2_merged.tsv" > "$OUTDIR/stage2_filtered.tsv"
  before=$(($(wc -l < "$OUTDIR/stage2_merged.tsv") - 1))
  after=$(($(wc -l < "$OUTDIR/stage2_filtered.tsv") - 1))
  echo "      -> $after papers (filtered $((before - after)) older papers)"
  cp "$OUTDIR/stage2_filtered.tsv" "$OUTDIR/stage2_merged.tsv"
else
  echo "[4/15] Skipping year filter (no --min-year specified)"
fi

# ============================================================
# STAGE 5: Enrich with Scite
# ============================================================
if [[ -n "$SCITE" && -f "$SCITE" && -s "$OUTDIR/stage2_merged.tsv" ]]; then
  echo "[5/15] Enriching with Scite..."
  awk -f "$AWK_DIR/scite_enrich.awk" "$SCITE" "$OUTDIR/stage2_merged.tsv" > "$OUTDIR/stage3_scored.tsv"
  echo "      -> enriched with citation contexts"
else
  echo "[5/15] Skipping Scite (no file provided or no papers to enrich)"
  cp "$OUTDIR/stage2_merged.tsv" "$OUTDIR/stage3_scored.tsv" 2>/dev/null || true
fi

# ============================================================
# STAGE 6: Select core papers
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[6/15] Selecting top $TOP_N core papers..."
  {
    head -n1 "$OUTDIR/stage3_scored.tsv"
    tail -n +2 "$OUTDIR/stage3_scored.tsv" | sort -t$'\t' -k8,8nr | head -n "$TOP_N"
  } > "$OUTDIR/stage4_core.tsv"
  echo "      -> $(($(wc -l < "$OUTDIR/stage4_core.tsv") - 1)) papers selected"
else
  echo "[6/15] No papers to select"
  touch "$OUTDIR/stage4_core.tsv"
fi

# ============================================================
# STAGE 7: Generate Markdown matrix
# ============================================================
if [[ -s "$OUTDIR/stage4_core.tsv" ]]; then
  echo "[7/15] Generating Markdown matrix..."
  awk -f "$AWK_DIR/matrix_generator.awk" "$OUTDIR/stage4_core.tsv" > "$OUTDIR/RELATED_WORK_MATRIX.md"
else
  echo "[7/15] Nothing to generate for matrix"
fi

# ============================================================
# STAGE 8: Generate Provenance Graph
# ============================================================
if [[ -s "$OUTDIR/stage2_merged.tsv" ]]; then
  echo "[8/15] Generating provenance graph..."
  awk -f "$AWK_DIR/provenance_graph.awk" "$OUTDIR/stage2_merged.tsv" > "$OUTDIR/provenance.dot"
  echo "      -> provenance.dot"
else
  echo "[8/15] Skipping provenance graph"
fi

# ============================================================
# STAGE 9: Generate Reading Queue
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[9/15] Generating reading queue..."
  awk -f "$AWK_DIR/reading_queue.awk" "$OUTDIR/stage3_scored.tsv" > "$OUTDIR/READING_QUEUE.md"
  echo "      -> READING_QUEUE.md"
else
  echo "[9/15] Skipping reading queue"
fi

# ============================================================
# STAGE 10: Generate Zettelkasten Notes
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[10/15] Generating Zettelkasten notes..."
  cd "$OUTDIR"
  awk -f "$AWK_DIR/zettel_generator.awk" "stage3_scored.tsv" > zettel_index.md
  cd "$SCRIPT_DIR"
  echo "      -> zettel/ directory with $(ls "$OUTDIR/zettel/" 2>/dev/null | wc -l) notes"
else
  echo "[10/15] Skipping Zettelkasten"
fi

# ============================================================
# STAGE 11: Zombie Paper Detector
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[11/15] Running zombie paper detector..."
  awk -f "$AWK_DIR/zombie_detector.awk" "$OUTDIR/stage3_scored.tsv" > "$OUTDIR/ZOMBIE_REPORT.md"
  echo "      -> ZOMBIE_REPORT.md"
else
  echo "[11/15] Skipping zombie detector"
fi

# ============================================================
# STAGE 12: Narrative Generator
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[12/15] Generating narrative draft..."
  awk -f "$AWK_DIR/narrative_generator.awk" "$OUTDIR/stage3_scored.tsv" > "$OUTDIR/NARRATIVE.md"
  echo "      -> NARRATIVE.md"
fi

# ============================================================
# STAGE 13: Contradiction Map
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[13/15] Generating contradiction map..."
  awk -f "$AWK_DIR/contradiction_map.awk" "$OUTDIR/stage3_scored.tsv" > "$OUTDIR/CONTRADICTION_MAP.md"
  echo "      -> CONTRADICTION_MAP.md"
else
  echo "[13/15] Skipping contradiction map"
fi

# ============================================================
# STAGE 14: Serendipity Report
# ============================================================
if [[ -s "$OUTDIR/stage2_merged.tsv" ]]; then
  echo "[14/15] Calculating serendipity scores..."
  awk -f "$AWK_DIR/serendipity.awk" "$OUTDIR/stage2_merged.tsv" > "$OUTDIR/SERENDIPITY_REPORT.md"
  echo "      -> SERENDIPITY_REPORT.md"
else
  echo "[14/15] Skipping serendipity report"
fi

# ============================================================
# STAGE 15: Bias Audit
# ============================================================
if [[ -s "$OUTDIR/stage2_merged.tsv" ]]; then
  echo "[15/15] Running bias audit..."
  awk -f "$AWK_DIR/bias_audit.awk" "$OUTDIR/stage2_merged.tsv" > "$OUTDIR/BIAS_AUDIT.md"
  echo "      -> BIAS_AUDIT.md"
else
  echo "[15/15] Skipping bias audit"
fi

# ============================================================
# STAGE 16: Zettel Links (post-processing)
# ============================================================
if [[ -d "$OUTDIR/zettel" && -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[BONUS] Adding bidirectional links to zettels..."
  cd "$OUTDIR"
  awk -f "$AWK_DIR/zettel_links.awk" "stage3_scored.tsv" > zettel_links.log 2>&1
  cd "$SCRIPT_DIR"
  echo "      -> zettels now have real [[links]]"
fi

# ============================================================
# STAGE 17: LaTeX Export (NEW v5.0)
# ============================================================
if [[ -s "$OUTDIR/stage3_scored.tsv" ]]; then
  echo "[BONUS] Generating LaTeX export..."
  cd "$OUTDIR"
  awk -f "$AWK_DIR/latex_export.awk" "stage3_scored.tsv" > RELATED_WORK.tex
  cd "$SCRIPT_DIR"
  echo "      -> RELATED_WORK.tex + literature_pipeline.bib"
  echo "      -> Compilar: pdflatex RELATED_WORK.tex && bibtex RELATED_WORK && pdflatex RELATED_WORK.tex"
fi

echo ""
echo "========================================"
echo "  Done! Outputs in: $OUTDIR/"
echo "========================================"
echo ""
echo "Generated files:"
ls -la "$OUTDIR/" 2>/dev/null | tail -n +4
echo ""

# Git commit automatico (opcional)
if git rev-parse --git-dir >/dev/null 2>&1; then
  git checkout -b literature 2>/dev/null || git checkout literature 2>/dev/null
  git add "$OUTDIR/" 2>/dev/null || true
  git commit -m "literature: $(date +%Y-%m-%d) | $(($(wc -l < "$OUTDIR/stage4_core.tsv" 2>/dev/null || echo 0) - 1)) core papers" 2>/dev/null || true
  git checkout - 2>/dev/null || true
  echo "📚 Committed to 'literature' branch"
fi
