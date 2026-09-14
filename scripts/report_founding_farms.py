#!/usr/bin/env python3
"""Report managed farm exports without treating endpoint probes as annual averages.

Crop harvest and edible production are cumulative. Crop energy is nominal energy
before processing, seed use, feeding and spoilage; it is not delivered food.
"""
import argparse
import json

LIMITS = {0: 'none', 1: 'nitrogen', 2: 'phosphorus', 3: 'water'}


def summarize(history):
    catalog = history['economy_catalog']
    crops = catalog['agriculture']['crops']
    goods = {g['id']: g for g in catalog['goods']}
    rows = []
    for site in history['sites']:
        economy = site['economy']
        if economy['management'][0] <= .5:
            raise ValueError('report requires managed farming')
        if len(crops) != len(economy['crops']):
            raise ValueError('crop catalog and state lengths differ')
        probe = economy['production_probe']
        stock = site['stocks']
        crop_rows = []
        for definition, state in zip(crops, economy['crops']):
            good = goods[definition['good']]
            crop_rows.append(dict(
                good=good['id'], land_share=state[0], standing_kg=state[1],
                seed_kg=state[2], cumulative_harvest_kg=state[3],
                cumulative_nominal_harvest_food_kg=state[3] * good['food_energy'],
                seasonal=definition.get('season') is not None))
        rows.append(dict(
            site=site['id'], abandoned=site['abandoned'],
            population=stock['stock'][0], food_kg=stock['stock'][1],
            cumulative_edible_production_kg=stock['ledger'][0],
            # Abandoned/zero-population sites can retain an old production probe.
            probe_may_be_stale=site['abandoned'] or stock['stock'][0] <= 0,
            last_tool_multiplier=probe[0], last_cultivated_ha=probe[1],
            available_ha=stock['habitat'][1],
            last_reported_limit=LIMITS.get(int(economy['diagnostics'][0]), 'unknown'),
            endpoint_soil_n_kg=economy['soil'][1],
            endpoint_soil_p_kg=economy['soil'][2],
            endpoint_water_m3=economy['water'][0], crops=crop_rows))
    return dict(seed=history['seed'], month=history['month'], sites=rows)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('histories', nargs='+')
    for path in parser.parse_args().histories:
        with open(path) as source:
            history = json.load(source)
        print(json.dumps(dict(path=path, **summarize(history))))


if __name__ == '__main__':
    main()
