#!/usr/bin/env python3
"""Compare annual demand-economy outcomes; do not confuse throughput with stocks."""
import argparse,json,tomllib
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('report');p.add_argument('--baseline');p.add_argument('--out',default='output/demand-economy-analysis.md');a=p.parse_args()
r=json.loads(Path(a.report).read_text());assert r['complete'],'Incomplete experiment'
c=tomllib.loads(Path('assets/economy.toml').read_text());dry=[i for i,g in enumerate(c['goods']) if i!=63 and g.get('food_energy',0)<=0]
lines=['# Demand-driven economy evaluation','','Diagnostic terrain/ecology edge 64; monthly living history, scarce-island defaults. Stored goods exclude the separately tracked prepared-food reserve. Dry goods exclude edible raw stocks. Timber and flax growth is controlled at production; their existing stocks are retained. Excess incidental animal byproducts enter recorded compost.','', '| Seed | Years | Population | Dry kg/person | Wood tonnes | Flax tonnes | Shortage site-years | Deliveries | Residual | Seconds |','|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
for run in r['runs']:
 s=run['samples'][-1];e=s['production'];g=e['goods_kg'];pop=s['population']
 lines.append(f"| {run['seed']} | {s['month']//12} | {pop:.0f} | {sum(g[k] for k in dry)/pop:.1f} | {g[0]/1000:.1f} | {g[13]/1000:.1f} | {run['shortage_site_years']} | {run['events'].get('market_arrival',0)} | {run['max_relative_residual']:.2e} | {run['seconds']:.1f} |")
 earlier=next((x for x in run['samples'] if x['month']==s['month']-600),None)
 if earlier:
  old=earlier['production']['goods_kg'];print(run['seed'],'last50year dry kg/person',sum(old[k] for k in dry)/earlier['population'],'->',sum(g[k] for k in dry)/pop)
if a.baseline:
 b=json.loads(Path(a.baseline).read_text());lines+=['','## Previous completed histories','','| Seed | Population before → after | Shortage observations before → after | Deliveries before → after |','|---|---:|---:|---:|']
 for run in r['runs']:
  old=next(x for x in b['runs'] if x['seed']==run['seed']);assert b['years']==r['years']
  lines.append(f"| {run['seed']} | {old['samples'][-1]['population']:.0f} → {run['samples'][-1]['population']:.0f} | {old['shortage_site_years']} → {run['shortage_site_years']} | {old['events'].get('market_arrival',0)} → {run['events'].get('market_arrival',0)} |")
 lines+=['','These are whole-system comparisons. Historical trajectories can diverge; differences do not isolate the effect of an individual rule. Use --legacy-production for a current-build control.']
Path(a.out).write_text('\n'.join(lines)+'\n');print('\n'.join(lines))
