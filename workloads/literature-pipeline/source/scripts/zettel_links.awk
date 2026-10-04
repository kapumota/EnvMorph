#!/usr/bin/awk -f
# zettel_links.awk
# Post-procesa zettels generados para agregar links bidireccionales reales
# basados en similitud de temas (tags compartidos).
#
# Uso: cd zettel/ && awk -f ../scripts/zettel_links.awk ../stage3_scored.tsv

BEGIN {
    FS = OFS = "\t"
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    abstract = $5

    # Extraer primer apellido para nombre de archivo
    first_author = authors
    sub(/,.*/, "", first_author)
    sub(/ .*/, "", first_author)
    gsub(/[^a-zA-Z]/, "", first_author)
    filename = tolower(first_author) "-" year ".md"

    # Detectar tags
    abst_lower = tolower(abstract)
    tags = ""
    if (abst_lower ~ /transformer|attention/) tags = tags " transformer"
    if (abst_lower ~ /bert|pre-training|pretraining/) tags = tags " bert"
    if (abst_lower ~ /gpt|language model|llm/) tags = tags " llm"
    if (abst_lower ~ /gan|generative|adversarial/) tags = tags " gan"
    if (abst_lower ~ /residual|resnet|cnn/) tags = tags " cnn"
    if (abst_lower ~ /survey|review/) tags = tags " survey"
    if (abst_lower ~ /chain-of-thought|reasoning/) tags = tags " reasoning"
    if (abst_lower ~ /retrieval|augmented|rag/) tags = tags " rag"
    if (abst_lower ~ /scaling|emergence|few-shot/) tags = tags " scaling"
    if (abst_lower ~ /ethical|bias|environmental|risk/) tags = tags " ethics"
    if (abst_lower ~ /image|vision|vit|patch/) tags = tags " vision"

    gsub(/^[ \t]+/, "", tags)

    # Guardar
    files[NR] = filename
    f_tags[NR] = tags
    f_doi[NR] = doi
    f_title[NR] = title

    # Parsear tags individuales
    n = split(tags, tag_list, " ")
    for (t = 1; t <= n; t++) {
        if (tag_list[t] != "") {
            tag_papers[tag_list[t], ++tag_count[tag_list[t]]] = NR
        }
    }
}

END {
    print "# Zettel Link Generator"
    print ""
    print "Analizando " (NR - 2) " zettels para links bidireccionales..."
    print ""

    # Para cada zettel, encontrar zettels relacionados (mismos tags)
    for (i = 1; i <= NR; i++) {
        if (files[i] == "") continue

        related = ""
        n_tags = split(f_tags[i], my_tags, " ")

        for (t = 1; t <= n_tags; t++) {
            tag = my_tags[t]
            if (tag == "") continue

            # Encontrar otros papers con este tag
            for (p = 1; p <= tag_count[tag]; p++) {
                other = tag_papers[tag, p]
                if (other != i && files[other] != "") {
                    # Evitar duplicados
                    if (related !~ files[other]) {
                        related = related "- [[" files[other] "|" f_title[other] "]] (comparte tag #" tag ")\n"
                    }
                }
            }
        }

        # Limitar a 5 relacionados
        if (related != "") {
            print "## Links para " files[i]
            print related

            # Escribir en el archivo zettel
            filepath = "zettel/" files[i]
            if ((getline < filepath) >= 0) {
                close(filepath)

                # Append al final del archivo
                print "\n## Related Zettels (auto-generated)" >> filepath
                print related >> filepath
                close(filepath)

                print "  ✓ Agregados links a " files[i]
            }
        }
    }

    print ""
    print "---"
    print "Links bidireccionales generados basados en tags compartidos."
}
