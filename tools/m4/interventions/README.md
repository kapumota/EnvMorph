# EnvMorph M4-G2, planificador tipado de intervenciones ambientales

## Estado

Prototipo de desarrollo para **preparar y materializar** intervenciones de dos clases:

- `environment_override`, contraste explícito de una variable ambiental;
- `workspace_file_replace`, sustitución de un archivo del workspace por contenido fijado mediante SHA-256.

**Nunca ejecuta el workflow, no observa dependencias y no emite decisiones de relevancia conductual.** No altera los expedientes congelados F8, F9, F10 ni P1. El código se ejecuta desde `~/Descargas` y no se instala en el repositorio. Las copias de baseline y treatment son preparación de ingeniería, no datos del artículo.

## Verificaciones

`python3.11 m4_interventions.py selftest` verifica los dos tipos de intervención, la no modificación del workspace, la comprobación de hashes y el rechazo de valores/rutas adversariales. Para crear planes reales, se necesitan los archivos de entrada y fixtures preseleccionados. No hay soporte para editar archivos del host ni para cambiar ejecutables.

## Ejecución de ejemplo (no científica)

Desde el directorio `EnvMorph_M4_G2_Intervenciones`:

```bash
python3.11 m4_interventions.py selftest
python3.11 m4_interventions.py prepare \
  --spec examples/SPEC.json \
  --workspace examples/toy_workspace \
  --fixtures examples/fixtures \
  --output "$HOME/envmorph-m4-g2-example-preflight"

sha=$(python3.11 - <<'PY'
import json
from pathlib import Path
p = Path.home() / 'envmorph-m4-g2-example-preflight/M4_G2_PREFLIGHT.json'
print(json.loads(p.read_text())['spec_sha256'])
PY
)

python3.11 m4_interventions.py materialize \
  --spec examples/SPEC.json \
  --workspace examples/toy_workspace \
  --fixtures examples/fixtures \
  --output "$HOME/envmorph-m4-g2-example-variants" \
  --expected-spec-sha256 "$sha" \
  --ack-development-only
```

**Atención:** la carpeta de salida debe ser nueva. Si ya existe, la herramienta falla sin sobrescribir. Las variables de entorno registradas en `M4_G2_MANIFEST.json` aún no se aplican a ningún proceso, pues no hay ejecutor de workflows en este prototipo.

## Restricciones

- Los archivos objetivos y fixtures son rutas relativas existentes y no pueden contener componentes simbólicos. No se admiten modificaciones de archivos del host.
- Valores baseline/treatment distintos y hasta seis factores.
- `LD_PRELOAD`, `PATH`, `HOME`, `PYTHONPATH` y otras variables de ejecución privilegiadas se rechazan inicialmente. La lista no constituye una política de seguridad universal.
- El plan se vincula a hashes de los archivos objetivos y fixtures. **No** fija todavía la identidad byte a byte de todo el repositorio upstream.
- La materialización copia un workspace de desarrollo, sin aislamiento de kernel ni invocación de comandos. No implica hermeticidad ni seguridad para ejecutar código no confiable.
- Cada materialización contiene solo dos brazos: baseline y tratamiento conjunto. No es un motor factorial ni aplica políticas adaptativas todavía.
- La familia `workspace_file_replace` debe seleccionar entradas cuyo uso esté justificado en una cohorte prospectiva, no asumir que cualquier archivo observado será conductualmente relevante.

## Siguiente paso

No correr tratamientos externos hasta congelar cohorte, factores, oráculos, presupuestos, entradas, aislamiento y baselines en M4-G3. Con los datos ya disponibles, primero construir un registro de oportunidades tipadas por workflow. Registrar también las que sigan fuera de alcance.
