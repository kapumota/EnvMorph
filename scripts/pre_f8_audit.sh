#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"
cd "$ROOT"

fail() { echo "FAIL: $*" >&2; exit 1; }
pass() { echo "PASS: $*"; }

[[ -f docs/F8_PROTOCOL.md ]] || fail "falta docs/F8_PROTOCOL.md"
[[ -f experiments/f8/protocol.toml ]] || fail "falta experiments/f8/protocol.toml"
[[ -f docs/PRIOR_ART.md ]] || fail "falta docs/PRIOR_ART.md"
[[ -f docs/PAPER_SCOPE.md ]] || fail "falta docs/PAPER_SCOPE.md"
[[ -f workloads/literature-pipeline/README.md ]] || fail "falta la declaración del fixture histórico"
[[ -f provenance/F8_PROTOCOL_FILES.sha256 ]] || fail "falta el snapshot SHA-256 del protocolo F8"
pass "artefactos pre-F8 presentes"

(
  cd workloads/literature-pipeline/source
  sha256sum -c --quiet "$ROOT/provenance/LITERATURE_PIPELINE_FILES.sha256"
) || fail "el fixture histórico cambió"
pass "fixture histórico intacto"

sha256sum -c --quiet provenance/F8_PROTOCOL_FILES.sha256 || fail "el protocolo F8 cambió respecto del snapshot congelado"
pass "protocolo F8 coincide con su snapshot"

[[ "$(grep -c '^#### F5\.' docs/ROADMAP.md)" -eq 1 ]] || fail "ROADMAP debe tener una sola sección F5"
[[ "$(grep -c '^#### F6\.' docs/ROADMAP.md)" -eq 1 ]] || fail "ROADMAP debe tener una sola sección F6"
[[ "$(grep -c '^#### F7\.' docs/ROADMAP.md)" -eq 1 ]] || fail "ROADMAP debe tener una sola sección F7"
grep -Fq 'F8. External Corpus and Validation' docs/ROADMAP.md || fail "ROADMAP no identifica F8"
grep -Fq 'fuera del camino crítico de Paper 1' docs/ROADMAP.md || fail "ROADMAP no desacopla F9 de Paper 1"
pass "ROADMAP reconciliado"

python3 - <<'PY'
from pathlib import Path
import re

files = [
    Path('README.md'),
    Path('docs/ROADMAP.md'),
    Path('docs/F8_PROTOCOL.md'),
    Path('docs/PRIOR_ART.md'),
    Path('docs/PAPER_SCOPE.md'),
    Path('workloads/literature-pipeline/README.md'),
]

bad_phrases = [
    'TU-USUARIO',
    '12 features únicas',
    '12 features unicas',
    'revolucionario',
    'mejor que la competencia',
]
emoji = re.compile('[\U0001F300-\U0001FAFF\u2600-\u27BF]')

errors = []
for path in files:
    text = path.read_text(encoding='utf-8')
    lower = text.lower()
    for phrase in bad_phrases:
        if phrase.lower() in lower:
            errors.append(f'{path}: contiene frase no permitida: {phrase}')
    if emoji.search(text):
        errors.append(f'{path}: contiene emojis o símbolos decorativos')
    if re.search(r'^#{1,2}(?:\s|$)', text, flags=re.MULTILINE):
        errors.append(f'{path}: contiene encabezados Markdown de nivel 1 o 2')

if errors:
    raise SystemExit('\n'.join(errors))
PY
pass "superficie editorial activa sin placeholders, promoción ni emojis"

for term in reprotest diffoscope 'Reproducible Builds' 'Delta Debugging' 'Metamorphic Testing' RepTrace; do
  grep -Fiq "$term" docs/PRIOR_ART.md || fail "prior art no contiene $term"
done
pass "prior art mínimo registrado"

grep -Fq 'selection_must_be_result_blind = true' experiments/f8/protocol.toml || fail "el protocolo no congela selección independiente de resultados"
grep -Fq 'retain_negative_results = true' experiments/f8/protocol.toml || fail "el protocolo no exige resultados negativos"
grep -Fq 'extend_only_when_selected_corpus_requires = true' experiments/f8/protocol.toml || fail "el protocolo no condiciona extensiones al corpus"
grep -Fq 'f9_on_critical_path = false' experiments/f8/protocol.toml || fail "F9 sigue en el camino crítico"
pass "reglas F8 congeladas"

if git show-ref --verify --quiet refs/heads/fase7/portabilidad; then
  git diff --quiet fase7/portabilidad -- src || fail "la reconciliación pre-F8 modificó src/"
  git diff --quiet fase7/portabilidad -- workloads/literature-pipeline/source || fail "la reconciliación pre-F8 modificó el fixture"
fi
pass "sin cambios científicos ocultos antes de F8"

echo "PRE-F8 AUDIT PASS"
