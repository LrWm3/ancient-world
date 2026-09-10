#!/usr/bin/env python3
"""Summarize completed paired history_evaluate waterworks coverage sweeps.

Annual snapshot means are population weighted, not full monthly exposure integrals.
Compare only reports sharing the same world regime and initial settings.
"""
import argparse
import json
from pathlib import Path


def summarize(run):
    samples = run['samples']
    last = samples[-1]
    exposure = {'disease': 0., 'waterlogging': 0., 'shortfall': 0., 'coverage': 0.}
    population = 0.
    for sample in samples:
        assets = {a['site']: a for a in sample['production']['waterworks_assets']}
        for site in sample['settlements']:
            p = site['population']
            population += p
            a = assets[site['id']]
            exposure['disease'] += p * site['disease']
            exposure['waterlogging'] += p * site['waterlogging']
            exposure['shortfall'] += p * a['water_shortage']
            exposure['coverage'] += p * a['coverage']
    assets = last['production']['waterworks_assets']
    return {
        'seed': run['seed'], 'population': last['population'],
        'shortage_site_years': run['shortage_site_years'],
        'abandonments': run['abandonments'],
        'annual_population_weighted': {k: v / max(population, 1.) for k, v in exposure.items()},
        'final_served_residents': sum(a['served_residents'] for a in assets),
        'construction_work': sum(a['construction_work'] for a in assets),
        'operating_work': sum(a['operating_work'] for a in assets),
        'installed_kg': sum(sum(a['materials']) for a in assets),
        'worn_kg': sum(a['worn'] for a in assets),
        'domestic_water_m3': sum(a['domestic_water'] for a in assets),
        'max_relative_residual': run['max_relative_residual'],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('reports', type=Path, nargs='+')
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    results = []
    reference = None
    keys = ['seeds', 'years', 'resolution', 'epochs', 'requested_civilizations',
            'crop_yield_scale', 'living_world', 'weather_rules', 'market_rules',
            'island_phosphorus_scale', 'settlement_plot_hectares', 'shipping',
            'expeditions', 'discoveries', 'household_economy', 'social_indicators',
            'protect_vulnerable_rations', 'witnessed_relief', 'expedition_rules']
    for path in args.reports:
        report = json.loads(path.read_text())
        if not report['complete']:
            raise ValueError(f'{path}: incomplete report')
        settings = {k: report[k] for k in keys}
        settings['production'] = {k: v for k, v in report['production_rules'].items()
                                  if k != 'waterworks_target_fraction'}
        if reference is not None and settings != reference:
            raise ValueError(f'{path}: comparison settings differ beyond coverage target')
        reference = settings
        results.append({'source': str(path), 'target': report['production_rules']['waterworks_target_fraction'],
                        'runs': [summarize(r) for r in report['runs']]})
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.with_suffix('.json').write_text(json.dumps(results, indent=2) + '\n')
    lines = ['Annual exposure means are population weighted snapshots; work is cumulative worker-months.', '',
             '| Target | Seed | Final people | Mean coverage | Mean disease | Mean waterlogging | Shortage site-years | Build work | Operate work | Max residual |',
             '|---|---|---|---|---|---|---|---|---|---|']
    for result in results:
        for run in result['runs']:
            e = run['annual_population_weighted']
            lines.append(f"| {result['target']:.0%} | {run['seed']} | {run['population']:.0f} | {e['coverage']:.1%} | {e['disease']:.4f} | {e['waterlogging']:.4f} | {run['shortage_site_years']} | {run['construction_work']:.0f} | {run['operating_work']:.0f} | {run['max_relative_residual']:.2e} |")
    args.output.with_suffix('.md').write_text('\n'.join(lines) + '\n')
    print('\n'.join(lines))


if __name__ == '__main__':
    main()
