#!/usr/bin/env python3
"""Guardias transaccionales de G3-C v0.3, sin dependencias externas.

El ledger solo puede crecer mediante este módulo. Los hashes detectan cambios
accidentales, pero no sustituyen la custodia independiente ni un sello externo.
"""
import contextlib
import datetime
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

BUDGET = 180
BOOKEND_RESERVED = 66
WORKFLOWS = ('findutils', 'miller', 'unzip', 'exiftool', 'patch')
S = ('000','100','010','001')
H = ('110','101','011','111')
N = ('NULLA','NULLB')


def fail(reason):
    raise ValueError('G3C_STOP: ' + reason)


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def json_read(path):
    return json.loads(Path(path).read_text(encoding='utf-8'))


def canonical(data):
    return (json.dumps(data, sort_keys=True, indent=2, ensure_ascii=False)+'\n').encode('utf-8')


def exclusive_write(path, raw):
    """CREATE_NEW. Nunca sustituye un archivo, ni siquiera tras un error."""
    p = Path(path)
    p.parent.mkdir(parents=True, exist_ok=True)
    fd = os.open(p, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, 'O_NOFOLLOW', 0), 0o600)
    try:
        with os.fdopen(fd,'wb') as dst:
            dst.write(raw)
            dst.flush()
            os.fsync(dst.fileno())
    except Exception:
        # Si hay escritura incompleta, se conserva para la auditoría.
        raise
    return digest(p)


def require_freeze(campaign, root, name=None):
    root=Path(root);campaign=Path(campaign)
    p=campaign/'PROTOCOL_FREEZE.json'
    if not p.is_file():fail('falta congelamiento')
    f=json_read(p)
    if f.get('schema') != 'g3c-freeze-v3':fail('freeze anterior o incompleto')
    if name is not None and name not in f.get('workflows',[]):fail('workflow fuera de la cohorte congelada')
    paths={'harness_sha256':root/'harness/g3c_harness.py',
           'state_sha256':root/'harness/g3c_state.py',
           'decide_sha256':root/'harness/g3c_decide.py',
           'factors_sha256':root/'factors.json',
           'pool_manifest_sha256':root/'pool_manifest.json'}
    for key, src in paths.items():
        if not src.is_file() or f.get(key)!=digest(src):fail('hash congelado distinto: '+key)
    for w in f['workflows']:
        p=root/'workflows'/w/'spec.json'
        if not p.is_file() or digest(p)!=f.get('spec_sha256',{}).get(w):fail('spec alterado: '+w)
    for rel, expected in f.get('file_sha256',{}).items():
        src=root/rel
        if (not src.is_file()) or digest(src)!=expected:
            fail('archivo auxiliar o fixture modificado: '+rel)
    if not f.get('file_sha256') or not f.get('tool_binary_sha256'):
        fail('freeze sin huellas de auxiliares')
    for w in f['workflows']:
        spec=json_read(root/'workflows'/w/'spec.json')
        b=Path(spec['tool_binary'])
        if not b.is_file() or digest(b)!=f['tool_binary_sha256'].get(w):
            fail('binario de campaña no coincide: '+w)
    import shutil
    for cmd, expected in f.get('aux_binary_sha256',{}).items():
        p=shutil.which(cmd,path='/usr/bin:/bin')
        if not p or digest(p)!=expected:fail('auxiliar de campaña distinto: '+cmd)
    pub=campaign/'CUSTODIAN_PUBLIC_KEY.pem'
    if not pub.is_file() or digest(pub)!=f.get('custodian_public_key_sha256'):
        fail('clave pública del custodio ausente o alterada')
    if f.get('budget_max')!=BUDGET or f.get('bookend_reserve')!=BOOKEND_RESERVED:
        fail('presupuesto no ratificado')
    return f


def get_run_records(results, name, cells):
    p=Path(results)/name
    result={}
    expected_prefixes=set(cells)
    for extra in p.glob('*.json') if p.is_dir() else ():
        stem_name=extra.stem
        if not re.fullmatch(r'(000|100|010|001|110|101|011|111|NULLA|NULLB)_c[12]_a1',stem_name):
            fail('registro ajeno, duplicado o reintento no autorizado: '+str(extra))
    for cell in cells:
        runs=[]
        for c in (1,2):
            q=p/f'{cell}_c{c}_a1.json'
            if not q.is_file():fail(f'faltan confirmaciones en {name}/{cell}')
            r=json_read(q)
            if r.get('cell')!=cell or r.get('workflow')!=name or r.get('confirmation')!=c or r.get('attempt')!=1:
                fail('metadata de corrida inconsistente: '+str(q))
            if not(r.get('ok') and r.get('applied_env_ok') and r.get('fixture_ok')):
                fail('corrida fallida, tratamiento bloqueado: '+str(q))
            stdout=q.with_suffix('.stdout')
            if not stdout.is_file() or digest(stdout)!=r['oracle_stdout_sha256']:
                fail('stdout ausente o modificado: '+str(stdout))
            runs.append(r)
        if runs[0]['oracle_stdout_sha256']!=runs[1]['oracle_stdout_sha256']:
            fail(f'oráculo inestable: {name}/{cell}')
        result[cell]=runs
    return result


