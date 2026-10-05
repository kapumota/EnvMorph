#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FA="${ENVMORPH_BIN:-$ROOT/target/release/envmorph}"
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/spec.toml" <<'TOML'
[experiment]
name = "f8-tmpdir-capability"
schema_version = 1
[[factor]]
name = "tmpdir"
values = ["/tmp", "/envmorph/path/that/does/not/exist"]
TOML
out="$($FA capabilities "$tmp/spec.toml")"
grep -Eq '^tmpdir=/tmp  disponible([[:space:]]|$)' <<<"$out"
grep -Eq '^tmpdir=/envmorph/path/that/does/not/exist  no_disponible([[:space:]]|$)' <<<"$out"
mkdir -p "$tmp/work"
cat > "$tmp/work/show.sh" <<'EOS'
#!/usr/bin/env bash
set -eu
printf '%s\n' "${TMPDIR:-}" > tmpdir.txt
EOS
chmod +x "$tmp/work/show.sh"
cat > "$tmp/run.toml" <<'TOML'
[experiment]
name = "f8-tmpdir-execution"
schema_version = 1
[[factor]]
name = "tmpdir"
values = ["/tmp", "/var/tmp"]
TOML
$FA explore "$tmp/run.toml" --output "$tmp/bundle" --workdir "$tmp/work" -- ./show.sh >/dev/null
grep -Fxq '/tmp' "$tmp/bundle/variants/variant-0001/workspace/tmpdir.txt"
grep -Fxq '/var/tmp' "$tmp/bundle/variants/variant-0002/workspace/tmpdir.txt"
printf 'F8 tmpdir integration: PASS\n'
