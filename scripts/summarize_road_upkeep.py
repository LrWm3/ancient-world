"""Summarize verified road-upkeep trajectories and a matched preceding build."""
import argparse
import json
from pathlib import Path
from summarize_institution_succession import read_run


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--current', type=Path, required=True)
    p.add_argument('--baseline', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    a = p.parse_args()
    rows = []
    for seed in (17, 81):
        name = f'operators-linked-{seed}'
        data, current = read_run(a.current, name)
        prior, baseline = read_run(a.baseline, name)
        settings = lambda d: {k:v for k,v in d.items() if k not in ("runs", "label")}
        if settings(data) != settings(prior):
            raise ValueError("Comparison settings differ")
        roads = current['roads']
        care = [r for r in roads if r['upkeep'] is not None]
        # A new road starts empty: remaining + weathered is total construction.
        material = sum(r['road_bricks'] for r in care)
        lost = sum(r['upkeep']['lost_kg'] for r in care)
        work = sum(r['upkeep']['work'] for r in care)
        if abs(material + lost - 100 * work) > .01 * max(1., work):
            raise ValueError('Road material/work totals disagree')
        rows.append(dict(seed=seed, roads=len(roads), maintained_state=len(care),
            improved=sum(r['road_bricks'] > 0 for r in roads),
            established=sum(r['upkeep']['matured'] for r in care),
            impaired=sum(r['upkeep']['impaired'] for r in care),
            material_kg=material, weathered_kg=lost, worker_months=work,
            deterioration_events=current['events'].get('road_deteriorated', 0),
            restoration_events=current['events'].get('road_restored', 0),
            population=current['samples'][-1]['population'],
            baseline_population=baseline['samples'][-1]['population'],
            max_residual=current['max_relative_residual']))
    a.output.write_text(json.dumps(rows, indent=2)+'\n')


if __name__ == '__main__':
    main()
