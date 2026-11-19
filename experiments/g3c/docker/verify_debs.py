#!/usr/bin/env python3
"""Valida que cada .deb sea único y esté fijado por paquete, versión y hash.

Exige que TODOS los .deb presentes, incluidos auxiliares, estén declarados.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

def info(path,field):
    return subprocess.check_output(['dpkg-deb','-f',str(path),field],text=True).strip()

def main():
    root=Path(sys.argv[1]);directory=Path(sys.argv[2]);expected={}
    for file in sorted((root/'workflows').glob('*/spec.json')):
        spec=json.loads(file.read_text())
        if spec.get('kind')!='campaign':continue
        name=spec['package']
        expected[name]={'version':spec['version'],'sha256':spec['deb_sha256']}
    extra=json.loads((root/'docker/dependency_debs.json').read_text())
    if extra.get('schema')!='g3c-dependency-debs-v1':raise ValueError('manifest de dependencias inválido')
    for dep in extra['packages']:
        if dep['name'] in expected:raise ValueError('dependencia duplicada en lista')
        expected[dep['name']]={'version':dep['version'],'sha256':dep['sha256']}
    found={}
    for path in sorted(directory.glob('*.deb')):
        name=info(path,'Package')
        if name not in expected:raise ValueError('paquete no declarado: '+name)
        if name in found:raise ValueError('paquete duplicado: '+name)
        version=info(path,'Version')
        h=hashlib.sha256(path.read_bytes()).hexdigest()
        if version!=expected[name]['version'] or h!=expected[name]['sha256']:
            raise ValueError('versión/hash incorrecto de '+name)
        found[name]=str(path)
    missing=set(expected)-set(found)
    if missing:raise ValueError('faltan .deb: '+', '.join(sorted(missing)))
    print('G3C_DEBS_VERIFIED=%d'%len(found))
if __name__=='__main__':
    try:main()
    except (ValueError,KeyError,OSError,subprocess.CalledProcessError) as exc:
        print('G3C_DEBS_FAIL='+str(exc),file=sys.stderr);sys.exit(3)
