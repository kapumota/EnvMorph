### Selección de backend F9

#### Evidencia disponible

F9.1A validó el candidato `agentguard-derived-ptrace-observer` sobre cinco micro-workloads y seis expectativas, con `strace -ff` como referencia y sin ejecutar el corpus F8.

F9.1B añadió una prueba de cobertura complementaria para variables de entorno y una auditoría no invasiva de factibilidad eBPF.

#### Ptrace

El candidato derivado de AgentGuard-FastPath permanece como backend primario candidato porque ya posee ground truth positivo para:

- accesos de archivo;
- rutas relativas;
- `openat` con `dirfd`;
- errores `ENOENT`;
- seguimiento `fork` y `exec`;
- equivalencia de stdout y exit code.

No se afirma que ptrace observe `getenv`, vDSO ni toda dependencia en memoria.

#### Interposición

La biblioteca `LD_PRELOAD` de F9.1B observa una llamada a `getenv` y preserva stdout y exit code en el micro-workload.

La prueba de acceso directo mediante `environ` permanece invisible para el hook. Ese blind spot es intencional y queda registrado.

Por tanto, la interposición solo puede ser una fuente complementaria.

#### eBPF

La auditoría se realizó sin instalar paquetes, sin usar sudo, sin montar tracefs y sin cambiar capabilities o configuración del kernel.

Estado del host:

- clang: True
- bpftool: True
- libbpf por pkg-config: False
- BTF vmlinux: True
- build ready: False
- unprivileged probe ok: True

F9 no selecciona eBPF solo por disponibilidad. Si el host está listo y el probe no privilegiado funciona, el siguiente gate debe construir un prototipo eBPF con ground truth comparable antes de decidir.

#### Estado de decisión

Estado: `selected_for_current_host_constraints`

Selección final congelada en F9.1B: `true`

Siguiente gate: `F9.2`

#### No claims

F9.1B no reclama cobertura completa, superioridad de ptrace, inferioridad de eBPF ni observación exhaustiva de variables de entorno.

El corpus F8 todavía no se ejecuta.
