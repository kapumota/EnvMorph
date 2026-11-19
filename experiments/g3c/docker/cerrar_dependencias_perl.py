#!/usr/bin/env python3
"""Cierra el runtime de Perl SIN cambiar los seis paquetes originales.

Se ejecuta una sola vez, antes de congelar G3-C. Usa APT del contenedor
F10 preexistente para descargar, sin instalar, el cierre de `perl`; valida
la instalacion en un contenedor EFIMERO, sin red, junto con los seis .deb.
Solo tras PASS anota paquete/version/SHA-256 en el manifiesto de dependencias.

Uso desde experiments/g3c:
  python3 docker/cerrar_dependencias_perl.py --base-image-id sha256:...
No usar un tag mutable. No usar si hay artefactos de freeze/decisiones.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
DEBS = ROOT / 'debs'
MANIFEST = ROOT / 'MANIFEST.sha256'
LOCK = HERE / 'dependency_debs.json'

sys.path.insert(0, str(HERE))
from preparar_debs import validar


def stop(message):
    raise RuntimeError('STOP_G3C_PERL: ' + message)


def sha(path):
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def call(argv, **kwargs):
    print('EJECUTANDO:', ' '.join(str(x) for x in argv[:6]), flush=True)
    subprocess.run(list(map(str, argv)), check=True, **kwargs)


def confirm_base(image_id):
    if re.fullmatch(r'sha256:[a-f0-9]{64}', image_id) is None:
        stop('se requiere ID de imagen Docker sha256:... (64 hexadecimales)')
    proc = subprocess.run(['docker', 'image', 'inspect', '--format', '{{.Id}}', image_id],
                          capture_output=True, text=True)
    if proc.returncode or proc.stdout.strip() != image_id:
        stop('la imagen base F10 no existe o su ID no coincide')


def verify_manifest():
    call(['sha256sum', '-c', 'MANIFEST.sha256'], cwd=ROOT, stdout=subprocess.DEVNULL)


def verify_preconditions(image_id):
    confirm_base(image_id)
    for name in ('PROTOCOL_FREEZE.json', 'decisions.json', 'SEALED.json'):
        if (ROOT / 'campaign' / name).exists():
            stop('existen artefactos de campaña; se prohíbe modificar dependencias')
    if (ROOT / 'results').exists() and any((ROOT / 'results').iterdir()):
        stop('ya existen resultados de campaña; se prohíbe modificar dependencias')
    verify_manifest()
    found = validar(DEBS)
    if set(found) != {'findutils', 'libimage-exiftool-perl', 'miller', 'patch', 'unzip', 'zip'}:
        stop('los seis paquetes fijados no coinciden con el catalogo previo')
    obj = json.loads(LOCK.read_text())
    if obj.get('base_f10_image_id'):
        stop('base F10 ya fijada en manifiesto; no ejecutar nuevamente')
    if [p['name'] for p in obj['packages']] != ['zip']:
        stop('manifiesto de dependencias modificado; requiere auditoría')
    return obj


def inspect_debs(stage):
    result = []
    for path in sorted(stage.glob('*.deb')):
        info = {}
        for field in ('Package', 'Version', 'Architecture'):
            info[field] = subprocess.check_output(
                ['dpkg-deb', '-f', str(path), field], text=True).strip()
        name = info['Package']
        if not re.fullmatch(r'[a-z0-9][a-z0-9+.-]*', name):
            stop('nombre Debian inesperado: ' + name)
        if info['Architecture'] not in ('amd64', 'all'):
            stop('arquitectura no admitida para: ' + name)
        if path.stat().st_size > 32 * 1024 * 1024:
            stop('dependencia demasiado grande: ' + name)
        result.append({'name': name, 'version': info['Version'], 'sha256': sha(path),
                       'uso': 'cierre de dependencias de Perl para exiftool; previo a congelacion',
                       '_path': path})
    names = [item['name'] for item in result]
    if not result or 'perl' not in names or len(names) != len(set(names)):
        stop('APT no produjo un cierre de Perl completo e inequívoco')
    return result


def with_base(image_id, stage):
    shell = ('set -eu; export DEBIAN_FRONTEND=noninteractive; '
             'apt-get update; '
             'apt-get install -y --download-only --no-install-recommends perl; '
             'cp /var/cache/apt/archives/*.deb /salida/; '
             'test -n "$(ls -A /salida)"')
    # Unicamente adquisición, no hay instalación en la imagen base ni en el host.
    call(['docker', 'run', '--rm', '--network', 'bridge', '--user', '0:0',
          '--mount', f'type=bind,source={stage},target=/salida',
          '--entrypoint', '/bin/sh', image_id, '-ec', shell])


def verify_offline(image_id, stage):
    # Instalación solo en contenedor descartable. Los .deb ya preparados no se alteran.
    shell = ('set -eu; '
             'dpkg -i /seis/*.deb /extras/*.deb; '
             'dpkg --audit; '
             'perl -e "print qq(G3C_PERL_RUNTIME=PASS\\n)"; '
             'exiftool -ver')
    call(['docker', 'run', '--rm', '--network', 'none', '--user', '0:0',
          '--mount', f'type=bind,source={DEBS.resolve()},target=/seis,readonly',
          '--mount', f'type=bind,source={stage},target=/extras,readonly',
          '--entrypoint', '/bin/sh', image_id, '-ec', shell])


def updated_sha_manifest(changes):
    entries = {}
    for line in MANIFEST.read_text(encoding='utf-8').splitlines():
        digest, sep, name = line.partition('  ')
        if not sep or name in entries:
            stop('MANIFEST.sha256 original inválido')
        entries[name] = digest
    if 'docker/dependency_debs.json' not in entries:
        stop('la dependencia no figura en MANIFEST.sha256')
    entries.update(changes)
    return ''.join(f'{entries[name]}  {name}\n' for name in sorted(entries)).encode()


def lock_closure(obj, image_id, stage, packages):
    previous_lock = LOCK.read_bytes()
    previous_manifest = MANIFEST.read_bytes()
    created = []
    try:
        for dep in packages:
            target = DEBS / dep['_path'].name
            if target.exists():
                stop('destino existente, no sobrescribir: ' + target.name)
            with dep['_path'].open('rb') as src, target.open('xb') as dst:
                shutil.copyfileobj(src, dst, 1024 * 1024)
                dst.flush(); os.fsync(dst.fileno())
            created.append(target)
            if sha(target) != dep['sha256']:
                stop('copia de dependencia no coincide: ' + dep['name'])
        obj['base_f10_image_id'] = image_id
        obj['nota_cierre'] = ('Dependencias Perl calculadas con APT firmado dentro de la imagen '
                             'F10 fijada; instaladas y verificadas offline en contenedor descartable. '
                             'La base debe conservarse y exportarse para reproducción independiente.')
        obj['packages'] += [dict((k, v) for k, v in dep.items() if k != '_path')
                            for dep in packages]
        new_lock = (json.dumps(obj, ensure_ascii=False, indent=2) + '\n').encode()
        changes = {'docker/dependency_debs.json': hashlib.sha256(new_lock).hexdigest()}
        new_manifest = updated_sha_manifest(changes)
        # No usar reemplazo de manifiesto hasta validar que todos los paquetes están presentes.
        LOCK.write_bytes(new_lock)
        MANIFEST.write_bytes(new_manifest)
        validar(DEBS)
        verify_manifest()
    except BaseException:
        LOCK.write_bytes(previous_lock)
        MANIFEST.write_bytes(previous_manifest)
        for path in created:
            path.unlink(missing_ok=True)
        raise
    print('G3C_PERL_CIERRE=PASS')
    print('BASE_F10_ID=' + image_id)
    print('DEPENDENCIAS_PERL_FIJADAS=' + str(len(packages)))
    print('SIGUIENTE=verificar_docker.sh (sin volver a construir F10)')


def main(argv=None):
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--base-image-id', required=True,
                   help='ID exacto de la imagen F10 que construyó Docker')
    args = p.parse_args(argv)
    obj = verify_preconditions(args.base_image_id)
    with tempfile.TemporaryDirectory(prefix='.g3c-perl-stage-', dir=DEBS) as temporary:
        stage = Path(temporary).resolve()
        with_base(args.base_image_id, stage)
        packages = inspect_debs(stage)
        print('CIERRE_PROPUESTO=' + ', '.join(
            f'{item["name"]}={item["version"]}:{item["sha256"][:12]}'
            for item in packages))
        verify_offline(args.base_image_id, stage)
        lock_closure(obj, args.base_image_id, stage, packages)


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, OSError, ValueError, KeyError,
            subprocess.CalledProcessError) as exc:
        print('STOP_G3C_PERL=' + str(exc), file=sys.stderr)
        sys.exit(3)
