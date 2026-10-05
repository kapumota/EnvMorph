### Prior art y posicionamiento de EnvMorph antes de F8

#### Propósito

Este documento fija el posicionamiento de EnvMorph antes de la validación externa. Su función principal es declarar qué partes del problema ya cuentan con soluciones o antecedentes sólidos y qué hipótesis de diferenciación deben someterse a prueba en F8.

No es una revisión sistemática completa para publicación. Es un gate de posicionamiento previo a experimentación externa y debe ampliarse durante la preparación editorial del Paper 1.

#### Reprotest

`reprotest` reconstruye software en ambientes distintos y compara los artefactos resultantes. Su configuración actual incluye variaciones de environment, build path, kernel, ASLR, número de CPU, tiempo, usuario y grupo, file ordering, dominio y hostname, home, locales, exec path, timezone y umask. También dispone de modos automáticos para intentar determinar variaciones específicas asociadas a irreproducibilidad.

Consecuencia para EnvMorph: variar el entorno y detectar si el resultado cambia no es una contribución novedosa por sí sola. La cobertura de factores de F8 debe reconocer explícitamente esta línea de trabajo.

Referencia principal: https://manpages.debian.org/trixie/reprotest/reprotest.1.en.html

#### Diffoscope

`diffoscope` realiza comparación profunda de archivos, directorios y numerosos formatos, con desempaquetado recursivo y representaciones legibles de diferencias.

Consecuencia para EnvMorph: comparación profunda de artefactos tampoco debe presentarse como contribución central. Los oráculos F3 persiguen semánticas explícitas y controlables para el flujo experimental, no competir en amplitud de formatos con diffoscope.

Referencia principal: https://diffoscope.org/

#### Reproducible Builds

La comunidad Reproducible Builds documenta múltiples fuentes de variabilidad ambiental, incluyendo idioma y configuración regional, tiempo, orden de archivos, metadata de archivos, hostname, usuario y build path. También enfatiza que el orden de entradas y el locale pueden interactuar y producir resultados no reproducibles.

Consecuencia para EnvMorph: los factores F8 deben derivarse del corpus y del conocimiento acumulado sobre variabilidad, no limitarse a locale, timezone y AWK por conveniencia de implementación.

Referencias principales:

- https://reproducible-builds.org/docs/env-variations/
- https://reproducible-builds.org/docs/stable-inputs/
- https://reproducible-builds.org/docs/adding-build-variance/

#### Delta Debugging

Zeller y Hildebrandt formalizaron Delta Debugging para simplificar e identificar circunstancias inductoras de fallo. `ddmin` persigue minimalidad respecto del test definido y constituye antecedente directo de cualquier claim sobre minimización de cambios.

Consecuencia para EnvMorph: F5 no se presenta como invención de minimización causal. Su diferencia operacional actual es una búsqueda exacta por cardinalidad sobre un conjunto ambiental finito, preservando todos los conjuntos mínimos que reproducen una firma F4. Esta diferencia debe tratarse como decisión metodológica, no como novedad algorítmica mayor.

Referencia: Andreas Zeller y Ralf Hildebrandt, "Simplifying and Isolating Failure-Inducing Input", IEEE Transactions on Software Engineering 28(2), 2002. https://www.st.cs.uni-saarland.de/papers/tse2002/

#### Metamorphic Testing

Metamorphic Testing fue introducido como estrategia para generar casos relacionados y verificar propiedades relacionales cuando el oracle problem dificulta conocer una salida esperada. La literatura posterior ha desarrollado identificación de relaciones metamórficas, generación de casos y combinación con otras técnicas.

Consecuencia para EnvMorph: tratar el entorno como transformación metamórfica es un encuadre de aplicación. No debe afirmarse que EnvMorph inventa metamorphic testing. La hipótesis defendible es que relaciones ambientales explícitas pueden convertirse en evidencia de propagación, contratos y envelopes para workflows Unix heredados.

