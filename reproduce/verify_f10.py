#!/usr/bin/env python3
"""Verifica integridad y evidencia semántica del benchmark F10."""
import argparse
import json
from pathlib import Path
import f10


def require(condition, message):
    if not condition:
        raise ValueError(message)


def load(path):
    return json.loads(path.read_text())


def verify(root):
    root = Path(root)
    for line in (root / 'SHA256SUMS').read_text().splitlines():
        sha, relative = line.split('  ', 1)
        path = (root / relative).resolve()
        require(path.is_relative_to(root.resolve()), 'Ruta fuera de evidencia')
        require(f10.digest(path.read_bytes()) == sha, 'Integridad incorrecta: ' + relative)
    benchmark = load(f10.ROOT / 'experiments/f10/benchmark.json')
    require(f10.digest((f10.ROOT / 'experiments/f10/benchmark.json').read_bytes()) == f10.FROZEN,
            'Benchmark alterado')
    result = load(root / 'results.json')
    require(result['benchmark_sha256'] == f10.FROZEN, 'Congelamiento diferente')
    selected = {c['workload_id']: c for c in benchmark['candidates'] if c['selected']}
    require([r['workload_id'] for r in result['results']] == list(selected), 'Casos sustituidos o incompletos')
    validator = f10.module('verify_validator', 'tools/f9/intervention-plan/validate_plan.py')
    for row in result['results']:
        case = selected[row['workload_id']]
        folder = root / row['workload_id']
        stages = row['stages']
        require(set(stages) == set(f10.STAGES), 'Funnel incompleto')
        require(all(v in ('PASS', 'FAIL', 'UNRESOLVED', 'NOT_APPLICABLE') for v in stages.values()), 'Estado inválido')
        if stages['known_positive'] == 'PASS':
            for variant in ('baseline', 'treatment'):
                require((folder / f'ground_truth_{variant}.stdout').read_bytes() == f10.expected(case, variant), 'Ground truth incorrecto')
                require(load(folder / 'ground_truth.json')[variant]['exit_code'] == 0, 'Ground truth falló')
        if stages['observed'] == 'PASS':
            raw = [json.loads(x) for x in (folder / 'getenv.jsonl').read_text().splitlines()]
            require(any(e.get('name') == case['environmental_factor'] and e.get('kind') == 'getenv' for e in raw), 'Observación inexistente')
        if stages['normalized'] == 'PASS':
            events = [json.loads(x) for x in (folder / 'events.jsonl').read_text().splitlines()]
            require(any(e['resource'] == case['environmental_factor'] and e['operation'] == 'getenv' for e in events), 'Normalización sin variable objetivo')
        if stages['eligible'] == 'PASS':
            plan = load(folder / 'plan.json')
            require(not validator.validate(plan, load(folder / 'graph.json')), 'Plan no elegible')
            require(plan['resource'] == case['environmental_factor'], 'Factor cambiado')
            for variant in ('baseline', 'treatment'):
                require(plan[variant + '_value'] == case[variant + '_value'], 'Intervención cambiada')
            require(plan['oracle_ref'] == 'f3:byte:stdout' and plan['confirmations'] == 2, 'Oráculo cambiado')
        if stages['behaviorally_relevant'] == 'PASS':
            require(all(v == 'PASS' for v in stages.values()), 'Positivo sin completar funnel')
            require(row['treatment_attempted'], 'Tratamiento no intentado')
            for repetition in (1, 2):
                for variant in ('baseline', 'treatment'):
                    stem = f'{variant}_{repetition}'
                    run = load(folder / (stem + '.json'))
                    raw = (folder / (stem + '.stdout')).read_bytes()
                    require(run['exit_code'] == 0, 'Ejecución fallida')
                    require(raw == f10.expected(case, variant), 'Salida semántica inesperada')
                    require(run['stdout_sha256'] == f10.digest(raw), 'Captura distinta del runner')
                    require(run['stderr_sha256'] == f10.digest((folder / (stem + '.stderr')).read_bytes()), 'Stderr distinto del runner')
                    require(run['applied_value_sha256'] == f10.digest(case[variant + '_value'].encode()), 'Valor no aplicado')
                    require(run['plan_id'] == plan['plan_id'] and run['variant'] == variant, 'Evidencia de otro plan o variante')
                left = (folder / f'baseline_{repetition}.stdout').read_bytes()
                right = (folder / f'treatment_{repetition}.stdout').read_bytes()
                require(left != right, 'Falso positivo, salidas iguales')
                require(load(folder / f'f3_{repetition}.json')['result'] == 'different', 'F3 no detectó diferencia')
                require(load(folder / f'oracle_{repetition}.json')['classification'] == 'behaviorally_relevant', 'Clasificación distinta')
    require(result['metrics'] == f10.summarize(result['results']), 'Agregados incorrectos')
    gate = 'GO' if any(r['stages']['behaviorally_relevant'] == 'PASS' for r in result['results']) else 'STOP'
    require(result['decision'] == gate, 'Gate incoherente')
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence', type=Path)
    args = parser.parse_args()
    result = verify(args.evidence)
    print('PASS: integridad, evidencia por etapa y agregados;', result['decision'])


if __name__ == '__main__':
    main()
