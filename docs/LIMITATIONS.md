### Limitaciones del baseline F0

#### Equivalencia

La comparación actual se apoya principalmente en igualdad basada en contenido y hashes.

Todavía no existen oráculos generales separados para igualdad estructural o semántica.

#### Causalidad ambiental

La captura ambiental existente registra diferencias observadas, pero una diferencia entre variables no demuestra causalidad.

F1 y fases posteriores deberán separar observación, perturbación controlada e inferencia causal.

#### Interacciones

F0 no modela todavía interacciones entre factores ambientales.

No puede inferir que una divergencia requiera, por ejemplo, una combinación específica de implementación AWK y locale.

#### Herramientas internas

La instrumentación explícita identifica principalmente el ejecutable superior de una etapa.

Herramientas invocadas internamente por scripts pueden no quedar modeladas de forma independiente.

#### Replay

F0 rechaza rutas lógicas absolutas no mapeadas y rutas con componentes `..` antes de materializar entradas, salidas, stdout, stderr o cwd.

Este hardening protege el materializador de archivos, pero no convierte el replay en un sandbox de sistema operativo. Un proceso ejecutado todavía puede acceder a recursos externos si su propia orden lo solicita.

#### Workloads

El `literature-pipeline` es el primer workload heredado, pero no basta por sí solo para sostener resultados generales.

Las fases experimentales deberán incorporar workloads externos reproducibles y, cuando sea posible, bugs históricos documentados.

#### Evidencia científica

Los experimentos heredados de FlowAttest se conservan por procedencia.

No constituyen evidencia científica de EnvMorph.
