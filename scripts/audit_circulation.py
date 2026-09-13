#!/usr/bin/env python3
"""Audit exported history stocks and cumulative operator flows; never mutate a world.

Vacancy is not extinction. In aggregate mode missing named residents cannot prove
that an estate has no beneficiaries. Cash categories below are disjoint; household
flags are overlapping diagnostic subsets, not additional money stocks.
"""
import argparse
import json
from pathlib import Path


def audit(history):
    society = history.get('society') or {}
    households = society.get('households', [])
    economy = society.get('household_economy') or {}
    accounts = economy.get('accounts', [])
    enterprises = history.get('enterprises') or {}
    sites = history['sites']
    people = {p['id']: p for p in history.get('people', [])}
    members = {}
    for resident in (history.get('participation') or {}).get('residents', {}).values():
        person = people.get(resident['person'])
        if person is not None and person['died'] is None and resident['household'] is not None:
            members.setdefault(resident['household'], []).append(resident)
    credit = history.get('credit') or {}
    cash = {
        'towns': sum(s['economy']['finance'][0] for s in sites),
        'households': sum(a['cash'] for a in accounts),
        'councils': sum(c['treasury'] for c in society.get('councils', [])),
        'institutions': sum(i['treasury'] for i in (history.get('culture') or {}).get('institutions', [])),
        'operators': sum(f['cash'] for f in enterprises.get('firms', [])),
        'service_escrow': sum(o['escrow'] for o in enterprises.get('orders', [])),
        'export_contract_escrow': sum(c['escrow'] for c in history.get('export_contracts', [])),
        'export_payment_escrow': sum(p['escrow'] for p in history.get('export_payments', [])),
        'expedition_purses': sum(v['purse'] for v in (history.get('expeditions') or {}).get('voyages', [])),
        'relocation_purses': sum(j['cash'] for j in society.get('relocation', {}).get('journeys', [])),
    }
    initial = sum(s['economy']['finance'][1] for s in sites)
    issued = sum(r['issued'] for r in credit.get('issuance', {}).get('receipts', []))
    households_by_site = {}
    retained = []
    for household in households:
        account = accounts[household['id']] if household['id'] < len(accounts) else None
        if account is None:
            continue
        living = members.get(household['id'], [])
        row = {
            'household': household['id'], 'site': household['site'],
            'cash': account['cash'], 'vacant_since': household.get('vacant_since'),
            'known_living_members': len(living),
            'known_travelers': sum(isinstance(r['presence'], dict) and 'Traveling' in r['presence'] for r in living),
            'food_need': account['need'], 'hunger': account['hunger'],
        }
        households_by_site.setdefault(household['site'], []).append(row)
        if row['vacant_since'] is not None or sites[household['site']]['abandoned']:
            retained.append(row)
    firms = []
    for firm in enterprises.get('firms', []):
        firms.append({
            'firm': firm['id'], 'site': firm['site'], 'closed': firm['closed'],
            'closing_reason': firm.get('closing_reason'),
            'revenue': firm['revenue'], 'wages': firm['wages'], 'rent': firm['rent'],
            'operating_margin': firm['revenue'] - firm['wages'] - firm['rent'],
            'interest_net': firm.get('financing', {}).get('interest_received', 0) - firm.get('financing', {}).get('interest_paid', 0),
            'paid_work': firm['paid_work'], 'completed_work': firm['completed_work'],
            'completion_per_paid_work': firm['completed_work'] / firm['paid_work'] if firm['paid_work'] else None,
        })
    return {
        'seed': history['seed'], 'month': history['month'],
        'individual_demography': (history.get('named_demography') or {}).get('individual', False),
        'initial_cash': initial, 'issued_cash': issued, 'cash': cash,
        'cash_total': sum(cash.values()),
        'relative_money_residual': (initial + issued - sum(cash.values())) / max(initial + issued, 1),
        'sites': [{
            'id': s['id'], 'name': s['name'], 'abandoned': s['abandoned'],
            'population': s['stocks']['stock'][0], 'food_stock': s['stocks']['stock'][1],
            'town_cash': s['economy']['finance'][0],
            'household_cash': sum(r['cash'] for r in households_by_site.get(s['id'], [])),
            'vacant_household_cash': sum(r['cash'] for r in households_by_site.get(s['id'], []) if r['vacant_since'] is not None),
        } for s in sites],
        'retained_households': retained, 'operators': firms,
        'reclaimed_cash': sum(r['cash'] for r in economy.get('reclamation', {}).get('receipts', [])),
        'reclamation_cases': len(economy.get('reclamation', {}).get('receipts', [])),
        'inheritance_transfers': len(economy.get('inheritance', {}).get('receipts', [])),
        'inherited_cash': sum(r['cash'] for r in economy.get('inheritance', {}).get('receipts', [])),
        'limitations': [
            'Endpoint balances do not establish why money accumulated.',
            'Vacancy means no eligible representative, not necessarily no beneficiaries.',
            'Sparse named membership cannot establish extinction in aggregate mode.',
            'Operating margins exclude financing, capital, dividends and liquidation.',
            'Food stocks and household hunger describe only the exported boundary.',
        ],
    }


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('history', type=Path)
    args = parser.parse_args()
    print(json.dumps(audit(json.loads(args.history.read_text())), indent=2, allow_nan=False))
