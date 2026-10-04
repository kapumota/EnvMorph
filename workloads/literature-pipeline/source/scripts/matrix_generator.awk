#!/usr/bin/awk -f
# matrix_generator.awk
# Genera RELATED_WORK_MATRIX.md desde el TSV enriquecido

BEGIN {
    FS = "\t"
    print "# Related Work Matrix"
    print ""
    print "Pipeline: Elicit -> ResearchRabbit -> Scite -> Core Selection"
    print ""
    print "| # | Paper | Year | Authors | Sources | Scite Support | Scite Contrast | Score | Seed |"
    print "|---|-------|------|---------|---------|---------------|----------------|-------|------|"
}

NR == 1 { next }

{
    doi      = $1
    title    = $2
    year     = $3
    authors  = $4
    sources  = $6
    seed     = $7
    score    = $8
    sup      = $9
    cont     = $10

    # Acortar autores
    if (authors ~ /,.*,/) {
        first = authors
        sub(/,.*/, "", first)
        authors = first " et al."
    }

    # Acortar titulo
    if (length(title) > 55)
        title = substr(title, 1, 55) "..."

    badge = (seed == "YES") ? "seed" : ""
    link = "[" title "](https://doi.org/" doi ")"

    printf "| %d | %s | %s | %s | %s | %s | %s | %s | %s |\n",
        NR - 1, link, year, authors, sources, sup, cont, score, badge
}

END {
    print ""
    print "## Summary"
    print "- **Total candidates evaluated:** " NR - 2
    print "- **Seed papers:** marcados con 'seed' (score >= 3 + multiples fuentes)"
    print "- **Score formula:** Elicit relevance + ResearchRabbit network + Scite confidence boost"
    print "- **Scite ratio:** supporting / total citations (penaliza papers muy contrastados)"
    print ""
    print "## Methodology"
    print "1. **Elicit**: busqueda semantica inicial, 15-25 candidatos."
    print "2. **ResearchRabbit**: expansion por red de citaciones."
    print "3. **Scite**: validacion de contextos de cita (soporte vs. contradiccion)."
    print "4. **Core selection**: top 20 por score compuesto."
}
