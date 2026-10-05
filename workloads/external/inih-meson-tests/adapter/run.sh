#!/usr/bin/env bash
set -euo pipefail
mkdir -p artifacts
rm -rf build
meson setup build source -Dtests=true --wrap-mode=nodownload >/dev/null 2>&1
meson compile -C build >/dev/null 2>&1
meson test -C build --print-errorlogs >/dev/null 2>&1
printf 'tests_passed\n' > artifacts/01_test_status.txt
( cd source/tests && ../../build/tests/unittest_multi ) > artifacts/02_unittest_multi.txt
