#!/usr/bin/env python3
"""Summarize the completed, reproducible patron history acceptance runs."""
import json
import struct
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--prefix', default='patron-complete')
args = parser.parse_args()
PREFIX = args.prefix

def read(name):
    report = json.loads((ROOT / 'output' / f'{name}.json').read_text())
    if not report['complete']:
        raise SystemExit(f'{name} is not complete; refusing a final report')
    return report

def records(name, seed):
    path = ROOT / 'output' / f'{name}.{seed}.world'
    if not path.exists():
        return None
    with path.open('rb') as f:
        f.read(8)
        size = struct.unpack('<Q', f.read(8))[0]
        return json.loads(f.read(size))['civilizations']

lines = ['# Patron history acceptance results', '',
         'Evaluation runs use monthly living environments at terrain/ecology edge 64, sixteen initial arriving communities, and the existing scarce-island defaults. These are diagnostic history runs, not production-resolution GPU benchmarks. Local concurrent workloads affect wall times.', '']
for name, title in [(PREFIX, '200-year seed suite'), (f'{PREFIX}-long', '500-year histories')]:
    report = read(name)
    lines += [f'## {title}', '', '| Seed | Population | Active towns | Traditions | Active institutions | Objects | Expeditions | Max residual |', '|---|---:|---:|---:|---:|---:|---:|---:|']
    for run in report['runs']:
        s, c = run['samples'][-1], run['culture']
        lines.append(f"| {run['seed']} | {s['population']:.0f} | {s['active_sites']} | {c['traditions']} | {c['institutions']} | {c['artifacts']} | {s['expeditions']['launched']} | {run['max_relative_residual']:.2e} |")
    lines += ['', 'Residuals are fractions, not percentages. Active towns include very small surviving settlements; the separate population and shortage measures matter.', '', '| Seed | Accounts | Known-practice links | Recorded relationships | Shortage site-years | People records | Event records | Wall seconds |', '|---|---:|---:|---:|---:|---:|---:|---:|']
    for run in report['runs']:
        c = run['culture']; h = records(name, run['seed'])
        lines.append(f"| {run['seed']} | {c['accounts']} | {c['knowledge_links']} | {c['relationships']} | {run['shortage_site_years']} | {len(h['people']) if h else 'n/a'} | {len(h['events']) if h else 'n/a'} | {run['seconds']:.1f} |")
        assert c['patrons'] == c['departed'] == 16
        if h:
            assert not h['culture']['legacy_baseline']
            assert all(a['author'] is not None or a['institution'] is not None for a in h['culture']['accounts'])
            assert len([e for e in h['events'] if e['kind'] == 'patron_arrival']) == 16
    lines += ['', '| Seed | Living adult knowledge links | Active / all institutions | Manuscripts | Institution-funded voyages |', '|---|---:|---:|---:|---:|']
    for run in report['runs']:
        h = records(name, run['seed'])
        if not h: continue
        c = h['culture']
        adults = [a for a in c['agents'] if h['people'][a['person']]['died'] is None and h['month'] - h['people'][a['person']]['born'] >= 180]
        knowledge = sum(len(a['knowledge']) for a in adults)
        manuscripts = sum(a['kind'] == 'inscribed manuscript' and not a['destroyed'] and not a.get('lost',False) for a in c['artifacts'])
        sponsored = sum(v.get('institution') is not None for v in h['expeditions']['voyages'])
        lines.append(f"| {run['seed']} | {knowledge} across {len(adults)} adults | {sum(n['active'] for n in c['institutions'])} / {len(c['institutions'])} | {manuscripts} | {sponsored} |")
    lines += ['', '| Seed | Pilgrimages | Office campaigns | Curated specimens | Lost objects | Knowledge-source links |', '|---|---:|---:|---:|---:|---:|']
    for run in report['runs']:
        c=run['culture']
        lines.append(f"| {run['seed']} | {c.get('pilgrimages',0)} | {c.get('office_campaigns',0)} | {c.get('specimens',0)} | {c.get('lost_objects',0)} | {c.get('knowledge_sources',0)} |")
    lines += ['', '| Seed | Living household faiths | Primary town roles | Secondary town roles |', '|---|---:|---|---|']
    from collections import Counter
    for run in report['runs']:
        h=records(name,run['seed'])
        if not h: continue
        c=h['culture']
        living_faiths={c['household_faith'][hh['id']] for hh in h['society']['households'] if h['people'][hh['head']]['died'] is None and not h['sites'][hh['site']]['abandoned']}
        active={site['id'] for site in h['sites'] if not site['abandoned']}
        roles=[x['roles'] for x in run['agriculture'] if x['site'] in active]
        primary=Counter(x[0][0] for x in roles if x and x[0][1] > 0)
        secondary=Counter(x[1][0] for x in roles if len(x)>1 and x[1][1] > 0)
        fmt=lambda counter: ', '.join(f'{role}: {n}' for role,n in counter.most_common())
        lines.append(f"| {run['seed']} | {len(living_faiths)} | {fmt(primary)} | {fmt(secondary)} |")
    lines += ['', 'Traditions and people in the first tables are historical record counts, including dead people and traditions no longer dominant. Living knowledge is shown separately; manuscripts and institutional sponsorship are contingent outcomes, not guaranteed in every seed.', '']
    lines += ['', 'All sixteen initial patrons departed; daughter towns inherited affiliations without receiving new patrons. Full per-town crop, herd and role data, annual shortages, expeditions and stage timings are in the JSON beside each archive.', '']
