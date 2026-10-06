### Erratum F9.8A: atomicidad del stream getenv

#### Causa

El interposer F9.2 escribía un único objeto JSON mediante múltiples llamadas `write`: prefijo, nombre escapado y sufijo.

En workloads concurrentes esas escrituras podían entrelazarse.

La evidencia preservada de `fd-build-completions` contiene JSON inválido en el stream `getenv`.

#### Corrección

El interposer nuevo construye el registro completo en memoria privada y realiza una sola llamada `write` por observación.

El contador `seq` pasa a ser atómico entre threads.

Los bytes de control y no ASCII del nombre se representan como `\u00xx`.

No se registran valores de variables.

#### Ground truth

La prueba usa ocho threads y mil llamadas `getenv` por thread.

Debe producir:

```text
8000 eventos objetivo
8000 secuencias distintas
0 líneas JSON inválidas
```

#### Alcance

No cambian backend ptrace, resolución de paths, baseline ambiental, exclusiones instrumentales, regla de candidatos, oráculos F3-F5 ni métricas F9.7.

Los resultados parciales anteriores no se usan para selección, elegibilidad, planes o clasificación.

No se han ejecutado treatments.
