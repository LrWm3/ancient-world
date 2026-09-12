#!/usr/bin/env python3
"""Matched council-policy evidence; run only on completed monthly-observed reports."""
import argparse
import json
from pathlib import Path
from compare_food_access import compare


def arrears(sample):
    """Separate retained historical counters from currently active administrations."""
    administrations = sample['administrations']
    stored = max((a['unpaid_months'] for a in administrations), default=0)
    if 'active_site_ids' not in sample:
        return stored, None  # Older reports cannot establish this retrospectively.
    ids = sample['active_site_ids']
    if (not isinstance(ids, list)
            or any(type(i) is not int or i < 0 or i >= len(administrations) for i in ids)
            or len(set(ids)) != len(ids)):
        raise ValueError('invalid active_site_ids for administration report')
    return stored, max((administrations[i]['unpaid_months'] for i in ids), default=0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('baseline', type=Path)
    parser.add_argument('treatment', type=Path)
    parser.add_argument('--allow-difference', action='append', default=[])
    args = parser.parse_args()
    baseline, treatment = (json.loads(p.read_text()) for p in (args.baseline, args.treatment))
    try:
        compare(baseline, treatment, set(args.allow_difference))
        for report in (baseline, treatment):
            for run in report['runs']:
                arrears(run['samples'][-1])
    except (ValueError, KeyError, TypeError) as error:
        parser.error(str(error))
    print('| Seed | Arm | Population | Food access gap % | Admin unpaid % | Longest stored unpaid streak | Longest active unpaid streak | Relief withheld by allowance |')
    print('|---|---|---:|---:|---:|---:|---:|---:|')
    for report, label in ((baseline, 'baseline'), (treatment, 'treatment')):
        for run in report['runs']:
            s = run['samples'][-1]
            a = s['council_funding']['administration']
            gap = 100 * a['shortfall'] / a['requested'] if a['requested'] else 0
            relief = s['council_relief_totals']
            stored, active = arrears(s)
            print(f"| {run['seed']} | {label} | {s['population']:.0f} | "
                  f"{100*s['food_totals'][5]/s['food_totals'][0]:.4f} | {gap:.3f} | "
                  f"{stored} | {active if active is not None else 'unknown'} | {relief[4]:.3f} |")
    print('\nStored arrears may include inactive sites; active streak is unknown for older reports. Use requested/paid totals for funding coverage.')
    print('\nEnding annual tax boundaries (not monthly revenue forecasts):')
    print('\n| Seed | Arm | Site | Opening town cash | Rate | Office capacity | Autonomy | Tax paid | Ending admin unpaid months |')
    print('|---|---|---|---:|---:|---:|---:|---:|---:|')
    for report, label in ((baseline, 'baseline'), (treatment, 'treatment')):
        for run in report['runs']:
            s = run['samples'][-1]
            for t in s['council_funding']['taxes']:
                a = s['administrations'][t['site']]
                if a['unpaid_months']:
                    print(f"| {run['seed']} | {label} | {t['site']} | {t['opening_cash']:.2f} | "
                          f"{t['rate']:.4f} | {t['office_capacity']:.3f} | {t['autonomy']:.3f} | "
                          f"{t['paid']:.3f} | {a['unpaid_months']} |")


if __name__ == '__main__':
    main()
