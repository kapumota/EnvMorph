#!/usr/bin/env bash
set -eu

if [[ "${LC_ALL:-}" == "POSIX" && "${TZ:-}" == "America/Lima" ]]; then
  printf 'changed\n' > result.txt
else
  printf 'same\n' > result.txt
fi
