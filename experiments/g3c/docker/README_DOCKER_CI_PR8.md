### Operacion Docker y CI de G3-C, precongelacion

Esta extension es **solo de ingenieria**. No autoriza la campana S/H, no altera el pool ni sustituye evidencia. La version cientifica G3-C v0.3 sigue pendiente de ratificacion.

#### 1. Pruebas locales

Desde la raiz del repositorio:

```bash
cargo fmt --check
cargo test --locked
cargo build --release --locked
cd experiments/g3c
sha256sum -c MANIFEST.sha256
python3 docker/preparar_debs.py plan
G3C_F6V2_BIN="$PWD/../../target/release/envmorph-f6v2" python3 -m unittest discover -s tests -p 'test_g3c*.py' -v
```

#### 2. Materializar los seis .deb fijados

Se requieren los cinco primarios mas el **auxiliar zip** para generar el fixture de `unzip`. El auxiliar es un paquete del pool anterior, pero permanece excluido del sorteo de primarios. Su SHA-256 procede de `pool_manifest.json`, no de una descarga no verificada.

```bash
python3 docker/preparar_debs.py descargar
python3 docker/preparar_debs.py validar
python3 docker/preparar_debs.py dependencias
```

Se intenta primero `apt-get download`, sin instalar nada en el host. Si el índice APT del equipo no ofrece una versión fijada, el descargador consulta únicamente rutas estáticas del archivo oficial `https://archive.ubuntu.com/ubuntu/` para los seis paquetes originales. Cada descarga queda supeditada a la coincidencia EXACTA de nombre de paquete, versión y SHA-256 predeclarado; se rechazan cambios de versión, redirecciones fuera del archivo oficial y contenido adulterado. El script falla si una version no esta disponible en los repositorios de Ubuntu 24.04 configurados o si el SHA no coincide. No se corrigen versiones ni hashes para que el proceso pase. Si se necesitan dependencias adicionales, documentar version, SHA y funcion en `docker/dependency_debs.json`, completar el cierre transitivo y actualizar `MANIFEST.sha256` **antes** del congelamiento. La construccion de Docker falla ante dependencias ausentes, no ejecuta `apt-get -f`.

#### 3. Docker en modo preparacion

Volver a la raiz y ejecutar:

```bash
cd ../..
bash experiments/g3c/docker/verificar_docker.sh
```

Esta rutina construye la base F10 y G3-C, anota los IDs por contenido, ejecuta `check-install`, `check-env` y los 17 tests dentro de un contenedor sin red y con privilegios reducidos. **No** ejecuta F10 como bookend, no prepara los cinco baselines, ni ejecuta S/H. Los logs nuevos quedan bajo `~/EnvMorph-G3C-preparacion/`.

Si falta `zip`, `openssl`, `tzdata`, `perl` u otra dependencia, o si los valores de versiones del contenedor no corresponden a los declarados, detenerse y resolver el cierre de dependencias mediante un nuevo commit auditable, sin modificar versiones del protocolo para acomodar el host.

**Atencion a la identificacion de imagen:** `docker image inspect .Id` obtiene un ID de imagen local, no necesariamente el digest de un manifiesto OCI publicado en registro. Congelar posteriormente la referencia adecuada y archivar la imagen y sus capas para un segundo host. No llamar al ID local "digest OCI externo".

#### 4. Evidencia y proteccion de archivos

- `debs/*.deb` y los logs locales estan excluidos de Git. Los hashes/versions fijados si estan versionados.
- No correr el script sobre una campaña ya congelada.
- El CI no descarga `.deb`, no requiere Docker y no ejecuta primarios.
- Nunca copiar evidencia historica a carpetas nuevas como si fueran mediciones confirmatorias.
- Antes del siguiente gate cientifico faltan: Docker con dependencias cerradas, F10 autentico, preflight baseline, custodia externa del held-out, prueba de restauracion en otro host y ratificacion firmada.

#### 5. Corrección del cierre transitivo Perl (PR #8, antes de congelar)

El build de Docker verificó seis paquetes fijados pero falló al configurar
`libimage-exiftool-perl` porque faltaba `perl:any` en la imagen F10. No se
modifican los seis paquetes originales. Los componentes de Perl se resuelven
con APT dentro de la imagen F10 local ya creada, y se descargan y prueban en
una instalación efímera sin red. Antes de incorporarlos a `debs/`, el script
comprueba la instalación offline de **todos** los paquetes y hace un lock de
nombre, versión y SHA-256 de cada dependencia.

No es necesario volver a ejecutar el build F10. Conservar su ID `sha256` de
los logs originales en `~/EnvMorph-G3C-preparacion/.../02-f10-image-id.txt`.
Ejemplo concreto del log del 19/11/2026:

```bash
cd experiments/g3c
python3 docker/cerrar_dependencias_perl.py \
  --base-image-id sha256:cc83624c2a797c314cb383a811571a97b7b6cd1e90e6e8c72bf0160862edc461
python3 docker/preparar_debs.py validar
cd ../..
bash experiments/g3c/docker/verificar_docker.sh
```

`verificar_docker.sh` reutiliza **exactamente** la imagen F10 fijada en
`dependency_debs.json`, sin reconstruirla por una etiqueta mutable. La
adquisición de Perl usa red en un contenedor efímero, nunca durante S/H;
la comprobación e instalación de los seis `.deb` originales y los añadidos
ocurre **sin red**. No se usan `apt-get -f`, versiones sustitutas, ni cambios
al código F10/F6-v2 o al protocolo G3-C.

**Limitaciones:** La firma del índice APT y los SHA-256 de los paquetes
adquiridos se comprueban durante la descarga, pero todavía falta archivar
la imagen F10/G3-C por contenido, el listado completo de paquetes instalados
(`dpkg-query -W`), validar F10 real y reproducir en segundo host. Esto sigue
siendo exclusivamente precongelación, no certificación científica.
