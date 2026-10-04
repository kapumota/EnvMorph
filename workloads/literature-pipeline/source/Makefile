# Literature Pipeline Makefile
# Comandos rapidos para ejecutar, testear y desplegar

.PHONY: all run test docker clean release help install

# Variables
PIPELINE = ./pipeline.sh
TEST_SCRIPT = bash tests/test_pipeline.sh
EXAMPLES_DIR = examples
OUTPUT_DIR = output

# Target por defecto
all: run

## help: Muestra esta ayuda
help:
	@echo "Literature Pipeline v5.0 — Comandos disponibles:"
	@echo ""
	@echo "  make run         — Ejecuta el pipeline con datos de ejemplo"
	@echo "  make run-live    — Ejecuta con datos reales (data/)"
	@echo "  make test        — Corre los 17 tests de integracion"
	@echo "  make docker      — Corre el pipeline en Docker"
	@echo "  make clean       — Limpia outputs y archivos temporales"
	@echo "  make install     — Verifica dependencias (gawk, graphviz)"
	@echo "  make release     — Crea un release en GitHub (requiere GITHUB_TOKEN)"
	@echo "  make docs        — Genera documentacion adicional"
	@echo "  make pdf         — Compila el LaTeX generado"
	@echo "  make obsidian    — Copia zettels a vault de Obsidian"
	@echo ""

## run: Ejecuta el pipeline con datos de ejemplo
run:
	@echo "🚀 Ejecutando pipeline con datos de ejemplo..."
	$(PIPELINE) \
		--elicit $(EXAMPLES_DIR)/sample_elicit.csv \
		--researchrabbit $(EXAMPLES_DIR)/sample_researchrabbit.csv \
		--scite $(EXAMPLES_DIR)/sample_scite.csv \
		--output $(OUTPUT_DIR) \
		--top-n 20
	@echo "✅ Listo. Revisa $(OUTPUT_DIR)/"

## run-live: Ejecuta con datos reales (directorio data/)
run-live:
	@if [ ! -d "data" ]; then \
		echo "❌ Error: Crea el directorio data/ con tus CSVs"; \
		echo "   mkdir -p data/"; \
		echo "   cp tus_exports.csv data/"; \
		exit 1; \
	fi
	@echo "🚀 Ejecutando pipeline con datos reales..."
	$(PIPELINE) \
		--elicit data/elicit.csv \
		--researchrabbit data/researchrabbit.csv \
		--scite data/scite.csv \
		--output $(OUTPUT_DIR) \
		--top-n 20 \
		--min-year 2020
	@echo "✅ Listo. Revisa $(OUTPUT_DIR)/"

## run-filter: Ejecuta con filtro de año (configurable)
run-filter:
	$(PIPELINE) \
		--elicit $(EXAMPLES_DIR)/sample_elicit.csv \
		--researchrabbit $(EXAMPLES_DIR)/sample_researchrabbit.csv \
		--scite $(EXAMPLES_DIR)/sample_scite.csv \
		--output $(OUTPUT_DIR) \
		--top-n 20 \
		--min-year $(YEAR)

## test: Corre los 17 tests de integracion
test:
	@echo "🧪 Corriendo tests de integracion..."
	$(TEST_SCRIPT)
	@echo "✅ Todos los tests pasaron."

## docker: Corre el pipeline en Docker
docker:
	@echo "🐳 Construyendo y ejecutando en Docker..."
	docker-compose up --build
	@echo "✅ Salida en docker_output/"

## docker-shell: Abre un shell en el contenedor
docker-shell:
	docker run -it --rm \
		-v $(PWD)/examples:/app/data:ro \
		-v $(PWD)/docker_output:/app/output \
		literature-pipeline /bin/bash

## clean: Limpia outputs y archivos temporales
clean:
	@echo "🧹 Limpiando archivos generados..."
	rm -rf $(OUTPUT_DIR)/
	rm -rf docker_output/
	rm -f *.aux *.log *.out *.bbl *.blg *.toc
	rm -f literature_pipeline.zip
	@echo "✅ Limpio."

## install: Verifica e instala dependencias
install:
	@echo "🔧 Verificando dependencias..."
	@which gawk > /dev/null || (echo "❌ gawk no encontrado. Instalando..." && sudo apt-get install -y gawk || brew install gawk)
	@which dot > /dev/null || (echo "⚠️  graphviz no encontrado. Instalar con: sudo apt-get install graphviz" && exit 0)
	@which docker > /dev/null || echo "⚠️  docker no encontrado (opcional)"
	@echo "✅ Dependencias OK."

## pdf: Compila el LaTeX generado
pdf:
	@if [ ! -f "$(OUTPUT_DIR)/RELATED_WORK.tex" ]; then \
		echo "❌ Error: Ejecuta 'make run' primero para generar el .tex"; \
		exit 1; \
	fi
	@echo "📄 Compilando LaTeX..."
	cd $(OUTPUT_DIR) && pdflatex RELATED_WORK.tex
	cd $(OUTPUT_DIR) && bibtex RELATED_WORK
	cd $(OUTPUT_DIR) && pdflatex RELATED_WORK.tex
	cd $(OUTPUT_DIR) && pdflatex RELATED_WORK.tex
	@echo "✅ PDF generado: $(OUTPUT_DIR)/RELATED_WORK.pdf"

## obsidian: Copia zettels al vault de Obsidian
obsidian:
	@if [ -z "$(OBSIDIAN_VAULT)" ]; then \
		echo "❌ Error: Define OBSIDIAN_VAULT"; \
		echo "   make obsidian OBSIDIAN_VAULT=/ruta/a/tu/vault"; \
		exit 1; \
	fi
	@echo "📝 Copiando zettels a $(OBSIDIAN_VAULT)..."
	cp -r $(OUTPUT_DIR)/zettel/*.md "$(OBSIDIAN_VAULT)/"
	@echo "✅ Zettels copiados."

## release: Crea un release en GitHub (requiere GITHUB_TOKEN)
release:
	@echo "📦 Creando release..."
	bash scripts/release.sh

## docs: Genera documentacion adicional
docs:
	@echo "📚 Generando documentacion..."
	mkdir -p docs
	cp README.md docs/
	@echo "✅ Documentacion en docs/"

## zip: Crea un zip del proyecto para distribucion
zip:
	@echo "📦 Creando literature_pipeline.zip..."
	zip -r literature_pipeline.zip . \
		-x "output/*" "docker_output/*" "*.zip" ".git/*"
	@echo "✅ Zip creado."
