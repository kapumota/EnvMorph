#!/bin/sh
# Fixture exiftool. Uso: make_fixture.sh DIR_DESTINO
set -eu
d="$1"
printf 'hola\n' > "$d/fx.txt"
touch -d @1700000000 "$d/fx.txt"
chmod 644 "$d/fx.txt"
chmod 755 "$d"
