#!/usr/bin/env python3
"""Reproducible pressure experiments; no simulation defaults are modified."""
import argparse
from concurrent.futures import ThreadPoolExecutor, as_completed
import json
from pathlib import Path
import subprocess

CASES = {
    'baseline': [],
    'yield_040': ['--crop-yield-scale', '0.4'],
    'yield_035': ['--crop-yield-scale', '0.35'],
    'yield_033': ['--crop-yield-scale', '0.33'],
    'yield_030': ['--crop-yield-scale', '0.3'],
    'yield_010': ['--crop-yield-scale', '0.1'],
    'land_080': ['--settlement-plot-hectares', '80'],
    'land_020': ['--settlement-plot-hectares', '20'],
    'phosphorus_005': ['--island-phosphorus-scale', '0.05'],
    'drought_090': ['--drought-severity', '0.9', '--drought-probability', '0.6', '--drought-regime-months', '96'],
    'storms_025': ['--storm-probability', '0.25'],
    'compound_hard': ['--crop-yield-scale', '0.3', '--settlement-plot-hectares', '80', '--island-phosphorus-scale', '0.1', '--drought-severity', '0.75', '--drought-probability', '0.4'],
    'compound_survival': ['--crop-yield-scale', '0.1', '--settlement-plot-hectares', '20', '--drought-severity', '0.95', '--drought-probability', '1', '--drought-regime-months', '120'],
    'compound_extreme': ['--crop-yield-scale', '0.1', '--settlement-plot-hectares', '20', '--island-phosphorus-scale', '0.01', '--drought-severity', '0.95', '--drought-probability', '1', '--drought-regime-months', '120'],
}

def summarize(report):
    rows = []
    for run in report['runs']:
        initial = {s['id']: s['population'] for s in run['initial_sites']}
        low = dict(initial)
        for sample in run['samples']:
            for s in sample['settlements']:
                if s['id'] in low:
                    low[s['id']] = min(low[s['id']], s['population'])
        final = run['samples'][-1]
        empty = next((s for s in run['samples'] if s['active_sites'] == 0), None)
        post_collapse = {k: v - empty['cumulative_events'].get(k, 0) for k, v in final['cumulative_events'].items() if empty and v > empty['cumulative_events'].get(k, 0)}
        rows.append({
            'seed': run['seed'], 'population': final['population'],
            'active_population': sum(s['population'] for s in final['settlements'] if not s['abandoned']),
            'initial_population': sum(initial.values()),
            'active_sites': final['active_sites'],
            'abandonments': len(run['abandonments']),
            'initial_abandonments': sum(e['site'] in initial for e in run['abandonments']),
            'first_abandonment_year': min((e['month']/12 for e in run['abandonments']), default=None),
            'initial_towns_below_half': sum(low[k] < v*.5 for k,v in initial.items()),
            'initial_towns_below_quarter': sum(low[k] < v*.25 for k,v in initial.items()),
            'minimum_initial_town_population': min(low.values()),
            # Annual snapshots miss hunger that resolves before the sample month.
            'shortage_site_years': run['shortage_site_years'],
            'food_crises': run['events'].get('food_crisis', 0),
            'recoveries': run['recoveries'],
            'post_total_abandonment_events': post_collapse,
            'max_relative_residual': run['max_relative_residual'],
        })
    return rows

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--cases', default=','.join(CASES))
    p.add_argument('--seeds', default='17,81,256')
    p.add_argument('--years', type=int, default=100)
    p.add_argument('--jobs', type=int, default=2)
    p.add_argument('--output', type=Path, default=Path('output/pressure-sweep'))
    p.add_argument('--save-worlds', action='store_true')
    p.add_argument('--summarize-only', action='store_true', help='Rebuild summaries from completed reports without running simulations')
    a = p.parse_args()
    names = a.cases.split(',')
    if any(n not in CASES for n in names) or a.jobs < 1:
        p.error('unknown case or invalid job count')
    a.output.mkdir(parents=True, exist_ok=True)
    def run_case(name):
        prefix = a.output / name
        cmd = ['target/release/examples/history_evaluate', '--seeds', a.seeds, '--years', str(a.years), '--discoveries', '--living-world', '--label', name, '--output', str(prefix), *CASES[name]]
        if a.save_worlds:
            cmd.append('--save-worlds')
        if a.summarize_only:
            report_path = prefix.with_suffix('.json')
            code = 0 if report_path.exists() and json.loads(report_path.read_text()).get('complete') else 1
        else:
            # A failed rerun must not inherit completed seeds from an older run.
            prefix.with_suffix('.json').unlink(missing_ok=True)
            with prefix.with_suffix('.log').open('w') as log:
                code = subprocess.run(cmd, stdout=log, stderr=subprocess.STDOUT).returncode
        row = {'case': name, 'command': cmd, 'exit_code': code, 'runs': []}
        report_path = prefix.with_suffix('.json')
        if report_path.exists():
            report = json.loads(report_path.read_text())
            if (code == 0 and not report['complete']) or report['years'] != a.years or report['seeds'] != [int(v) for v in a.seeds.split(',')]:
                raise ValueError(f'{name}: incomplete or mismatched experiment')
            # Only fully completed seeds are written by the evaluator; retain
            # them even if a later seed failed. The case remains marked failed.
            row['runs'] = summarize(report)
        print(name, 'exit', code, 'abandonments', [r['abandonments'] for r in row['runs']], flush=True)
        return row
    results = {}
    with ThreadPoolExecutor(max_workers=a.jobs) as pool:
        futures = {pool.submit(run_case, n): n for n in names}
        for f in as_completed(futures):
            results[futures[f]] = f.result()
            (a.output / 'summary.json').write_text(json.dumps({'years': a.years, 'seeds': a.seeds, 'complete': len(results)==len(names), 'cases': [results[n] for n in names if n in results]}, indent=2))
    lines = ['# Pressure sweep', '', f'{a.years} years; seeds {a.seeds}; current settlement lifecycle (abandonment after twelve consecutive months below one person-equivalent). Declines are annual samples relative to each initial town at landfall; daughter towns are included only in total abandonments. Shortage observations are year-end samples, not a count of all hungry months; food crises count episodes reaching three consecutive hungry months. Failed cases are not classified as survival.', '', '| Case | Seed | Active people | Active settlements | Abandoned (initial) | Initial towns below half | First abandonment year | Year-end shortage observations | Food crises | Residual |', '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for name in names:
        case = results[name]
        if case['exit_code']:
            lines.append(f'| {name} FAILED ({case["exit_code"]}) | | | | | | | | | |')
        for r in case['runs']:
            year = r['first_abandonment_year']
            lines.append(f'| {name} | {r["seed"]} | {r["active_population"]:.0f} | {r["active_sites"]} | {r["abandonments"]} ({r["initial_abandonments"]}) | {r["initial_towns_below_half"]} | {"—" if year is None else f"{year:.1f}"} | {r["shortage_site_years"]} | {r["food_crises"]} | {r["max_relative_residual"]:.2e} |')
    (a.output/'summary.md').write_text('\n'.join(lines)+'\n')
    if any(r['exit_code'] for r in results.values()):
        raise SystemExit('Some cases failed; see retained logs.')

if __name__ == '__main__':
    main()