Referencias:

- T. Y. Chen, S. C. Cheung y S. M. Yiu, "Metamorphic Testing: A New Approach for Generating Next Test Cases", HKUST-CS98-01, 1998. https://www.cse.ust.hk/faculty/scc/publ/CS98-01-metamorphictesting.pdf
- T. Y. Chen et al., "Metamorphic Testing: A Review of Challenges and Opportunities", ACM Computing Surveys 51(1), 2018, DOI 10.1145/3143561.

#### Fallos dependientes del entorno

Cavezza et al. estudiaron reproducibilidad de fallos dependientes del entorno y mostraron experimentalmente que componentes ambientales pueden influir en la capacidad de reproducir fallos de software.

Consecuencia para EnvMorph: environment-sensitive behavior tiene antecedente empírico directo. F8 debe posicionarse como caracterización estructurada de divergencia, suficiencia operacional y contratos, no como descubrimiento de que el entorno afecta al software.

Referencia: Davide G. Cavezza et al., "Reproducibility of Environment-Dependent Software Failures: An Experience Report", ISSRE 2014, DOI 10.1109/ISSRE.2014.19.

#### Variabilidad y builds no reproducibles

Trabajos recientes estudian variabilidad profunda y opciones de configuración como causas de irreproducibilidad. Por ejemplo, investigaciones de 2024 exploran puntos de variabilidad de lenguaje, librerías, compilador, máquina virtual, sistema operativo y variables ambientales, y otros trabajos identifican opciones de configuración responsables de builds no reproducibles en sistemas altamente configurables.

Consecuencia para EnvMorph: el estudio de interacciones entre factores y configuración no puede tratarse como espacio vacío. Paper 2 debe demostrar qué aporta específicamente el envelope de EnvMorph más allá de exploración de configuraciones.

Referencias:

- Mathieu Acher et al., "Embracing Deep Variability For Reproducibility and Replicability", 2024, DOI 10.1145/3641525.3663621.
- Georges Aaron Randrianaina et al., "Options Matter: Documenting and Fixing Non-Reproducible Builds in Highly-Configurable Systems", 2024, DOI 10.1145/3643991.3644913.

#### Hermeticidad y tracing

Bazel y Nix representan una línea distinta: prevenir dependencias ambientales mediante builds herméticos y sandboxing. Además, RepTrace utiliza tracing de llamadas al sistema y análisis causal para localizar causas de builds no reproducibles.

Consecuencia para EnvMorph: F9 no debe ser requisito para Paper 1. Un backend ptrace o equivalente entraría en una zona de prior art más competitiva y requiere una revisión separada antes de implementarse.

Referencias:

- Bazel, Hermeticity: https://bazel.build/basics/hermeticity
- Nix, reproducible builds: https://reproducible.nixos.org/
- Zhilei Ren et al., "Root cause localization for unreproducible builds via causality analysis over system call tracing", ASE 2019, DOI 10.1109/ASE.2019.00056.

#### Qué no reclama EnvMorph

EnvMorph no reclama como novedad aislada:

- variar entornos de ejecución
- comparar dos artefactos
- detectar irreproducibilidad
- minimizar un conjunto de cambios
- usar metamorphic testing
- perseguir hermeticidad de builds
- localizar causas mediante tracing de sistema

#### Hipótesis de diferenciación a validar

La contribución potencial que F8 debe poner a prueba es la composición de:

- equivalencia explícita por artefacto
- localización temporal o por etapas de primera divergencia
- propagación y absorción observables
- enumeración exacta de conjuntos ambientales mínimos dentro de un espacio declarado
- contratos ambientales delimitados por evidencia
- envelopes finitos con cobertura y regiones `out_of_scope`
- evaluación sobre workflows Unix heredados que no fueron diseñados para EnvMorph

Estas características son hipótesis de diferenciación, no claims editoriales confirmados hasta completar F8 y una revisión bibliográfica más exhaustiva.
