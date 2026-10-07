### Frontera del artefacto EnvMorph

#### Composición

EnvMorph no es actualmente un único binario end-to-end.

El artefacto está compuesto por:

- un núcleo y CLI en Rust que implementa la cadena F0-F7;
- un harness experimental F9 en Python y C para observación, normalización,
  construcción de grafos, planes de intervención, ejecución y clasificación;
- un harness F10 en Python y shell para los controles externos `known-positive`;
- scripts de reproducción y auditoría.

Por tanto, cualquier paper, README o descripción pública debe presentar el
artefacto como:

```text
Rust core + Python/C experimental harness
```

y no afirmar que el CLI Rust expone actualmente toda la cadena F9/F10.

#### Alcance del observador F9

F9 combina dos fuentes complementarias.

El backend ptrace observa operaciones de sistema seleccionadas:

- `open`;
- `openat`;
- `creat`;
- `execve`;
- `execveat`.

La interposición `LD_PRELOAD` observa llamadas dinámicamente interponibles a
`getenv`.

Esta combinación no equivale a observación completa de dependencias
ambientales. Entre los blind spots conocidos están:

- lecturas directas de `environ`;
- accesos mediante interfaces distintas de `getenv`, incluido
  `secure_getenv` cuando no atraviesa el símbolo interpuesto;
- consumo interno de variables de locale dentro de libc;
- binarios estáticos para los que `LD_PRELOAD` no aplica;
- dependencias resueltas en memoria, cachés, descriptores preabiertos o vDSO;
- syscalls fuera del conjunto observado por el backend congelado.

El backend ptrace puede observar archivos de locale abiertos durante una
ejecución, pero eso no demuestra qué variable ambiental originó la selección.

#### Consecuencia científica

F10 demuestra que EnvMorph recupera tres controles externos positivos cuyas
variables objetivo pasan por la superficie `getenv` observada.

F10 no mide cobertura sobre todas las formas de consumo del entorno y no debe
presentarse como una estimación general de recall del observador.

La contribución defendible es una cadena de evidencia delimitada por el
mecanismo de observación y los controles declarados.
