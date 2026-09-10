"""Compare matched policy runs and check that trajectories agree before intervention."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
from analyze_integrated_history import summarize


def load_suite(directory):
    manifest = json.loads((directory/'manifest.json').read_text())
    if not manifest['complete']:
        raise ValueError('Incomplete suite')
    result = {}
    for run in manifest['runs']:
        if run['status'] != 'complete':
            raise ValueError('Incomplete run')
        raw = (directory/(run['name']+'.json')).read_bytes()
        if hashlib.sha256(raw).hexdigest() != run['json_sha256']:
            raise ValueError('Invalid raw checksum')
        result[run['name']] = json.loads(raw)
    return result


def compare(before, after):
    a, b = summarize(before), summarize(after)
    if a['seed'] != b['seed'] or before['years'] != after['years']:
        raise ValueError('Unmatched seed or duration')
    pairs = list(zip(before['runs'][0]['samples'], after['runs'][0]['samples']))
    first = next((y['month'] for _, y in pairs
                  if y['cumulative_events'].get('autonomy_negotiated', 0)), None)
    # This suite changes only council response, not initial state or environment.
    # Before the first recorded concession, every annual mediator must still agree.
    for x,y in pairs:
        # Explicit scenario toggles leave a factual event even before any response.
        # Ignore only that declared intervention record, not resulting mediator changes.
        x, y = copy.deepcopy(x), copy.deepcopy(y)
        for sample in (x, y):
            sample['cumulative_events'].pop('autonomy_negotiation_policy', None)
        if first is None or y['month'] < first:
            if x != y:
                raise ValueError(f"Trajectory changed before intervention at month {y['month']}")
    return {'seed':a['seed'], 'first_concession_observed_month':first,
            'identical_pre_intervention_years':sum(first is None or y['month'] < first for _,y in pairs),
            'before':a, 'after':b}


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('before', type=Path)
    p.add_argument('after', type=Path)
    a=p.parse_args()
    before, after = load_suite(a.before), load_suite(a.after)
    if before.keys() != after.keys():
        raise ValueError('Suites must contain the same profiles and seeds')
    rows = [dict(name=name, **compare(before[name], after[name])) for name in before]
    (a.after/'comparison.json').write_text(json.dumps(rows,indent=2)+'\n')
    lines = ['# Council response: matched century trajectories','',
             '| Run | Identical years before concession | Crises before → after | Concessions | Recoveries before → after | Secessions before → after | Population before → after |',
             '|---|---:|---:|---:|---:|---:|---:|']
    for r in rows:
        x,y = r['before'],r['after']
        lines.append(f"| {r['name']} | {r['identical_pre_intervention_years']} | {x['crises']} → {y['crises']} | {y['negotiations']} | {x['recoveries']} → {y['recoveries']} | {x['secessions']} → {y['secessions']} | {x['population']:.0f} → {y['population']:.0f} |")
    lines += ['', 'Annual mediators match before the first observed concession, excluding only the explicit policy-toggle event count. Controlled fixtures establish the immediate policy, fiscal and control effects. Later population and conflict differences are coupled trajectories, not isolated effect sizes. Zero concessions require identical complete trajectories. These are game-balance comparisons, not historical parameter estimates.', '']
    (a.after/'comparison.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    main()
