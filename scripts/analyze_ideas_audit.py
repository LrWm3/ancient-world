"""Read matching-grid world archives and evaluator reports for design-outcome audits.

Usage: python3 scripts/analyze_ideas_audit.py REPORT_PREFIX [...]
Simulation and budget validation remain in the GPU evaluator. This is read-only analysis.
"""
import collections
import json
import math
import struct
import sys
from pathlib import Path


def analyze(prefix):
    report = json.loads(Path(prefix + '.json').read_text())
    if not report['complete']:
        raise ValueError('Incomplete evaluation')
    results = []
    for run in report['runs']:
        with open(f"{prefix}.{run['seed']}.world", 'rb') as archive:
            if archive.read(8) != b'ANCIENT2':
                raise ValueError('Unsupported archive')
            size = struct.unpack('<Q', archive.read(8))[0]
            if size > 64 * 1024 * 1024:
                raise ValueError('Oversized header')
            header = json.loads(archive.read(size))
            config = header['config']
            if config['resolution'] != config['ecology_resolution']:
                raise ValueError('This diagnostic requires matching grids')
            n = 6 * config['resolution'] ** 2
            terrain_stride = 176 if header["version"] >= 6 else 160
            terrain = archive.read(n * terrain_stride)
            eco_floats = 152 if header['version'] >= 5 else 128
            env_floats = 88 if header['version'] >= 4 else 32
            eco = list(struct.iter_unpack(f'<{eco_floats}f', archive.read(n * eco_floats * 4)))
            env = list(struct.iter_unpack(f'<{env_floats}f', archive.read(n * env_floats * 4)))
        h = header['civilizations']
        tags = [struct.unpack_from('<I', terrain, i * terrain_stride + 128)[0] for i in range(n)]
        active = [s for s in h['sites'] if not s['abandoned']]
        islands = collections.Counter(s['island'] for s in active)
        polity = collections.Counter(h['politics']['controllers'][s['id']] for s in active)
        voyages = h['expeditions']['voyages']
        phase = collections.Counter(v['phase'] for v in voyages)
        regions = {}
        for region in [2, 3]:
            ids = [i for i in range(n) if tags[i] == region]
            area = sum(env[i][12] for i in ids)
            mass = lambda k: sum(eco[i][k * 4] * env[i][12] for i in ids)
            photo = sum(eco[i][112] * env[i][12] for i in ids)
            chemo = sum(eco[i][113] * env[i][12] for i in ids)
            layers = [mass(k) / area for k in range(5)]
            strong_area = sum(env[i][12] for i in ids if eco[i][113] > 1e-8
                              and eco[i][113] / max(eco[i][112] + eco[i][113], 1e-20) >= .1)
            regions[region] = {
                'area_m2': area, 'layer_C_kg_m2': layers,
                'chemo_share_of_production': chemo / max(photo + chemo, 1e-20),
                'area_fraction_at_least_10pct_chemo': strong_area / area,
                'guild_C_kg_m2': [mass(k) / area for k in range(5, 17)],
            }
        lake = {}
        for name, lo, hi in [('core', 0., .6), ('rim', .75, 1.05), ('all', 0., math.pi)]:
            ids = [i for i in range(n) if tags[i] == 1
                   and lo <= math.acos(max(-1., min(1., env[i][30]))) < hi]
            area = sum(env[i][12] for i in ids)
            stocks = {k: sum(eco[i][k * 4 + 2] * env[i][12] for i in ids)
                      for k in [20, 21, 22, 23, 24]}
            surface_volume = sum(min(100., max(1., env[i][14])) * env[i][12] for i in ids)
            lake[name] = {'area_m2': area, 'P_kg': stocks,
                          'surface_P_kg_m3': stocks[20] / max(1., surface_volume),
                          'exchange_m_month': sum(eco[i][118] * env[i][12] for i in ids) / max(1., area)}
        kin = h['politics']['kin']
        important = {'war_declared', 'food_crisis', 'persistent_inundation', 'expedition_lost', 'secession'}
        result = {
            'seed': run['seed'], 'founders': report['requested_civilizations'],
            'initial_climate_converged': header['progress']['climate_converged'],
            'initial_climate_cycles': header['progress']['climate_cycles'],
            'population': run['samples'][-1]['population'],
            'stored_food_kg': sum(s['stocks']['stock'][1] for s in h['sites']),
            'cumulative_crop_production_kg': sum(s['stocks']['ledger'][0] for s in h['sites']),
            'cumulative_food_consumed_kg': sum(s['stocks']['ledger'][1] for s in h['sites']),
            'cumulative_food_lost_kg': sum(s['stocks']['ledger'][2] for s in h['sites']),
            'food_reserve_months': sum(s['stocks']['stock'][1] for s in h['sites']) / max(1., 18 * sum(s['stocks']['stock'][0] for s in active)),
            'town_population_range': [min((s['stocks']['stock'][0] for s in active), default=0), max((s['stocks']['stock'][0] for s in active), default=0)],
            'granary_bricks_kg': sum(f.get('granary_bricks', 0) for f in h['living']['floods'].values()),
            'managed_soil_P_kg': sum(s['economy']['soil'][2] for s in h['sites']),
            'active_sites': len(active), 'towns_per_occupied_island': sorted(islands.values()),
            'towns_per_polity': sorted(polity.values()),
            'off_island_towns': [s['name'] for s in h['sites'] if tags[s['cell']] != 2],
            'outer_route_destinations': [tags[r['cells'][-1]] for r in h['expeditions']['routes']],
            'events': run['events'], 'shortage_site_years': run['shortage_site_years'],
            'people': len(h['people']), 'with_recorded_parents': sum(any(p is not None for p in k['parents']) for k in kin),
            'marriages': len(h['politics']['marriages']), 'factions': len(h['politics']['factions']),
            'voyage_phases': phase, 'objectives': collections.Counter(v['objective'] for v in voyages),
            'crew_deaths': sum(not c['alive'] for v in voyages for c in v['crew']),
            'discoveries': run['samples'][-1]['discoveries'],
            'regional_budget': run['samples'][-1]['ecology_budget']['regions'],
            'ecology_endpoint': regions, 'lake_endpoint': lake,
            'floods': run['samples'][-1]['floods'],
            'causal_examples': [e for e in h['events'] if e['kind'] in important][:8],
        }
        results.append(result)
        print(run['seed'], 'towns/island', sorted(islands.values()), 'polity towns', sorted(polity.values()),
              'wars', run['events'].get('war_declared', 0), 'voyages', dict(phase))
    Path(prefix + '-ideas.json').write_text(json.dumps(results, indent=2))


if __name__ == '__main__':
    for prefix in sys.argv[1:]:
        analyze(prefix)
