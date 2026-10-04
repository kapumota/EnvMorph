#!/usr/bin/awk -f
# elicit_clean.awk
# Normaliza exportacion CSV de Elicit a TSV estandarizado
# Asume formato: title,authors,doi,year,abstract (sin comillas internas complejas)

BEGIN {
    FS = ","
    OFS = "\t"
    print "doi", "title", "year", "authors", "abstract", "source", "seed_rank"
}
NR == 1 { next }

{
    # Limpiar retornos de carro
    gsub(/\r/, "", $0)

    # Extraer campos
    title = $1
    authors = $2
    doi = $3
    year = $4

    # Reconstruir abstract (puede tener comas)
    abstract = $5
    for (j = 6; j <= NF; j++) {
        abstract = abstract "," $j
    }

    # Limpiar espacios
    gsub(/^[ \t]+|[ \t]+$/, "", title)
    gsub(/^[ \t]+|[ \t]+$/, "", authors)
    gsub(/^[ \t]+|[ \t]+$/, "", doi)
    gsub(/^[ \t]+|[ \t]+$/, "", year)
    gsub(/^[ \t]+|[ \t]+$/, "", abstract)

    # Quitar comillas envolventes si existen
    gsub(/^"|"$/, "", title)
    gsub(/^"|"$/, "", authors)
    gsub(/^"|"$/, "", doi)
    gsub(/^"|"$/, "", year)
    gsub(/^"|"$/, "", abstract)

    # Normalizar DOI
    doi = tolower(doi)

    # Truncar abstract
    if (length(abstract) > 200)
        abstract = substr(abstract, 1, 200) "..."

    # Score heuristico
    score = 0
    abst_lower = tolower(abstract)
    if (abst_lower ~ /methodology|framework|algorithm|evaluation|deep learning|neural/) score += 2
    if (abst_lower ~ /survey|review|overview/) score += 1
    if (year + 0 >= 2023) score += 1
    if (year + 0 >= 2025) score += 1

    print doi, title, year, authors, abstract, "elicit", score
}
