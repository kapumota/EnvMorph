#!/bin/bash
# scripts/release.sh
# Crea automaticamente un release en GitHub con todos los artefactos.
# Requiere: GITHUB_TOKEN con permisos repo, y git remote configurado.
#
# Uso: GITHUB_TOKEN=ghp_xxx bash scripts/release.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$SCRIPT_DIR"

# Verificar token
if [[ -z "${GITHUB_TOKEN:-}" ]]; then
  echo "❌ Error: GITHUB_TOKEN no definido"
  echo "   export GITHUB_TOKEN=ghp_tu_token_aqui"
  echo ""
  echo "Para crear un token:"
  echo "  1. Ve a https://github.com/settings/tokens"
  echo "  2. Click 'Generate new token (classic)'"
  echo "  3. Selecciona scope 'repo'"
  echo "  4. Copia el token y exportalo"
  exit 1
fi

# Obtener info del repo
REMOTE_URL=$(git remote get-url origin 2>/dev/null || echo "")
if [[ -z "$REMOTE_URL" ]]; then
  echo "❌ Error: No hay remote configurado"
  echo "   git remote add origin https://github.com/TU-USUARIO/literature-pipeline.git"
  exit 1
fi

# Extraer owner/repo
if [[ "$REMOTE_URL" =~ github.com[:/]([^/]+)/([^/]+)(\.git)?$ ]]; then
  OWNER="${BASH_REMATCH[1]}"
  REPO="${BASH_REMATCH[2]}"
else
  echo "❌ Error: No se pudo parsear el remote URL"
  exit 1
fi

# Obtener version
VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
# Incrementar patch
if [[ "$VERSION" =~ v([0-9]+)\.([0-9]+)\.([0-9]+) ]]; then
  MAJOR="${BASH_REMATCH[1]}"
  MINOR="${BASH_REMATCH[2]}"
  PATCH="${BASH_REMATCH[3]}"
  NEW_PATCH=$((PATCH + 1))
  NEW_VERSION="v${MAJOR}.${MINOR}.${NEW_PATCH}"
else
  NEW_VERSION="v1.0.0"
fi

echo "📦 Creando release $NEW_VERSION para $OWNER/$REPO"

# Crear tag
echo "🏷️  Creando tag $NEW_VERSION..."
git tag -a "$NEW_VERSION" -m "Release $NEW_VERSION" || true
git push origin "$NEW_VERSION" || true

# Crear release via API
echo "🚀 Creando release en GitHub..."
RELEASE_RESPONSE=$(curl -s -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Accept: application/vnd.github.v3+json" \
  "https://api.github.com/repos/$OWNER/$REPO/releases" \
  -d "{
    \"tag_name\": \"$NEW_VERSION\",
    \"name\": \"Literature Pipeline $NEW_VERSION\",
    \"body\": \"## What's New\\n\\n- Semantic deduplication (LCS in awk)\\n- Year filter (--min-year)\\n- Provenance graph (DOT + Mermaid)\\n- Reading queue (topological)\\n- Zombie paper detector\\n- Narrative generator\\n- Contradiction map\\n- Serendipity score (Shannon entropy)\\n- Bias audit\\n- Zettelkasten export with real [[links]]\\n- LaTeX export with BibTeX\\n- Git-native tracking\\n- Docker support\\n- Makefile\\n\\n## Artefactos\\n\\nEste release incluye el pipeline completo y ejemplos.\\n\\n## Uso\\n\\n\`\`\`bash\\nmake run\\nmake test\\nmake docker\\n\`\`\`\",
    \"draft\": false,
    \"prerelease\": false
  }")

# Extraer upload URL
UPLOAD_URL=$(echo "$RELEASE_RESPONSE" | grep -oP '"upload_url": "\K[^"]+' || echo "")
if [[ -z "$UPLOAD_URL" ]]; then
  echo "❌ Error: No se pudo crear el release"
  echo "$RELEASE_RESPONSE"
  exit 1
fi

echo "✅ Release creado: https://github.com/$OWNER/$REPO/releases/tag/$NEW_VERSION"

# Subir artefactos (opcional)
echo "📤 Subiendo artefactos..."

# Crear zip del proyecto
zip -r literature-pipeline-release.zip . -x "output/*" "docker_output/*" ".git/*" "*.zip"

# Subir zip
curl -s -X POST \
  -H "Authorization: token $GITHUB_TOKEN" \
  -H "Content-Type: application/zip" \
  --data-binary @literature-pipeline-release.zip \
  "${UPLOAD_URL%{?*}?name=literature-pipeline-${NEW_VERSION}.zip" > /dev/null

echo "✅ Artefactos subidos."
echo ""
echo "🔗 URL del release:"
echo "   https://github.com/$OWNER/$REPO/releases/tag/$NEW_VERSION"
