### Limitaciones del baseline inicial

#### Equivalencia

La comparación actual se apoya principalmente en igualdad basada en hashes. Todavía no existen oráculos separados para igualdad estructural o semántica.

#### Dependencias ambientales

La captura ambiental existente registra información útil, pero una diferencia observada entre variables no demuestra causalidad.

#### Herramientas internas

La instrumentación explícita identifica principalmente el ejecutable superior de una etapa. Herramientas invocadas internamente por scripts pueden no quedar modeladas de forma independiente.

#### Replay

F0.2 rechaza rutas lógicas absolutas no mapeadas y rutas con componentes `..` antes de materializar inputs, outputs, stdout, stderr o cwd dentro del replay.

Este hardening protege el materializador de archivos, pero no convierte el replay en un sandbox de sistema operativo. Un proceso ejecutado todavía puede acceder a recursos externos si su propia orden lo solicita. Ese aislamiento fuerte queda fuera de Fase 0.

#### Lenguaje de interfaz

Los comentarios, ayuda CLI, errores y reportes humanos del núcleo activo se normalizan al español en F0.2. Los identificadores técnicos de protocolo y JSON permanecen estables en inglés cuando forman parte del formato de datos o de categorías del modelo.
