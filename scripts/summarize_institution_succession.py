"""Compare retained evaluator trajectories, validating their recorded content hashes."""
import argparse
import gzip
import hashlib
import json
from pathlib import Path


def read_run(directory, name):
    manifest_path = directory / 'manifest.json'
    manifest = json.loads(manifest_path.read_bytes() if manifest_path.exists() else
        gzip.decompress((directory / 'run-manifest.json.gz').read_bytes()))
    if not manifest['complete']:
        raise ValueError('Incomplete ensemble')
    record = next(r for r in manifest['runs'] if r['name'] == name)
    path = directory / (name + '.json')
    # The evaluator harness records the raw JSON hash after successful completion.
    expected = record['json_sha256']
    raw = path.read_bytes() if path.exists() else gzip.decompress(path.with_suffix('.json.gz').read_bytes())
    if hashlib.sha256(raw).hexdigest() != expected:
        raise ValueError(f'Changed trajectory: {path}')
    data = json.loads(raw)
    run = data['runs'][0]
    if not data['complete'] or run['max_relative_residual'] >= .001:
        raise ValueError('Incomplete or unbalanced trajectory')
    return data, run


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline', required=True, type=Path)
    p.add_argument('--current', required=True, type=Path)
    p.add_argument('--seeds', default='17,81')
    p.add_argument('--output', required=True, type=Path)
    a = p.parse_args()
    rows = []
    for seed in map(int, a.seeds.split(',')):
        metadata = None
        for mode, directory in [('before', a.baseline), ('mandates', a.current)]:
            data, run = read_run(directory, f'operators-linked-{seed}')
            settings = {k:v for k,v in data.items() if k not in ('runs','label')}
            if metadata is not None and settings != metadata:
                raise ValueError('Comparison settings differ')
            metadata = settings
            if run['seed'] != seed:
                raise ValueError('Seed mismatch')
            active = [n for n in run['culture']['institution_capacity'] if n['active']]
            mandates = [n['capacity']['mandate'] for n in active if n['capacity'].get('mandate')]
            all_mandates = [n['capacity']['mandate'] for n in run['culture']['institution_capacity'] if n['capacity'].get('mandate')]
            rows.append(dict(seed=seed, mode=mode, years=data['years'],
                population=run['samples'][-1]['population'], shortage_site_years=run['shortage_site_years'],
                active_institutions=len(active), operational=sum(n['operational'] for n in active),
                vacancies=run['events'].get('institution_vacant',0),
                contests=run['events'].get('institution_succession_contested',0),
                selections=run['events'].get('institution_successor_selected',0),
                active_vacant=sum(m['holder'] is None for m in mandates),
                longest_open_vacancy_months=max((data['years']*12-m['vacant_since'] for m in mandates if m['holder'] is None),default=0),
                unrepresented_vacancies=sum(n['capacity'].get('mandate',{}).get('holder',-1) is None and n.get('eligible_local_members',-1) == 0 for n in active),
                oldest_currently_represented_vacancy_months=max((data['years']*12-n['capacity']['mandate']['vacant_since'] for n in active if n.get('eligible_local_members',0) > 0 and n['capacity'].get('mandate') and n['capacity']['mandate']['holder'] is None),default=0),
                assembly_work=sum(m['work'] for m in all_mandates),
                max_relative_residual=run['max_relative_residual']))
    a.output.write_text(json.dumps(rows,indent=2)+'\n')
    for row in rows:
        print(json.dumps(row))


if __name__ == '__main__':
    main()
