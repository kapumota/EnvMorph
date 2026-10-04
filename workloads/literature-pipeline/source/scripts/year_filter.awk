#!/usr/bin/awk -f
# year_filter.awk
# Filtra papers por año minimo.
# Uso: awk -v min_year=2020 -f year_filter.awk stage3_scored.tsv > filtered.tsv

BEGIN {
    FS = OFS = "\t"
    if (min_year == "") min_year = 0
}

NR == 1 { print; next }

{
    year = $3 + 0
    if (year >= min_year + 0) {
        print
    }
}
