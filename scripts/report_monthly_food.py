#!/usr/bin/env python3
"""Monthly town food balances from optional GPU-boundary audit observations.

Unclassified net flow is an algebraic remainder, not a conservation check and not
necessarily trade. It can include provisions, cargo, relief and relocation.
"""
import argparse
import json

SHORTAGE_TOLERANCE_KG = 0.01


def flow(before, after):
    if before is None or after is None:
        return None
    production, consumption, spoilage = [
        y - x for x, y in zip(before['ledger_kg'], after['ledger_kg'])]
    change = after['food_kg'] - before['food_kg']
    return dict(produced_kg=production, recorded_consumption_kg=consumption,
                recorded_spoilage_kg=spoilage, stock_change_kg=change,
                unclassified_net_kg=change - production + consumption + spoilage)


def rows(history):
    trace = (history.get('demographic_audit') or {}).get('food', {}).get('months')
    if not trace:
        raise ValueError('monthly food trace is absent; rerun with --demographic-audit=true')
    seen = set()
    for sample in trace:
        key = (sample['month'], sample['site'])
        if key in seen:
            raise ValueError('duplicate monthly food sample')
        seen.add(key)
        need, eaten = sample['need_kg'], sample['eaten_kg']
        unmet = max(0, need - eaten)
        physical = min(unmet, max(0, need - sample['physically_available_kg']))
        yield dict(seed=history['seed'], month=sample['month'], site=sample['site'],
                   shortage=unmet > SHORTAGE_TOLERANCE_KG,
                   need_kg=need, eaten_kg=eaten,
                   physical_shortfall_kg=physical, access_shortfall_kg=unmet - physical,
                   opening_kg=None if sample['opening'] is None else sample['opening']['food_kg'],
                   closing_kg=None if sample['closing'] is None else sample['closing']['food_kg'],
                   whole_month=flow(sample['opening'], sample['closing']),
                   before_production=flow(sample['opening'], sample['before_gpu']),
                   gpu=flow(sample['before_gpu'], sample['after_gpu']),
                   after_production=flow(sample['after_gpu'], sample['closing']),
                   cultivated_ha=sample['cultivated_ha'],
                   tool_multiplier=sample['tool_multiplier'],
                   limiting_resource=sample['limiting_resource'],
                   cumulative_crop_harvest_kg=sample['crop_harvest_kg'])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('histories', nargs='+')
    for path in parser.parse_args().histories:
        with open(path) as source:
            history = json.load(source)
        for row in rows(history):
            print(json.dumps(dict(path=path, **row)))


if __name__ == '__main__':
    main()
