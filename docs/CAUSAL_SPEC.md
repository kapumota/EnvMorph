### Especificación causal F5

#### Propósito

F5 busca un Minimal Causal Environment sobre un conjunto finito de factores ambientales declarados.

El término causal se usa en sentido operacional. Un conjunto mínimo es suficiente para reproducir la misma firma observable F4 obtenida por la intervención completa. Esto no constituye por sí solo una prueba de causalidad física ni excluye factores no declarados.

#### Formato

```toml
[causal]
name = "ejemplo-f5"
schema_version = 1
confirmations = 2

[[factor]]
name = "locale"
baseline = "C"
treatment = "POSIX"

[[factor]]
name = "timezone"
baseline = "UTC"
treatment = "America/Lima"
```

Cada factor tiene un valor `baseline` y un valor `treatment` distintos.

`confirmations` controla cuántas ejecuciones independientes deben reproducir la misma firma F4. El valor predeterminado es 2 y F5 admite de 1 a 5 confirmaciones.

#### Firma objetivo

F5 ejecuta primero el baseline y la intervención completa mediante F2. Los artefactos se comparan con F3 y se clasifican con F4.

La firma objetivo conserva:

- estado global del trazado
- primera divergencia observada
- fronteras de absorción
- secuencia de equivalencia por observación
- secuencia de propagación por observación

Si la intervención completa produce firmas distintas entre confirmaciones, F5 se detiene y no declara minimalidad.

#### Minimalidad

F5 realiza una búsqueda exacta por cardinalidad creciente. En la primera cardinalidad que reproduce la firma objetivo se evalúan todos los subconjuntos de ese tamaño y se reportan todos los conjuntos mínimos.

La versión inicial admite hasta 10 factores. Ese límite evita presentar una búsqueda truncada como si fuera exacta.

`missing`, `error`, `unavailable` y `execution_failed` no se interpretan como evidencia negativa. Interrumpen la minimización porque no permiten establecer suficiencia observable.

#### CLI

```text
envmorph minimize-environment CAUSAL.toml --trace TRACE.tsv --workdir DIR --output DIR -- CMD [ARGS...]
```

La salida admite formato humano o JSON. `mce-result.json` se guarda siempre dentro del directorio de salida.
