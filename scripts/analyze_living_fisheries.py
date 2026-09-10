#!/usr/bin/env python3
"""Summarize natural-stock fishery branches; does not fit simulation parameters."""
import argparse
import gzip
import json
from pathlib import Path


def read_rows(path):
    opener = gzip.open if path.suffix == '.gz' else open
    with opener(path, 'rt') as stream:
        return [json.loads(line) for line in stream]


def metadata_for(directory, seed):
    path = directory/f'metadata-{seed}.json'
    if path.exists():
        return json.loads(path.read_text())
    with gzip.open(str(path)+'.gz', 'rt') as stream:
        return json.load(stream)


def summarize(rows, fish_food_energy=0.5):
    measured = rows[1:]
    sites = [s for row in measured for s in row['sites']]
    need = sum(s['ration_need'][3] for s in sites)
    eaten = sum(s['ration_eaten'][3] for s in sites)
    observed = sum((s['household_stress'] or [0]*4)[3] for s in sites)
    deprived = sum((s['food_security_households'] or [0]*4)[3] for s in sites)
    final = rows[-1]
    monthly = [sum(s['catch_kg'] for s in r['sites']) for r in measured]
    return {
        'seed': final['seed'], 'branch': final['branch'],
        'months': final['elapsed_months'], 'catch_kg': sum(monthly),
        'fishing_worker_months': sum(s.get('fishery_plan', [0]*4)[1] for s in sites),
        'equipment_worker_months': sum(s.get('fishery_plan', [0]*4)[2] for s in sites),
        'site_months_seeking_fishing': sum(s.get('fishery_plan', [0]*4)[0] > 0 for s in sites),
        'site_months_fishing': sum(s.get('fishery_plan', [0]*4)[1] > 0 for s in sites),
        'final_installed_equipment_kg': sum(sum(s.get('fishery_equipment', [0]*4)[:3]) for s in final['sites']),
        'final_trap_timber_kg': sum(s.get('fishery_traps', [0]*4)[0] for s in final['sites']),
        'trap_timber_installed_kg': sum(s.get('fishery_traps', [0]*4)[1] for s in final['sites']),
        'trap_timber_worn_kg': sum(s.get('fishery_traps', [0]*4)[2] for s in final['sites']),
        'peak_site_month_catch_kg': max(s['catch_kg'] for s in sites),
        'catch_calorie_equivalent_fraction_of_need': sum(monthly)*fish_food_energy/need if need else None,
        'catch_first_half_kg': sum(monthly[:len(monthly)//2]),
        'catch_second_half_kg': sum(monthly[len(monthly)//2:]),
        'last_year_catch_kg': sum(monthly[-12:]),
        'food_reserve_final_kg_equivalent': sum(s['food_reserve_equivalent_kg'] for s in final['sites']),
        'population_final': sum(s['population'] for s in final['sites']),
        'abandoned_final': sum(s['abandoned'] for s in final['sites']),
        'unmet_ration_fraction': max(0., 1-eaten/need) if need else None,
        'severe_household_observation_fraction': deprived/observed if observed else None,
        'max_economy_relative_error': max(abs(v) for r in rows for v in r['economy_residuals']),
        'max_ecology_relative_error': max(abs(v) for r in rows if r['budget'] for v in r['budget']['relative_error']),
        'max_water_relative_error': max(abs(r['budget']['water_relative_error']) for r in rows if r['budget']),
        'lake_guild_final_carbon_kg': [final['wildlife']['carbon_kg'][1][k] for k in (8,9,10)],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('directory', type=Path)
    args = parser.parse_args()
    paths = sorted(args.directory.glob('*.jsonl*'))
    runs = {}
    for path in paths:
        rows = read_rows(path)
        if len(rows) < 2:
            continue
        key = rows[0]['seed'], rows[0]['branch']
        if key in runs:
            raise ValueError(f'duplicate input {key}')
        metadata = metadata_for(args.directory, key[0])
        expected = metadata['months']
        if [r['elapsed_months'] for r in rows] != list(range(expected+1)):
            raise ValueError(f'incomplete or reordered trajectory {path}')
        runs[key] = rows
    for seed in {key[0] for key in runs}:
        required = {'baseline','sham','removed','restored','closed','removed_closed'}
        expected_branches = set(metadata_for(args.directory, seed).get('branches', required))
        if not required <= expected_branches <= required | {'work_ablation'}:
            raise ValueError(f'invalid declared branch suite for seed {seed}')
        if {key[1] for key in runs if key[0] == seed} != expected_branches:
            raise ValueError(f'incomplete branch suite for seed {seed}')
    if not runs:
        raise ValueError('no trajectories found')
    summaries = []
    for (seed, _), rows in runs.items():
        metadata = metadata_for(args.directory, seed)
        fish = next(g for g in metadata['economy_catalog']['goods'] if g['id'] == 'fish')
        # The production shader caps food conversion by embodied C/N/P as well
        # as the nominal food-energy factor. This is potential ration output,
        # not a claim that all caught fish was actually eaten.
        effective_energy = min(fish['food_energy'], *(v / need for v, need in zip(fish['cnp'], (.45, .02, .003))))
        summaries.append(summarize(rows, effective_energy))
    summaries.sort(key=lambda s: (s['seed'], s['branch']))
    checks = []
    for seed in sorted({key[0] for key in runs}):
        baseline = runs.get((seed, 'baseline'))
        sham = runs.get((seed, 'sham'))
        if baseline and sham:
            checks.append({'seed':seed, 'check':'sham physical trajectories equal baseline',
                           'passed':len(baseline)==len(sham) and all(a['sites']==b['sites'] and a['wildlife']==b['wildlife'] for a,b in zip(baseline,sham))})
        removed = runs.get((seed, 'removed'))
        restored = runs.get((seed, 'restored'))
        if removed and restored:
            half = len(removed)//2 + 1
            checks.append({'seed':seed, 'check':'restoration branch matches removal before restoration',
                           'passed':all(a['sites']==b['sites'] and a['wildlife']==b['wildlife'] for a,b in zip(removed[:half],restored[:half]))})
        for branch in ('closed', 'removed_closed'):
            rows = runs.get((seed,branch))
            if rows:
                checks.append({'seed':seed,'check':f'{branch} has no new catch',
                               'passed':all(s['catch_kg']==0 for r in rows[1:] for s in r['sites'])})
    result = {'summaries':summaries,'checks':checks}
    (args.directory/'summary.json').write_text(json.dumps(result,indent=2)+'\n')
    print('| Seed | Branch | Catch kg | Final food kg equiv. | Population | Unmet rations | Severe household observations |')
    print('|---|---|---:|---:|---:|---:|---:|')
    for s in summaries:
        hunger = s['severe_household_observation_fraction']
        print(f"| {s['seed']} | {s['branch']} | {s['catch_kg']:.3f} | {s['food_reserve_final_kg_equivalent']:.2f} | {s['population_final']:.2f} | {s['unmet_ration_fraction']:.2%} | {hunger:.2%} |" if hunger is not None else str(s))
    for check in checks:
        print(check)
    if any(not c['passed'] for c in checks):
        raise SystemExit('Control check failed; inspect trajectories before attributing effects.')


if __name__ == '__main__':
    main()
