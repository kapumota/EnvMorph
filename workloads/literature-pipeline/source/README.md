# Literature Pipeline v5.0

[![CI](https://github.com/TU-USUARIO/literature-pipeline/actions/workflows/ci.yml/badge.svg)](https://github.com/TU-USUARIO/literature-pipeline/actions)
[![Tests](https://img.shields.io/badge/tests-17%2F17-brightgreen)](tests/test_pipeline.sh)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Docker](https://img.shields.io/badge/docker-ready-blue)](Dockerfile)
[![Awk](https://img.shields.io/badge/powered%20by-awk-orange)]()

> Pipeline de línea de comandos para construir matrices de *related work* a partir de múltiples fuentes bibliográficas: **Elicit**, **ResearchRabbit**, **Scite** y **SciSpace**.
> 
> **Novedad**: No es solo un buscador. Es un **sistema de gestión del conocimiento científico** con 12 features únicas que ninguna herramienta SaaS ofrece.

Escrito 100 % en `bash` + `awk`. Sin dependencias de Python, Node o Docker. Funciona en cualquier sistema Unix/Linux/macOS con `gawk` instalado.

---

## 🚀 Las 12 features únicas (ninguna herramienta SaaS las tiene todas)

| # | Feature | Descripción | Estado |
|---|---------|-------------|--------|
| 1 | **Semantic Deduplication** | Detecta duplicados arXiv vs. publicado por similitud LCS | ✅ |
| 2 | **Year Filter** | Filtra papers por año mínimo (`--min-year 2020`) | ✅ |
| 3 | **Provenance Graph** | Grafo de cómo llegó cada paper (Elicit, RR, ambos) | ✅ |
| 4 | **Reading Queue Topológica** | Orden de lectura por fundamentalidad | ✅ |
| 5 | **Zombie Paper Detector** | Alerta de papers sobrevalorados | ✅ |
| 6 | **Narrative Generator** | Borrador automático de sección "Related Work" | ✅ |
| 7 | **Contradiction Map** | Mapa de disputas académicas con grafo Mermaid | ✅ |
| 8 | **Serendipity Score** | Entropía de Shannon (puentes interdisciplinarios) | ✅ |
| 9 | **Bias Audit** | Auditoría de sesgo temporal y de fuente | ✅ |
| 10 | **Zettelkasten Export** | Notas atómicas con links bidireccionales para Obsidian | ✅ |
| 11 | **Git-native Tracking** | Commits automáticos en rama `literature` | ✅ |
| 12 | **100% Offline** | Sin conexión a internet, sin APIs, sin tokens | ✅ |

---

## 📊 Tabla comparativa vs. la competencia

| Feature | Elicit | RR | Scite | Conn.Papers | **Tu Pipeline** |
|-------------------------|--------|----|-------|-------------|-------------|
| Búsqueda semántica | ✅ | ✅ | ❌ | ❌ | ✅ |
| Grafo de citas | ❌ | ✅ | ❌ | ✅ | ✅ |
| **Semantic dedup** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Year filter** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Reading queue** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Zombie detector** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Narrative generator** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Contradiction map** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Serendipity score** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Bias audit** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Zettelkasten + links** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **LaTeX export** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **Git tracking** | ❌ | ❌ | ❌ | ❌ | ✅* |
| **100% offline** | ❌ | ❌ | ❌ | ❌ | ✅* |

\* = **ÚNICO** — ninguna herramienta SaaS ofrece esta feature

---

## 📦 Instalación

### Opción A: Nativo (Linux/macOS)

```bash
git clone https://github.com/tu-usuario/literature-pipeline.git
cd literature-pipeline
```

Requisitos:
- `bash` ≥ 4.0
- `gawk` (GNU awk) — `sudo apt install gawk` / `brew install gawk`
- Opcional: `graphviz` para renderizar `provenance.dot`

### Opción B: Docker (Windows/Linux/macOS)

```bash
docker-compose up --build
```

O manualmente:

```bash
docker build -t literature-pipeline .
docker run -v $(pwd)/data:/app/data -v $(pwd)/output:/app/output literature-pipeline \
  --elicit /app/data/elicit.csv \
  --researchrabbit /app/data/researchrabbit.csv \
  --scite /app/data/scite.csv \
  --output /app/output
```

---

## 🎯 Uso rápido

### 1. Exporta tus datos

| Herramienta | Formato esperado | Archivo de entrada |
|-------------|------------------|--------------------|
| Elicit | CSV: `title,authors,doi,year,abstract` | `data/elicit.csv` |
| ResearchRabbit | CSV: `title,authors,doi,year,abstract` | `data/researchrabbit.csv` |
| Scite | CSV: `doi,supporting,mentioning,contrasting,total` | `data/scite.csv` |

### 2. Ejecuta el pipeline

```bash
# Basico
./pipeline.sh \
  --elicit data/elicit.csv \
  --researchrabbit data/researchrabbit.csv \
  --scite data/scite.csv \
  --output output/ \
  --top-n 20

# Con filtro de año
./pipeline.sh \
  --elicit data/elicit.csv \
  --researchrabbit data/researchrabbit.csv \
  --scite data/scite.csv \
  --output output/ \
  --top-n 20 \
  --min-year 2020
```

### 3. Revisa los 12 artefactos generados

```bash
# 1. Tabla de related work
cat output/RELATED_WORK_MATRIX.md

# 2. Grafo de procedencia
dot -Tpng output/provenance.dot -o output/provenance.png

# 3. Cola de lectura topologica
cat output/READING_QUEUE.md

# 4. Notas atomicas para Obsidian
ls output/zettel/

# 5. Zombie detector
cat output/ZOMBIE_REPORT.md

# 6. Borrador de Related Work
cat output/NARRATIVE.md

# 7. Contradiction map
cat output/CONTRADICTION_MAP.md

# 8. Serendipity score
cat output/SERENDIPITY_REPORT.md

# 9. Bias audit
cat output/BIAS_AUDIT.md

# 10. LaTeX con bibliografia
cat output/RELATED_WORK.tex

# 11. BibTeX
cat output/literature_pipeline.bib

# 12. Deduplicacion semantica log
cat output/dedup.log
```

---

## 🏗️ Estructura del proyecto

```
literature-pipeline/
├── scripts/
│   ├── elicit_clean.awk          # Stage 1: Normaliza CSV
│   ├── semantic_dedup.awk        # Stage 2: Deduplicacion LCS
│   ├── year_filter.awk           # Stage 4: Filtro por año
│   ├── merge_sources.awk         # Stage 5: Fusiona fuentes
│   ├── scite_enrich.awk          # Stage 6: Enriquece Scite
│   ├── matrix_generator.awk      # Stage 8: Tabla Markdown
│   ├── provenance_graph.awk      # Stage 9: Grafo DOT
│   ├── reading_queue.awk         # Stage 10: Cola de lectura
│   ├── zettel_generator.awk      # Stage 11: Notas Obsidian
│   ├── zombie_detector.awk       # Stage 12: Detector zombie
│   ├── narrative_generator.awk   # Stage 13: Narrativa
│   ├── contradiction_map.awk     # Stage 14: Mapa contradicciones
│   ├── serendipity.awk           # Stage 15: Score sorpresa
│   ├── bias_audit.awk            # Stage 16: Auditoria sesgo
│   ├── zettel_links.awk          # Stage 17: Links bidireccionales
│   └── latex_export.awk          # Stage 18: Export LaTeX
├── examples/
│   ├── sample_elicit.csv
│   ├── sample_researchrabbit.csv
│   └── sample_scite.csv
├── tests/
│   └── test_pipeline.sh          # 17 tests de integracion
├── Dockerfile                    # Docker para Windows/Linux/macOS
├── docker-compose.yml            # Docker Compose
├── pipeline.sh                   # Orquestador principal
├── README.md
└── LICENSE
```

---

## 🎨 Las 12 features novedosas en detalle

### 1. Semantic Deduplication
Detecta duplicados semanticos (arXiv vs. NeurIPS) por similitud de titulo > 85% usando **Longest Common Subsequence (LCS)** implementado en awk.

### 2. Year Filter (`--min-year`)
Filtra papers por año minimo antes del scoring. Ideal para enfocarse en trabajo reciente o excluir obsoleto.

### 3. Provenance Graph
Visualiza **como llego cada paper** a tu lista. Nodos verdes = ambas fuentes, azules = solo Elicit, rosas = solo RR.

### 4. Reading Queue Topologica
Ordena papers por **fundamentalidad**: antiguedad + citas totales + ratio Scite + status seed.

### 5. Zombie Paper Detector
Detecta papers sobrevalorados con `zombie_score = total_citations / supporting_citations`.

### 6. Narrative Generator
Genera borrador de seccion "Related Work" agrupando por temas emergentes con conectores textuales.

### 7. Contradiction Map
Mapa de disputas academicas: papers con >10 contradicciones se marcan como 🔥 polemicos.

### 8. Serendipity Score
Entropia de Shannon sobre fuentes de descubrimiento. Papers en 4+ fuentes son 🌟 **High Surprise**.

### 9. Bias Audit
Auditoria de sesgo temporal y de fuente. Alerta con recomendaciones de diversificacion.

### 10. Zettelkasten Export
Notas atomicas con frontmatter YAML, checkboxes, y **links bidireccionales reales** `[[...]]`.

### 11. LaTeX Export
Genera `RELATED_WORK.tex` listo para compilar con `pdflatex` + `literature_pipeline.bib` con entradas BibTeX y `ibliography`.

### 12. Git-native Tracking
Commits automaticos en rama `literature`. Usa `git diff literature~7 literature -- output/` para ver evolucion.

---

## 🧪 Tests

```bash
bash tests/test_pipeline.sh
```

17 tests de integracion verifican todo el pipeline incluyendo filtro de año y export LaTeX.

---

## 📄 Licencia

MIT © [Tu Nombre](https://github.com/tu-usuario)
