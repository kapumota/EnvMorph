#!/usr/bin/env python3
"""Prepara SOLO paquetes Debian fijados previamente (precongelacion).

No instala nada en el host, no modifica specs, pool ni protocolo. Acepta solo
versiones y SHA-256 declarados, incluyendo 'zip' como auxiliar de fixture.

Uso desde experiments/g3c:
  python3 docker/preparar_debs.py plan
  python3 docker/preparar_debs.py descargar
  python3 docker/preparar_debs.py validar

'descargar' exige que debs/ no contenga ningun .deb y copia de forma exclusiva.
Si hay un fallo no elimina evidencia ni rellena manifiestos de forma automatica.
"""
import argparse
import hashlib
import json
import os
from urllib.parse import quote, urlparse
from urllib.request import Request, urlopen
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DEST = ROOT / 'debs'

# Rutas oficiales para el catalogo previamente fijado (Ubuntu Noble, amd64).
# NO se infieren rutas nuevas, NO se cambian versiones y NO se aprueban mirrors.
# El contenido descargado debe cumplir la huella SHA-256 del spec/manifiesto.
OFFICIAL_POOL = {
    ('findutils', '4.9.0-5build1'):
        'pool/main/f/findutils/findutils_4.9.0-5build1_amd64.deb',
    ('libimage-exiftool-perl', '12.76+dfsg-1'):
        'pool/universe/libi/libimage-exiftool-perl/libimage-exiftool-perl_12.76+dfsg-1_all.deb',
    ('miller', '6.11.0-1ubuntu0.3'):
        'pool/universe/m/miller/miller_6.11.0-1ubuntu0.3_amd64.deb',
    ('patch', '2.7.6-7build3'):
        'pool/main/p/patch/patch_2.7.6-7build3_amd64.deb',
    ('unzip', '6.0-28ubuntu4.1'):
        'pool/main/u/unzip/unzip_6.0-28ubuntu4.1_amd64.deb',
    ('zip', '3.0-13ubuntu0.2'):
        'pool/main/z/zip/zip_3.0-13ubuntu0.2_amd64.deb',
}
ORIGIN = 'https://archive.ubuntu.com/ubuntu/'
MAX_DEB_BYTES = 32 * 1024 * 1024


def abort(msg):
    raise RuntimeError('STOP_G3C_DEBS: ' + msg)


def specs():
    seen = {}
    for p in sorted((ROOT/'workflows').glob('*/spec.json')):
        obj = json.loads(p.read_text(encoding='utf-8'))
        if obj.get('kind') != 'campaign':
            continue
        name = obj['package']
        if name in seen:
            abort('paquete repetido: ' + name)
        seen[name] = (obj['version'], obj['deb_sha256'], p.name)
    manifest=json.loads((ROOT/'docker/dependency_debs.json').read_text(encoding='utf-8'))
    if manifest.get('schema')!='g3c-dependency-debs-v1':
        abort('manifest de dependencias sin esquema')
    for obj in manifest.get('packages',[]):
        name = obj['name']
        if name in seen:abort('paquete auxiliar duplicado: '+name)
        seen[name] = (obj['version'],obj['sha256'],'auxiliar')
    for name, (version, digest, _) in seen.items():
        if not name or not version or len(digest)!=64 or any(c not in '0123456789abcdef' for c in digest):
            abort('identidad no fijada: '+name)
    if len([p for p in seen.values() if p[2]!='auxiliar'])!=5:
        abort('cohorte primaria no corresponde a cinco paquetes')
    return seen


def run(args,cwd=None):
    return subprocess.run(args,cwd=cwd,check=True,text=True,capture_output=True)


def validar(ruta):
    expected=specs()
    current={}
    for p in sorted(ruta.glob('*.deb')):
        name=run(['dpkg-deb','-f',str(p),'Package']).stdout.strip()
        version=run(['dpkg-deb','-f',str(p),'Version']).stdout.strip()
        sha=hashlib.sha256(p.read_bytes()).hexdigest()
        if name not in expected:abort('paquete ajeno: '+name)
        if name in current:abort('paquete repetido: '+name)
        if (version,sha)!=expected[name][:2]:abort('version o SHA incorrecto: '+name)
        current[name]=p
    missing=set(expected)-set(current)
    if missing:abort('faltan paquetes: '+', '.join(sorted(missing)))
    print('G3C_PAQUETES_VERIFICADOS='+str(len(current)))
    return current


def official_url(name, version):
    route = OFFICIAL_POOL.get((name, version))
    if not route:
        abort('no existe ruta oficial fijada para ' + name + '=' + version)
    return ORIGIN + quote(route, safe='/._-')


