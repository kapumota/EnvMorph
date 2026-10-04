#!/usr/bin/awk -f
# zombie_detector.awk
# Detecta "zombie papers": muchas citas totales pero pocas de soporte activo.
# Un paper con zombie_score > 10 es ampliamente mencionado pero raramente
# usado como fundamento real.
#
# Uso: awk -f zombie_detector.awk stage3_scored.tsv > ZOMBIE_REPORT.md

BEGIN {
    FS = OFS = "\t"

    print "# Zombie Paper Report"
    print ""
    print "> Papers con muchas citas totales pero bajo soporte activo."
    print "> Son ampliamente mencionados pero raramente usados como fundamento."
    print ""
    print "| # | Paper | Year | Total Citas | Supporting | Zombie Score | Status |"
    print "|---|-------|------|-------------|----------|--------------|--------|"
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    sup = ($9 == "" ? 0 : $9 + 0)
    tot = ($11 == "" ? 0 : $11 + 0)

    # Evitar division por cero
    if (sup <= 0) sup = 1

    zombie_score = tot / sup

    # Clasificacion
    if (zombie_score >= 10) {
        status = "🧟 Zombie"
    } else if (zombie_score >= 5) {
        status = "⚠️  At Risk"
    } else if (zombie_score >= 2) {
        status = "😐 Normal"
    } else {
        status = "✅ Solid"
    }

    # Acortar titulo
    if (length(title) > 50)
        title = substr(title, 1, 50) "..."

    # Acortar autores
    if (authors ~ /,.*,/) {
        first = authors
        sub(/,.*/, "", first)
        authors = first " et al."
    }

    link = "[" title "](https://doi.org/" doi ")"

    printf "| %d | %s | %s | %s | %s | %.1f | %s |\n",
        NR - 1, link, year, tot, sup, zombie_score, status

    # Guardar para resumen
    all_zombie[NR] = zombie_score
    all_status[NR] = status
    all_title[NR] = title
}

END {
    print ""
    print "## Interpretacion del Zombie Score"
    print ""
    print "```"
    print "zombie_score = total_citations / supporting_citations"
    print "```"
    print ""
    print "| Rango | Significado |"
    print "|-------|-------------|"
    print "| < 2.0 | ✅ **Solid**: Fuertemente respaldado por la comunidad |"
    print "| 2.0 - 4.9 | 😐 **Normal**: Mencionado y respaldado en proporcion equilibrada |"
    print "| 5.0 - 9.9 | ⚠️  **At Risk**: Muchas menciones pasivas, poco soporte activo |"
    print "| >= 10.0 | 🧟 **Zombie**: Ampliamente mencionado pero raramente usado como fundamento |"
    print ""

    # Contar por categoria
    zombie_count = 0
    at_risk_count = 0
    solid_count = 0
    for (i in all_zombie) {
        if (all_status[i] ~ /Zombie/) zombie_count++
        else if (all_status[i] ~ /At Risk/) at_risk_count++
        else if (all_status[i] ~ /Solid/) solid_count++
    }

    print "## Resumen"
    print "- Total papers analizados: " (NR - 2)
    print "- 🧟 Zombies: " zombie_count
    print "- ⚠️  At Risk: " at_risk_count
    print "- ✅ Solid: " solid_count
    print ""
    print "## Recomendacion"
    print ""
    if (zombie_count > 0) {
        print "⚠️  **Cuidado**: Tienes " zombie_count " paper(s) zombie en tu lista."
        print "Estos papers pueden estar sobrevalorados. Considera:"
        print "1. Verificar si el paper realmente aporta a tu investigacion"
        print "2. Buscar alternativas mas respaldadas"
        print "3. Si lo incluyes, justificar por que lo usas a pesar del bajo soporte"
    } else {
        print "✅ **Excelente**: No hay papers zombie en tu lista."
        print "Todos los papers seleccionados tienen soporte activo de la comunidad."
    }
}
