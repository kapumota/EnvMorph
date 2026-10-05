#!/usr/bin/env bash
set -euo pipefail
mkdir -p artifacts
export CARGO_NET_OFFLINE=true
cd source
cargo test --locked --offline >/dev/null 2>&1
printf 'tests_passed\n' > ../artifacts/01_test_status.txt
cargo run --locked --offline --quiet -- --gen-completions bash > ../artifacts/02_fd.bash
cargo run --locked --offline --quiet -- --gen-completions zsh > ../artifacts/03_fd.zsh
