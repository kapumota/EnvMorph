#!/bin/bash
# tests/test_pipeline.sh
# Tests de integracion para literature-pipeline v6.0

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMPDIR=$(mktemp -d)
trap "rm -rf $TMPDIR" EXIT

echo "Running integration tests for literature-pipeline v6.0..."

# Test 1: Pipeline completo
echo "[TEST 1] Full pipeline with sample data..."
bash "$SCRIPT_DIR/pipeline.sh" \
  --elicit "$SCRIPT_DIR/examples/sample_elicit.csv" \
  --researchrabbit "$SCRIPT_DIR/examples/sample_researchrabbit.csv" \
  --scite "$SCRIPT_DIR/examples/sample_scite.csv" \
  --output "$TMPDIR/out" \
  --top-n 10

if [[ ! -f "$TMPDIR/out/RELATED_WORK_MATRIX.md" ]]; then
  echo "FAIL: RELATED_WORK_MATRIX.md not generated"
  exit 1
fi
echo "[TEST 1] PASS"

# Test 2: Semantic deduplication
echo "[TEST 2] Semantic deduplication..."
if [[ ! -f "$TMPDIR/out/stage1_deduped.tsv" ]]; then
  echo "FAIL: stage1_deduped.tsv not generated"
  exit 1
fi
echo "[TEST 2] PASS"

# Test 3: DOI deduplication
echo "[TEST 3] DOI deduplication..."
count=$(grep -c "10.48550/arxiv.1706.03762" "$TMPDIR/out/stage2_merged.tsv" || true)
if [[ "$count" -ne 1 ]]; then
  echo "FAIL: DOI deduplication failed (found $count occurrences)"
  exit 1
fi
echo "[TEST 3] PASS"

# Test 4: Scite enrichment
echo "[TEST 4] Scite enrichment..."
if ! grep -q "312" "$TMPDIR/out/stage3_scored.tsv"; then
  echo "FAIL: Scite supporting count not found"
  exit 1
fi
echo "[TEST 4] PASS"

# Test 5: Top-N filtering
echo "[TEST 5] Top-N filtering..."
lines=$(($(wc -l < "$TMPDIR/out/stage4_core.tsv") - 1))
if [[ "$lines" -gt 10 ]]; then
  echo "FAIL: Top-N filtering failed ($lines > 10)"
  exit 1
fi
echo "[TEST 5] PASS"

# Test 6: Provenance graph
echo "[TEST 6] Provenance graph generation..."
if [[ ! -f "$TMPDIR/out/provenance.dot" ]]; then
  echo "FAIL: provenance.dot not generated"
  exit 1
fi
if ! grep -q "digraph Provenance" "$TMPDIR/out/provenance.dot"; then
  echo "FAIL: DOT file malformed"
  exit 1
fi
echo "[TEST 6] PASS"

# Test 7: Reading queue
echo "[TEST 7] Reading queue generation..."
if [[ ! -f "$TMPDIR/out/READING_QUEUE.md" ]]; then
  echo "FAIL: READING_QUEUE.md not generated"
  exit 1
fi
if ! grep -q "Foundation Score" "$TMPDIR/out/READING_QUEUE.md"; then
  echo "FAIL: Reading queue missing foundation scores"
  exit 1
fi
echo "[TEST 7] PASS"

# Test 8: Zettelkasten
echo "[TEST 8] Zettelkasten generation..."
if [[ ! -d "$TMPDIR/out/zettel" ]]; then
  echo "FAIL: zettel/ directory not created"
  exit 1
fi
zettel_count=$(ls "$TMPDIR/out/zettel/" 2>/dev/null | wc -l)
if [[ "$zettel_count" -lt 1 ]]; then
  echo "FAIL: No zettels generated"
  exit 1
fi
echo "[TEST 8] PASS ($zettel_count zettels)"

# Test 9: Zombie detector
echo "[TEST 9] Zombie paper detector..."
if [[ ! -f "$TMPDIR/out/ZOMBIE_REPORT.md" ]]; then
  echo "FAIL: ZOMBIE_REPORT.md not generated"
  exit 1
fi
if ! grep -q "Zombie Score" "$TMPDIR/out/ZOMBIE_REPORT.md"; then
  echo "FAIL: Zombie report missing score column"
  exit 1
fi
echo "[TEST 9] PASS"

# Test 10: Narrative generator
echo "[TEST 10] Narrative generator..."
if [[ ! -f "$TMPDIR/out/NARRATIVE.md" ]]; then
  echo "FAIL: NARRATIVE.md not generated"
  exit 1
fi
if ! grep -q "Related Work" "$TMPDIR/out/NARRATIVE.md"; then
  echo "FAIL: Narrative missing Related Work section"
  exit 1
fi
echo "[TEST 10] PASS"

