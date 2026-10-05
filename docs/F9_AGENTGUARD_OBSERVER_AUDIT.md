### Auditoría AgentGuard-FastPath para F9.1A

#### Fuente congelada

Repositorio: `kapumota/AgentGuard-FastPath`

Commit: `2891a512e265cce07c64fc6a0367bd8a78bba3cb`

Licencia: MIT.

La copia seleccionada se conserva en `third_party/f9/agentguard-fastpath` y su identidad se registra en `provenance/f9/agentguard_selected_source.sha256`.

Los archivos upstream se preservan byte-exactos. Cualquier whitespace heredado se conserva deliberadamente y se audita por hash, no mediante reglas de estilo de EnvMorph.

#### Componentes técnicos reutilizados

El candidato observer-only reutiliza conceptos presentes en el monitor upstream:

- lanzamiento con `PTRACE_TRACEME`;
- parada inicial con `SIGSTOP`;
- `PTRACE_SYSCALL`;
- seguimiento de `fork`, `vfork` y `clone`;
- lectura de argumentos mediante `PTRACE_PEEKDATA`;
- lectura de registros;
- resolución de rutas relativas de `openat` usando `dirfd`.

El candidato no enlaza contra AgentGuard ni incorpora su motor de políticas.

#### Lógica eliminada

El monitor upstream puede modificar `orig_rax` mediante `PTRACE_SETREGS` y sustituir syscalls por `getpid` cuando una política decide bloquear una operación.

Esa semántica es válida para una herramienta defensiva, pero invalida el rol de observador de F9.

El candidato F9 elimina:

- evaluación de políticas;
- acciones allow o block;
- `PTRACE_SETREGS`;
- sustitución de syscalls;
- sandboxing;
- scoring de riesgo;
- severidades de seguridad;
- decisiones de enforcement.

#### Semántica del candidato

El candidato registra eventos neutrales JSONL en syscall exit y conserva:

- PID;
- PPID;
- clase de evento;
- syscall;
- recurso;
- valor de retorno;
- errno derivado;
- estado success o error.

No modifica registros del tracee.

#### Ground truth

F9.1A usa cinco micro-workloads que no pertenecen al corpus F8:

1. open absoluto exitoso;
2. open relativo contra cwd;
3. openat relativo contra dirfd;
4. open inexistente con ENOENT;
5. fork seguido de exec y acceso del hijo a un archivo.

Se compara el candidato con `strace -ff`. Además se exige equivalencia exacta de stdout y exit code con una ejecución sin observador.

#### Resultado

Consultar:

- `experiments/f9/backend_eval/metrics.json`;
- `experiments/f9/backend_eval/comparison.tsv`;
- `experiments/f9/backend_eval/toolchain.txt`.

Un PASS de F9.1A significa que el candidato es apto para continuar la evaluación de backend. No significa que ptrace haya sido seleccionado como backend definitivo.

#### Limitaciones activas

El candidato inicial observa solo:

- `open`;
- `openat`;
- `creat`;
- `execve`;
- `execveat`.

No observa directamente `getenv` resuelto en memoria, accesos vDSO ni todas las clases de dependencia declaradas en F9.0.

La resolución de rutas es lexical y no constituye una identidad completa frente a symlinks, mounts o namespaces.

#### Siguiente gate

F9.1B debe evaluar factibilidad eBPF y el rol complementario de interposición a nivel de biblioteca antes de congelar el backend definitivo.

El corpus F8 no se ejecuta en F9.1A.
