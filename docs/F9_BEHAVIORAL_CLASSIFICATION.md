### Clasificación conductual F9

#### Objetivo

F9.5 convierte evidencia de intervención controlada en tres estados:

```text
behaviorally_relevant
observed_only
unresolved
```

La clasificación es relativa al plan de intervención y al oráculo declarado.

No es una afirmación de causalidad general.

#### Precondiciones

Toda clasificación requiere un `InterventionPlan v1` válido de F9.4.

La dependencia debe haber sido observada antes de la intervención y el plan debe cumplir:

```text
confirmations >= 2
result_blind_selection = true
baseline_value != treatment_value
```

#### Oráculo inicial

F9.5 implementa únicamente:

```text
micro:stdout
```

Su firma es el SHA-256 de stdout capturado por el runner F9.4.

La integración con los oráculos F3 ocurre en F9.6.

#### behaviorally_relevant

Se asigna cuando:

1. baseline tiene al menos las confirmaciones requeridas;
2. treatment tiene al menos las confirmaciones requeridas;
3. todas las ejecuciones tienen exit code cero;
4. baseline es internamente consistente;
5. treatment es internamente consistente;
6. la firma del oráculo baseline es diferente de treatment.

El resultado significa:

```text
la intervención preregistrada produjo
una diferencia reproducible bajo el oráculo declarado
```

No significa que se haya probado causalidad universal.

#### observed_only

Se asigna cuando se cumplen las mismas condiciones de ejecución y consistencia, pero:

```text
baseline_signature == treatment_signature
```

El resultado significa que la dependencia fue observada e intervenida válidamente, pero el oráculo declarado no detectó una diferencia reproducible.

#### unresolved

Se asigna conservadoramente cuando no existe evidencia suficiente para decidir.

Razones iniciales:

```text
insufficient_confirmations
execution_failed
within_variant_instability
unsupported_oracle
```

`unresolved` no se convierte en `observed_only`.

#### Confirmaciones

Las confirmaciones se evalúan por variante.

Dos ejecuciones baseline y dos treatment constituyen el mínimo actual.

Las ejecuciones adicionales pueden conservarse, pero todas las usadas para decisión deben respetar la consistencia interna del oráculo.

#### Ground truth F9.5

F9.5 usa tres micro-workloads controlados:

```text
F9_RELEVANT
getenv observado
baseline y treatment producen stdout diferente
resultado esperado: behaviorally_relevant

F9_STABLE
getenv observado
baseline y treatment producen el mismo stdout
resultado esperado: observed_only

F9_UNRESOLVED
getenv observado
treatment falla de manera controlada
resultado esperado: unresolved
```

Los estados esperados se usan solo en la prueba del clasificador.

No forman parte de los planes de intervención.

#### No selección por resultado

Los planes no incluyen el estado esperado.

Los valores baseline y treatment quedan fijados antes de ejecutar la clasificación.

#### Relación con F9.4

F9.4 responde:

```text
¿la intervención es válida y aplicable?
```

F9.5 responde:

```text
¿qué dice el oráculo después de confirmaciones?
```

#### Siguiente gate

F9.6 integrará la clasificación con los oráculos y contratos existentes de F3, F4 y F5 antes de cualquier validación sobre el corpus F8.
