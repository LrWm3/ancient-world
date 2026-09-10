"""Matched 2x2 wage/employer experiments using one immutable, verified evaluator."""
import argparse
from collections import Counter
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import time
from integrated_history import digest

MODES = {
    'communal-equal': ['--no-enterprises', '--equal-payroll'],
    'communal-linked': ['--no-enterprises'],
    'operators-equal': ['--equal-payroll'],
    'operators-linked': [],
}

def summarize(data):
    if not data.get('complete') or len(data['runs']) != 1:
        raise ValueError('Expected a completed single-seed run')
    run = data['runs'][0]
    samples = run['samples']
    end = data['years'] * 12
    if [s['month'] for s in samples] != list(range(12, end + 1, 12)):
        raise ValueError('Incomplete annual trajectory')
    firms = (run.get('enterprises') or {}).get('firms', [])
    closed = [f for f in firms if f['closed'] is not None]
    eligible = [f for f in firms if f['founded'] + 60 <= end]
    survived = [f for f in eligible if f['closed'] is None or f['closed'] - f['founded'] >= 60]
    sums = {k: sum(f[k] for f in firms) for k in (
        'capital', 'cash', 'revenue', 'wages', 'rent', 'dividends', 'liquidation',
        'written_off', 'paid_work', 'completed_work')}
    for f in firms:
        residual = f['cash'] - f['capital'] - f['revenue'] + f['wages'] + f['rent'] + f['dividends'] + f['liquidation']
        if abs(residual) > 1e-7 * (1 + f['capital'] + f['revenue']):
            raise ValueError('Firm cash does not reconcile')
    if run['max_relative_residual'] >= .001:
        raise ValueError('Failed integrated conservation tolerance')
    return dict(seed=run['seed'], population=samples[-1]['population'],
        active_sites=samples[-1]['active_sites'], shortage_site_years=run['shortage_site_years'],
        abandonments=len(run['abandonments']), founded=len(firms), active=len(firms)-len(closed),
        closed=len(closed), closure_reasons=dict(Counter(f['closing_reason'] for f in closed)),
        median_closed_lifetime_months=statistics.median(f['closed']-f['founded'] for f in closed) if closed else None,
        five_year_eligible=len(eligible), five_year_survivors=len(survived),
        utilization=sums['completed_work']/sums['paid_work'] if sums['paid_work'] else None,
        max_residual=run['max_relative_residual'], **sums)

def write_report(directory, rows):
    (directory/'summary.json').write_text(json.dumps(rows, indent=2)+'\n')
    lines=['# Matched employer and household-income experiments', '',
        'These are game-balance checks, not empirical wage or business-survival calibration. '
        'Modes share seeds, catalogs, geography and history duration. Later population differences '
        'include feedback; controlled fixtures identify the immediate cash and capacity mediators.', '',
        '| Mode | Seed | Population | Active towns | Shortage site-years | Firms started / active / closed | Paid-work utilization | Max residual |',
        '|---|---:|---:|---:|---:|---:|---:|---:|']
    for r in rows:
        utilization='—' if r['utilization'] is None else f"{r['utilization']:.1%}"
        lines.append(f"| {r['mode']} | {r['seed']} | {r['population']:.0f} | {r['active_sites']} | {r['shortage_site_years']} | {r['founded']} / {r['active']} / {r['closed']} | {utilization} | {r['max_residual']:.2e} |")
    # Report each intervention within matched seeds; interaction is a difference of differences.
    effects=[]
    by_seed={}
    for row in rows: by_seed.setdefault(row['seed'], {})[row['mode']]=row
    for seed, group in by_seed.items():
        if set(group) != set(MODES): continue
        for metric in ('population','active_sites','shortage_site_years','abandonments'):
            ce=group['communal-equal'][metric];cl=group['communal-linked'][metric]
            oe=group['operators-equal'][metric];ol=group['operators-linked'][metric]
            effects.append(dict(seed=seed,metric=metric,wages_without_operators=cl-ce,
                operators_with_equal_wages=oe-ce,operators_with_linked_wages=ol-cl,
                interaction=(ol-cl)-(oe-ce)))
    (directory/'effects.json').write_text(json.dumps(effects,indent=2)+'\n')
    lines += ['', 'Closure counts come from persistent firm records, not annual snapshots. '
        'Five-year survival uses only firms founded at least five years before the end; '
        'still-active younger firms are censored. Accounts, closure reasons and eligible counts '
        'are retained in summary.json and raw trajectories. No runtime here is an isolated hardware benchmark.', '']
    (directory/'summary.md').write_text('\n'.join(lines))

def main():
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('--binary', type=Path, required=True)
    p.add_argument('--build-manifest', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    p.add_argument('--seeds', default='17,81,409')
    p.add_argument('--years', type=int, default=50)
    p.add_argument('--yield-scale', type=float, default=.33)
    p.add_argument('--modes', default=','.join(MODES))
    a=p.parse_args()
    seeds=[int(s) for s in a.seeds.split(',')]
    modes=a.modes.split(',')
    if len(set(seeds)) != len(seeds) or not seeds or a.years < 1 or not 0 < a.yield_scale <= 1:
        p.error('Expected unique seeds, positive years, and yield scale in (0, 1]')
    if len(set(modes)) != len(modes) or not modes or any(m not in MODES for m in modes):
        p.error('Invalid or duplicate modes')
    build=json.loads(a.build_manifest.read_text())
    if digest(a.binary) != build['binary_sha256']:
        raise ValueError('Executable does not match verified build')
    a.output.mkdir(parents=True, exist_ok=False)
    binary=(a.output/'evaluator.bin').resolve()
    shutil.copy2(a.binary, binary)
    manifest=dict(complete=False, build=build, seeds=seeds, modes=modes, years=a.years,
        yield_scale=a.yield_scale, runs=[])
    rows=[]
    def persist():
        (a.output/'manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
        write_report(a.output, rows)
    persist()
    for seed in seeds:
        for mode in modes:
            name=f'{mode}-{seed}'
            output=a.output/name
            command=[str(binary),'--seeds',str(seed),'--years',str(a.years),
                '--resolution','64','--epochs','1','--civilizations','16',
                '--crop-yield-scale',str(a.yield_scale),'--discoveries','--living-world',
                '--waterworks-repair-priority','--offices','--output',str(output),'--label',name]+MODES[mode]
            record=dict(name=name, command=command, status='running')
            manifest['runs'].append(record);persist()
            print(f'Starting {name}',flush=True);start=time.monotonic()
            with output.with_suffix('.log').open('w') as log:
                result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
            record.update(seconds=time.monotonic()-start,exit_code=result.returncode)
            if result.returncode:
                record['status']='failed';persist();raise SystemExit(f'Failed {name}; inspect its log')
            data=json.loads(output.with_suffix('.json').read_text())
            if data['runs'][0]['seed'] != seed or data['years'] != a.years or data['resolution'] != 64:
                raise ValueError('Trajectory does not match requested scenario')
            row=summarize(data);row['mode']=mode;rows.append(row)
            record.update(status='complete',json_sha256=digest(output.with_suffix('.json')))
            persist();print(f"Finished {name}: {row['founded']} firms, {row['active']} active; {record['seconds']:.1f}s",flush=True)
    manifest['complete']=True;persist()

if __name__ == '__main__':
    main()
