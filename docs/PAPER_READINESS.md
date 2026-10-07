### Estado editorial post F10

#### Resumen

El repositorio está suficientemente estable para comenzar la redacción de
manuscritos y para congelar la arquitectura y los resultados F8-F10.

Esto no equivale a que el Paper 1 definido en `docs/PAPER_SCOPE.md` esté listo
para envío.

#### Redacción

Estado:

```text
MANUSCRIPT_DRAFTING = GO
```

Pueden escribirse ya:

- arquitectura y metodología F0-F10;
- protocolo result-blind F8;
- resultado negativo F8;
- observación e intervención F9;
- validación de sensibilidad F10;
- amenazas a la validez y frontera del artefacto.

#### Paper 1 bajo el alcance actual

Estado:

```text
PAPER1_SUBMISSION = NO_GO
```

Motivos:

1. `docs/PAPER_SCOPE.md` exige al menos una evaluación held-out cuando sea
   técnicamente posible.
2. F8 intentó una evaluación held-out, pero obtuvo cero predicciones decididas
   y una abstención `out_of_scope`.
3. F10 no es validación held-out de un contrato F6; es un benchmark
   `known-positive` de sensibilidad observada e intervención.
4. F8 no produjo una divergencia externa positiva sobre la cual demostrar
   end-to-end la derivación de un contrato F6 y su evaluación held-out.

Por tanto, el paper original centrado en "Environmental Metamorphic Contracts"
requiere una última validación específica del paper o una reformulación
explícita y no retrospectiva de su claim.

#### F10 como evidencia

F10 sí resuelve la objeción de ausencia total de positivos externos:

```text
known_positive = 3
behaviorally_relevant = 3
end_to_end_sensitivity = 1.0
```

Estos valores son descriptivos sobre tres controles positivos previamente
seleccionados. No constituyen una estimación de recall poblacional y no permiten
estimar especificidad porque F10-MVP no contiene controles negativos.

#### Trabajo científico mínimo restante para el Paper 1 actual

No se recomienda abrir otra fase grande.

El mínimo es un experimento editorial separado que produzca, cuando sea
técnicamente viable:

- al menos un caso externo con divergencia reproducible;
- derivación F5/F6 sobre ese caso;
- una configuración held-out fijada antes de evaluarla;
- una decisión held-out, no una abstención.

Si el dominio elegido no permite held-out fuerte, el claim del paper debe
reducirse de forma explícita antes del envío.

#### Claims permitidos ahora

Permitido:

- EnvMorph posee una cadena experimental reproducible F0-F10;
- F8 preservó resultados externos negativos sin retuning post-hoc;
- F9 separa dependencia observada de relevancia conductual;
- F10 produjo 3/3 positivos en un benchmark `known-positive`;
- el artefacto combina un núcleo Rust y un harness Python/C;
- la observación tiene blind spots documentados.

No permitido:

- observación completa de dependencias ambientales;
- recall general de 1.0;
- especificidad demostrada;
- representatividad del software Unix;
- prevalencia de sensibilidad;
- validación held-out positiva de contratos externos.
