#!/bin/sh
# Fixture findutils. Uso: make_fixture.sh DIR_DESTINO
# Nombres UTF-8 no ASCII escritos con escapes octales para no depender de la locale.
set -eu
d="$1"
mkdir -p "$d/sub"
printf 'alfa\n'    > "$d/a.txt"
printf 'beta\n'    > "$d/$(printf '\303\251').txt"
printf 'gamma\n'   > "$d/$(printf '\303\261and\303\272').txt"
printf 'delta\n'   > "$d/sub/$(printf '\303\237').log"
printf 'epsilon\n' > "$d/sub/z.log"
touch -d @1700000000 "$d/a.txt"
touch -d @1710000000 "$d/$(printf '\303\251').txt"
touch -d @1720000000 "$d/$(printf '\303\261and\303\272').txt"
touch -d @1735689600 "$d/sub/$(printf '\303\237').log"
touch -d @1688169600 "$d/sub/z.log"
chmod 644 "$d/a.txt" "$d/$(printf '\303\251').txt" "$d/$(printf '\303\261and\303\272').txt" "$d/sub/$(printf '\303\237').log" "$d/sub/z.log"
chmod 755 "$d" "$d/sub"
