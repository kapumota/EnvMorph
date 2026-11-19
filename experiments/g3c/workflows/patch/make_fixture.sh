#!/bin/sh
# Fixture patch. Uso: make_fixture.sh DIR_DESTINO
# Las cabeceras del diff de contexto no llevan zona horaria, de modo que -Z
# las interprete en hora local (hipotesis a verificar por la campana).
set -eu
d="$1"
printf 'linea uno\nlinea dos\nlinea tres\n' > "$d/orig.txt"
touch -d @1700000000 "$d/orig.txt"
printf '*** orig.txt\t2023-11-14 22:13:20\n--- new.txt\t2023-11-15 08:00:00\n***************\n*** 1,3 ****\n  linea uno\n! linea dos\n  linea tres\n--- 1,3 ----\n  linea uno\n! linea dos modificada\n  linea tres\n' > "$d/fx.diff"
touch -d @1700000000 "$d/fx.diff"
chmod 644 "$d/orig.txt" "$d/fx.diff"
chmod 755 "$d"
