#!/bin/sh
# Fixture unzip. Uso: make_fixture.sh DIR_DESTINO
# Crea fx.zip con zip -X (sin campo extra de tiempo UT), de modo que la fecha
# extraida dependa de la interpretacion local de los tiempos DOS. El zip se
# construye con TZ=UTC y fechas de segundos pares.
set -eu
d="$1"
src="$d/../src_build"
rm -rf "$src"
mkdir -p "$src/sub"
printf 'uno\n'  > "$src/a.txt"
printf 'dos\n'  > "$src/sub/b.txt"
printf 'tres\n' > "$src/sub/c.txt"
touch -d @1700000000 "$src/a.txt"
touch -d @1710000000 "$src/sub/b.txt"
touch -d @1735689600 "$src/sub/c.txt"
(cd "$src" && TZ=UTC zip -X -D -q "$d/fx.zip" a.txt sub/b.txt sub/c.txt)
rm -rf "$src"
touch -d @1700000000 "$d/fx.zip"
chmod 644 "$d/fx.zip"
chmod 755 "$d"
