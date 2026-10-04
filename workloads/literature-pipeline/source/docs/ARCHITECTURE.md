# Arquitectura del Literature Pipeline

## Vision general

El Literature Pipeline es un sistema de procesamiento de datos bibliograficos
escrito 100% en `bash` + `awk`. No tiene dependencias externas (Python, Node, etc.)
y puede ejecutarse offline.

## Diagrama de flujo

```
┌─────────────┐     ┌─────────────────┐     ┌──────────────────┐
│   Elicit    │────→│  elicit_clean   │────→│ semantic_dedup   │
│   (CSV)     │     │    .awk         │     │     .awk         │
└─────────────┘     └─────────────────┘     └──────────────────┘
                                                    │
┌─────────────────┐     ┌─────────────────┐        │
│ ResearchRabbit  │────→│  merge_sources  │←───────┘
│    (CSV)        │     │     .awk        │
└─────────────────┘     └─────────────────┘
                                │
                        ┌───────┴───────┐
                        │  year_filter   │  ← --min-year
                        │    .awk        │
                        └───────┬───────┘
                                │
                        ┌───────┴───────┐
                        │  scite_enrich  │  ← Scite CSV
                        │     .awk       │
                        └───────┬───────┘
                                │
                    ┌───────────┼───────────┐
                    │           │           │
            ┌───────┴──┐ ┌────┴────┐ ┌───┴────┐
            │  matrix   │ │ reading │ │ zettel │
            │ generator │ │  queue  │ │generator│
            └─────┬─────┘ └────┬────┘ └───┬────┘
                  │            │          │
            RELATED_WORK    READING    zettel/
            _MATRIX.md      _QUEUE.md
                    │            │          │
            ┌───────┴──┐ ┌────┴────┐ ┌───┴────┐
            │  zombie   │ │narrative│ │contrad.│
            │ detector  │ │generator│ │  map   │
            └─────┬─────┘ └────┬────┘ └───┬────┘
                  │            │          │
            ZOMBIE_      NARRATIVE   CONTRADICTION
            REPORT.md    .md         _MAP.md
                    │            │          │
            ┌───────┴──┐ ┌────┴────┐ ┌───┴────┐
            │serendipity│ │  bias   │ │  latex │
            │   .awk    │ │  audit  │ │ export │
            └─────┬─────┘ └────┬────┘ └───┬────┘
                  │            │          │
            SERENDIPITY    BIAS_      RELATED_WORK
            _REPORT.md    AUDIT.md    .tex + .bib
```

## Principios de diseno

1. **Unix Philosophy**: Cada script hace una cosa bien. Se componen con pipes.
2. **Zero Dependencies**: Solo bash + gawk. No Python, no Node, no Docker obligatorio.
3. **Idempotencia**: Correr el pipeline dos veces produce el mismo resultado.
4. **Git-native**: Todo el output es texto plano (Markdown, TSV, DOT) versionable con git.
5. **Offline-first**: No requiere internet ni APIs despues de obtener los CSVs iniciales.

## Formato de datos interno

Todos los scripts usan **TSV** (tab-separated values) como formato de intercambio:

```
doi\ttitle\tyear\tauthors\tabstract\tsources\tseed_flag\ttotal_score\tscite_support\tscite_contrast\tscite_total\tscite_ratio
```

Esto permite que cualquier script awk pueda leer la salida de otro sin parseo complejo.

## Extension

Para agregar una nueva fuente (ej. Semantic Scholar):

1. Crea `scripts/semantic_scholar_enrich.awk`
2. Sigue el patron de `scite_enrich.awk` (dos pasadas: carga fuente, merge con pipeline)
3. Agrega la etapa en `pipeline.sh` entre stages 6 y 7
4. Agrega un test en `tests/test_pipeline.sh`
