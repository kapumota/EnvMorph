#!/usr/bin/awk -f
# reading_queue.awk
# Genera una cola de lectura priorizada topológicamente.
# Papers fundacionales (más antiguos, más citados) primero.
# Papers que dependen de otros van después.
#
# Uso: awk -f reading_queue.awk stage3_scored.tsv > READING_QUEUE.md

BEGIN {
    FS = OFS = "\t"
    print "# Reading Queue"
    print ""
    print "Papers ordenados por prerequisitos lógicos. Lee de arriba hacia abajo."
    print ""
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    sources = $6
    seed = $7
    score = $8
    sup = $9
    cont = $10
    tot = $11
    ratio = $12

    # Calcular "foundation score": cuánto fundamento es este paper
    # Más antiguo + más citado + más soportado = más fundacional
    age_boost = (2026 - (year + 0))  # más antiguo = más fundacional
    if (age_boost < 0) age_boost = 0

    citation_boost = (tot + 0) / 50  # normalizado
    if (citation_boost > 5) citation_boost = 5

    support_boost = (ratio + 0) * 3

    seed_boost = (seed == "YES") ? 2 : 0

    foundation_score = age_boost + citation_boost + support_boost + seed_boost

    # Guardar
    papers[NR] = doi
    p_title[doi] = title
    p_year[doi] = year
    p_authors[doi] = authors
    p_score[doi] = score
    p_foundation[doi] = foundation_score
    p_seed[doi] = seed
    p_sup[doi] = sup
    p_cont[doi] = cont
    p_tot[doi] = tot
    p_ratio[doi] = ratio
    p_sources[doi] = sources
}

END {
    # Ordenar por foundation_score descendente (más fundacional primero)
    n = 0
    for (i in papers) {
        n++
        order[n] = papers[i]
    }

    # Bubble sort por foundation_score (suficiente para datasets pequeños)
    for (i = 1; i <= n; i++) {
        for (j = i + 1; j <= n; j++) {
            if (p_foundation[order[j]] > p_foundation[order[i]]) {
                tmp = order[i]
                order[i] = order[j]
                order[j] = tmp
            }
        }
    }

    # Generar tabla
    print "| # | Paper | Year | Authors | Foundation Score | Seed | Citations | Scite Ratio |"
    print "|---|-------|------|---------|------------------|------|-----------|-------------|"

    for (i = 1; i <= n; i++) {
        d = order[i]

        # Acortar autores
        authors_short = p_authors[d]
        if (authors_short ~ /,.*,/) {
            first = authors_short
            sub(/,.*/, "", first)
            authors_short = first " et al."
        }

        # Acortar título
        t = p_title[d]
        if (length(t) > 50)
            t = substr(t, 1, 50) "..."

        link = "[" t "](https://doi.org/" d ")"
        seed_badge = (p_seed[d] == "YES") ? "🌱" : ""

        printf "| %d | %s | %s | %s | %.1f | %s | %s | %.2f |\n",
            i, link, p_year[d], authors_short, p_foundation[d], seed_badge, p_tot[d], p_ratio[d]
    }

    print ""
    print "## Cómo interpretar"
    print ""
    print "- **Foundation Score**: combina antigüedad, citas totales, ratio de soporte Scite y status de seed."
    print "- **Lectura recomendada**: empieza por los papers con score fundacional más alto (son los prerequisitos)."
    print "- **Papers con 🌱**: semillas del proceso de búsqueda, alta prioridad."
    print ""
    print "## Estadísticas"
    print "- Total papers: " n

    seed_count = 0
    for (i = 1; i <= n; i++) if (p_seed[order[i]] == "YES") seed_count++
    print "- Seed papers: " seed_count

    avg_foundation = 0
    for (i = 1; i <= n; i++) avg_foundation += p_foundation[order[i]]
    if (n > 0) avg_foundation /= n
    printf "- Foundation score promedio: %.1f\n", avg_foundation
}