# Test 11: Contradiction Map
echo "[TEST 11] Contradiction map..."
if [[ ! -f "$TMPDIR/out/CONTRADICTION_MAP.md" ]]; then
  echo "FAIL: CONTRADICTION_MAP.md not generated"
  exit 1
fi
if ! grep -q "mermaid" "$TMPDIR/out/CONTRADICTION_MAP.md"; then
  echo "FAIL: Contradiction map missing mermaid diagram"
  exit 1
fi
echo "[TEST 11] PASS"

# Test 12: Serendipity Report
echo "[TEST 12] Serendipity report..."
if [[ ! -f "$TMPDIR/out/SERENDIPITY_REPORT.md" ]]; then
  echo "FAIL: SERENDIPITY_REPORT.md not generated"
  exit 1
fi
if ! grep -q "Entropy Score" "$TMPDIR/out/SERENDIPITY_REPORT.md"; then
  echo "FAIL: Serendipity report missing entropy scores"
  exit 1
fi
echo "[TEST 12] PASS"

# Test 13: Bias Audit
echo "[TEST 13] Bias audit..."
if [[ ! -f "$TMPDIR/out/BIAS_AUDIT.md" ]]; then
  echo "FAIL: BIAS_AUDIT.md not generated"
  exit 1
fi
if ! grep -q "Sesgo" "$TMPDIR/out/BIAS_AUDIT.md"; then
  echo "FAIL: Bias audit missing sections"
  exit 1
fi
echo "[TEST 13] PASS"

# Test 14: Zettel bidirectional links
echo "[TEST 14] Zettel bidirectional links..."
if ! grep -q "Related Zettels" "$TMPDIR/out/zettel/"*.md 2>/dev/null; then
  echo "FAIL: No zettels have bidirectional links"
  exit 1
fi
echo "[TEST 14] PASS"

# Test 15: All outputs exist
echo "[TEST 15] All expected outputs exist..."
for file in "RELATED_WORK_MATRIX.md" "READING_QUEUE.md" "ZOMBIE_REPORT.md" \
            "NARRATIVE.md" "CONTRADICTION_MAP.md" "SERENDIPITY_REPORT.md" \
            "BIAS_AUDIT.md" "provenance.dot"; do
  if [[ ! -f "$TMPDIR/out/$file" ]]; then
    echo "FAIL: $file not found"
    exit 1
  fi
done
echo "[TEST 15] PASS"

# Test 16: Year filter
echo "[TEST 16] Year filter (--min-year)..."
bash "$SCRIPT_DIR/pipeline.sh" \
  --elicit "$SCRIPT_DIR/examples/sample_elicit.csv" \
  --researchrabbit "$SCRIPT_DIR/examples/sample_researchrabbit.csv" \
  --scite "$SCRIPT_DIR/examples/sample_scite.csv" \
  --output "$TMPDIR/out_filtered" \
  --top-n 20 \
  --min-year 2020

if grep -q $'\t2016\t' "$TMPDIR/out_filtered/stage2_merged.tsv" 2>/dev/null; then
  echo "FAIL: Year filter not working (found 2016 paper)"
  exit 1
fi
echo "[TEST 16] PASS"

# Test 17: LaTeX export
echo "[TEST 17] LaTeX export..."
if [[ ! -f "$TMPDIR/out/RELATED_WORK.tex" ]]; then
  echo "FAIL: RELATED_WORK.tex not generated"
  exit 1
fi
if ! grep -q "\\\\documentclass" "$TMPDIR/out/RELATED_WORK.tex"; then
  echo "FAIL: LaTeX file malformed"
  exit 1
fi
if [[ ! -f "$TMPDIR/out/literature_pipeline.bib" ]]; then
  echo "FAIL: literature_pipeline.bib not generated"
  exit 1
fi
echo "[TEST 17] PASS"

# Test 18: Makefile exists (NEW v6.0)
echo "[TEST 18] Makefile exists..."
if [[ ! -f "$SCRIPT_DIR/Makefile" ]]; then
  echo "FAIL: Makefile not found"
  exit 1
fi
if ! grep -q "run:" "$SCRIPT_DIR/Makefile"; then
  echo "FAIL: Makefile missing 'run' target"
  exit 1
fi
if ! grep -q "test:" "$SCRIPT_DIR/Makefile"; then
  echo "FAIL: Makefile missing 'test' target"
  exit 1
fi
echo "[TEST 18] PASS"

# Test 19: Documentation exists (NEW v6.0)
echo "[TEST 19] Documentation exists..."
for doc in "docs/ARCHITECTURE.md" "docs/CONTRIBUTING.md" "docs/USAGE.md" "docs/FAQ.md"; do
  if [[ ! -f "$SCRIPT_DIR/$doc" ]]; then
    echo "FAIL: $doc not found"
    exit 1
  fi
done
echo "[TEST 19] PASS"

echo ""
echo "========================================"
echo "  All 19 tests passed!"
echo "========================================"
