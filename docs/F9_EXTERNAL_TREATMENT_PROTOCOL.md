### F9.8C treatments y clasificación externa

#### Scope

F9.8C ejecuta únicamente los `InterventionPlan v1` congelados en F9.8B.

El número esperado de planes externos es dos.

No se crean nuevos factores, treatments, oráculos ni planes.

#### Confirmaciones

Cada plan obtiene:

```text
baseline  rep 1
baseline  rep 2
treatment rep 1
treatment rep 2
```

Cada ejecución parte de una copia nueva de `.envmorph/f8-runtime/<workload>`.

Los demás factores F8 del mismo workload permanecen en su valor baseline.

#### Ejecución

Los adapters F8 congelados se ejecutan offline mediante:

```text
bash adapter/run.sh
```

Cada repetición conserva:

```text
exit_code
timeout
stdout SHA-256
stderr SHA-256
artefactos F8
```

#### Clasificación

Para cada repetición se compara baseline contra treatment usando el trace F8 congelado y `envmorph analyze-propagation`.

Se requieren dos confirmaciones semánticamente iguales.

Mapeo F4 ya congelado:

```text
stable     -> observed_only
absorbed   -> behaviorally_relevant
persistent -> behaviorally_relevant
unresolved -> unresolved
```

La evidencia decidible se genera mediante el bridge F9.6, no mediante una reimplementación semántica.

#### Efecto del observador

Si F9.8A registró `observer_effect != preserved` para el workload, F9.8C no puede producir una clasificación confirmada. El caso queda como `observer_failure`.

#### Fallos

Un fallo o timeout de ejecución queda `execution_failed`.

Inestabilidad entre las dos confirmaciones F4 queda `unresolved`.

No se convierte ningún fallo en `observed_only`.

#### Agregación

Al final se materializa un `f9.metrics_census.v1` completo con las 12,717 dependencias y se evalúa con el agregador congelado en F9.7.

El denominador de relevancia sigue siendo solo el conjunto decidible.

No se define un Behavioral Hermeticity Gap escalar.
