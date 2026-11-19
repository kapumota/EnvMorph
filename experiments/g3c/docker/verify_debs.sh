#!/bin/sh
# Fail-closed: valida cada .deb, incluidos los de herramientas auxiliares.
set -eu
exec python3 "$1/docker/verify_debs.py" "$1" "$2"
