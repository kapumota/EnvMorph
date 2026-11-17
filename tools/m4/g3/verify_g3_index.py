#!/usr/bin/env python3
"""Verifica anclas locales G3. No certifica autenticidad ni independencia."""
import argparse
import hashlib
import json
from pathlib import Path


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, default=Path.cwd())
    args = parser.parse_args()
    repo = args.repo.resolve()
    index = json.loads((repo / 'experiments/m4/g3/G3_EVIDENCE_INDEX.json').read_text())
    if (index.get('schema') != 'g3.engineering_index.v1'
        or index.get('status') != 'ENGINEERING_ONLY_NOT_CONFIRMATORY'
        or index.get('scientific_confirmatory') is not False
        or index.get('g3c_protocol_frozen') is not False
        or index.get('f9_observer_integrated') is not False
        or index.get('f6_contract_evaluated') is not False
        or index.get('independent_second_host_reproduction') != 'DEFERRED'):
        raise SystemExit('G3_VERIFY=BLOCKED: estado editorial inconsistente')
    paths = (
        ('pilot_evidence_path', 'pilot_evidence_sha256'),
        ('bridge_evidence_path', 'bridge_evidence_sha256'),
        ('pilot_script_path', 'pilot_script_sha256'),
        ('stop_rules_path', 'stop_rules_sha256'),
    )
    for path_key, hash_key in paths:
        relative = Path(index[path_key])
        if relative.is_absolute() or '..' in relative.parts:
            raise SystemExit('G3_VERIFY=BLOCKED: ruta no relativa')
        path = repo / relative
        if not path.is_file() or path.is_symlink() or not path.resolve().is_relative_to(repo):
            raise SystemExit(f'G3_VERIFY=BLOCKED: archivo no regular {relative}')
        if digest(path) != index[hash_key]:
            raise SystemExit(f'G3_VERIFY=BLOCKED: SHA-256 distinto {relative}')
        print('G3_VERIFIED=' + str(relative))
    print('G3_INDEX=PASS')
    print('G3_SCIENTIFIC_CONFIRMATORY=NO')
    print('G3_RAW_PRIMARY_EVIDENCE=NOT_INCLUDED')
    print('G3_SECOND_HOST_REPRODUCTION=DEFERRED')


if __name__ == '__main__':
    main()
