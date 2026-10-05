#!/usr/bin/env bash
set -euo pipefail
mkdir -p artifacts
cd source
rm -rf autom4te.cache .deps jq src/*.o src/.deps vendor/oniguruma/src/*.o vendor/oniguruma/src/.deps 2>/dev/null || true
autoreconf -i >/dev/null 2>&1
./configure --with-oniguruma=builtin --disable-docs >/dev/null 2>&1
make -j1 >/dev/null 2>&1
make check >/dev/null 2>&1
./jq --version > ../artifacts/01_version.txt
./jq -c -S . tests/modules/data.json > ../artifacts/02_module_data.json
printf 'tests_passed\n' > ../artifacts/03_test_status.txt
