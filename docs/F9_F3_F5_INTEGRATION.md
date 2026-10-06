### Integración F9 con F3, F4 y F5

#### Propósito

F9.6 reemplaza el oráculo sintético exclusivo `micro:stdout` como única demostración de clasificación y valida que F9 pueda consumir las capas científicas ya cerradas.

La integración no reimplementa sus semánticas.

#### F3

F3 conserva autoridad sobre equivalencia de artefactos mediante:

```text
EquivalenceOracle
ByteOracle
TextOracle
JsonOracle
CsvOracle
```

F9.6 consume únicamente la salida JSON de:

```text
envmorph compare-artifacts
```

Mapeo F9:

```text
identical  -> observed_only
equivalent -> observed_only
different  -> behaviorally_relevant
missing    -> unresolved
error      -> unresolved
```

La clasificación sigue siendo relativa al oráculo F3 seleccionado.

#### F4

F4 conserva autoridad sobre propagación y absorción mediante:

```text
PropagationAnalyzer
```

F9.6 consume la salida JSON de:

```text
envmorph analyze-propagation
```

Mapeo F9:

```text
stable     -> observed_only
absorbed   -> behaviorally_relevant
persistent -> behaviorally_relevant
unresolved -> unresolved
```

`absorbed` cuenta como diferencia conductual porque existe al menos una divergencia observable confirmada, aunque desaparezca en una etapa posterior.

F9 no convierte `divergence_start` en una causa.

#### F5

F5 conserva autoridad sobre Minimal Causal Environment y suficiencia operacional.

F9.6 consume `mce-result.json`.

Mapeo F9:

```text
minimal              -> behaviorally_relevant
no_observable_effect -> observed_only
```

`minimal` agrega evidencia de suficiencia operacional dentro de los factores declarados.

No cambia el alcance original de F5:

```text
operational_sufficiency_not_physical_proof
```

#### Jerarquía de evidencia

Las capas no se fusionan como si fueran equivalentes.

```text
F3 artifact_equivalence
F4 propagation_trace
F5 operational_sufficiency
```

F5 consume F4 y F4 consume F3. F9.6 preserva esa jerarquía.

#### Confirmaciones

Los casos F3 y F4 se ejecutan dos veces en F9.6 y deben producir salidas JSON byte-identical.

F5 utiliza `confirmations = 2` dentro de su contrato y además se valida determinismo de su resultado machine-readable sobre ejecuciones independientes.

#### Contradicciones

F9.6 no resuelve contradicciones por votación.

Si capas relacionadas producen estados incompatibles, el caso debe quedar `unresolved` en una fase posterior de composición.

F9.6 valida solo ground truth coherente.

#### No claims

F9.6 no afirma:

- causalidad física;
- que toda diferencia F3 sea externamente importante;
- que toda absorción sea persistente;
- que un MCE cubra factores no declarados;
- cobertura completa de dependencias ambientales.

#### Corpus externo

F9.6 no ejecuta `csvkit`, `jq`, `fd` ni `inih`.

#### Siguiente gate

F9.7 congela métricas, reglas agregadas y criterios de decisión antes de F9.8 sobre el corpus externo.
