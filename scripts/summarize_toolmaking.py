#!/usr/bin/env python3
"""Summarize complete toolmaking development comparisons; no parameter fitting."""
import gzip
import json
import sys
from pathlib import Path


def render(data):
    if not data.get('complete'):
        raise ValueError('Incomplete experiment')
    groups = {}
    for r in data['runs']:
        group = groups.setdefault(r['seed'], {})
        if r['policy'] in group:
            raise ValueError('Duplicate policy')
        group[r['policy']] = r
    lines = ['# Toolmaking development results', '', f"GPU: {data['gpu']}", '',
             'Ten-year totals after matched spinup, with starting tools held in experimental custody. Differences are relative to food-only, with no broad maintenance reservation. Development seeds; not empirical calibration.', '',
             '| Seed | Policy | Settled population Δ | Harvest Δ kg | Unmet rations Δ kg | Tools made Δ kg | Mean expertise | Maximum expertise | Priority work, worker-months |',
             '|---|---|---:|---:|---:|---:|---:|---:|---:|']
    for seed, group in groups.items():
        if set(group) != {'food-only', 'replacement-jobs', 'jobs-and-expertise'}:
            raise ValueError('Missing policy')
        base = group['food-only']['months']
        for policy, r in group.items():
            months = r['months']
            if len(months) != 120 or [m['month'] for m in months] != list(range(13,133)):
                raise ValueError('Incomplete or shifted clock')
            total = lambda m, k: sum(s[k] for s in m['sites'])
            pop = total(months[-1], 'population')-total(base[-1], 'population')
            harvest = sum(total(a, 'harvest')-total(b, 'harvest') for a,b in zip(months,base))
            unmet = sum(total(a, 'unmet_rations')-total(b, 'unmet_rations') for a,b in zip(months,base))
            made = total(months[-1], 'tools_made')-total(base[-1], 'tools_made')
            skill = [s['craft'][0] for s in months[-1]['sites']]
            work = sum(s['work'][1] for m in months for s in m['sites'])
            lines.append(f'| {seed} | {policy} | {pop:.3f} | {harvest:.2f} | {unmet:.2f} | {made:.3f} | {sum(skill)/len(skill):.6f} | {max(skill):.6f} | {work:.3f} |')
    residual = max(abs(v) for r in data['runs'] for m in r['months'] for v in m['residual'])
    lines += ['', f'Maximum absolute normalized economy residual: {residual:.6e}.', '',
              'Population includes settled residents; demand changes with population. Ten-year differences combine feedback, migration and other history outcomes. Skill is a fraction, not a percentage. All branch results are retained; no coefficients were fitted against them.', '']
    return '\n'.join(lines)


if __name__ == '__main__':
    path = Path(sys.argv[1])
    with (gzip.open(path, 'rt') if path.suffix == '.gz' else path.open()) as f:
        print(render(json.load(f)), end='')
