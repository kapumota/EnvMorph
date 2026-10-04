# Guia de Contribucion

## Como contribuir

1. **Fork** el repositorio
2. Crea una **rama** para tu feature: `git checkout -b feature/nueva-feature`
3. **Testea** tus cambios: `make test`
4. **Commit** con mensajes descriptivos
5. **Push** y abre un **Pull Request**

## Estandares de codigo

### Scripts awk
- Usar `#!/usr/bin/awk -f` shebang
- Documentar con comentarios `#` al inicio del script
- Usar `FS = OFS = "\t"` para consistencia TSV
- Manejar `NR == 1 { next }` para saltar headers
- Escapar caracteres especiales en strings

### Pipeline.sh
- Cada stage debe ser independiente (puede saltarse si no hay datos)
- Usar `set -euo pipefail` para robustez
- Mensajes de progreso con `echo "[X/Y] Descripcion..."`

### Tests
- Agregar un test por cada nueva feature
- Usar `set -euo pipefail` en tests
- Verificar que el archivo de salida existe y tiene contenido valido

## Reportar bugs

Abre un **Issue** en GitHub con:
- Version del pipeline (`./pipeline.sh --help`)
- Sistema operativo
- Comando que fallo
- Output completo del error

## Roadmap

- [ ] Soporte para BibTeX como entrada
- [ ] Integracion con Zotero API
- [ ] Export a Notion
- [ ] Plugin para VS Code
- [ ] Interfaz web minimalista (HTML estatico generado por awk)
