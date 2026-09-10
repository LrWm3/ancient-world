#!/usr/bin/env python3
"""Standard-library evidence runner. Never treats missing GPU coverage as a pass."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import platform
import re
import shlex
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    h = hashlib.sha256()
    with path.open('rb') as f:
        for block in iter(lambda: f.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def capture(cmd):
    try:
        r = subprocess.run(cmd, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        return {'command': cmd, 'returncode': r.returncode, 'output': r.stdout.strip()}
    except OSError as e:
        return {'command': cmd, 'returncode': None, 'output': str(e)}


def test_counts(text):
    return [dict(zip(('passed', 'failed', 'ignored', 'measured', 'filtered'), map(int, m)))
            for m in re.findall(r'test result: .*? (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out', text)]


def source_snapshot(exclude):
    files = capture(['git', 'ls-files', '--cached', '--others', '--exclude-standard'])['output'].splitlines()
    return {f: digest(ROOT / f) for f in files if (ROOT / f).is_file() and not (ROOT / f).resolve().is_relative_to(exclude)}


def stage_passed(returncode, summaries, is_test):
    return returncode == 0 and (not is_test or (sum(t["passed"] for t in summaries) > 0 and all(t["failed"] == 0 for t in summaries)))


def summary(data):
    lines = ['# Model evidence run', '', f"Profile: {data['profile']}; status: {data['status']}",
             f"Commit: `{data['commit']['output']}`", '',
             'Completion applies only to the selected profile. This is verification and model intervention evidence, not empirical calibration.', '',
             '| Stage | Status | Seconds | Passed / failed / ignored |', '|---|---|---:|---|']
    for s in data['stages']:
        counts = s.get('test_summaries', [])
        totals = [sum(c[k] for c in counts) for k in ('passed', 'failed', 'ignored')]
        lines.append(f"| {s['name']} | {s['status']} | {s.get('seconds', 0):.1f} | {' / '.join(map(str, totals)) if counts else 'not a test count'} |")
    lines += ['', '## Coverage limits', '', '- CPU profile skips hardware tests; focused profile runs selected GPU boundaries; full profile executes all Rust tests including ignored cases.',
              '- Test counts describe executions, not unique requirements or code coverage.',
              '- Other GPUs/backends, global sensitivity, empirical fitting and resolution-response convergence remain unverified by this run.',
              '- See docs/model-evidence.md for traceability, equations and interpretation limits.', '']
    return '\n'.join(lines)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--profile', choices=['cpu', 'focused', 'full'], default='focused')
    p.add_argument('--cargo', default='cargo', help='Executable prefix, e.g. "mise exec rust@1.89.0 -- cargo"')
    p.add_argument('--output', type=Path)
    p.add_argument('--seeds', default='17,81,256')
    p.add_argument('--tool-fractions', default='1')
    policy = p.add_mutually_exclusive_group()
    policy.add_argument('--food-security', action='store_true')
    policy.add_argument('--policy-suite', action='store_true')
    p.add_argument('--reference', help='Previous tool-reserve results JSON or gzip JSON')
    p.add_argument('--closure-months', type=int, default=60)
    p.add_argument('--recovery-months', type=int, default=60)
    a = p.parse_args()
    if a.policy_suite and a.reference:
        p.error("policy suites use within-run references")
    out = (a.output or ROOT / 'output' / ('evidence-' + datetime.datetime.now(datetime.timezone.utc).strftime('%Y%m%dT%H%M%S%fZ'))).resolve()
    out.mkdir(parents=True, exist_ok=False)  # Never combine stale results with a new run.
    cargo = shlex.split(a.cargo)
    source_hashes = source_snapshot(out)
    data = {'schema': 1, 'profile': a.profile, 'status': 'running', 'complete': False,
            'started_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
            'commit': capture(['git', 'rev-parse', 'HEAD']), 'working_tree': capture(['git', 'status', '--porcelain']),
            'source_sha256': source_hashes, 'platform': platform.platform(),
            'cargo': capture(cargo + ['--version']), 'rustc': capture(cargo[:-1] + ['rustc', '--version']), 'gpu_driver': capture(['nvidia-smi', '--query-gpu=name,driver_version,memory.total', '--format=csv']),
            'parameters': {'seeds': a.seeds, 'tool_fractions': a.tool_fractions, 'food_security': a.food_security, 'policy_suite': a.policy_suite, 'reference': a.reference, 'closure_months': a.closure_months, 'recovery_months': a.recovery_months},
            'stages': []}

    def save():
        tmp = out / 'manifest.tmp'
        tmp.write_text(json.dumps(data, indent=2))
        tmp.replace(out / 'manifest.json')
        (out / 'report.md').write_text(summary(data))

    def run(name, args, is_test=False):
        stage = {'name': name, 'command': args, 'status': 'running'}
        data['stages'].append(stage)
        save()
        print(f'{name}: started; log {out / (name + ".log")}', flush=True)
        started = time.monotonic()
        try:
            with (out / (name + '.log')).open('w') as f:
                r = subprocess.run(args, cwd=ROOT, stdout=f, stderr=subprocess.STDOUT)
            stage['returncode'] = r.returncode
            stage['test_summaries'] = test_counts((out / (name + '.log')).read_text())
            stage['status'] = 'passed' if stage_passed(r.returncode, stage['test_summaries'], is_test) else 'failed'
        except OSError as e:
            stage.update(status='failed', error=str(e))
        stage['seconds'] = time.monotonic() - started
        save()
        print(f'{name}: {stage["status"]} ({stage["seconds"]:.1f}s)', flush=True)
        return stage['status'] == 'passed'

    save()
    try:
        run('runner-tests', [sys.executable, '-m', 'unittest', 'discover', '-s', 'scripts', '-p', 'test_evidence.py'])
        run('format', cargo + ['fmt', '--check'])
        run('clippy', cargo + ['clippy', '--locked', '--all-targets', '--', '-D', 'warnings'])
        run('cpu', cargo + ['test', '--locked', '--lib', '--tests', '--', '--test-threads=1'], True)
        if a.profile == 'full':
            run('gpu-full', cargo + ['test', '--locked', '--lib', '--tests', '--', '--ignored', '--test-threads=1'], True)
        elif a.profile == 'focused':
            run('gpu-coupling', cargo + ['test', '--locked', '--test', 'resources', '--test', 'environmental_returns', '--test', 'economy', '--test', 'alloy_processing', '--', '--ignored', '--test-threads=1'], True)
            run('gpu-muster', cargo + ['test', '--locked', '--lib', 'provisioned_raids_conserve_and_resume_in_transit', '--', '--ignored'], True)
            run('gpu-drainage', cargo + ['test', '--locked', '--test', 'gpu', 'gpu_drainage_matches_priority_flood', '--', '--ignored', '--exact'], True)
            run('gpu-exchange', cargo + ['test', '--locked', '--test', 'ecology', 'isolated_lake_exchange_matches_cpu_two_box_reference', '--', '--ignored', '--exact'], True)
        if a.profile != 'cpu':
            if all(s['status'] == 'passed' for s in data['stages']):
                passed = run('mine-intervention', cargo + ['run', '--locked', '--example', 'coupling_evidence', '--', '--output', str(out / 'mine'), '--seeds', a.seeds, '--tool-fractions', a.tool_fractions, '--closure-months', str(a.closure_months), '--recovery-months', str(a.recovery_months)] + (['--food-security'] if a.food_security else []) + (['--policy-suite'] if a.policy_suite else []))
                result = out / 'mine' / 'results.json'
                if passed and (not result.exists() or not json.loads(result.read_text()).get('complete')):
                    data['stages'][-1]['status'] = 'failed'
                if data['stages'][-1]['status'] == 'passed':
                    run('mine-report', [sys.executable, str(ROOT / 'scripts' / 'summarize_coupling.py'), str(result)] + ([a.reference] if a.reference else []))
            else:
                data['stages'].append({'name': 'mine-intervention', 'status': 'blocked_by_failed_checks'})
        data['complete'] = all(s['status'] == 'passed' for s in data['stages'])
        data['status'] = 'passed' if data['complete'] else 'failed'
    except (KeyboardInterrupt, Exception) as e:
        data.update(status='interrupted' if isinstance(e, KeyboardInterrupt) else 'failed', error=repr(e), complete=False)
    finally:
        end_hashes = source_snapshot(out)
        data['source_changed_during_run'] = sorted(f for f in source_hashes.keys() | end_hashes.keys() if source_hashes.get(f) != end_hashes.get(f))
        if data['source_changed_during_run']:
            data.update(status='source_changed', complete=False)
        data['artifacts_sha256'] = {str(f.relative_to(out)): digest(f) for f in out.rglob('*') if f.is_file() and f.name not in ('manifest.json', 'report.md', 'manifest.tmp')}
        save()
    print(f'Report: {out / "report.md"}', flush=True)
    return 0 if data['complete'] else 1


if __name__ == '__main__':
    sys.exit(main())
