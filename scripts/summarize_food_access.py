#!/usr/bin/env python3
"""Summarize complete yield x common-entitlement suites; no pooling across towns."""
import argparse
import json
from collections import defaultdict

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('input')
args = parser.parse_args()
months = defaultdict(list)
results = {}
settings = None
with open(args.input) as source:
    for line in source:
        row = json.loads(line)
        if row['type'] == 'settings':
            settings = row
            continue
        key = row['seed'], round(row['yield'], 4), round(row['common_share'], 4)
        if row['type'] == 'month':
            months[key].append(row)
        elif row['type'] == 'result':
            assert key not in results, f'duplicate result {key}'
            results[key] = row
assert settings and settings['affordability'], 'not an affordability suite'
expected = {(s, round(y, 4), round(c, 4)) for s in settings['seeds'] for y in settings['yields'] for c in settings['common_shares']}
assert results.keys() == expected, f'incomplete suite: missing {expected - results.keys()}'
print('| Seed | Yield | Common share | Final population | Physical gap / need | Access gap / need | Produced food kg-eq |')
print('|---|---:|---:|---:|---:|---:|---:|')
for key in sorted(expected):
    rows = months[key]
    assert [r['month'] for r in rows] == list(range(1, settings['years'] * 12 + 1)), f'incomplete months {key}'
    for r in rows:
        n, a, funded, eaten, physical, access = r['food']
        assert abs(n - eaten - physical - access) <= 1e-4 * (1 + n), f'gap mismatch {key}'
        if key[2] == 1.0:
            assert access <= 1e-4 * (1 + n), f'full entitlement still blocked {key}'
    need = sum(r['food'][0] for r in rows)
    physical = sum(r['food'][4] for r in rows)
    access = sum(r['food'][5] for r in rows)
    print(f'| {key[0]} | {key[1]} | {key[2]} | {results[key]["population"]:.0f} | {physical / max(need, 1e-12):.3%} | {access / max(need, 1e-12):.3%} | {sum(r["produced"] for r in rows):.0f} |')
print('\nMaximum population residual:', max(r['max_population_residual'] for r in results.values()))
print('Maximum relative food residual:', max(r['max_food_residual'] for r in results.values()))
print('\nOpening-month controls (need, available, funded, eaten, physical gap, access gap):')
for key in sorted(expected):
    print(key, [round(x, 3) for x in months[key][0]['food']])

# Before historical divergence, the access intervention must not change the food source.
for seed in settings['seeds']:
    for y in settings['yields']:
        opening = [months[seed, round(y, 4), round(c, 4)][0]['food'] for c in settings['common_shares']]
        assert all(abs(f[0] - opening[0][0]) < 1e-6 and abs(f[1] - opening[0][1]) < 1e-6 for f in opening), 'opening supply changed across access controls'
