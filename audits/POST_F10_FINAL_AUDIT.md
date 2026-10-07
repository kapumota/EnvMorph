### Auditoría post F10

#### Baseline

Commit auditado:

```text
9476e6c6c2bae67ebbecc1537ff468f189080634
```

Freeze científico anterior:

```text
v0.1.0 -> 29cc7961766f783ee4d642f3acf446fb6f9bdb27
```

Rama de cierre:

```text
cierre/auditoria-post-f10
```

#### Gates ejecutados

- rustc local: `1.95.0`
- cargo fmt --check: PASS
- cargo test --locked: PASS (108 passed; 0 failed)
- cargo build --release --locked: PASS
- warnings observados durante build actual: 11
- provenance/F9_10_FILES.sha256: PASS
- auditoría de rutas públicas: PASS
- ocurrencias actuales del home histórico detectadas por el auditor: 4143
- blockers actuales de rutas públicas: 0
- reproducción F10 Docker: PASS
- controles F10 behaviorally_relevant: 3/3

#### Revisión de los nuevos hallazgos

##### 1. Cobertura del observador

Estado: ACCEPTED_LIMITATION_AND_CLAIM_NARROWED

La afirmación de que F9 usa únicamente un observador `getenv` es incorrecta.
El repositorio contiene además un backend ptrace que observa `open`,
`openat`, `creat`, `execve` y `execveat`.

La crítica sustantiva sí es válida: la atribución de lecturas de variables
ambientales depende de la fuente complementaria `getenv`, que no cubre todas
las formas de consumo del entorno. Lecturas directas de `environ`, consumo
interno de locale dentro de libc, binarios estáticos y otras rutas pueden no
producir un evento de variable ambiental.

Consecuencia: queda prohibido reclamar observación completa o recall general
del observador.

##### 2. F10 como benchmark positivo

Estado: ACCEPTED_LIMITATION

F10 contiene tres controles `known-positive` y ningún control negativo. El
3/3 demuestra sensibilidad end-to-end en esos controles y constituye una
prueba de existencia y sensibilidad en condiciones favorables.

No permite estimar especificidad y no constituye una estimación poblacional de
recall.

##### 3. Producto end-to-end

Estado: RESOLVED_BY_ARTIFACT_DEFINITION

El CLI Rust implementa F0-F7. F9 se implementa mediante 14 scripts
Python (3285 líneas por `wc -l`) y 5 archivos C (1328 líneas),
y F10 usa un harness de reproducción adicional.

El artefacto se define formalmente como `Rust core + Python/C experimental
harness`. No se afirma que un único binario exponga toda la cadena.

##### 4. Gate held-out de Paper 1

Estado: OPEN_PAPER_BLOCKER

`docs/PAPER_SCOPE.md` exige al menos una evaluación held-out cuando sea
técnicamente posible.

F8 tuvo una evaluación held-out intentada, cero predicciones decididas y una
abstención `out_of_scope`.

F10 no es una evaluación held-out de un contrato F6.

Por tanto, el Paper 1 bajo su alcance actual no está listo para envío.

##### 5. Reproducibilidad histórica

Estado: ACCEPTED_RESIDUAL_LIMITATION

F10 tiene una ruta Docker reproducible y fue regenerado durante esta auditoría.

F8/F9 históricos no son completamente regenerables desde raw público. Los
artefactos históricos conservan rutas absolutas y evidencia de procedencia.

El auditor de rutas actual reporta 4143 ocurrencias y
0 blockers públicos. El número bruto de ocurrencias no debe
interpretarse por sí solo como número de defectos, porque artefactos de
auditoría e históricos pueden reproducir textualmente rutas congeladas.

##### 6. Toolchain e higiene

Estado: PARTIALLY_RESOLVED

Se declara `rust-version = "1.85"`, consistente con la dependencia
`toml 1.1.6`.

Cargo.lock v4 no es compatible con Cargo 1.75, y la dependencia TOML usa
edición Rust 2024.

Los warnings de código no utilizado y la dependencia de utilidades Unix en
tests shell permanecen como deuda de ingeniería no bloqueante para la evidencia
científica actual.

#### Estado de las observaciones anteriores

| Observación | Estado final |
|---|---|
| Falta de positivo externo real | RESOLVED |
| Embudo F9 12717 -> 2 | EXPLAINED_BUT_NOT_GENERALIZED |
| Sesgo de selección del corpus | ACCEPTED_LIMITATION |
| Reproducibilidad F10 end-to-end | RESOLVED_FOR_F10 |
| Reproducibilidad histórica F8/F9 | ACCEPTED_RESIDUAL_LIMITATION |
| Proceso frente a evidencia | CLOSED_WITH_BOUNDED_CLAIMS |

#### Readiness

```text
REPOSITORY_AUDIT = PASS_WITH_LIMITATIONS
MANUSCRIPT_DRAFTING = GO
PAPER1_SUBMISSION_CURRENT_SCOPE = NO_GO
```

La auditoría de repositorio queda cerrada para evitar una secuencia abierta de
nuevas fases de ingeniería.

El trabajo restante debe ser específico del paper, no otra auditoría general
del repositorio.

Para el Paper 1 actual, el mínimo científico restante es obtener una validación
externa con divergencia reproducible que permita F5/F6 y una decisión held-out,
o reformular explícitamente el claim editorial antes del envío.
