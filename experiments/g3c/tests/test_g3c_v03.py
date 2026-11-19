"""Pruebas adversariales de seguridad metodológica para G3-C v0.3.

Estos tests NO son evidencia confirmatoria de los workflows primarios.
"""
import contextlib
import hashlib
import io
import itertools
import json
import pathlib
import sys
import tempfile
import subprocess
import unittest
HERE=pathlib.Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parent/'harness'))
import g3c_harness as H
import g3c_state as G

class Guards(unittest.TestCase):
    def test_no_overwrite(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'a.json'
            G.exclusive_write(p,b'primero')
            with self.assertRaises(FileExistsError):G.exclusive_write(p,b'segundo')
            self.assertEqual(p.read_bytes(),b'primero')

    def test_budget_hard_cap(self):
        with tempfile.TemporaryDirectory() as t:
            for i in range(57):
                used=G.reserve(t,'w',f'c{i:03d}','S',2)
            self.assertEqual(used,180)
            with self.assertRaises(ValueError):G.reserve(t,'w','overflow','H',2)
            self.assertEqual(len(G.read_ledger(t)),57)

    def test_budget_duplicate(self):
        with tempfile.TemporaryDirectory() as t:
            G.reserve(t,'w','000','preflight',2)
            with self.assertRaises(ValueError):G.reserve(t,'w','000','preflight',2)

    def test_confirmations_mandatory(self):
        with tempfile.TemporaryDirectory() as t:
            for n in (0,1,3,5):
                with self.assertRaises(ValueError):G.reserve(t,'w','000','S',n)

    def test_retries_not_unilaterally_allowed(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(ValueError):G.reserve(t,'w','000','S',2,attempt=2)

    def test_no_h_without_receipt(self):
        with tempfile.TemporaryDirectory() as t:
            c=pathlib.Path(t)
            G.exclusive_write(c/'decisions.json',b'{}')
            G.exclusive_write(c/'SEALED.json',G.canonical({'decisions_sha256':G.digest(c/'decisions.json')}))
            with self.assertRaises(ValueError):G.require_heldout_authorization(c)

    def test_h_after_receipt_and_tamper(self):
        with tempfile.TemporaryDirectory() as t:
            c=pathlib.Path(t)
            priv=c/'private.pem';pub=c/'CUSTODIAN_PUBLIC_KEY.pem'
            subprocess.run(['openssl','genpkey','-algorithm','ED25519','-out',str(priv)],check=True,capture_output=True)
            subprocess.run(['openssl','pkey','-in',str(priv),'-pubout','-out',str(pub)],check=True,capture_output=True)
            G.exclusive_write(c/'PROTOCOL_FREEZE.json',b'{}')
            G.exclusive_write(c/'decisions.json',b'{}')
            h=G.digest(c/'decisions.json');fs=G.digest(c/'PROTOCOL_FREEZE.json')
            G.exclusive_write(c/'SEALED.json',G.canonical({'decisions_sha256':h,'evidence_sha256':'abc'}))
            receipt={'schema':'g3c-external-receipt-v1','decisions_sha256':h,'freeze_sha256':fs,
                'evidence_sha256':'abc','url':'https://osf.io/example/','custodian':'firmante externo','attestation_utc':'2026-11-17T00:00:00Z'}
            rp=c/'received.json';G.exclusive_write(rp,G.canonical(receipt))
            sig=c/'received.sig'
            subprocess.run(['openssl','pkeyutl','-sign','-inkey',str(priv),'-rawin','-in',str(rp),
                '-out',str(sig)],check=True,capture_output=True)
            G.record_receipt(c,rp,sig)
            G.require_heldout_authorization(c)
            with self.assertRaises(FileExistsError):G.record_receipt(c,rp,sig)
            (c/'decisions.json').write_text('{ }')
            with self.assertRaises(ValueError):G.require_heldout_authorization(c)
            (c/'decisions.json').write_text('{}')
            (c/'EXTERNAL_RECEIPT.sig').write_bytes(b'firma falsa')
            with self.assertRaises(ValueError):G.require_heldout_authorization(c)

    def test_negative_controls_fail_closed(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'w';p.mkdir()
            for cell in G.S+G.N:
                for k in (1,2):
                    path=p/f'{cell}_c{k}_a1'
                    data=(b'DIFERENTE' if cell=='NULLB' else b'NORMAL')
                    G.exclusive_write(path.with_suffix('.stdout'),data)
                    rec={'cell':cell,'workflow':'w','confirmation':k,'attempt':1,
                         'ok':True,'applied_env_ok':True,'fixture_ok':True,
                         'oracle_stdout_sha256':hashlib.sha256(data).hexdigest()}
                    G.exclusive_write(path.with_suffix('.json'),G.canonical(rec))
            with self.assertRaisesRegex(ValueError,'CONTROL_NEGATIVE_FAIL'):
                G.validate_negatives(t,['w'])

    def test_no_h_leak(self):
        with tempfile.TemporaryDirectory() as t:
            p=pathlib.Path(t)/'w';p.mkdir()
            (p/'110_c1_a1.stdout').write_bytes(b'leak')
            with self.assertRaisesRegex(ValueError,'fuga'):
                G.ensure_s_only(t,['w'])

    def test_selftest_cannot_overwrite(self):
        with tempfile.TemporaryDirectory() as t:
            x=pathlib.Path(t)
            args=['run-cell','--workflow','_selftest_null','--cell','100','--results',str(x/'res'),
                  '--campaign',str(x/'camp'),'--work-root',str(x/'work')]
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(H.main(args),0)
            evidence=(x/'res/_selftest_null/100_c1_a1.stdout')
            self.assertEqual(evidence.read_bytes(),b'constante\n')
            with self.assertRaises(SystemExit) as ex,contextlib.redirect_stderr(io.StringIO()):
                H.main(args)
            self.assertEqual(ex.exception.code,3)
            self.assertEqual(evidence.read_bytes(),b'constante\n')

    def test_preflight_isolated_from_S(self):
        with tempfile.TemporaryDirectory() as t:
            x=pathlib.Path(t)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(H.main(['preflight','--workflow','_selftest_null','--results',str(x/'res'),
                    '--campaign',str(x/'camp')]),0)
            self.assertFalse((x/'res/_selftest_null/000_c1_a1.json').exists())
            self.assertTrue((x/'res/preflight/_selftest_null/000_c1_a1.json').exists())

class ModelProof(unittest.TestCase):
    def test_no_possible_win_over_C2_with_S_H(self):
        """Prueba exhaustiva sobre las 8 asignaciones de etiquetas de singletons."""
        family=[lambda bits:False,lambda bits:True]
        for length in (1,2):
            for idx in itertools.combinations(range(3),length):
                for val in itertools.product((0,1),repeat=length):
                    family.append(lambda bits,idx=idx,val=val:
                                  all(bits[i]==v for i,v in zip(idx,val)))
        self.assertEqual(len(family),20)
        s=[(0,0,0),(1,0,0),(0,1,0),(0,0,1)]
        held=[(1,1,0),(1,0,1),(0,1,1),(1,1,1)]
        for labels in itertools.product((False,True),repeat=3):
            observations=[False,*labels]
            viable=[f for f in family if all(f(x)==y for x,y in zip(s,observations))]
            for cell in held:
                preds={f(cell) for f in viable}
                c2=any(labels[i] for i,on in enumerate(cell) if on)
                if len(preds)==1:
                    self.assertEqual(next(iter(preds)),c2,(labels,cell))

if __name__=='__main__':unittest.main()
