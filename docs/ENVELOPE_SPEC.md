### Portability Envelopes F7

#### Propósito

F7 convierte un Environmental Contract F6 en un sobre finito de portabilidad sobre el lattice baseline/treatment declarado por el contrato.

F7 no reejecuta el workload, no recalcula equivalencia, no repite propagación y no vuelve a minimizar factores. Cada celda se clasifica llamando al evaluador contractual F6.

#### Dominio finito

Para `n` factores, F7 enumera exactamente `2^n` configuraciones usando únicamente los valores baseline y treatment observados. La versión inicial admite hasta 10 factores.

Cada celda queda clasificada como:

```text
satisfied
violated
out_of_scope
```

Una celda `invalid` generada internamente se considera un error, porque F7 siempre construye configuraciones completas.

#### Estado del sobre

`complete` significa que todas las configuraciones del lattice declarado fueron decididas por F6 como `satisfied` o `violated`.

`partial` significa que al menos una configuración sigue `out_of_scope`. Esto no es un fallo: expresa evidencia insuficiente dentro de una combinación del contraste declarado.

#### Fronteras

Dos celdas son vecinas si difieren en un solo factor. F7 registra una frontera cuando sus estados contractuales son distintos.

Estas fronteras son observacionales y finitas. No representan una frontera continua ni una ley de portabilidad para valores ambientales no ensayados.

#### CLI

```text
envmorph build-envelope CONTRACT.toml --output ENVELOPE.json [--format human|json]
```

La salida JSON contiene factores, todas las celdas, conteos por estado, cobertura decidida y fronteras entre celdas vecinas.

#### Límite científico

El claim F7 es `finite_observed_contrast_envelope_not_universal_portability`.

F7 caracteriza exhaustivamente el producto cartesiano baseline/treatment declarado. No extrapola a versiones, locales, zonas horarias, implementaciones u otros valores que no formen parte del contrato F6.
