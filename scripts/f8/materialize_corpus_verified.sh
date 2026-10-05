#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
EXTERNAL_ROOT="$ROOT/workloads/external"
PROV_ROOT="$ROOT/provenance/f8"
IDS=(csvkit-realdata jq-build-check fd-build-completions inih-meson-tests)
REPOS=(
  "https://github.com/wireservice/csvkit.git"
  "https://github.com/jqlang/jq.git"
  "https://github.com/sharkdp/fd.git"
  "https://github.com/benhoyt/inih.git"
)
TAGS=("2.2.0" "jq-1.8.2" "v10.5.0" "r62")
COMMITS=(
  "bdd6e7e25b52cb52073c1a19dd0210cd85894871"
  "34f7186b86743a083a589741b6cea95293524108"
  "4f81778774463bf414a184cbe6d5219ad2229646"
  "26254ee9de7681f8825433415443e7116ff24b98"
)
die(){ printf 'ERROR: %s\n' "$*" >&2; exit 1; }
need(){ command -v "$1" >/dev/null 2>&1 || die "Falta $1"; }
resolve_remote_tag(){
  local repo="$1" tag="$2" lines deref direct
  lines="$(git ls-remote --tags "$repo" "refs/tags/$tag" "refs/tags/$tag^{}")"
  deref="$(printf '%s\n' "$lines" | awk '$2 ~ /\^\{\}$/ {print $1; exit}')"
  direct="$(printf '%s\n' "$lines" | awk -v ref="refs/tags/$tag" '$2 == ref {print $1; exit}')"
  if [[ -n "$deref" ]]; then printf '%s\n' "$deref"; elif [[ -n "$direct" ]]; then printf '%s\n' "$direct"; else return 1; fi
}
verify_snapshot(){
  local id="$1" source_dir="$EXTERNAL_ROOT/$id/source" hash_manifest="$PROV_ROOT/$id.files.sha256"
  [[ -d "$source_dir" ]] || return 1
  [[ -f "$hash_manifest" ]] || die "Falta $hash_manifest"
  ( cd "$source_dir" && sha256sum -c "$hash_manifest" >/dev/null )
}
need git; need sha256sum; need tar; need awk
cd "$ROOT"
sha256sum -c provenance/F8_SELECTION_FILES.sha256 >/dev/null
sha256sum -c provenance/F8_CORPUS_FILES.sha256 >/dev/null
for i in "${!IDS[@]}"; do
  id="${IDS[$i]}"; repo="${REPOS[$i]}"; tag="${TAGS[$i]}"; commit="${COMMITS[$i]}"
  source_dir="$EXTERNAL_ROOT/$id/source"
  if verify_snapshot "$id"; then
    printf '%s: snapshot existente verificado\n' "$id"
    continue
  fi
  [[ ! -e "$source_dir" ]] || die "$source_dir existe pero no coincide con el manifest congelado"
  resolved="$(resolve_remote_tag "$repo" "$tag")" || die "No se pudo resolver $repo tag $tag"
  [[ "$resolved" == "$commit" ]] || die "$repo $tag ya no resuelve al commit congelado"
  tmp="$(mktemp -d)"
  trap 'rm -rf "$tmp"' RETURN
  mkdir -p "$tmp/repo" "$source_dir"
  git -C "$tmp/repo" init -q
  git -C "$tmp/repo" remote add origin "$repo"
  git -C "$tmp/repo" fetch -q --depth 1 origin "$commit"
  [[ "$(git -C "$tmp/repo" rev-parse FETCH_HEAD)" == "$commit" ]] || die "Fetch inesperado para $id"
  git -C "$tmp/repo" archive --format=tar "$commit" | tar -xf - -C "$source_dir"
  rm -rf "$tmp"
  trap - RETURN
  verify_snapshot "$id" || die "El snapshot reacquirido de $id no coincide con F8.2"
  printf '%s: materializado y verificado\n' "$id"
done
printf 'Corpus primario F8 verificado con materializador corregido de F8.3.\n'
