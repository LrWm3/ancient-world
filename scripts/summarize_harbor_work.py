"""Compare verified harbor-work ensembles against matching road-upkeep runs."""
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
        data, run = read_run(a.current, name)
        prior, old = read_run(a.baseline, name)
        settings = lambda d: {k:v for k,v in d.items() if k not in ('runs', 'label')}
        if settings(data) != settings(prior):
            raise ValueError('Comparison settings differ')
        ports = run['harbors']
        rows.append(dict(seed=seed, harbors=len(ports),
            commissioned=sum(p['commissioned'] is not None for p in ports),
            impaired=sum(p['work']['impaired'] for p in ports if p['work']),
            worker_months=sum(p['work']['worker_months'] for p in ports if p['work']),
            first_commission_month=min((p['commissioned'] for p in ports if p['commissioned'] is not None), default=None),
            structural_capacity=sum(1000*min(a/t for a,t in zip(p['assets'],(200,10,100))) for p in ports if p['commissioned'] is not None),
            operational_ports=run['samples'][-1]['operational_ports'],
            prior_operational_ports=old['samples'][-1]['operational_ports'],
            population=run['samples'][-1]['population'],
            prior_population=old['samples'][-1]['population'],
            deterioration_events=run['events'].get('harbor_deteriorated',0),
            restoration_events=run['events'].get('harbor_restored',0),
            max_residual=run['max_relative_residual']))
    a.output.write_text(json.dumps(rows,indent=2)+'\n')


if __name__ == '__main__':
    main()
