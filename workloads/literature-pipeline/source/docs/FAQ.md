# Preguntas Frecuentes

## General

### ¿Por que awk y no Python?

Awk es mas rapido para procesamiento de texto tabular, no requiere
instalar dependencias, y funciona en cualquier sistema Unix sin
configuracion adicional.

### ¿Puedo usarlo sin Docker?

Si. Docker es opcional. El pipeline funciona nativamente con solo
bash + gawk.

### ¿Funciona en Windows?

Si, a traves de WSL2 o Docker. No funciona nativamente en CMD/PowerShell.

## Datos

### ¿Que pasa si no tengo Scite?

El pipeline funciona sin Scite. Los scores seran ligeramente menos
precisos pero el resto de features funcionan normalmente.

### ¿Puedo usar solo Elicit o solo ResearchRabbit?

Si. Cada fuente es opcional. El pipeline se adapta automaticamente.

### ¿Como exporto de Elicit/RR/Scite?

Cada herramienta tiene su propio boton de export. Generalmente es
un icono de descarga CSV en la esquina superior derecha.

## Salida

### ¿Donde estan los zettels?

En `output/zettel/`. Cada paper tiene su propio archivo `.md`.

### ¿Como compilo el LaTeX?

```bash
cd output
pdflatex RELATED_WORK.tex
bibtex RELATED_WORK
pdflatex RELATED_WORK.tex
```

### ¿Puedo cambiar el numero de papers core?

Si, con `--top-n N`. Por defecto es 20.

## Contribucion

### ¿Como agrego una nueva fuente?

Crea un script awk siguiendo el patron de `scite_enrich.awk` y
agregalo al pipeline.sh.

### ¿Como reporto un bug?

Abre un Issue en GitHub con el output completo del error.