def descargar_archivo_oficial(name, version, sha, stage):
    if run(['dpkg','--print-architecture']).stdout.strip() != 'amd64':
        abort('esta descarga oficial esta fijada exclusivamente para amd64')
    url = official_url(name, version)
    filename = OFFICIAL_POOL[(name, version)].rsplit('/', 1)[-1]
    output = stage / filename
    print('Consultando archivo Ubuntu:', url, flush=True)
    request = Request(url, headers={'User-Agent': 'EnvMorph-G3C-pr8/1.1'})
    digest = hashlib.sha256()
    count = 0
    try:
        with urlopen(request, timeout=60) as response:
            if urlparse(response.geturl()).hostname != 'archive.ubuntu.com':
                abort('redireccion fuera del archivo oficial Ubuntu')
            with output.open('xb') as target:
                while True:
                    chunk = response.read(1024*1024)
                    if not chunk:
                        break
                    count += len(chunk)
                    if count > MAX_DEB_BYTES:
                        abort('paquete excede tamaño maximo: ' + name)
                    digest.update(chunk)
                    target.write(chunk)
                target.flush()
                os.fsync(target.fileno())
    except (OSError, ValueError) as exc:
        abort('no se pudo descargar ' + name + ' del archivo oficial: ' + str(exc))
    if count == 0 or digest.hexdigest() != sha:
        abort('SHA-256 incorrecto en paquete oficial: ' + name)
    if run(['dpkg-deb', '-f', str(output), 'Package']).stdout.strip() != name:
        abort('identidad Debian incorrecta: ' + name)
    if run(['dpkg-deb', '-f', str(output), 'Version']).stdout.strip() != version:
        abort('version Debian incorrecta: ' + name)
    print('DESCARGA_OFICIAL_VERIFICADA=' + name)


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('accion',choices=['plan','descargar','validar','dependencias'])
    args=parser.parse_args()
    expected=specs()
    if args.accion=='plan':
        for name, (version,sha,origin) in sorted(expected.items()):
            print(f'{name}\t{version}\t{sha}\t{origin}')
        print('G3C_PLAN_PAQUETES='+str(len(expected)))
        return
    if args.accion=='validar':
        validar(DEST)
        return
    if args.accion=='dependencias':
        validar(DEST)
        for p in sorted(DEST.glob('*.deb')):
            name=run(['dpkg-deb','-f',str(p),'Package']).stdout.strip()
            depende=run(['dpkg-deb','-f',str(p),'Depends']).stdout.strip()
            print(f'{name}: {depende or "sin Depends"}')
        print('Revisar el cierre transitivo frente a la imagen F10 antes del freeze.')
        return
    if args.accion=='descargar':
        DEST.mkdir(parents=True,exist_ok=True)
        if any(DEST.glob('*.deb')):
            abort('ya existen .deb: no sobreescribir; ejecutar validar o revisar manualmente')
        if not shutil_which('apt-get') or not shutil_which('dpkg-deb'):
            abort('se requieren apt-get y dpkg-deb en el host de preparacion Ubuntu 24.04')
        with tempfile.TemporaryDirectory(prefix='g3c-debs-') as tmp:
            stage=Path(tmp)
            for name,(version, _,_) in sorted(expected.items()):
                print('Descargando:',name,version,flush=True)
                try:
                    run(['apt-get','download',f'{name}={version}'],cwd=stage)
                except subprocess.CalledProcessError as exc:
                    print('APT no encuentra la version fijada; se prueba archivo oficial. Motivo: '
                          + (exc.stderr[-250:] if exc.stderr else 'sin detalle'), flush=True)
                    try:
                        descargar_archivo_oficial(name, version, expected[name][1], stage)
                    except Exception as failure:
                        abort(f'no se obtuvo {name}={version}; no se han cambiado versiones ni hashes. '
                              + str(failure))
            files=validar(stage)
            for name,p in sorted(files.items()):
                destination=DEST/p.name
                with open(p,'rb') as src,open(destination,'xb') as dst:
                    while True:
                        block=src.read(1024*1024)
                        if not block:break
                        dst.write(block)
                    dst.flush();os.fsync(dst.fileno())
            validar(DEST)
            print('No se han instalado paquetes ni alterado el protocolo.')


def shutil_which(name):
    from shutil import which
    return which(name) is not None

if __name__=='__main__':
    try:
        main()
    except (RuntimeError,OSError,KeyError,ValueError,subprocess.CalledProcessError) as exc:
        print(str(exc),file=sys.stderr)
        sys.exit(3)
