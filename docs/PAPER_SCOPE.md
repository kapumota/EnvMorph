### Alcance editorial provisional

#### Paper 1

Título de trabajo: Environmental Metamorphic Contracts for Legacy Unix Workflows.

Camino crítico: F2, F3, F4, F5, F6 y F8. F1 aporta modelado y F0 reproducibilidad del artefacto. F7 puede presentarse como análisis complementario, pero no es necesario para sostener el claim central de contratos si F8 demuestra validez externa.

Claim máximo provisional: EnvMorph puede derivar contratos ambientales delimitados por evidencia a partir de divergencias observables reproducibles en workflows Unix heredados y evaluar esos contratos sobre variantes separadas cuando el dominio ofrece configuraciones held-out.

No claims:

- causalidad física
- portabilidad universal
- exhaustividad sobre factores no declarados
- superioridad general frente a reprotest o diffoscope
- novedad de delta debugging
- hermeticidad conductual global

Gate editorial mínimo antes de considerar Paper 1 listo:

- entre tres y cinco workloads externos válidos
- selección del corpus independiente de resultados
- resultados negativos preservados
- factores y oráculos adicionales justificados por el corpus
- al menos una evaluación held-out donde sea técnicamente posible
- related work ampliado desde `docs/PRIOR_ART.md`
- README, ROADMAP y reportes coherentes con el estado real

#### Paper 2

Objeto provisional: Portability Envelopes and Environmental Interactions.

Debe demostrar valor adicional de F7 en espacios con más factores e interacciones reales. Un lattice 2^3 del fixture interno no basta como evidencia científica.

#### Paper 3

Objeto provisional: estudio empírico de sensibilidad ambiental en workflows reales.

Solo se justifica si F8 produce un corpus suficientemente diverso, reusable y con resultados cuantitativos que excedan una evaluación de Paper 1.

#### Paper 4

Objeto provisional: Behavioral Hermeticity.

F9 queda fuera del camino crítico. Antes de implementarlo se requiere comparar explícitamente con RepTrace, hermetic build systems, Nix, Bazel y trabajos de sensibilidad a comportamiento del sistema operativo. Un backend ptrace es una posible implementación, no una contribución por sí mismo.
