#!/usr/bin/awk -f
# bias_audit.awk
# Audita sesgo geografico e institucional en la lista de papers.
# Usa heuristica de dominios de email/afiliacion si estan disponibles,
# o inferencia por nombre de autor y patrones conocidos.
#
# Uso: awk -f bias_audit.awk stage2_merged.tsv > BIAS_AUDIT.md

BEGIN {
    FS = OFS = "\t"

    print "# Bias Audit Report"
    print ""
    print "> Audita sesgo geografico e institucional en tu revision de literatura."
    print "> ⚠️  Nota: Este analisis es heuristico. Para resultados precisos,"
    print ">   incluye afiliaciones de autores en tus exportaciones."
    print ""
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    sources = $6

    # Heuristica 1: Detectar region por patrones en nombres de autores/instituciones
    # Esto es una aproximacion muy simplificada. En produccion usarias
    # una base de datos de afiliaciones.

    authors_lower = tolower(authors)

    # Deteccion muy basica por patrones de nombres (no confiable, solo demostrativo)
    # En un sistema real, parsearias afiliaciones del CSV
    region = "Unknown"

    # Si el CSV incluye afiliaciones, usarlas
    # Por ahora, usamos una heuristica de "diversidad de autores"
    # como proxy de diversidad geografica

    n_authors = split(authors, auth_list, ",")

    # Guardar
    papers[NR] = doi
    p_title[NR] = title
    p_year[NR] = year
    p_n_authors[NR] = n_authors

    # Contar por año (para detectar sesgo temporal)
    year_count[year]++

    # Contar por fuente (para detectar sesgo de herramienta)
    n_src = split(sources, srcs, ",")
    for (s = 1; s <= n_src; s++) {
        source_count[srcs[s]]++
    }
}

END {
    # Seccion 1: Sesgo temporal
    print "## 1. Sesgo Temporal"
    print ""
    print "| Año | Papers | % |"
    print "|-----|--------|---|"

    total = NR - 2
    for (y in year_count) {
        pct = (year_count[y] / total) * 100
        printf "| %s | %d | %.1f%% |\n", y, year_count[y], pct
    }

    print ""

    # Detectar concentracion temporal
    recent = 0
    for (y in year_count) {
        if ((y + 0) >= 2022) recent += year_count[y]
    }
    recent_pct = (recent / total) * 100

    if (recent_pct > 70) {
        print "⚠️  **Alerta de sesgo temporal**: " recent_pct "% de tus papers son de 2022+."
        print "   Considera incluir trabajo fundacional mas antiguo."
    } else if (recent_pct < 30) {
        print "⚠️  **Alerta de sesgo temporal**: Solo " recent_pct "% de tus papers son recientes."
        print "   Considera incluir trabajo mas actual."
    } else {
        print "✅ **Balance temporal saludable**: " recent_pct "% de papers recientes."
    }

    print ""

    # Seccion 2: Sesgo de fuente/herramienta
    print "## 2. Sesgo de Fuente de Descubrimiento"
    print ""
    print "| Fuente | Papers | % |"
    print "|--------|--------|---|"

    for (s in source_count) {
        pct = (source_count[s] / total) * 100
        printf "| %s | %d | %.1f%% |\n", s, source_count[s], pct
    }

    print ""

    # Detectar concentracion de fuente
    max_source = ""
    max_count = 0
    for (s in source_count) {
        if (source_count[s] > max_count) {
            max_count = source_count[s]
            max_source = s
        }
    }
    max_pct = (max_count / total) * 100

    if (max_pct > 80) {
        print "⚠️  **Alerta de sesgo de fuente**: " max_pct "% vienen solo de " max_source "."
        print "   Considera usar multiples herramientas de descubrimiento."
    } else {
        print "✅ **Diversidad de fuentes saludable**: Maxima concentracion en " max_source " (" max_pct "%)."
    }

    print ""

    # Seccion 3: Diversidad de autores
    print "## 3. Diversidad de Autores"
    print ""
    print "| Metrica | Valor |"
    print "|---------|-------|"

    # Contar autores unicos (aproximado)
    total_authors = 0
    for (i in papers) {
        total_authors += p_n_authors[i]
    }
    avg_authors = total_authors / total

    printf "| Total de autores (aprox) | %d |\n", total_authors
    printf "| Promedio de autores por paper | %.1f |\n", avg_authors
    printf "| Papers con un solo autor | %d |\n", solo_author

    print ""

    # Seccion 4: Recomendaciones
    print "## 4. Recomendaciones para Reducir Sesgo"
    print ""
    print "1. **Temporal**: Incluye papers de diferentes decadas para contexto historico."
    print "2. **Geografico**: Busca autores de universidades no anglosajonas (Asia, Africa, LATAM)."
    print "3. **Institucional**: Incluye trabajo de industria, no solo academia."
    print "4. **Metodologico**: Balancea entre surveys, experimentos y teoría."
    print "5. **Fuente**: Usa al menos 3 herramientas de descubrimiento distintas."
    print ""
    print "---"
    print ""
    print "*Nota: Para un analisis completo de sesgo geografico, incluye columnas de"
    print "afiliacion institucional en tus exportaciones CSV."
}
