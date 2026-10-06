### Erratum F9.8A0: serialización byte-safe del observer

#### Incidente de adquisición

La primera ejecución F9.8A1 bajo el protocolo efectivo anterior alcanzó `csvkit-realdata`, `jq-build-check` y `fd-build-completions`, y falló al normalizar el stream ptrace de `fd-build-completions`.

El error fue una secuencia que no podía decodificarse como UTF-8 estricto.

`inih-meson-tests` no llegó a ejecutarse.

El rollback anterior eliminó la evidencia cruda parcial y no existió commit de censo.

#### Causa técnica

Linux representa nombres de ruta como secuencias de bytes y no exige UTF-8.

El observer F9.1A escapaba caracteres JSON de control, comillas y backslash, pero emitía directamente los bytes mayores o iguales a `0x80`.

Eso puede producir JSONL que no es UTF-8 válido.

#### Corrección

No se cambia qué syscalls observa el backend ni cómo resuelve rutas.

Solo cambia la representación JSON:

```text
byte ASCII seguro -> se conserva
byte de control   -> \u00xx
byte >= 0x80      -> \u00xx
```

La transformación proyecta cada byte a `U+00XX`. Es inyectiva sobre los bytes originales y puede invertirse mediante Latin-1.

No se usa `errors=replace`, porque eso perdería identidad y podría fusionar recursos distintos.

#### Reinicio de campaña

Los cuatro workloads se vuelven a ejecutar desde el inicio bajo el transporte corregido.

Los resultados parciales previos no se usan para filtrar dependencias, asignar elegibilidad, crear planes, seleccionar treatments o clasificar relevancia.

#### Evidencia de fallos futuros

La orquestación deja de eliminar `.envmorph/f9-8a-evidence` si una nueva adquisición falla.

#### Alcance científico

Este erratum no cambia corpus, factores baseline, conjunto de syscalls, complemento getenv, normalización lexical, exclusiones instrumentales, reglas del censo, métricas F9.7, oráculos F3-F5 ni política de treatments.
