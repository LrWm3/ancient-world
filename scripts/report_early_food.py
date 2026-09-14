#!/usr/bin/env python3
"""Read-only early-food report. Cumulative requests are not deliveries.

Managed production's food ledger uses calorie-equivalent kg. Raw crop harvest
mass is deliberately not summed into that ledger: crops differ in food energy.
"""
import argparse
import json


def summarize(h):
    sites = h['sites']
    if any(s['economy'].get('management', [0])[0] <= .5 for s in sites):
        raise ValueError('report requires managed farming; legacy ledger records standing growth')
    rows = []
    for s in sites:
        samples = s['lifecycle']['timeline']['samples']
        rows.append(dict(site=s['id'], population=s['stocks']['stock'][0],
                         food_kg=s['stocks']['stock'][1],
                         food_produced_kg=s['stocks']['ledger'][0],
                         food_eaten_kg=s['stocks']['ledger'][1],
                         food_spoiled_kg=s['stocks']['ledger'][2],
                         retained_sample_months=len(samples),
                         empty_food_sample_months=sum(r['food'] < .01 for r in samples)))
    shipping = h.get('shipping') or {}
    requests = h.get('trade_contact', {}).get('food_requests')
    return dict(seed=h['seed'], month=h['month'], initial_food_kg=h['initial_food'], sites=rows,
                **{k: sum(r[k] for r in rows) for k in
                   ('food_kg', 'food_produced_kg', 'food_eaten_kg', 'food_spoiled_kg')},
                surveyed_ports=len(shipping.get('ports', [])),
                commissioned_ports=sum(p['commissioned'] is not None for p in shipping.get('ports', [])),
                vessels=sum(len((p.get('fleet') or {}).get('vessels', [])) for p in shipping.get('ports', [])),
                food_dispatched_kg=None if requests is None else sum(r['dispatched_kg'] for r in requests),
                food_dispatches=None if requests is None else sum(r['dispatches'] for r in requests))


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('histories', nargs='+')
    args = p.parse_args()
    for path in args.histories:
        with open(path) as f:
            h = json.load(f)
        print(json.dumps(dict(path=path, **summarize(h))))


if __name__ == '__main__':
    main()
