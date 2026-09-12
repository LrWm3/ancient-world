#!/usr/bin/env python3
"""Compare complete matched balance reports without changing simulation state."""
import argparse
import json
import math
from pathlib import Path


def compare(baseline, treatment, allowed):
    for report in (baseline, treatment):
        if report.get('complete') is not True:
            raise ValueError('both reports must be complete')
        if report.get('observation_interval_months') != 1:
            raise ValueError('monthly observations required')
    # Require explicit acknowledgement of every metadata difference, including
    # enabled mechanisms. Hardware differences are evidence too, not ignored.
    keys = (baseline.keys() | treatment.keys()) - {'runs', 'complete'}
    changed = {key for key in keys if baseline.get(key) != treatment.get(key)}
    if changed - allowed:
        raise ValueError(f'unacknowledged differences: {sorted(changed - allowed)}')
    if allowed - keys:
        raise ValueError(f'unknown metadata keys: {sorted(allowed - keys)}')
    rows = []
    indexed = []
    for report in (baseline, treatment):
        seeds = report['seeds']
        if not seeds or len(set(seeds)) != len(seeds):
            raise ValueError('empty or duplicate declared seeds')
        by_seed = {run['seed']: run for run in report['runs']}
        if len(by_seed) != len(report['runs']) or set(by_seed) != set(report['seeds']):
            raise ValueError('missing or duplicate seed results')
        indexed.append(by_seed)
    if indexed[0].keys() != indexed[1].keys():
        raise ValueError('seed sets differ')
    for seed in sorted(indexed[0]):
        pair = []
        for by_seed, report in zip(indexed, (baseline, treatment)):
            samples = by_seed[seed]['samples']
            if not samples:
                raise ValueError('missing observation samples')
            sample = samples[-1]
            if sample['year'] != report['years']:
                raise ValueError('run does not reach declared horizon')
            food = sample['food_totals']
            values = [sample['population'], *food, sample['max_population_residual'],
                      sample['max_food_residual']]
            if len(food) != 6 or not all(math.isfinite(v) and v >= 0 for v in values) or food[0] <= 0:
                raise ValueError('invalid food/population diagnostics')
            if abs(food[0] - food[3] - food[4] - food[5]) > 1e-4 * (1 + food[0]):
                raise ValueError('physical/access gap decomposition failed')
            pair.append((sample['population'], 100 * food[4] / food[0],
                         100 * food[5] / food[0], sample['active_sites'],
                         sample['max_population_residual'], sample['max_food_residual']))
        rows.append((seed, *pair))
    return changed, rows


def recent_interval(report, run):
    """Last observed interval, never a subtraction from an invented zero baseline."""
    samples = run['samples']
    if len(samples) < 2:
        return None
    before, after = samples[-2:]
    start, end = before['year'], after['year']
    if not (math.isfinite(start) and math.isfinite(end) and 0 <= start < end == report['years']):
        raise ValueError('invalid recent interval dates')
    stocks = [before['population'], after['population']]
    if not all(math.isfinite(v) and v >= 0 for v in stocks):
        raise ValueError('invalid recent population')
    totals = (before['food_totals'], after['food_totals'])
    if any(len(t) != 6 or not all(math.isfinite(v) and v >= 0 for v in t) for t in totals):
        raise ValueError('invalid recent food totals')
    food = [b - a for a, b in zip(*totals)]
    if any(v < 0 for v in food):
        raise ValueError('cumulative food totals decreased')
    if abs(food[0] - food[3] - food[4] - food[5]) > 1e-4 * (1 + food[0]):
        raise ValueError('recent food gap decomposition failed')
    return {
        'start': start, 'end': end,
        'population_change': stocks[1] - stocks[0],
        'population_change_pct': 100 * (stocks[1] / stocks[0] - 1) if stocks[0] else None,
        'physical_gap_pct': 100 * food[4] / food[0] if food[0] else None,
        'access_gap_pct': 100 * food[5] / food[0] if food[0] else None,
        'food_need': food[0],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('treatment', type=Path)
    parser.add_argument('--allow-difference', action='append', default=[])
    args = parser.parse_args()
    try:
        baseline = json.loads(args.baseline.read_text())
        treatment = json.loads(args.treatment.read_text())
        changed, rows = compare(baseline, treatment, set(args.allow_difference))
    except (ValueError, KeyError, IndexError, TypeError) as error:
        parser.error(str(error))
    try:
        recent = []
        for a, b in zip(sorted(baseline['runs'], key=lambda r: r['seed']),
                        sorted(treatment['runs'], key=lambda r: r['seed'])):
            first, second = recent_interval(baseline, a), recent_interval(treatment, b)
            if first is None or second is None:
                continue
            if (first['start'], first['end']) != (second['start'], second['end']):
                raise ValueError('recent comparison intervals differ')
            recent.append((a['seed'], first, second))
    except (ValueError, KeyError, IndexError, TypeError) as error:
        parser.error(str(error))
    print('Acknowledged differences: ' + ', '.join(sorted(changed)))
    print('\n| Seed | Population baseline → treatment | Physical gap % | Access gap % | Active sites | Max population residual | Max food residual |')
    print('|---|---|---|---|---|---|---|')
    for seed, base, trial in rows:
        print(f'| {seed} | {base[0]:.2f} → {trial[0]:.2f} | '
              f'{base[1]:.4f} → {trial[1]:.4f} | {base[2]:.4f} → {trial[2]:.4f} | '
              f'{base[3]} → {trial[3]} | {max(base[4], trial[4]):.3g} | '
              f'{max(base[5], trial[5]):.3g} |')

    if recent:
        print('\nLatest observed interval (not cumulative):')
        print('\n| Seed | Years | Population change baseline → treatment | Physical gap % | Access gap % |')
        print('|---|---|---|---|---|')
        def fmt(value):
            return 'n/a' if value is None else f'{value:.4f}'
        for seed, base, trial in recent:
            print(f"| {seed} | {base['start']}–{base['end']} | "
                  f"{base['population_change']:+.0f} → {trial['population_change']:+.0f} | "
                  f"{fmt(base['physical_gap_pct'])} → {fmt(trial['physical_gap_pct'])} | "
                  f"{fmt(base['access_gap_pct'])} → {fmt(trial['access_gap_pct'])} |")


if __name__ == '__main__':
    main()
