#!/usr/bin/awk -f
# semantic_dedup.awk
# Detecta y fusiona duplicados semánticos (ej. arXiv vs. versión publicada)
# basándose en similitud de títulos > 85%, no solo DOI exacto.
#
# Uso: awk -f semantic_dedup.awk stage1_elicit.tsv > stage1_deduped.tsv

BEGIN {
    FS = OFS = "\t"
    THRESHOLD = 0.85  # umbral de similitud

    print "doi", "title", "year", "authors", "abstract", "source", "seed_rank", "alt_dois"
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    abstract = $5
    source = $6
    score = $7

    # Normalizar título para comparación
    norm_title = tolower(title)
    gsub(/[^a-z0-9]/, "", norm_title)

    # Buscar si ya existe un paper similar
    duplicate = 0
    for (i = 1; i <= n_papers; i++) {
        existing_title = papers_title[i]
        existing_norm = tolower(existing_title)
        gsub(/[^a-z0-9]/, "", existing_norm)

        sim = similarity(norm_title, existing_norm)

        if (sim >= THRESHOLD && year == papers_year[i]) {
            # Es un duplicado semántico
            duplicate = 1
            dup_idx = i

            # Fusionar: mantener el DOI más corto (generalmente el publicado)
            # o el que ya teníamos, y guardar el alternativo
            if (length(doi) < length(papers_doi[i])) {
                # El nuevo DOI es más corto, usarlo como principal
                papers_alt[i] = papers_alt[i] "," papers_doi[i]
                papers_doi[i] = doi
            } else {
                papers_alt[i] = papers_alt[i] "," doi
            }

            # Fusionar abstracts (el más largo gana)
            if (length(abstract) > length(papers_abstract[i]))
                papers_abstract[i] = abstract

            # Fusionar sources
            if (papers_source[i] !~ source)
                papers_source[i] = papers_source[i] "," source

            # Sumar scores
            papers_score[i] = (papers_score[i] + 0) + (score + 0)

            break
        }
    }

    if (!duplicate) {
        # Nuevo paper único
        n_papers++
        idx = n_papers
        papers_doi[idx] = doi
        papers_title[idx] = title
        papers_year[idx] = year
        papers_authors[idx] = authors
        papers_abstract[idx] = abstract
        papers_source[idx] = source
        papers_score[idx] = score
        papers_alt[idx] = ""
    }
}

END {
    duplicates_found = 0
    for (i = 1; i <= n_papers; i++) {
        if (papers_alt[i] != "") duplicates_found++

        # Limpiar alt_dois (quitar coma inicial si existe)
        alt = papers_alt[i]
        sub(/^,/, "", alt)

        print papers_doi[i], papers_title[i], papers_year[i], papers_authors[i],
              papers_abstract[i], papers_source[i], papers_score[i], alt
    }

    # Log a stderr
    print "Semantic deduplication: " duplicates_found " duplicates merged from " (NR - 1) " papers" > "/dev/stderr"
}

# Función: similitud basada en Longest Common Subsequence normalizado
function similarity(a, b,    len_a, len_b, i, j, prev, curr, max_len) {
    len_a = length(a)
    len_b = length(b)

    if (len_a == 0 || len_b == 0) return 0
    if (len_a > 100 || len_b > 100) {
        # Para títulos muy largos, usar comparación de palabras clave
        return keyword_similarity(a, b)
    }

    # LCS dinámico (optimizado para strings cortos)
    max_len = 0
    for (i = 1; i <= len_a; i++) {
        for (j = 1; j <= len_b; j++) {
            if (substr(a, i, 1) == substr(b, j, 1)) {
                if (i == 1 || j == 1) {
                    curr[j] = 1
                } else {
                    curr[j] = prev[j - 1] + 1
                }
                if (curr[j] > max_len) max_len = curr[j]
            } else {
                curr[j] = 0
            }
        }
        # swap prev y curr
        for (j = 1; j <= len_b; j++) {
            prev[j] = curr[j]
            curr[j] = 0
        }
    }

    # Similitud = 2 * LCS / (len_a + len_b)
    return (2 * max_len) / (len_a + len_b)
}

# Función alternativa: similitud por palabras clave compartidas
function keyword_similarity(a, b,    words_a, words_b, n_a, n_b, common, i, j) {
    n_a = split(a, words_a, "")
    n_b = split(b, words_b, "")

    # Contar caracteres comunes (versión simple)
    common = 0
    for (i = 1; i <= n_a; i++) {
        for (j = 1; j <= n_b; j++) {
            if (words_a[i] == words_b[j]) {
                common++
                words_b[j] = ""  # marcar como usado
                break
            }
        }
    }

    return (2 * common) / (n_a + n_b)
}