lines += ['## Matched seed-17 controls', '', '| Mode | Population | Towns | Aid effort | Traditions | Institutions | Expeditions | Max residual |', '|---|---:|---:|---:|---:|---:|---:|---:|']
for name, label in [(PREFIX,'Diverse farms + aid'), (f'{PREFIX}-no-aid','Diverse farms, aid disabled'), (f'{PREFIX}-legacy','Legacy farms + aid')]:
    r=read(name)['runs'][0]; s,c=r['samples'][-1],r['culture']
    lines.append(f"| {label} | {s['population']:.0f} | {s['active_sites']} | {c['aid_effort']:.1f} | {c['traditions']} | {c['institutions']} | {s['expeditions']['launched']} | {r['max_relative_residual']:.2e} |")
lines += ['', 'The full 80-test hardware/catalog suite passed, followed by the additional office-campaign fixture and all four practices tests (81 distinct passing tests). Formatting and Clippy with warnings denied pass. Controlled fixtures cover conservation, finite patron service, schism ancestry, cultural identity under conquest, institutional expedition escrow/refund, ownership and custody, local pilgrimage, annual role inertia, specimen curation and checkpoint continuation.', '', 'The aid control changes assistance, not arrival supplies or initial doctrine. Later population and expedition outcomes can diverge in either direction because succession, shortages and political conflict interact. Legacy farming is a comparison model, not the abundance target.', '', '## Observed limits', '', 'Seed 81 reached the explicit 256-tradition capacity by year 500; further schisms are suppressed at that limit and the explorer reports it. Knowledge is concentrated in active teachers and institutions; historical links do not imply universal education. Regional pilgrimages currently execute only on open routes whose round trip fits the monthly labor budget, so distant sacred sites can remain inaccessible. Unique specimens are preserved only after actual workshop deliveries; their absence in a seed is valid.', '', '## Calibration', '', 'The earlier `patron-transmission-long` histories fell to 40 and 95 people at year 500. Controlled continuations from the same seed-17 year-200 archive isolated an unsustainably high nitrogen-fixation energy cost. Over the next century, the old-cost control reached 1,346 people, finite tool recycling alone reached 1,375, and the calibrated fixation cost plus recycling reached 5,639. The last intervention had a maximum final residual of 1.55e-5. This is a regional game calibration: fixation still consumes production energy and atmospheric nitrogen is recorded as an external input. It is not a general island fertility multiplier.', '', 'The cost is editable as `fixation_cost_kg` in the agriculture catalog. Old catalogs retain 80; new worlds use 12. Worn tools recover 90% as scrap, and remelting with fuel recovers 90% of that metal. Both stages retain irreversible losses. Raw control files are `output/patron-recycling-control.json`, `output/patron-recycling-intervention.json`, and `output/patron-nitrogen-intervention.json`.', '', '## Reproducibility and performance', '', 'The GPU pipeline/buffer cache preserved the original 30-year samples while reducing observed time from 18.74 to 10.27 seconds. Indexed genealogy validation preserved an exact two-year continuation fingerprint and reduced observed time from 6.87 to 4.72 seconds. The event-archive transaction comparison is recorded separately in `output/patron-clone-before2.json` and `output/patron-clone-after2.json`. Concurrent compilation and GPU work make these local timings unsuitable as fixed performance promises.', '', 'An original version-two 100-year archive imported with all 33 towns’ original stock arrays intact. It received an explicit cultural baseline at month 1200 and zero fabricated patrons. The final implementation advanced it another year (`output/patron-complete-legacy-import.json`). A 200-year current archive advanced for two years with different batch sizes and a midpoint checkpoint: history, terrain and ecology matched bitwise (`output/patron-complete-replay.json`). The desktop loaded that mature history and produced `output/patron-complete-explorer.png`. Independent causal-record audits are in `output/patron-complete-audit.json`. ', '']
profile_path = ROOT / 'output' / f'{PREFIX}-profile.json'
if profile_path.exists():
    profile=json.loads(profile_path.read_text())
    timings=profile['timings_ms']
    lines += ['## Final mature-history profile', '',
        f"A {profile['months']}-month continuation of the final seed-17 year-200 archive took {profile['seconds']:.2f} seconds while the long seed runs were also active. CPU social processing and validation took {timings['history_social_and_validation_wall_ms']:.0f} ms, GPU production plus readback {timings['history_production_and_readback_wall_ms']:.0f} ms, and transaction setup {timings['history_setup_wall_ms']:.0f} ms. The history fingerprint `{profile['history_fingerprint']}` also matches the independent checkpoint/batch replay. Social record processing remains the largest measured component in this diagnostic workload.", '']
memory_path = ROOT / 'output' / f'{PREFIX}-memory.json'
if memory_path.exists():
    memory = json.loads(memory_path.read_text())
    lines += ['## Memory observations', '', '| Process | Host high-water mark |', '|---|---:|']
    for name, values in memory['processes'].items():
        lines.append(f"| {name} | {values['VmHWM']} |")
    lines += ['', 'Host figures come from `/proc` lifetime high-water marks sampled before exit. The diagnostic GPU processes each reported approximately 301 MiB in `nvidia-smi`, including context overhead. These 64-edge worlds do not establish performance at production terrain resolutions.', '']
(ROOT/'docs'/'patron-results.md').write_text('\n'.join(lines))
print('Wrote docs/patron-results.md')
