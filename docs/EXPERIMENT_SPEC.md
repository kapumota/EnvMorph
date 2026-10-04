### Especificación experimental F1

#### Propósito

Un archivo experimental describe factores ambientales y sus valores posibles. F1 modela y valida la especificación, pero todavía no ejecuta el workflow.

#### Versión de esquema

El formato inicial usa `schema_version = 1`.

Un cambio incompatible del formato deberá incrementar esta versión. Los campos desconocidos se rechazan para evitar que un error tipográfico cambie silenciosamente un experimento.

#### Forma mínima

```toml
[experiment]
name = "literature-pipeline-f1"
schema_version = 1

[[factor]]
name = "locale"
values = ["C", "en_US.UTF-8"]

[[factor]]
name = "timezone"
values = ["UTC", "America/Lima"]

[[factor]]
name = "awk_implementation"
values = ["mawk", "gawk"]
```

#### Invariantes

- el experimento debe declarar al menos un factor
- cada factor debe tener un nombre único
- cada factor debe declarar al menos dos valores distintos
- los nombres usan letras minúsculas ASCII, dígitos y `_`
- los valores no pueden ser vacíos ni tener espacios externos
- el orden de factores y valores se conserva porque F1.3 lo utilizará para producir un plan determinista

#### Alcance

Los tres factores del primer experimento son `locale`, `timezone` y `awk_implementation`.

El modelo no afirma todavía que esos factores sean causalmente relevantes. Esa evidencia deberá provenir de ejecuciones controladas en fases posteriores.