def validate_negatives(results, workflows):
    statuses={}
    for w in workflows:
        cells=get_run_records(results,w,S+N)
        base=cells['000'][0]['oracle_stdout_sha256']
        nulls=[cells[n][0]['oracle_stdout_sha256']==base for n in N]
        statuses[w]=nulls
        if not all(nulls):fail(f'CONTROL_NEGATIVE_FAIL en {w}')
    return statuses


def audit_present_results(results,workflows):
    """Verifica bytes primarios de toda corrida registrada, incluso las H parciales."""
    for wf in workflows:
        directory=Path(results)/wf
        if not directory.is_dir():fail('directorio de resultados ausente: '+wf)
        all_stems=set()
        for p in directory.glob('*.json'):
            stem=p.stem
            if not re.fullmatch(r'(000|100|010|001|110|101|011|111|NULLA|NULLB)_c[12]_a1',stem):
                fail('corrida fuera del protocolo: '+str(p))
            rec=json_read(p)
            cell=rec.get('cell')
            expected_stage=('H' if cell in H else ('N' if cell in N else 'S'))
            if (rec.get('workflow')!=wf or rec.get('stage')!=expected_stage or
                rec.get('attempt')!=1 or
                rec.get('confirmation')!=int(stem.split('_c')[1][0]) or
                stem!=f'{cell}_c{rec["confirmation"]}_a1'):
                fail('metadatos no concuerdan con registro: '+str(p))
            target=p.with_suffix('.stdout')
            if not target.is_file() or digest(target)!=rec.get('oracle_stdout_sha256'):
                fail('evidencia stdout alterada: '+str(target))
            all_stems.add(stem)
        for p in directory.glob('*.stdout'):
            if p.stem not in all_stems:
                fail('stdout sin registro asociado: '+str(p))


def evidence_snapshot(results,workflows):
    """Huella de evidencia S/N sobre rutas y bytes, excluyendo H."""
    lines=[]
    for w in workflows:
        get_run_records(results,w,S+N)
        for cell in S+N:
            for conf in (1,2):
                for ext in ('json','stdout'):
                    p=Path(results)/w/f'{cell}_c{conf}_a1.{ext}'
                    lines.append(f'{w}/{p.name} {digest(p)}')
    return hashlib.sha256(('\n'.join(lines)+'\n').encode()).hexdigest()


def ensure_s_only(results, workflows):
    for name in workflows:
        d=Path(results)/name
        if d.is_dir():
            for c in H:
                if any(d.glob(c+'_*.json')) or any(d.glob(c+'_*.stdout')):
                    fail('fuga de held-out en '+name+'/'+c)


def require_f10_start(campaign,freeze):
    p=Path(campaign)/'F10_START.json'
    if not p.is_file():fail('F10 3/3 de inicio no registrado')
    row=json_read(p)
    if (row.get('schema')!='g3c-f10-bookend-v3' or row.get('phase')!='start'
        or row.get('image_digest_declarado')!=freeze.get('image_digest')
        or row.get('target_invocations')!=33):
        fail('F10 inicio distinto de la imagen congelada')
    e=Path(row.get('evidence_path',''))
    if (not e.is_relative_to(Path(campaign).resolve()) or
        not (e/'results.json').is_file() or not (e/'SHA256SUMS').is_file() or
        digest(e/'results.json')!=row.get('results_sha256') or
        digest(e/'SHA256SUMS')!=row.get('manifest_sha256')):
        fail('evidencia F10 inicial alterada o inaccesible')


def ensure_not_sealed(campaign):
    c=Path(campaign)
    if any((c/n).exists() for n in ('SEALED.json','decisions.json','HELDOUT_AUTHORIZATION.json')):
        fail('campaña ya sellada, no volver a decidir ni ejecutar S/N')


def verify_signature(campaign,receipt_path,signature_path):
    """Se requiere firma Ed25519 del recibo con clave pública congelada."""
    key=Path(campaign)/'CUSTODIAN_PUBLIC_KEY.pem'
    if not key.is_file():fail('falta clave pública del custodio')
    result=subprocess.run(['openssl','pkeyutl','-verify','-pubin','-inkey',str(key),
        '-rawin','-in',str(receipt_path),'-sigfile',str(signature_path)],
        capture_output=True,text=True,timeout=20)
    if result.returncode!=0:fail('firma del custodio inválida o ausente')


