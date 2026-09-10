"""Summarize evaluator reports and matching saved-world history metadata.

Usage: python3 scripts/analyze_flood_audit.py output/flood-audit-normal [...]
The evaluator validates full archives/budgets; this reads only their JSON headers.
"""
import collections
import json
import struct
import sys
from pathlib import Path


def inspect(prefix):
    report = json.loads(Path(prefix + '.json').read_text())
    if not report.get('complete'):
        raise ValueError(f'{prefix}: evaluation is incomplete')
    results = []
    for run in report['runs']:
        seed = run['seed']
        with open(f'{prefix}.{seed}.world', 'rb') as archive:
            if archive.read(8) != b'ANCIENT2':
                raise ValueError('Unsupported archive')
            length = struct.unpack('<Q', archive.read(8))[0]
            if length > 64 * 1024 * 1024:
                raise ValueError('Oversized archive header')
            history = json.loads(archive.read(length))['civilizations']
        ongoing, durations = {}, []
        starts = collections.Counter()
        for event in history['events']:
            site = event['site']
            if event['kind'] == 'flood':
                ongoing[site] = event['month']
                starts[site] += 1
            elif event['kind'] == 'flood_receded' and site in ongoing:
                durations.append(event['month'] - ongoing.pop(site))
        towns = []
        for key, flood in history['living']['floods'].items():
            site_id = int(key)
            site = history['sites'][site_id]
            age = max(1, history['month'] - site['founded'])
            if flood['flooded_months']:
                towns.append({
                    'id': site_id, 'name': site['name'],
                    'abandoned': site['abandoned'],
                    'population': site['stocks']['stock'][0],
                    'age_months': age,
                    'exposure_fraction': flood['flooded_months'] / age,
                    'ongoing_wet_months': history['month'] - ongoing[site_id]
                    if site_id in ongoing else 0,
                    'episodes': starts[site_id], **flood,
                })
        delayed = [c for c in history['cargo'] if c['weather_delay_months']]
        samples = run['samples']
        results.append({
            'seed': seed, 'events': run['events'],
            'final': samples[-1]['floods'],
            'population': samples[-1]['population'],
            'active_sites': samples[-1]['active_sites'],
            'shortage_site_years': run['shortage_site_years'],
            'max_social_residual': run['max_relative_residual'],
            'max_eco_cnp': max(abs(v) for s in samples
                               for v in s['ecology_budget']['relative_error']),
            'max_eco_water': max(abs(s['ecology_budget']['water_relative_error'])
                                 for s in samples),
            'towns': towns,
            'max_completed_wet_months': max(durations, default=0),
            'ongoing_cargo': len(delayed),
            'max_cargo_delay': max((c['weather_delay_months'] for c in delayed),
                                   default=0),
            'held_cargo_kg': sum(c['kg'] for c in delayed),
            'storm_probability': history['economy_catalog']['weather']['storm_probability'],
        })
    Path(prefix + '-analysis.json').write_text(json.dumps(results, indent=2))
    for result in results:
        chronic = [(t['name'], t['ongoing_wet_months']) for t in result['towns']
                   if t['ongoing_wet_months'] >= 12]
        print(result['seed'], 'population', round(result['population']),
              'wet town-months', result['final']['site_months'],
              'pending cargo', result['ongoing_cargo'],
              'max delay', result['max_cargo_delay'], 'chronic floods', chronic)


if __name__ == '__main__':
    if len(sys.argv) < 2:
        raise SystemExit('Usage: analyze_flood_audit.py REPORT_PREFIX [...]')
    for prefix in sys.argv[1:]:
        print(prefix)
        inspect(prefix)
