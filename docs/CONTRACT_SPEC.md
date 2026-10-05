### Environmental Contracts F6

#### Propósito

F6 convierte evidencia F5 en un contrato ambiental versionable y verificable. El contrato no descubre nuevos factores ni ejecuta nuevamente el workflow durante `check-contract`.

La derivación sí consume F5 mediante `derive-contract`, que ejecuta la minimización causal y conserva su evidencia.

#### Semántica conservadora

Un contrato tiene política `preserve_reference_behavior` y alcance `declared_contrasts_only`.

Los estados de evaluación son:

```text
satisfied
violated
out_of_scope
invalid
```

`satisfied` significa que la configuración coincide con una configuración observada que preservó el comportamiento de referencia.

`violated` significa que la configuración coincide exactamente con un MCE observado o con la intervención completa que produjo la firma F4 objetivo.

`out_of_scope` significa que la configuración usa un valor no ensayado, un factor no declarado o una combinación parcial cuya semántica no fue establecida por F5.

`invalid` significa que falta información requerida por el contrato.

F6 no supone monotonicidad. Si un MCE contiene dos factores, no se concluye que cualquier superconjunto de esos factores viole el contrato, salvo la intervención completa que F5 confirmó explícitamente.

#### Derivación

```text
envmorph derive-contract CAUSAL.toml --trace TRACE.tsv --workdir DIR --evidence DIR --output CONTRACT.toml -- CMD [ARGS...]
```

La evidencia F5 queda bajo `DIR` y el contrato conserva:

- nombre del experimento causal
- baseline y treatment por factor
- número de confirmaciones
- estado causal F5
- estado objetivo F4
- primera divergencia observada
- fronteras de absorción
- todos los MCEs mínimos
- factores esenciales

#### Evaluación

```text
envmorph check-contract CONTRACT.toml --environment ENVIRONMENT.toml
```

Una configuración usa:

```toml
[environment]
name = "candidate"
schema_version = 1

[[factor]]
name = "locale"
value = "C"
```

La evaluación no ejecuta el workload y es determinista para el mismo contrato y la misma configuración.

#### Límite científico

Un Environmental Contract F6 codifica evidencia experimental delimitada. No constituye todavía un portability envelope y no generaliza a valores ambientales no observados.
