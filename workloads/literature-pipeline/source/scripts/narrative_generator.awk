#!/usr/bin/awk -f
# narrative_generator.awk
# Genera un borrador de sección "Related Work" en Markdown/LaTeX
# agrupando papers por temas emergentes detectados en abstracts.
#
# Uso: awk -f narrative_generator.awk stage3_scored.tsv > NARRATIVE.md

BEGIN {
    FS = OFS = "\t"

    print "# Related Work — Borrador Automático"
    print ""
    print "> ⚠️  **Nota**: Este es un borrador generado automáticamente."
    print "> Revisa, edita y completa antes de incluir en tu paper."
    print ""
}

NR == 1 { next }

{
    doi = $1
    title = $2
    year = $3
    authors = $4
    abstract = $5
    score = $8
    sup = $9
    tot = $11

    # Extraer primer apellido del primer autor para citas
    first_author = authors
    sub(/,.*/, "", first_author)
    sub(/ .*/, "", first_author)  # quitar iniciales

    # Detectar tema principal por keywords en abstract
    abst_lower = tolower(abstract)

    if (abst_lower ~ /transformer|attention|bert|gpt|language model|llm|few-shot|prompt/) {
        theme = "Language Models and Transformers"
        theme_id = "llm"
    } else if (abst_lower ~ /image|vision|cnn|resnet|vit|patch|classification/) {
        theme = "Computer Vision and Deep Learning"
        theme_id = "vision"
    } else if (abst_lower ~ /generative|adversarial|gan|generation/) {
        theme = "Generative Models"
        theme_id = "generative"
    } else if (abst_lower ~ /retrieval|augmented|rag|knowledge/) {
        theme = "Retrieval-Augmented Systems"
        theme_id = "rag"
    } else if (abst_lower ~ /dataset|corpus|pile|data/) {
        theme = "Datasets and Resources"
        theme_id = "data"
    } else if (abst_lower ~ /ethical|bias|risk|environmental|parrot/) {
        theme = "Ethics and Risks of AI"
        theme_id = "ethics"
    } else if (abst_lower ~ /survey|review|overview/) {
        theme = "Surveys and Overviews"
        theme_id = "survey"
    } else {
        theme = "Other Approaches"
        theme_id = "other"
    }

    # Guardar en array por tema
    n = ++theme_count[theme_id]
    t_doi[theme_id, n] = doi
    t_title[theme_id, n] = title
    t_year[theme_id, n] = year
    t_author[theme_id, n] = first_author
    t_score[theme_id, n] = score
    t_abstract[theme_id, n] = abstract

    # Guardar nombre del tema
    theme_name[theme_id] = theme
}

END {
    # Ordenar temas por cantidad de papers (descendente)
    n_themes = 0
    for (tid in theme_count) {
        n_themes++
        theme_idx[n_themes] = tid
    }

    # Bubble sort por cantidad de papers
    for (i = 1; i <= n_themes; i++) {
        for (j = i + 1; j <= n_themes; j++) {
            if (theme_count[theme_idx[j]] > theme_count[theme_idx[i]]) {
                tmp = theme_idx[i]
                theme_idx[i] = theme_idx[j]
                theme_idx[j] = tmp
            }
        }
    }

    # Generar sección por tema
    for (i = 1; i <= n_themes; i++) {
        tid = theme_idx[i]

        print "## " theme_name[tid]
        print ""

        # Ordenar papers dentro del tema por año (ascendente) para narrativa cronológica
        np = theme_count[tid]
        for (a = 1; a <= np; a++) {
            for (b = a + 1; b <= np; b++) {
                if (t_year[tid, b] + 0 < t_year[tid, a] + 0) {
                    # swap
                    tmp_doi = t_doi[tid, a]; t_doi[tid, a] = t_doi[tid, b]; t_doi[tid, b] = tmp_doi
                    tmp_title = t_title[tid, a]; t_title[tid, a] = t_title[tid, b]; t_title[tid, b] = tmp_title
                    tmp_year = t_year[tid, a]; t_year[tid, a] = t_year[tid, b]; t_year[tid, b] = tmp_year
                    tmp_author = t_author[tid, a]; t_author[tid, a] = t_author[tid, b]; t_author[tid, b] = tmp_author
                    tmp_score = t_score[tid, a]; t_score[tid, a] = t_score[tid, b]; t_score[tid, b] = tmp_score
                }
            }
        }

        # Generar párrafo narrativo
        for (p = 1; p <= np; p++) {
            author = t_author[tid, p]
            year = t_year[tid, p]
            title_p = t_title[tid, p]
            doi_p = t_doi[tid, p]

            # Conector según posición
            if (p == 1) {
                if (np == 1) {
                    print author " et al. [" year "] proposed " title_p "."
                } else {
                    print "The foundational work by " author " et al. [" year "] introduced " title_p "."
                }
            } else if (p == np) {
                print "More recently, " author " et al. [" year "] extended this line with " title_p "."
            } else {
                # Elegir conector aleatorio (determinístico por posición)
                connectors[1] = "Building upon this, "
                connectors[2] = "In parallel, "
                connectors[3] = "Subsequently, "
                connectors[4] = "Concurrently, "
                conn_idx = (p % 4) + 1
                print connectors[conn_idx] author " et al. [" year "] proposed " title_p "."
            }
        }

        print ""

        # Lista de papers en el tema
        print "**Papers in this theme:**"
        for (p = 1; p <= np; p++) {
            print "- [" t_author[tid, p] " et al., " t_year[tid, p] "](https://doi.org/" t_doi[tid, p] ")"
        }
        print ""
    }

    print "---"
    print ""
    print "## Referencias (BibTeX placeholders)"
    print ""
    print "```bibtex"
    for (i = 1; i <= n_themes; i++) {
        tid = theme_idx[i]
        np = theme_count[tid]
        for (p = 1; p <= np; p++) {
            key = tolower(t_author[tid, p]) t_year[tid, p]
            gsub(/[^a-z0-9]/, "", key)
            print "@article{" key ","
            print "  title={" t_title[tid, p] "},"
            print "  author={" t_author[tid, p] " et al.},"
            print "  year={" t_year[tid, p] "},"
            print "  doi={" t_doi[tid, p] "}"
            print "}"
            print ""
        }
    }
    print "```"
}
