#!/usr/bin/awk -f
# serendipity.awk
# Calcula un score de "sorpresa" basado en entropia de Shannon
# sobre las fuentes de descubrimiento de cada paper.
# Papers que aparecen en muchas fuentes distintas son puentes interdisciplinarios.
#
# Uso: awk -f serendipity.awk stage2_merged.tsv > SERENDIPITY_REPORT.md

BEGIN {
    FS = OFS = "\t"

    print "# Serendipity Report"
    print ""
    print "> Score de sorpresa basado en entropia de Shannon sobre fuentes de descubrimiento."
    print "> Papers con alta entropia son puentes interdisciplinarios valiosos."
    print ""
    print "| # | Paper | Year | Sources | Entropy Score | Surprise Level |"
    print "|---|-------|------|---------|---------------|----------------|"
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    sources = $6

    # Contar fuentes distintas
    n_sources = split(sources, srcs, ",")

    # Calcular entropia de Shannon (simplificada)
    # Asumimos distribucion uniforme entre fuentes
    # H = -sum(p_i * log2(p_i))
    # Para n fuentes uniformes: H = log2(n)
    if (n_sources > 1) {
        entropy = log(n_sources) / log(2)  # log2(n)
    } else {
        entropy = 0
    }

    # Normalizar a escala 0-10 (maximo teorico con 5 fuentes = log2(5) ≈ 2.32)
    max_entropy = log(5) / log(2)
    normalized = (entropy / max_entropy) * 10

    # Clasificar
    if (normalized >= 7) {
        level = "🌟 High Surprise"
    } else if (normalized >= 4) {
        level = "✨ Medium Surprise"
    } else {
        level = "📌 Standard"
    }

    # Acortar titulo
    if (length(title) > 45)
        title = substr(title, 1, 45) "..."

    # Acortar autores
    if (authors ~ /,.*,/) {
        first = authors
        sub(/,.*/, "", first)
        authors = first " et al."
    }

    link = "[" title "](https://doi.org/" doi ")"

    printf "| %d | %s | %s | %s | %.2f | %s |\n",
        NR - 1, link, year, sources, normalized, level

    # Guardar para resumen
    all_entropy[NR] = normalized
    all_level[NR] = level
}

END {
    print ""
    print "## Formula"
    print ""
    print "```"
    print "entropy = log2(n_sources)"
    print "normalized = (entropy / log2(5)) * 10"
    print "```"
    print ""
    print "| Rango | Nivel | Significado |"
    print "|-------|-------|-------------|"
    print "| 0 - 3.9 | 📌 **Standard** - Aparece en 1 fuente | Paper tipico de un solo descubrimiento |"
    print "| 4 - 6.9 | ✨ **Medium Surprise** - Aparece en 2-3 fuentes | Puente entre comunidades |"
    print "| 7 - 10 | 🌟 **High Surprise** - Aparece en 4+ fuentes | Descubrimiento interdisciplinario raro |"
    print ""

    # Contar por categoria
    high = 0
    medium = 0
    standard = 0
    for (i in all_entropy) {
        if (all_level[i] ~ /High/) high++
        else if (all_level[i] ~ /Medium/) medium++
        else standard++
    }

    print "## Resumen"
    print "- Total papers: " (NR - 2)
    print "- 🌟 High Surprise: " high
    print "- ✨ Medium Surprise: " medium
    print "- 📌 Standard: " standard
    print ""
    print "## Recomendacion"
    print ""
    if (high > 0) {
        print "🌟 **Descubrimiento valioso** - Tienes " high " paper(s) de alta sorpresa."
        print "Estos papers conectan multiples comunidades y pueden ofrecer perspectivas unicas."
    } else if (medium > 0) {
        print "✨ **Buena diversidad** - Tienes " medium " paper(s) de sorpresa media."
        print "Estos puentes entre comunidades enriquecen tu revision."
    } else {
        print "📌 **Busqueda homogenea** - Todos tus papers vienen de una sola fuente."
        print "Considera expandir tu busqueda a otras herramientas o bases de datos."
    }
}
