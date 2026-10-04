#!/usr/bin/awk -f
# surprise_score.awk
# Calcula un "surprise score" para cada paper basado en:
# 1. Diversidad de fuentes de descubrimiento (entropía de Shannon)
# 2. Diversidad temática del abstract (cuántos dominios toca)
# 3. Ratio de contradicción (papers que desafían el status quo)
#
# Uso: awk -f surprise_score.awk stage3_scored.tsv > SURPRISE_REPORT.md

BEGIN {
    FS = OFS = "\t"

    print "# Surprise Score Report"
    print ""
    print "> Detector de ideas fronterizas e inesperadas."
    print "> Un paper con alto surprise score aparece en contextos diversos"
    print "> o desafía el consenso establecido."
    print ""
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    abstract = $5
    sources = $6
    cont = ($10 == "" ? 0 : $10 + 0)
    tot = ($11 == "" ? 0 : $11 + 0)
    ratio = ($12 == "" ? 0 : $12 + 0)

    # 1. Entropía de fuentes
    # Contar cuántas fuentes distintas trajeron este paper
    n_sources = 0
    if (sources ~ /elicit/) n_sources++
    if (sources ~ /researchrabbit/) n_sources++

    # Entropía de Shannon (máxima cuando todas las fuentes son igualmente probables)
    # Con 2 fuentes, máx entropía = 1.0
    source_entropy = 0
    if (n_sources == 1) source_entropy = 0.0
    else if (n_sources == 2) source_entropy = 1.0

    # 2. Diversidad temática del abstract
    # Contar cuántos dominios diferentes menciona
    abst_lower = tolower(abstract)
    themes = 0
    if (abst_lower ~ /transformer|attention|bert|gpt|language model|llm/) themes++
    if (abst_lower ~ /image|vision|cnn|resnet|vit|patch|classification/) themes++
    if (abst_lower ~ /generative|adversarial|gan|generation/) themes++
    if (abst_lower ~ /retrieval|augmented|rag|knowledge/) themes++
    if (abst_lower ~ /dataset|corpus|pile|data/) themes++
    if (abst_lower ~ /ethical|bias|risk|environmental|parrot|danger|concern/) themes++
    if (abst_lower ~ /survey|review|overview/) themes++
    if (abst_lower ~ /scaling|emergence|few-shot|prompt|chain/) themes++

    theme_diversity = themes / 8.0  # normalizar a 0-1

    # 3. Factor de contradicción (desafía el status quo)
    # Papers muy contrastados a veces son "sorpresivos"
    if (tot > 0) {
        contrast_ratio = cont / tot
    } else {
        contrast_ratio = 0
    }
    # Invertir: más contradicción = más sorpresa, pero con techo
    contradiction_factor = (contrast_ratio > 0.3) ? 0.5 : (contrast_ratio * 1.5)

    # 4. Factor de antigüedad (papers recientes que revolucionan son sorpresivos)
    age = 2026 - (year + 0)
    if (age < 0) age = 0
    recency_bonus = (age <= 2) ? 0.3 : 0.0

    # Surprise Score compuesto (0-10)
    surprise = (source_entropy * 3.0) + (theme_diversity * 4.0) + (contradiction_factor * 2.0) + recency_bonus
    if (surprise > 10) surprise = 10

    # Guardar
    n++
    papers[n] = doi
    p_title[doi] = title
    p_year[doi] = year
    p_authors[doi] = authors
    p_surprise[doi] = surprise
    p_entropy[doi] = source_entropy
    p_themes[doi] = theme_diversity
    p_contrast[doi] = contradiction_factor
    p_sources[doi] = sources
    p_cont[doi] = cont
    p_tot[doi] = tot
}

END {
    # Ordenar por surprise score descendente
    for (i = 1; i <= n; i++) {
        for (j = i + 1; j <= n; j++) {
            if (p_surprise[papers[j]] > p_surprise[papers[i]]) {
                tmp = papers[i]
                papers[i] = papers[j]
                papers[j] = tmp
            }
        }
    }

    print "| # | Paper | Year | Surprise | Entropy | Themes | Contradiction | Sources |"
    print "|---|-------|------|----------|---------|--------|---------------|---------|"

    for (i = 1; i <= n; i++) {
        d = papers[i]

        # Acortar
        t = p_title[d]
        if (length(t) > 45) t = substr(t, 1, 45) "..."

        a = p_authors[d]
        if (a ~ /,.*,/) { first = a; sub(/,.*/, "", first); a = first " et al." }

        link = "[" t "](https://doi.org/" d ")"

        # Badge
        if (p_surprise[d] >= 7) badge = "🔥"
        else if (p_surprise[d] >= 5) badge = "✨"
        else if (p_surprise[d] >= 3) badge = "💡"
        else badge = ""

        printf "| %d | %s | %s | %.1f %s | %.1f | %.2f | %.2f | %s |\n",
            i, link, p_year[d], p_surprise[d], badge, p_entropy[d], p_themes[d], p_contrast[d], p_sources[d]
    }

    print ""
    print "## Cómo interpretar el Surprise Score"
    print ""
    print "```"
    print "surprise = source_entropy*3 + theme_diversity*4 + contradiction*2 + recency"
    print "```"
    print ""
    print "| Componente | Peso | Qué mide |"
    print "|------------|------|----------|"
    print "| **Source Entropy** | 3x | Diversidad de fuentes de descubrimiento |"
    print "| **Theme Diversity** | 4x | Cuántos dominios toca el abstract |"
    print "| **Contradiction Factor** | 2x | Si desafía el status quo (>30% contrasting) |"
    print "| **Recency Bonus** | +0.3 | Papers recientes (≤2 años) que revolucionan |"
    print ""
    print "| Score | Interpretación |"
    print "|-------|----------------|"
    print "| ≥ 7.0 | 🔥 **High Surprise**: Idea fronteriza, interdisciplinaria o disruptiva |"
    print "| 5.0 - 6.9 | ✨ **Medium Surprise**: Conexión inesperada entre dominios |"
    print "| 3.0 - 4.9 | 💡 **Low Surprise**: Diversidad moderada, dentro de lo esperado |"
    print "| < 3.0 | 📋 **Predictable**: Paper estándar de su dominio |"
    print ""

    print "## Estadísticas"
    print "- Total papers analizados: " n

    high = 0; med = 0; low = 0
    for (i = 1; i <= n; i++) {
        s = p_surprise[papers[i]]
        if (s >= 7) high++
        else if (s >= 5) med++
        else if (s >= 3) low++
    }
    print "- 🔥 High surprise: " high
    print "- ✨ Medium surprise: " med
    print "- 💡 Low surprise: " low
    print ""

    if (high > 0) {
        print "## 🔥 Papers de Alta Sorpresa"
        print ""
        print "Estos papers conectan dominios inesperados o desafían el consenso."
        print "Son candidatos para descubrimientos fronterizos en tu investigación."
        print ""
        for (i = 1; i <= n; i++) {
            if (p_surprise[papers[i]] >= 7) {
                d = papers[i]
                print "- **" p_title[d] "** (" p_year[d] ") — Score: " p_surprise[d]
            }
        }
    }
}
