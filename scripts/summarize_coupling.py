#!/usr/bin/env python3
"""Render observed paired effects, without interpreting them as empirical coefficients."""
import gzip
import json
from pathlib import Path
import sys


def sum_field(sample, field, index=None):
    return sum(s[field] if index is None else s[field][index] for s in sample['sites'])


def tool_multiplier(sample):
    if not sample['sites'] or any('production_probe' not in s for s in sample['sites']):
        return None
    weight = sum(s['production_probe'][3] for s in sample['sites'])
    return sum(s['production_probe'][0]*s['production_probe'][3] for s in sample['sites'])/weight if weight > 0 else None


def compare_policy(data, reference):
    key = lambda r: (r['seed'], r.get('tool_fraction', 1), r['fixed_labor'])
    refs = {key(r): r for r in reference['runs']}
    if not reference.get('complete') or len(refs) != len(reference['runs']):
        raise ValueError('Incomplete or ambiguous reference')
    lines = ['', '## Food-security policy versus previous adaptive policy', '', 'Matched reserve and mine policy. Differences are new minus previous; fixed staffing must reproduce every previously measured monthly value.', '', '| Seed | Initial access | Mines closed | Closure population Δ | Closure harvest Δ kg | Mean food-shortfall Δ percentage points | Closure tool multiplier Δ |', '|---|---|---|---:|---:|---:|---:|']
    checked = 0
    for run in data['runs']:
        old = refs[key(run)]
        if run['config'] != old['config'] or data['closure_months'] != reference['closure_months'] or data['recovery_months'] != reference['recovery_months']:
            raise ValueError('Unmatched experiment configurations')
        for branch, control in zip(run['branches'], old['branches']):
            a, b = branch['months'], control['months']
            if branch['closed'] != control['closed'] or len(a) != len(b):
                raise ValueError('Unmatched branches')
            if run['fixed_labor']:
                for m, n in zip(a, b):
                    clean = dict(m, sites=[{k:v for k,v in s.items() if k != 'food_labor'} for s in m['sites']])
                    if clean != n:
                        raise ValueError('Fixed control changed')
                    checked += 1
                continue
            n = data['closure_months']
            shortfall = lambda seq: sum(s['population']*s['shortage_fraction'] for m in seq[:n] for s in m['sites'])/max(1, sum(sum_field(m, 'population') for m in seq[:n]))
            population = sum_field(a[n-1], 'population')-sum_field(b[n-1], 'population')
            harvest = sum(sum_field(m, 'production_kg')-sum_field(k, 'production_kg') for m,k in zip(a[:n], b[:n]))
            tool = tool_multiplier(a[n-1])-tool_multiplier(b[n-1])
            lines.append(f"| {run['seed']} | {run.get('tool_fraction', 1):g} | {branch['closed']} | {population:.2f} | {harvest:.1f} | {100*(shortfall(a)-shortfall(b)):.3f} | {tool:.5f} |")
    lines += ['', f'Unchanged fixed-control monthly observations: {checked}. Food shortfall is population-weighted over the closure interval; population weights can themselves change.']
    return lines


