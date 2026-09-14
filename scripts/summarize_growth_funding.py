#!/usr/bin/env python3
"""Compare native history exports; cash columns are selected accounts, not a ledger.

Audit coverage is shown explicitly: retained annual rows may not cover the entire
history. Institution identity survival is separate from operational readiness.
"""
import argparse
import json
from pathlib import Path

OPERATIONAL_READINESS = 0.25
OPERATIONAL_BUILDING_CONDITION = 0.25


def operational(institution):
    """Mirror Institution::operational, including vacancies and unfinished halls."""
    if not institution['active']:
        return False
    capacity = institution['capacity']
    if capacity is None:
        return True
    mandate = capacity.get('mandate')
    if mandate is not None and mandate['holder'] is None:
        return False
    building = capacity.get('building')
    return capacity['readiness'] >= OPERATIONAL_READINESS and (
        building is None or (
            (building.get('facility') is not None or building['construction_remaining'] == 0)
            and building['condition'] >= OPERATIONAL_BUILDING_CONDITION))


def summarize(history, after_year=None):
    sites = history['sites']
    society = history['society']
    accounts = society['household_economy']['accounts']
    institutions = history['culture']['institutions']
    years = history['demographic_audit']['years']
    if after_year is not None:
        years = [y for y in years if y['year'] > after_year]
    if not years:
        raise ValueError('enable demographic-audit; no retained annual observations')
    last_year = history['month'] // 12
    decade = [y for y in years if last_year - 10 < y['year'] <= last_year]
    need = sum(y['food_need_kg'] for y in years)
    active = [n for n in institutions if n['active']]
    municipal = society['household_economy'].get('municipal_relief', {})
    municipal_receipts = municipal.get('receipts', [])
    return {
        'seed': history['seed'], 'month': history['month'],
        'municipal_relief_month': municipal.get('month'),
        'municipal_relief_requested': sum(r['requested'] for r in municipal_receipts),
        'municipal_relief_paid': sum(r['paid'] for r in municipal_receipts),
        'population': sum(s['stocks']['stock'][0] for s in sites),
        'active_towns': sum(not s['abandoned'] for s in sites),
        'total_towns': len(sites),
        'audit_after_year': after_year,
        'audit_first_year': min(y['year'] for y in years),
        'audit_last_year': max(y['year'] for y in years),
        'window_births': sum(y['observed_births'] for y in years),
        'window_deaths': sum(y['observed_deaths'] for y in years),
        'audit_months': sum(y['months'] for y in years),
        'final_decade_months': sum(y['months'] for y in decade),
        'final_decade_births': sum(y['observed_births'] for y in decade),
        'final_decade_deaths': sum(y['observed_deaths'] for y in decade),
        'physical_gap_pct': 100 * sum(y['physical_shortfall_kg'] for y in years) / need if need else None,
        'access_gap_pct': 100 * sum(y['access_shortfall_kg'] for y in years) / need if need else None,
        'town_cash': sum(s['economy']['finance'][0] for s in sites),
        'council_cash': sum(c['treasury'] for c in society['councils']),
        'household_cash': sum(a['cash'] for a in accounts),
        'institution_cash': sum(n['treasury'] for n in institutions),
        'wealth_tax_paid': sum(a['wealth_tax_paid'] for a in accounts),
        'household_food_spending': sum(a['food_spending'] for a in accounts),
        'active_institutions': len(active),
        'operational_institutions': sum(operational(n) for n in active),
        'institution_dues': sum(n['dues'] for n in institutions),
        'town_support_requested': society['council_funding']['emergency_town_support']['requested'],
        'town_support_paid': society['council_funding']['emergency_town_support']['paid'],
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('histories', nargs='+', type=Path)
    parser.add_argument('--after-year', type=int, help='Restrict annual audit to years strictly after this completed boundary; balances remain endpoint stocks')
    args = parser.parse_args()
    for path in args.histories:
        print(json.dumps({'file': str(path), **summarize(json.loads(path.read_text()), args.after_year)}, allow_nan=False))


if __name__ == '__main__':
    main()
