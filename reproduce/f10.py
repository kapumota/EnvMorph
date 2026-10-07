#!/usr/bin/env python3
"""Ejecuta exclusivamente los tres controles preregistrados de F10."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FROZEN = '59db7a371740870eda0b50a08a39826e276409cc91b42bcfd48fc7f798a2dff9'
STAGES = ['known_positive', 'observed', 'normalized', 'intervenable', 'eligible',
          'treatment_executed', 'oracle_decidable', 'behaviorally_relevant']


def digest(data):
    return hashlib.sha256(data).hexdigest()


def save(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + '\n')


def module(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    obj = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(obj)
    return obj


def execute(command, env=None, cwd=ROOT):
    return subprocess.run(list(map(str, command)), env=env, cwd=cwd,
                          capture_output=True, timeout=60)


def checked(command, env=None):
    proc = execute(command, env)
    if proc.returncode:
        raise RuntimeError(proc.stderr.decode(errors='replace') + proc.stdout.decode(errors='replace'))
    return proc


def expected(case, variant):
    data = case['expected_observable_difference']
    value = data[variant]
    return bytes.fromhex(value) if data['encoding'] == 'hex' else value.encode()


def summarize(results):
    n = len(results)
    count = lambda key: sum(r['stages'][key] == 'PASS' for r in results)
    attempted = sum(r.get('treatment_attempted', False) for r in results)
    ratio = lambda a, b: a / b if b else None
    return dict(known_positive_count=count('known_positive'),
                benchmark_control_count=n,
                observation_recall=ratio(count('observed'), n),
                normalization_recall=ratio(count('normalized'), n),
                intervenability_recall=ratio(count('intervenable'), n),
                eligibility_recall=ratio(count('eligible'), n),
                attempted_interventions=attempted,
                successful_interventions=count('treatment_executed'),
                intervention_success_rate=ratio(count('treatment_executed'), attempted),
                oracle_decidability=ratio(count('oracle_decidable'), attempted),
                end_to_end_sensitivity=ratio(count('behaviorally_relevant'), n))


def run_case(case, out, observer, validator, bridge):
    target = out / case['workload_id']
    target.mkdir()
    stages = {s: 'UNRESOLVED' for s in STAGES}
    stages['known_positive'] = 'PASS'
    result = dict(workload_id=case['workload_id'], stages=stages, evidence={}, treatment_attempted=False)
    with tempfile.TemporaryDirectory(prefix='envmorph-f10-') as td:
        workspace = Path(td) / 'workspace'
        workspace.mkdir()
        (workspace / 'a b').touch()
        env = dict(PATH='/usr/bin:/bin', LC_ALL='C', HOME=str(workspace), TZ='UTC0', PYTHONNOUSERSITE='1')
        env[case['environmental_factor']] = case['baseline_value']
        # Comprobación independiente del contrato upstream, sin cambiar selección.
        direct = {}
        for variant in ('baseline', 'treatment'):
            variant_env = dict(env)
            variant_env[case['environmental_factor']] = case[variant + '_value']
            proc = execute(case['command'], variant_env, workspace)
            (target / ('ground_truth_' + variant + '.stdout')).write_bytes(proc.stdout)
            (target / ('ground_truth_' + variant + '.stderr')).write_bytes(proc.stderr)
            direct[variant] = dict(exit_code=proc.returncode, stdout_sha256=digest(proc.stdout),
                                   matches_expected=proc.stdout == expected(case, variant))
        save(target / 'ground_truth.json', direct)
        result['evidence']['known_positive'] = 'ground_truth.json y documentación preregistrada'
        if not all(v['matches_expected'] and v['exit_code'] == 0 for v in direct.values()):
            stages['known_positive'] = 'FAIL'
            result['diagnosis'] = 'invalid_ground_truth, caso conservado sin sustitución'
            return result
        raw = target / 'getenv.jsonl'
        raw.touch()
        observed_env = dict(env, LD_PRELOAD=str(observer), ENVMORPH_F9_INTERPOSE_LOG=str(raw))
        observed = execute(case['command'], observed_env, workspace)
        (target / 'observed.stdout').write_bytes(observed.stdout)
        (target / 'observed.stderr').write_bytes(observed.stderr)
        events_raw = [json.loads(line) for line in raw.read_text().splitlines()]
        found = any(e.get('name') == case['environmental_factor'] for e in events_raw)
        stages['observed'] = 'PASS' if found else 'FAIL'
        result['evidence']['observed'] = 'getenv.jsonl'
        if observed.returncode or observed.stdout != expected(case, 'baseline'):
            stages['observed'] = 'UNRESOLVED'
            result['diagnosis'] = 'La instrumentación altera la salida o falla'
            return result
        (target / 'ptrace.jsonl').touch()
        checked([sys.executable, ROOT / 'tools/f9/event-normalizer/normalize_events.py',
                 '--ptrace', target / 'ptrace.jsonl', '--getenv', raw,
                 '--run-id', 'f10-' + case['workload_id'], '--output', target / 'events.jsonl',
                 '--dependencies', target / 'dependencies.json', '--metrics', target / 'normalization.json'])
        events = [json.loads(line) for line in (target / 'events.jsonl').read_text().splitlines()]
        matches = [e for e in events if e['resource'] == case['environmental_factor']]
        stages['normalized'] = 'PASS' if matches else ('FAIL' if found else 'UNRESOLVED')
        result['evidence']['normalized'] = 'events.jsonl'
        if not matches:
            result['diagnosis'] = 'No se observó la variable objetivo, no se inventa elegibilidad'
            return result
        checked([sys.executable, ROOT / 'tools/f9/dependency-graph/build_graph.py',
                 '--events', target / 'events.jsonl', '--output', target / 'graph.json',
                 '--metrics', target / 'graph_metrics.json'])
        graph = json.loads((target / 'graph.json').read_text())
        node = next(n for n in graph['nodes'] if n.get('node_type') == 'dependency' and n.get('resource') == case['environmental_factor'])
        stages['intervenable'] = 'PASS' if node['kind'] == 'environment' and node['operation'] == 'getenv' else 'FAIL'
        plan = {k: node[k] for k in ('dependency_key', 'kind', 'resource_type', 'resource', 'operation')}
        plan.update(schema_version='f9.intervention_plan.v1', source_run_id=graph['run_id'],
                    mechanism='environment_override', isolation='workspace_copy',
                    baseline_value=case['baseline_value'], treatment_value=case['treatment_value'],
                    oracle_ref='f3:byte:stdout', confirmations=2, result_blind_selection=True)
        plan['plan_id'] = validator.expected_plan_id(plan)
        save(target / 'plan.json', plan)
        problems = validator.validate(plan, graph)
        save(target / 'eligibility.json', {'problems': problems, 'status': 'eligible' if not problems else 'invalid'})
        stages['eligible'] = 'FAIL' if problems else 'PASS'
        result['evidence']['intervenable'] = 'plan.json: environment_override'
        result['evidence']['eligible'] = 'eligibility.json, validación F9.4'
        if problems:
            return result
        runs = {'baseline': [], 'treatment': []}
        applied = True
        for variant in runs:
            for repetition in (1, 2):
                stem = f'{variant}_{repetition}'
                if variant == 'treatment':
                    result['treatment_attempted'] = True
                proc = execute([sys.executable, ROOT / 'tools/f9/intervention-runner/run_intervention.py',
                                '--plan', target / 'plan.json', '--variant', variant,
                                '--workspace-source', workspace, '--output', target / (stem + '.json'),
                                '--', *case['command']], env)
                (target / (stem + '.runner.stderr')).write_bytes(proc.stderr)
                if not (target / (stem + '.json')).exists():
                    applied = False
                    continue
                run = json.loads((target / (stem + '.json')).read_text())
                # Captura aislada para F3, verificando identidad con stdout del runner F9.
                with tempfile.TemporaryDirectory(prefix='envmorph-f10-capture-') as cp:
                    copy = Path(cp) / 'workspace'
                    shutil.copytree(workspace, copy)
                    variant_env = dict(env)
                    variant_env[case['environmental_factor']] = case[variant + '_value']
                    capture = execute(case['command'], variant_env, copy)
                (target / (stem + '.stdout')).write_bytes(capture.stdout)
                (target / (stem + '.stderr')).write_bytes(capture.stderr)
                ok = (proc.returncode == capture.returncode == run['exit_code'] == 0 and
                      run['applied_value_sha256'] == digest(case[variant + '_value'].encode()) and
                      run['stdout_sha256'] == digest(capture.stdout) and
                      run['stderr_sha256'] == digest(capture.stderr))
                applied = applied and ok
                run['f10_matches_expected'] = capture.stdout == expected(case, variant)
                runs[variant].append(run)
        stages['treatment_executed'] = 'PASS' if applied and all(len(v) == 2 for v in runs.values()) else 'FAIL'
        result['evidence']['treatment_executed'] = 'baseline_*.json, treatment_*.json y capturas con hashes coincidentes'
        if stages['treatment_executed'] != 'PASS':
            return result
        evidence = []
        for repetition in (1, 2):
            proc = execute([ROOT / 'target/release/envmorph', 'compare-artifacts', '--oracle', 'byte',
                            '--left', target / f'baseline_{repetition}.stdout',
                            '--right', target / f'treatment_{repetition}.stdout', '--format', 'json'])
            (target / f'f3_{repetition}.json').write_bytes(proc.stdout)
            f3 = json.loads(proc.stdout)
            adapted = bridge.adapt_f3(f3, 2)
            save(target / f'oracle_{repetition}.json', adapted)
            evidence.append(adapted)
        stable = all(len({v['stdout_sha256'] for v in rs}) == 1 for rs in runs.values())
        valid = all(v['f10_matches_expected'] for rs in runs.values() for v in rs)
        decidable = stable and valid and all(e['classification'] != 'unresolved' for e in evidence)
        stages['oracle_decidable'] = 'PASS' if decidable else 'UNRESOLVED'
        stages['behaviorally_relevant'] = ('PASS' if all(e['classification'] == 'behaviorally_relevant' for e in evidence)
                                           else 'FAIL') if decidable else 'UNRESOLVED'
        result['evidence']['oracle_decidable'] = 'f3_*.json, oracle_*.json y expectativas congeladas'
        result['evidence']['behaviorally_relevant'] = 'Dos confirmaciones F3 interpretadas por puente F9.6'
        result['diagnosis'] = 'Control completado' if decidable else 'Oráculo inestable o ground truth no confirmado'
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    benchmark_path = ROOT / 'experiments/f10/benchmark.json'
    if digest(benchmark_path.read_bytes()) != FROZEN:
        raise SystemExit('El benchmark no coincide con el protocolo congelado')
    out = args.output.resolve()
    if out.exists():
        raise SystemExit('El destino ya existe, preservar ejecución previa y usar otro directorio')
    benchmark = json.loads(benchmark_path.read_text())
    selected = [c for c in benchmark['candidates'] if c['selected']]
    if len(selected) != 3:
        raise SystemExit('Se requieren exactamente tres casos congelados')
    versions = {}
    for case in selected:
        path = case['command'][0]
        proc = checked([path, '--version'])
        version = proc.stdout.decode().splitlines()[0]
        if version.split()[-1] != case['upstream_version']:
            raise SystemExit('Versión incompatible: ' + version)
        versions[path] = {'version': version, 'sha256': digest(Path(path).read_bytes())}
    out.mkdir(parents=True)
    save(out / 'environment.json', {'binaries': versions, 'platform': os.uname()._asdict() if hasattr(os.uname(), '_asdict') else list(os.uname()), 'benchmark_sha256': FROZEN})
    observer = out / 'observer.so'
    checked(['cc', '-shared', '-fPIC', '-O2', '-Wall', '-Wextra', '-o', observer,
             ROOT / 'tools/f9/external-observer/getenv_observer_atomic.c'])
    validator = module('f10_validator', 'tools/f9/intervention-plan/validate_plan.py')
    bridge = module('f10_bridge', 'tools/f9/f3-f5-bridge/adapt_oracle_evidence.py')
    results = []
    for case in selected:
        try:
            result = run_case(case, out, observer, validator, bridge)
        except (RuntimeError, ValueError, OSError, subprocess.TimeoutExpired) as exc:
            result = {'workload_id': case['workload_id'], 'stages': {s: 'UNRESOLVED' for s in STAGES}, 'error': str(exc)}
        results.append(result)
        save(out / 'results.partial.json', results)
        print(case['workload_id'], result['stages'], flush=True)
    metrics = summarize(results)
    gate = 'GO' if any(r['stages']['behaviorally_relevant'] == 'PASS' for r in results) else 'STOP'
    summary = {'benchmark_sha256': FROZEN, 'results': results, 'metrics': metrics, 'decision': gate,
               'scope': 'Sensibilidad descriptiva en tres controles documentados, no prevalencia poblacional'}
    save(out / 'results.json', summary)
    manifest = ''.join(f'{digest(p.read_bytes())}  {p.relative_to(out)}\n' for p in sorted(out.rglob('*')) if p.is_file())
    (out / 'SHA256SUMS').write_text(manifest)
    print(json.dumps(metrics, indent=2), gate)


if __name__ == '__main__':
    main()
