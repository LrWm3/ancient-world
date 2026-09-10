"""Summarize completed joint diagnostics without treating annual samples as monthly events."""
import argparse
import json
import hashlib
from pathlib import Path


def summarize(data):
    if not data.get('complete') or len(data['runs']) != 1:
        raise ValueError('Expected one complete seed run')
    run = data['runs'][0]
    samples = run['samples']
    if [s['month'] for s in samples] != list(range(12, data['years'] * 12 + 1, 12)):
        raise ValueError('Incomplete annual observations')
    site_years = sum(len(s['integration']['sites']) for s in samples)
    saturated = sum(s['planned_land_freight'] and s['free_land_freight_kg'] < 1
                    for y in samples for s in y['integration']['sites'])
    high_unrest = sum(s['unrest'] is not None and s['unrest'] > .65
                      for y in samples for s in y['integration']['sites'])
    losses = gains = 0
    previous = {}
    for year in samples:
        current = {s['site']: s['knowledge_holders'] for s in year['integration']['sites']}
        for site in current.keys() & previous.keys():
            if current[site] is not None and previous[site] is not None:
                losses += sum(a > 0 and b == 0 for a,b in zip(previous[site], current[site]))
                gains += sum(a == 0 and b > 0 for a,b in zip(previous[site], current[site]))
        previous = current
    last = samples[-1]
    local = last['integration']['sites']
    mean = lambda key: sum(s[key] for s in local)/max(len(local), 1)
    return {'seed':run['seed'], 'population':last['population'], 'active_sites':last['active_sites'],
            'shortage_site_years':run['shortage_site_years'], 'site_years':site_years,
            'freight_saturated_site_years':saturated,'high_unrest_site_years':high_unrest,
            'knowledge_losses':losses, 'knowledge_gains':gains,
            'final_single_holder_topics':sum(v == 1 for s in local for v in (s['knowledge_holders'] or [])),
            'water_coverage':mean('water_coverage'), 'crowding':mean('crowding'),
            'crises':run['events'].get('governance_crisis',0),
            'negotiations':run['events'].get('autonomy_negotiated',0),
            'recoveries':run['events'].get('governance_recovery',0),
            'secessions':run['events'].get('secession',0), 'wars':run['events'].get('war_declared',0),
            'deliveries':run['events'].get('market_arrival',0),
            'abandonments':len(run['abandonments']), 'max_relative_residual':run['max_relative_residual'],
            'seconds':run['seconds'],
            'history_wall_seconds':{k: v/1000 for k,v in run.get('timings_ms',{}).items()
                                    if k.startswith('history_') and k.endswith('_wall_ms')}}


def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('directory', type=Path)
    a=p.parse_args()
    manifest=json.loads((a.directory/'manifest.json').read_text())
    if not manifest['complete'] or any(r['status'] != 'complete' for r in manifest['runs']):
        raise ValueError('Suite is incomplete; inspect its failure logs instead')
    rows=[]
    for record in manifest['runs']:
        raw=(a.directory/(record['name']+'.json')).read_bytes()
        if hashlib.sha256(raw).hexdigest() != record['json_sha256']:
            raise ValueError('Run checksum differs from manifest')
        row=summarize(json.loads(raw))
        row['profile']=record['name'].split('-')[0]
        rows.append(row)
    (a.directory/'summary.json').write_text(json.dumps(rows,indent=2)+'\n')
    lines=['# Integrated history diagnostics','',
           'Selected profiles are shown below (baseline yield 0.5; harsh yield 0.33). Living ecology, expeditions and repair priority enabled. See the manifest for policy and freight interventions. This is game-balance evaluation, not empirical fitting.', '',
           '| Profile | Seed | People | Active sites | Abandonments | Shortage site-years | Freight saturated site-years | High-unrest site-years | Knowledge losses / gains | Water coverage | Max residual |',
           '|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    for r in rows:
        lines.append(f"| {r['profile']} | {r['seed']} | {r['population']:.0f} | {r['active_sites']} | {r['abandonments']} | {r['shortage_site_years']} | {r['freight_saturated_site_years']} | {r['high_unrest_site_years']} | {r['knowledge_losses']} / {r['knowledge_gains']} | {r['water_coverage']:.1%} | {r['max_relative_residual']:.2e} |")
    lines+=['','Knowledge transitions compare consecutive annual observations of continuously active towns; losses during abandonment are excluded. Counts can miss within-year losses/recovery. Water coverage is the final unweighted active-town mean. Low capacity saturation does not prove there is demand. This suite uses one resolution and one GPU; all initial-state and catalog metadata remain in the raw JSON.','']
    (a.directory/'summary.md').write_text('\n'.join(lines))


if __name__ == '__main__':
    main()
