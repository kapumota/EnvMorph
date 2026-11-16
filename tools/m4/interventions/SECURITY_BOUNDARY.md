# M4-G2: frontera de seguridad de la materialización

El planificador **no ejecuta workflows** y no constituye un sandbox.

La política de desarrollo acepta árboles de directorios y archivos regulares
únicamente. Rechaza enlaces simbólicos, FIFO, sockets y dispositivos en todo el
workspace y directorio de fixtures, aunque no sean factores declarados.
Limita la inspección de cada árbol a 50 000 entradas y 2 GiB de datos.
Antes de copiar, fija una huella integral de nombres y contenidos; verifica
que cada copia coincide y que los árboles originales siguen iguales al final.
Estas restricciones pueden excluir proyectos upstream con venv o symlinks;
esas exclusiones deben registrarse, no retocarse sin protocolo.

El control de salida canónica de A1 bloquea alias simbólicos que escriban
accidentalmente dentro del workspace; esta defensa no garantiza seguridad
contra escritores concurrentes, mutación de directorios entre comprobaciones
(TOCTOU), rutas montadas de forma maliciosa ni ataques desde root.

Antes de incorporar un ejecutor externo se requiere aislamiento real de
procesos/sistema de archivos, usuario sin privilegios, límites CPU/RAM/tiempo,
restricciones de red, control de mounts y revisión específica de la semántica
de entrada de cada aplicación. No ejecutar repositorios no confiables con
la mera protección de este preflight.
