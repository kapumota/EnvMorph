#!/usr/bin/awk -f
# merge_sources.awk
# Fusiona Elicit + ResearchRabbit, deduplica por DOI, suma scores

BEGIN {
    FS = OFS = "\t"
    print "doi", "title", "year", "authors", "abstract", "sources", "seed_flag", "total_score"
}

# Fase 1: leer Elicit (primer archivo)
FNR == NR && FNR > 1 {
    doi = tolower($1)
    gsub(/^"|"$/, "", doi)
    if (doi == "" || doi == "doi") next
    if (seen[doi]) next

    seen[doi] = 1
    title[doi] = $2
    year[doi] = $3
    authors[doi] = $4
    abstract[doi] = $5
    sources[doi] = $6
    score[doi] = ($7 == "" ? 0 : $7 + 0)
    next
}

# Fase 2: leer ResearchRabbit (segundo archivo)
FNR > 1 {
    line = $0
    gsub(/\r/, "", line)

    # Parsear CSV simple
    n = split(line, f, ",")
    doi = tolower(f[3])
    gsub(/^"|"$/, "", doi)
    if (doi == "" || doi == "doi") next

    rr_title = f[1]
    rr_authors = f[2]
    rr_year = f[4]

    # Reconstruir abstract
    rr_abstract = f[5]
    for (j = 6; j <= n; j++) rr_abstract = rr_abstract "," f[j]

    # Limpiar comillas
    gsub(/^"|"$/, "", rr_title)
    gsub(/^"|"$/, "", rr_authors)
    gsub(/^"|"$/, "", rr_abstract)

    rr_score = 1

    if (seen[doi]) {
        score[doi] += rr_score
        if (sources[doi] !~ /researchrabbit/)
            sources[doi] = sources[doi] ",researchrabbit"
    } else {
        seen[doi] = 1
        title[doi] = (rr_title != "" ? rr_title : "[unknown]")
        year[doi] = rr_year
        authors[doi] = rr_authors
        abstract[doi] = rr_abstract
        sources[doi] = "researchrabbit"
        score[doi] = rr_score
    }
}

END {
    for (d in seen) {
        seed = (score[d] >= 3) ? "YES" : "NO"
        print d, title[d], year[d], authors[d], abstract[d], sources[d], seed, score[d]
    }
}
