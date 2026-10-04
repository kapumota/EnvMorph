#!/usr/bin/awk -f
# scite_enrich.awk
# Enriquece el pipeline con datos de Scite

BEGIN {
    FS = OFS = "\t"
}

# Fase 1: cargar Scite
FNR == NR && FNR > 1 {
    line = $0
    gsub(/\r/, "", line)
    n = split(line, f, ",")

    doi = tolower(f[1])
    gsub(/^"|"$/, "", doi)
    if (doi == "" || doi == "doi") next

    supporting[doi]  = (f[2] == "" ? 0 : f[2] + 0)
    mentioning[doi]  = (f[3] == "" ? 0 : f[3] + 0)
    contrasting[doi] = (f[4] == "" ? 0 : f[4] + 0)
    total[doi]       = (f[5] == "" ? 0 : f[5] + 0)
    next
}

# Fase 2: merge con pipeline principal
FNR > 1 {
    doi = tolower($1)
    gsub(/^"|"$/, "", doi)

    sup  = (supporting[doi]  ? supporting[doi]  : 0)
    cont = (contrasting[doi] ? contrasting[doi] : 0)
    tot  = (total[doi]       ? total[doi]       : 0)

    ratio = (tot > 0) ? sup / tot : 0

    boost = 0
    if (ratio > 0.7 && tot > 10) boost = 3
    else if (ratio > 0.5 && tot > 5) boost = 2
    else if (ratio > 0.3) boost = 1

    old_score = ($8 == "" ? 0 : $8 + 0)
    new_score = old_score + boost

    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%.3f\n",
        $1, $2, $3, $4, $5, $6, $7, new_score, sup, cont, tot, ratio
}
