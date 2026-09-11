#!/usr/bin/env python3
"""Summarize the complete paired suite emitted by nutrition_evaluate."""
import argparse
import json
from collections import defaultdict
from statistics import mean

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
        key = row['seed'], round(row['yield'], 4), row['enabled']
        if row['type'] == 'month':
            months[key].append(row)
        elif row['type'] == 'result':
            if key in results:
                raise ValueError(f'duplicate result {key}')
            results[key] = row
assert settings is not None, 'missing settings'
expected = {(s, round(y, 4), e) for s in settings['seeds'] for y in settings['yields'] for e in [False, True]}
assert results.keys() == expected, f'incomplete suite: missing {expected - results.keys()}'
for key in expected:
    assert [r['month'] for r in months[key]] == list(range(1, settings['years'] * 12 + 1)), f'incomplete months {key}'
print('| Seed | Yield | Nutrition | Final population | Active sites | Mean hunger | Mean accounts >50% hunger | Mean personal capacity | Mean committed work |')
print('|---|---:|---|---:|---:|---:|---:|---:|---:|')
for key in sorted(expected):
    rows = months[key]
    print(f'| {key[0]} | {key[1]} | {"on" if key[2] else "off"} | {results[key]["population"]:.0f} | {rows[-1]["active_sites"]} | {mean(r["need_weighted_hunger"] for r in rows):.3f} | {mean(r["hungry_accounts"] / max(r["accounts"], 1) for r in rows):.3f} | {mean(r["personal_capacity"] for r in rows):.2f} | {mean(r["committed"] for r in rows):.2f} |')
print('\nMaximum absolute population residual:', max(r['max_population_residual'] for r in results.values()))
print('Maximum relative food residual:', max(r['max_food_residual'] for r in results.values()))
print('\nFirst-month immediate household-exposure changes in expected deaths (on):')
for key in sorted(expected):
    if not key[2]:
        continue
    row = months[key][0]
    effects = [value for receipt in row['resolution'] for metric in receipt['metrics'] for name, value in metric['explained'] if name == 'household food exposure (expected deaths)']
    assert effects, f'comparison metrics absent {key}'
    print(key[:2], sum(effects))
print('\n| Seed | Yield | First population difference (month) | First personal capacity difference (month) |')
print('|---|---:|---:|---:|')
for seed in settings['seeds']:
    for value in settings['yields']:
        value = round(value, 4)
        pairs = list(zip(months[seed, value, False], months[seed, value, True]))
        def first_difference(field, tolerance):
            return next((a['month'] for a, b in pairs if abs(a[field] - b[field]) > tolerance), 'none')
        print(f'| {seed} | {value} | {first_difference("population", 1e-6)} | {first_difference("personal_capacity", 1e-5)} |')
