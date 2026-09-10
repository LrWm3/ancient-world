"""Extract matched expedition outcomes from verified evaluator ensembles."""
import argparse
import json
from pathlib import Path
from summarize_institution_succession import read_run


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--baseline', type=Path, required=True)
    p.add_argument('--current', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--seeds', default='17,81')
    a = p.parse_args()
    rows = []
    for seed in map(int, a.seeds.split(',')):
        metadata = None
        for mode, directory in [('shared-skill',a.baseline), ('crew-competence',a.current)]:
            data, run = read_run(directory, f'operators-linked-{seed}')
            settings = {k:v for k,v in data.items() if k not in ('runs','label')}
            if metadata is not None and metadata != settings:
                raise ValueError('Mismatched comparison settings')
            metadata = settings
            if run['seed'] != seed:
                raise ValueError('Mismatched seed')
            sample = run['samples'][-1]
            voyages = sample['expeditions']
            rows.append(dict(seed=seed,mode=mode,years=data['years'],
                launched=voyages['launched'],returned=voyages['returned'],lost=voyages['lost'],
                active=voyages['active'],crew_away=voyages['crew_away'],
                delivered_knowledge=sum(voyages['knowledge']),
                casualties=run['events'].get('expedition_casualty',0),
                strandings=run['events'].get('expedition_stranded',0),
                repairs=run['events'].get('expedition_repaired',0),
                rescues=run['events'].get('expedition_rescue',0),
                population=sample['population'],
                max_relative_residual=run['max_relative_residual']))
    a.output.write_text(json.dumps(rows,indent=2)+'\n')
    for row in rows:
        print(json.dumps(row))


if __name__ == '__main__':
    main()
