### Prior art inicial F9

#### Bazel

Bazel define un build hermético como uno aislado de cambios del host y enfatiza aislamiento y source identity. Su enfoque principal es prevenir influencias externas mediante herramientas y dependencias administradas.

Referencia:

https://bazel.build/versions/9.1.0/basics/hermeticity

Consecuencia para F9: prevenir dependencia y medir relevancia conductual son preguntas distintas. F9 no debe presentarse como alternativa a la hermeticidad de Bazel.

#### Nix

Nix usa sandboxing para limitar visibilidad del filesystem y, en Linux, aislar recursos mediante namespaces. Esto reduce dependencias no declaradas, pero no clasifica por sí solo si una dependencia observada habría cambiado un artefacto bajo una intervención específica.

Referencias:

https://reproducible.nixos.org/
https://releases.nixos.org/nix/nix-2.32.3/manual/command-ref/conf-file.html

#### RepTrace

Ren et al., ASE 2019, proponen RepTrace para localizar causas de builds no reproducibles a partir de system-call traces y un grafo de dependencias proceso/archivo.

Referencia:

https://www.se.cs.uni-saarland.de/conferences/ASE/ase2019/details/ase-2019-papers/86/Root-Cause-Localization-for-Unreproducible-Builds-via-Causality-Analysis-over-System-.html

Consecuencia para F9: tracing más causalidad sobre builds irreproducibles ya existe. La novedad no puede ser "usar syscall tracing para encontrar dependencias".

#### Hermeticidad empírica de Bazel

Shenyu Zheng, Bram Adams y Ahmed E. Hassan estudiaron 70 proyectos Bazel mediante aproximadamente 150 millones de llamadas de filesystem y encontraron dependencias no herméticas incluso en proyectos que usan un build system orientado a hermeticidad.

Referencia:

https://mcislab.github.io/publications/2025/ieeesw-shenyu.pdf
DOI: 10.1109/MS.2024.3472016

Consecuencia para F9: detectar dependencias externas a partir de trazas también tiene prior art empírico fuerte. F9 debe separar detección de dependencia y relevancia sobre outputs.

#### Identificación de requisitos mediante tracing

Granat et al., 2024, estudian identificación de build requisites mediante system-call tracing y presentan buildography, incluyendo ptrace.

Referencia:

https://doi.org/10.15514/ISPRAS-2024-36(5)-2

Consecuencia para F9: ptrace no constituye una contribución científica por sí mismo.

#### Reproducible Builds

El proyecto Reproducible Builds documenta variaciones del entorno, incluyendo build path, timestamps y otros canales de no determinismo.

Referencias:

https://reproducible-builds.org/docs/env-variations/
https://reproducible-builds.org/docs/build-path/

Consecuencia para F9: la taxonomía ambiental debe alinearse con causas conocidas de irreproducibilidad y no inventar categorías sin relación con el estado del arte.

#### AgentGuard-FastPath

El repositorio `kapumota/AgentGuard-FastPath` contiene un monitor Linux en C basado en ptrace y un analizador `agfast`. El monitor sigue fork, vfork y clone, observa syscalls seleccionadas y actualmente combina observación con enforcement.

Archivos relevantes:

https://github.com/kapumota/AgentGuard-FastPath/blob/main/src/monitor.c
https://github.com/kapumota/AgentGuard-FastPath/tree/main/ebpf

Auditoría inicial:

- reutilización potencial: lectura de strings/buffers del tracee, seguimiento de procesos y normalización básica de paths;
- no reutilizar sin cambios: lógica que bloquea o reemplaza syscalls;
- cobertura actual insuficiente para F9 completo: el monitor se centra en open/openat/creat, execve/execveat, connect, señales y policy enforcement;
- F9 requiere un formato de eventos neutral y separado de policy enforcement;
- la implementación F9 debe declarar explícitamente blind spots como getenv en memoria y vDSO.

Conclusión provisional: AgentGuard-FastPath puede aportar componentes técnicos, pero no debe importarse como backend F9 completo ni usarse como claim de novedad.

#### Hipótesis de diferenciación

La hipótesis a validar en F9 es más estrecha:

`observed environmental dependency != behaviorally relevant environmental dependency`

La contribución potencial es un procedimiento reproducible para enlazar observación transparente, intervención controlada y oráculos de comportamiento ya existentes en EnvMorph.

Esta hipótesis sigue siendo provisional hasta F9.1-F9.8.
