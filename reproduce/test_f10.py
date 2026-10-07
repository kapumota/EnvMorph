#!/usr/bin/env python3
"""Pruebas del verificador ante evidencia alterada, sin modificar F8/F9."""
import json
from pathlib import Path
import shutil
import sys
import tempfile
import unittest
import f10
import verify_f10

EVIDENCE = Path(sys.argv.pop(1)).resolve()


class EvidenceTests(unittest.TestCase):
    def test_original_evidence(self):
        self.assertEqual(verify_f10.verify(EVIDENCE)['decision'], 'GO')

    def test_empty_denominator(self):
        result = f10.summarize([{'stages': {s: 'UNRESOLVED' for s in f10.STAGES}}])
        self.assertIsNone(result['intervention_success_rate'])
        self.assertIsNone(result['oracle_decidability'])

    def altered_copy(self, mutate, renew_manifest):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td) / 'evidence'
            shutil.copytree(EVIDENCE, root)
            mutate(root)
            if renew_manifest:
                (root / 'SHA256SUMS').write_text(''.join(
                    f'{f10.digest(p.read_bytes())}  {p.relative_to(root)}\n'
                    for p in sorted(root.rglob('*')) if p.is_file() and p.name != 'SHA256SUMS'))
            with self.assertRaises(ValueError):
                verify_f10.verify(root)

    def test_raw_tampering(self):
        self.altered_copy(lambda p: (p / 'date-tz/treatment_1.stdout').write_bytes(b'alterado'), False)

    def test_semantic_tampering_with_updated_hashes(self):
        self.altered_copy(lambda p: (p / 'date-tz/treatment_1.stdout').write_bytes(b'alterado'), True)

    def test_aggregate_tampering_with_updated_hashes(self):
        def mutate(root):
            path = root / 'results.json'
            result = json.loads(path.read_text())
            result['metrics']['eligibility_recall'] = 0
            f10.save(path, result)
        self.altered_copy(mutate, True)


if __name__ == '__main__':
    unittest.main()