def record_receipt(campaign,receipt_path,signature_path):
    """Se acepta exclusivamente un recibo firmado; el sello temporal se audita aparte."""
    c=Path(campaign)
    verify_signature(c,receipt_path,signature_path)
    rec=json_read(receipt_path)
    if rec.get('schema')!='g3c-external-receipt-v1':fail('receipt inválido')
    seal=json_read(c/'SEALED.json')
    if rec.get('decisions_sha256')!=seal.get('decisions_sha256'):
        fail('recibo externo no corresponde a las decisiones')
    if rec.get('freeze_sha256')!=digest(c/'PROTOCOL_FREEZE.json'):
        fail('recibo externo no corresponde al freeze')
    url=rec.get('url','')
    if not isinstance(url,str) or not re.fullmatch(r'https://[^\s]{8,500}',url):
        fail('faltan URL y sello externo verificables')
    if not rec.get('attestation_utc') or not rec.get('custodian'):
        fail('falta fecha o custodio')
    if rec.get('evidence_sha256')!=seal.get('evidence_sha256'):
        fail('recibo externo no incluye la misma evidencia S/N')
    exclusive_write(c/'EXTERNAL_RECEIPT.json',Path(receipt_path).read_bytes())
    exclusive_write(c/'EXTERNAL_RECEIPT.sig',Path(signature_path).read_bytes())
    raw=canonical({'schema':'g3c-heldout-authorization-v1',
                   'decisions_sha256':seal['decisions_sha256'],
                   'freeze_sha256':digest(c/'PROTOCOL_FREEZE.json'),
                   'receipt_sha256':digest(c/'EXTERNAL_RECEIPT.json'),
                   'signature_sha256':digest(c/'EXTERNAL_RECEIPT.sig'),
                   'receipt_url':url,
                   'external_timestamp_unverified':True})
    exclusive_write(c/'HELDOUT_AUTHORIZATION.json',raw)


def require_heldout_authorization(campaign):
    c=Path(campaign)
    seal=c/'SEALED.json';dec=c/'decisions.json';auth=c/'HELDOUT_AUTHORIZATION.json'
    if not all(p.is_file() for p in (seal,dec,auth)):
        fail('H bloqueado: faltan decisiones, sello y recibo externo')
    se=json_read(seal);au=json_read(auth)
    if se.get('decisions_sha256')!=digest(dec) or au.get('decisions_sha256')!=digest(dec):
        fail('decisiones modificadas tras sellado')
    if au.get('freeze_sha256')!=digest(c/'PROTOCOL_FREEZE.json'):
        fail('protocolo modificado tras sellado')
    if not au.get('receipt_sha256') or not au.get('receipt_url'):
        fail('recibo externo ausente')
    r=c/'EXTERNAL_RECEIPT.json'
    sig=c/'EXTERNAL_RECEIPT.sig'
    if (not r.is_file() or digest(r)!=au['receipt_sha256'] or
        not sig.is_file() or digest(sig)!=au.get('signature_sha256')):
        fail('evidencia externa o firma no coincide con el sello')
    verify_signature(c,r,sig)


@contextlib.contextmanager
def ledger_lock(campaign):
    c=Path(campaign);c.mkdir(parents=True, exist_ok=True)
    lock=c/'BUDGET.lock'
    with open(lock,'a+b') as f:
        fcntl.flock(f,fcntl.LOCK_EX)
        yield c
        fcntl.flock(f,fcntl.LOCK_UN)


def read_ledger(c):
    p=Path(c)/'BUDGET.jsonl'
    entries=[]
    if p.exists():
        for line in p.read_text(encoding='utf-8').splitlines():
            if line.strip():entries.append(json.loads(line))
    return entries


def reserve(campaign,workflow,cell,stage,confirmations,attempt=1):
    if confirmations!=2:fail('cada celda exige exactamente 2 confirmaciones')
    if attempt!=1:fail('reintentos requieren revisión externa y nueva autorización, no están automatizados')
    if stage not in ('preflight','S','N','H'):fail('etapa no permitida')
    with ledger_lock(campaign) as c:
        p=c/'BUDGET.jsonl';entries=read_ledger(c)
        used=sum(x['cost'] for x in entries)
        if used+confirmations+BOOKEND_RESERVED>BUDGET:fail('presupuesto 180 agotado, contando 66 invocaciones F10 reservadas')
        key=f'{workflow}:{cell}:{stage}:a{attempt}'
        if any(x['key']==key for x in entries):fail('ejecución ya reservada: '+key)
        event={'key':key,'cost':confirmations,'stage':stage,'utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
               'bookend_reserve':BOOKEND_RESERVED}
        with open(p,'ab') as o:
            o.write((json.dumps(event,sort_keys=True,separators=(',',':'))+'\n').encode('utf-8'));o.flush();os.fsync(o.fileno())
        return used+confirmations+BOOKEND_RESERVED
