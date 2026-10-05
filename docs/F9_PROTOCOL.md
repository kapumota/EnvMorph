### Protocolo F9: hermeticidad conductual

#### Estado

Este protocolo se congela antes de implementar el backend de observación F9 y antes de ejecutar experimentos de relevancia conductual.

F9 parte del cierre F8 sin modificarlo ni reinterpretarlo. F8 terminó con tres de cuatro workloads evaluables, cero divergencias observables y una abstención held-out. F9 formula una pregunta nueva: una dependencia ambiental puede ser observada durante la ejecución sin ser conductualmente relevante para los artefactos definidos por los oráculos.

#### Objetivo

Distinguir entre:

- dependencia ambiental observada;
- dependencia ambiental conductualmente relevante;
- dependencia observada sin efecto detectable;
- dependencia no evaluable o fuera de alcance.

F9 no busca convertir resultados negativos de F8 en resultados positivos. Busca explicar qué recursos del entorno son consultados y cuáles de ellos cambian el comportamiento observable cuando se intervienen de manera controlada.

#### Preguntas de investigación

RQ1. Qué dependencias ambientales externas puede observar un backend transparente durante la ejecución de un workflow Unix heredado.

RQ2. Qué subconjunto de las dependencias observadas produce una diferencia reproducible bajo una intervención controlada y un oráculo F3 previamente declarado.

RQ3. Qué dependencias observadas permanecen conductualmente irrelevantes dentro del dominio de intervención declarado.

RQ4. En los workloads estables de F8, existen dependencias ambientales observadas que expliquen por qué dependencia observada no implica sensibilidad observable.

RQ5. Cuál es el costo y la cobertura del backend de observación elegido frente a alternativas razonables.

#### Definiciones operacionales

`observed_dependency` es una relación registrada entre un proceso del workflow y un recurso ambiental externo bajo una categoría declarada.

`behaviorally_relevant` requiere:

1. evidencia de observación previa;
2. una intervención finita y declarada sobre ese recurso o su resolución;
3. baseline estable;
4. al menos dos confirmaciones consistentes;
5. diferencia detectada por un oráculo F3 congelado antes de la intervención afectada.

`observed_only` significa que la dependencia fue observada pero las intervenciones declaradas no produjeron diferencia detectable.

`unresolved` significa que la estabilidad, la observación o la intervención no permiten una conclusión confirmatoria.

`out_of_scope` significa que el recurso, valor o mecanismo excede el dominio preregistrado.

`observer_failure` significa que el backend no pudo observar la ejecución con las garantías mínimas declaradas.

#### Categorías iniciales de dependencia

Las categorías candidatas son:

- acceso a contenido de archivos;
- acceso a metadatos y resolución de rutas;
- resolución y ejecución de herramientas;
- árbol de procesos;
- variables ambientales transmitidas en fronteras de ejecución;
- directorios temporales;
- locale y timezone cuando exista una representación observable;
- red, únicamente como dependencia observada, con ejecución experimental offline salvo protocolo específico posterior.

Estas categorías no implican que un único backend pueda observarlas todas.

#### Regla de observación

F9 separa observación de intervención.

La observación no debe alterar deliberadamente la decisión del workflow. Un backend reutilizado desde una herramienta de enforcement debe ejecutarse en modo observador y no reemplazar, bloquear o reescribir syscalls durante los experimentos F9.

#### Gate de backend

F9.0 no selecciona `ptrace` por defecto.

Antes de F9.2, F9.1 debe comparar como mínimo:

- ptrace o strace;
- eBPF cuando el host lo permita;
- interposición a nivel de biblioteca solo como complemento;
- mecanismos de sandbox o enforcement únicamente como contraste, no como sustituto de observación.

Los criterios de selección se fijan antes de ejecutar workloads F9:

- cobertura de categorías;
- seguimiento de fork, clone y exec;
- fidelidad de rutas;
- visibilidad de argumentos y resultados;
- privilegios requeridos;
- sobrecarga;
- efecto del observador;
- reproducibilidad del formato de eventos;
- compatibilidad con el runner F2;
- capacidad de operar sin modificar el workload upstream.

#### Limitaciones conocidas que el backend debe declarar

Un tracer de syscalls no observa automáticamente toda dependencia lógica.

Ejemplos:

- `getenv` puede resolverse completamente en memoria y no generar syscall;
- algunas lecturas de reloj pueden usar vDSO;
- una dependencia cargada por `mmap` puede generar acceso inicial pero no una syscall por cada lectura posterior;
- cachés de runtime pueden ocultar accesos repetidos;
- resolución de rutas relativas requiere estado de cwd, dirfd y enlaces;
- un backend de enforcement puede introducir comportamiento que invalide el experimento.

Estas limitaciones deben permanecer visibles en los resultados.

#### Corpus

El corpus F8 de cuatro workloads se conserva como corpus de continuidad y no se selecciona nuevamente por resultado.

Su uso en F9 permite estudiar la relación entre:

- dependencia observada;
- resultado F8 estable;
- relevancia conductual bajo intervenciones F9.

Un corpus adicional, si se incorpora, requiere un protocolo de selección nuevo y congelado antes de observar resultados F9. Ningún workload F8 puede sustituirse por su resultado previo.

#### Intervenciones

Una intervención solo puede proponerse para una dependencia ya observada o para un control explícito preregistrado.

Toda intervención confirmatoria debe registrar:

- recurso objetivo;
- categoría;
- baseline;
- treatment;
- mecanismo de intervención;
- oracle;
- precondiciones;
- disponibilidad;
- al menos dos confirmaciones.

No se amplía un factor solo para obtener una divergencia.

#### Estados de resultado

Los estados confirmatorios de F9 son:

```text
behaviorally_relevant
observed_only
unresolved
unavailable
out_of_scope
observer_failure
execution_failed
```

Todos los estados se preservan.

#### Behavioral Hermeticity Gap

F9.0 no congela todavía una fórmula escalar denominada `Behavioral Hermeticity Gap`.

Los resultados primarios serán conjuntos y cardinalidades:

- dependencias observadas;
- dependencias conductualmente relevantes;
- dependencias observadas sin efecto;
- dependencias unresolved;
- cobertura del observador.

Una métrica escalar solo podrá añadirse mediante un preregistro separado antes de la validación externa F9, con justificación frente al prior art. No se ajustará una fórmula después de observar resultados para maximizar separación.

#### Relación con F3-F7

F9 reutiliza:

- F3 para equivalencia de artefactos;
- F4 para localización de primera divergencia y propagación cuando corresponda;
- F5 para minimizar conjuntos de intervenciones cuando existan múltiples dependencias;
- F6 para expresar contratos solo si la nueva evidencia satisface sus precondiciones;
- F7 únicamente para espacios finitos explícitamente declarados.

F9 no redefine retrospectivamente los resultados F8.

#### No claims

F9 no reclama por defecto:

- hermeticidad universal;
- observación completa de todas las dependencias;
- causalidad física;
- sustitución de Bazel, Nix o sandboxes;
- novedad de usar ptrace, strace o eBPF;
- que toda dependencia observada sea un defecto;
- que ausencia de efecto bajo una intervención implique irrelevancia universal.

#### Criterio de cierre F9.0

F9.0 cierra cuando:

- el protocolo está versionado;
- el prior art inicial está documentado;
- el threat model del observador está documentado;
- la reutilización de AgentGuard-FastPath está clasificada como parcial o descartada;
- no existe todavía backend F9 implementado;
- F8 permanece byte-identical según sus manifests de provenance.