def main(path=None, reference_path=None):
    path = Path(path or sys.argv[1])
    data = json.loads(path.read_text())
    if not data.get('complete'):
        raise SystemExit('Refusing to summarize an incomplete experiment')
    if any(r.get('policy') == 'food-only' for r in data['runs']):
        from policy_suite import render
        if reference_path:
            raise ValueError('Policy suites use within-run references')
        path.with_suffix('.md').write_text(render(data))
        return
    lines = ['# Mine-access intervention evidence', '', f"GPU: {data['gpu']}; backend: {data.get('backend', 'not recorded')}; driver: {data.get('driver', 'not recorded')}", '',
             'Treatment minus its matched baseline. Fixed staffing is a diagnostic 62/8/10/20 percent allocation; it does not freeze worker headcount or provide free inputs.', '',
             '| Seed | Accessible initial tools | Fixed shares | Month | Settled population Δ | Farm workers Δ | Food produced Δ kg/month | Food eaten Δ kg/month | Tool stock Δ kg |',
             '|---|---|---|---|---:|---:|---:|---:|---:|']
    if any(r.get('food_security', False) for r in data['runs']):
        lines.insert(4, 'In this experiment, adaptive rows use the experimental food-security policy. Fixed rows retain the original fixed-share control.')
    errors = []
    source_errors = []
    ecology_errors = []
    effects = {}
    mediators = []
    population_errors = []
    for run in data['runs']:
        base, treatment = [b['months'] for b in run['branches']]
        for samples in (base, treatment):
            errors.extend(abs(x) for m in samples for x in m['economy_residuals'])
            population_errors.extend(abs(m['population_residual']) for m in samples if 'population_residual' in m)
            source_errors.extend(abs(m['source_residual']) for m in samples)
            ecology_errors.extend(abs(x) for m in samples for x in m['ecology_relative_error'])
        for idx in sorted({0, min(11, len(base)-1), data['closure_months']-1, len(base)-1}):
            a, b = base[idx], treatment[idx]
            delta = lambda f, i=None: sum_field(b, f, i)-sum_field(a, f, i)
            eaten = sum(delta('ration_eaten_kg', k) for k in range(3))
            lines.append(f"| {run['seed']} | {run.get('tool_fraction', 1):g} | {run['fixed_labor']} | {idx+1} | {delta('population'):.2f} | {delta('labor_workers',0):.2f} | {delta('production_kg'):.2f} | {eaten:.2f} | {delta('tool_stock_kg'):.2f} |")
        n = data['closure_months']
        def integrated(seq, key, index=None):
            return sum(sum_field(m, key, index) for m in seq[:n])
        def mean_share(seq):
            values = [sum_field(m, 'labor_workers', 0)/max(1e-12, sum(sum_field(m, 'labor_workers', k) for k in range(4))) for m in seq[:min(12, n)]]
            return sum(values)/len(values)
        mediators.append((run['seed'], run.get('tool_fraction', 1), run['fixed_labor'], 100*(mean_share(treatment)-mean_share(base)), integrated(treatment, 'production_kg')-integrated(base, 'production_kg'), integrated(treatment, 'ration_eaten_kg', 3)-integrated(base, 'ration_eaten_kg', 3)))
        idx = data['closure_months']-1
        effects.setdefault((run['seed'], run.get('tool_fraction', 1)), {})[run['fixed_labor']] = sum_field(treatment[idx], 'population')-sum_field(base[idx], 'population')
    lines += ['', '| Seed | Accessible initial tools | Closure settled population effect, adaptive | Closure settled population effect, fixed | Fixed minus adaptive effect |', '|---|---|---:|---:|---:|']
    for (seed, fraction), values in effects.items():
        lines.append(f"| {seed} | {fraction:g} | {values[False]:.2f} | {values[True]:.2f} | {values[True]-values[False]:.2f} |")
    lines += ['', '| Seed | Accessible initial tools | Fixed shares | First-year farm share Δ percentage points | Closure harvest Δ kg | Closure rations eaten Δ kg |', '|---|---|---|---:|---:|---:|']
    for seed, fraction, fixed, share, harvest, rations in mediators:
        lines.append(f"| {seed} | {fraction:g} | {fixed} | {share:.3f} | {harvest:.1f} | {rations:.1f} |")
    lines += ['', '| Seed | Accessible initial tools | Fixed shares | Closure total living population Δ including travelers |', '|---|---|---|---:|']
    for run in data['runs']:
        a, b = [branch['months'][data['closure_months']-1] for branch in run['branches']]
        if 'population_in_transit' in a:
            delta = sum_field(b, 'population')+b['population_in_transit']-sum_field(a, 'population')-a['population_in_transit']
            lines.append(f"| {run['seed']} | {run.get('tool_fraction', 1):g} | {run['fixed_labor']} | {delta:.2f} |")
    lines += ['', '| Seed | Initial access | Fixed shares | Mines closed | First-month tool multiplier | First-year mean multiplier | First-month cultivated ha | Restricted tools kg |', '|---|---|---|---|---:|---:|---:|---:|']
    for run in data['runs']:
        for closed, branch in zip((False, True), run['branches']):
            months = branch['months']
            probes = [tool_multiplier(m) for m in months[:12]]
            mean = sum(probes)/len(probes) if all(v is not None for v in probes) else None
            fmt = lambda v: f'{v:.6f}' if v is not None else 'not recorded'
            area = sum_field(months[0], 'production_probe', 1) if probes[0] is not None else None
            restricted = sum(s.get('restricted_tools_kg', 0) for s in months[0]['sites'])
            lines.append(f"| {run['seed']} | {run.get('tool_fraction', 1):g} | {run['fixed_labor']} | {closed} | {fmt(probes[0])} | {fmt(mean)} | {fmt(area)} | {restricted:.3f} |")
    lines += ['', 'Tool multipliers are population-weighted actual GPU production inputs. Restricted starting tools stay in conserved experimental custody; new output/imports remain usable.', '', '| Seed | Initial access | Fixed shares | Mines closed | Stock intervention: closure population Δ | Stock intervention: closure harvest Δ kg |', '|---|---|---|---|---:|---:|']
    references = {(r['seed'], r['fixed_labor']): r for r in data['runs'] if r.get('tool_fraction', 1) == 1}
    for run in data['runs']:
        ref = references.get((run['seed'], run['fixed_labor']))
        if ref is None or run.get('tool_fraction', 1) == 1:
            continue
        for closed, branch, control in zip((False, True), run['branches'], ref['branches']):
            n = data['closure_months']
            delta_pop = sum_field(branch['months'][n-1], 'population')-sum_field(control['months'][n-1], 'population')
            delta_food = sum(sum_field(a, 'production_kg')-sum_field(b, 'production_kg') for a,b in zip(branch['months'][:n], control['months'][:n]))
            lines.append(f"| {run['seed']} | {run['tool_fraction']:g} | {run['fixed_labor']} | {closed} | {delta_pop:.2f} | {delta_food:.2f} |")
    population_error = f'{max(population_errors):.3e}' if population_errors else 'not recorded'
    lines += ['', f'Maximum monthly normalized economy residual: {max(errors):.3e}; normalized source residual: {max(source_errors):.3e}; ecology relative C/N/P error: {max(ecology_errors):.3e}; population relative error: {population_error}.', '',
              'No-op checks compare complete serialized social history and measured environmental budgets after one month. They do not prove all GPU buffers identical.',
              'Monthly records include site identity, source inventories, births/deaths/migration, rations and all labor pools. Positive long-run population differences alone do not identify a mechanism.',
              'These runs test legacy shared ore/clay extraction, matching the original experiment; mineral-specific/alloy processing is not enabled.',
              'No fitted parameters, confidence intervals, held-out validation or empirical calibration are claimed.', '']
    if reference_path:
        reference_path = Path(reference_path)
        opener = gzip.open if reference_path.suffix == '.gz' else open
        with opener(reference_path, 'rt') as f:
            lines += compare_policy(data, json.load(f))
    path.with_suffix('.md').write_text('\n'.join(lines))

if __name__ == '__main__':
    main(reference_path=sys.argv[2] if len(sys.argv) > 2 else None)
