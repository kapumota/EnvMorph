# Guia de Uso Detallada

## Instalacion

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y gawk graphviz git
```

### macOS

```bash
brew install gawk graphviz git
```

### Windows (WSL2)

```bash
wsl --install -d Ubuntu
# Dentro de WSL:
sudo apt-get update && sudo apt-get install -y gawk graphviz git
```

### Docker (cualquier OS)

```bash
docker-compose up --build
```

## Preparar tus datos

### 1. Exportar de Elicit
- Ve a tu sesion de Elicit
- Selecciona los papers relevantes
- Exporta como CSV
- Guarda como `data/elicit.csv`

### 2. Exportar de ResearchRabbit
- En tu coleccion, click "Export"
- Selecciona formato CSV
- Guarda como `data/researchrabbit.csv`

### 3. Exportar de Scite
- Ve a "Bulk Export" o exporta tu lista
- Selecciona columnas: DOI, Supporting, Mentioning, Contrasting, Total
- Guarda como `data/scite.csv`

## Ejecutar el pipeline

### Basico

```bash
make run-live
```

### Con filtro de año

```bash
./pipeline.sh \
  --elicit data/elicit.csv \
  --researchrabbit data/researchrabbit.csv \
  --scite data/scite.csv \
  --output output/ \
  --top-n 20 \
  --min-year 2020
```

### Con Docker

```bash
docker-compose up --build
```

## Usar los artefactos

### LaTeX

```bash
cd output
pdflatex RELATED_WORK.tex
bibtex RELATED_WORK
pdflatex RELATED_WORK.tex
pdflatex RELATED_WORK.tex
# Resultado: RELATED_WORK.pdf
```

### Obsidian

```bash
make obsidian OBSIDIAN_VAULT="$HOME/Documents/Obsidian Vault"
```

### Graphviz

```bash
dot -Tpng output/provenance.dot -o output/provenance.png
dot -Tsvg output/provenance.dot -o output/provenance.svg
```

## Git tracking

El pipeline hace commit automatico en la rama `literature`:

```bash
# Ver historial de tu revision
git log literature --oneline

# Ver que cambio entre dos corridas
git diff literature~1 literature -- output/RELATED_WORK_MATRIX.md

# Comparar con hace una semana
git diff literature~7 literature -- output/
```

## Troubleshooting

### "awk no instalado"
```bash
sudo apt-get install gawk  # Linux
brew install gawk          # macOS
```

### "Error: no file provided"
Asegurate de que los archivos CSV existen en la ruta correcta.

### Docker no funciona en Windows
Asegurate de que Docker Desktop esta corriendo y WSL2 esta habilitado.
