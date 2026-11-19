#!/usr/bin/env python3
"""F10 inicial/final controlado, una sola vez y conservando intentos fallidos.

Debe ejecutarse en la MISMA imagen congelada del retículo G3-C. Nunca reutiliza
F10 previo, ni modifica expedientes F10 históricos del repositorio.
"""
import argparse
from pathlib import Path
import subprocess
import sys
import re
sys.path.insert(0,str(Path(__file__).resolve().parent))
import g3c_state as G

F10_BENCHMARK='59db7a371740870eda0b50a08a39826e276409cc91b42bcfd48fc7f798a2dff9'

def main(argv=None):
    ap=argparse.ArgumentParser()
    ap.add_argument('--phase',choices=('start','end'),required=True)
    ap.add_argument('--campaign',required=True)
    ap.add_argument('--image-digest',required=True)
    ap.add_argument('--repo-root',required=True)
    a=ap.parse_args(argv)
    try:
        campaign=Path(a.campaign).resolve()
        f=G.json_read(campaign/'PROTOCOL_FREEZE.json')
        if f.get('image_digest')!=a.image_digest or not re.fullmatch(r'sha256:[0-9a-f]{64}',a.image_digest):
            G.fail('digest de imagen distinto al congelado')
        if a.phase=='end' and not (campaign/'F10_START.json').is_file():
            G.fail('falta F10 inicial')
        root=Path(a.repo_root).resolve()
        scripts=['f10.py','verify_f10.py','test_f10.py']
        if any(not(root/'reproduce'/s).is_file() for s in scripts):
            G.fail('falta el ejecutor o verificador F10')
        out=campaign/('F10_'+a.phase.upper()+'_EVIDENCE')
        reservation=campaign/('F10_'+a.phase.upper()+'_RESERVATION.json')
        G.exclusive_write(reservation,G.canonical({'schema':'g3c-f10-reservation-v1',
            'phase':a.phase,'image_digest':a.image_digest,'expected_target_invocations':33}))
        if out.exists():G.fail('evidencia F10 ya existe, se prohíbe reemplazar')
        # Toda falla deja reserva, logs y resultados parciales para auditoría.
        for script,extra in [('f10.py',['--output',str(out)]),
                             ('verify_f10.py',[str(out)]),
                             ('test_f10.py',[str(out)])]:
            command=[sys.executable,str(root/'reproduce'/script),*extra]
            try:
                run=subprocess.run(command,cwd=root,capture_output=True,timeout=300)
            except subprocess.TimeoutExpired as exc:
                G.exclusive_write(campaign/(f'F10_{a.phase.upper()}_{script}.timeout'),
                    b'timeout de etapa F10\n')
                G.fail('timeout en '+script)
            G.exclusive_write(campaign/(f'F10_{a.phase.upper()}_{script}.stdout'),run.stdout)
            G.exclusive_write(campaign/(f'F10_{a.phase.upper()}_{script}.stderr'),run.stderr)
            if run.returncode:G.fail('F10 FALLA en '+script)
        summary=G.json_read(out/'results.json')
        if summary.get('benchmark_sha256')!=F10_BENCHMARK or len(summary.get('results',[]))!=3:
            G.fail('benchmark F10 alterado o incompleto')
        if any(row.get('stages',{}).get('behaviorally_relevant')!='PASS' for row in summary['results']):
            G.fail('F10 no es 3/3')
        if summary.get('decision')!='GO':G.fail('F10 no pasó')
        flag=campaign/('F10_'+a.phase.upper()+'.json')
        G.exclusive_write(flag,G.canonical({'schema':'g3c-f10-bookend-v3','phase':a.phase,
            'benchmark_sha256':F10_BENCHMARK,'results_sha256':G.digest(out/'results.json'),
            'manifest_sha256':G.digest(out/'SHA256SUMS'),
            'image_digest_declarado':a.image_digest,
            'evidence_path':str(out),
            'target_invocations':33,
            'nota':'La equivalencia imagen-id/ejecución requiere auditar logs Docker.'}))
        print('G3C_F10_'+a.phase.upper()+'=PASS_3_DE_3_REGISTRADO')
        return 0
    except (ValueError,FileNotFoundError,subprocess.SubprocessError) as exc:
        print(str(exc),file=sys.stderr)
        return 3
if __name__=='__main__':sys.exit(main())
