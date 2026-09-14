#!/usr/bin/env python3
"""Summarize optional GPU-boundary demographic diagnostics from history exports.

Annual rows are observed windows, possibly partial. The birth suppression split
is ordered arithmetic attribution, not an independent causal experiment.
"""
import argparse
import json


def summarize(history, through_year):
    audit = history.get('demographic_audit')
    if audit is None:
        raise ValueError('history has no demographic audit; enable --demographic-audit')
    rows = [r for r in audit['years'] if r['year'] <= through_year]
    if not rows:
        raise ValueError('no retained observations within requested window')
    def total(key):
        return sum(r[key] for r in rows)
    exposure = [sum(r['person_months'][b] for r in rows) for b in range(3)]
    deaths = sum(total(k) for k in ('base_deaths', 'nutrition_deaths', 'illness_deaths'))
    births = total('potential_births') - total('hunger_suppressed_births') - total('illness_suppressed_births')
    need = total('food_need_kg')
    return dict(seed=history['seed'], first_year=rows[0]['year'], last_year=rows[-1]['year'],
                observed_months=total('months'), person_months=exposure,
                adult_exposure_share=exposure[1] / sum(exposure) if sum(exposure) else None,
                **{k: total(k) for k in ('observed_births', 'observed_deaths',
                   'base_deaths', 'nutrition_deaths', 'illness_deaths',
                   'potential_births', 'hunger_suppressed_births', 'illness_suppressed_births')},
                birth_reconstruction_error=births-total('observed_births'),
                death_reconstruction_error=deaths-total('observed_deaths'),
                physical_deficit_share=total('physical_shortfall_kg') / need if need else None,
                access_deficit_share=total('access_shortfall_kg') / need if need else None)


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('histories', nargs='+')
    p.add_argument('--years', type=int, nargs='+', default=[1, 5, 20])
    a = p.parse_args()
    for path in a.histories:
        with open(path) as f:
            h = json.load(f)
        for year in a.years:
            print(json.dumps(dict(path=path, through_year=year, **summarize(h, year))))


if __name__ == '__main__':
    main()
