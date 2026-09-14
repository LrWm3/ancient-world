#!/usr/bin/env python3
"""Local retained food windows and latest cultural work, not a causal attribution.

Reuse monthly boundary accounting. Report incomplete coverage and distinguish
GPU production from transfers; an unclassified net flow is not necessarily trade.
Cultural work plans describe only their dated boundary, not annual service output.
"""
import argparse
import json
from collections import defaultdict

from report_monthly_food import SHORTAGE_TOLERANCE_KG, rows


def summarize(history):
    grouped = defaultdict(list)
    for row in rows(history):
        grouped[row['site']].append(row)
    sites = []
    for site, samples in sorted(grouped.items()):
        samples.sort(key=lambda r: r['month'])
        need = sum(r['need_kg'] for r in samples)
        def total(key):
            return sum(r[key] for r in samples)
        complete = all(r['whole_month'] is not None for r in samples)
        sites.append(dict(
            site=site, first_month=samples[0]['month'], last_month=samples[-1]['month'],
            observed_months=len(samples),
            missing_months=samples[-1]['month'] - samples[0]['month'] + 1 - len(samples),
            complete_boundaries=complete, need_kg=need,
            physical_shortfall_kg=total('physical_shortfall_kg'),
            physical_shortage_months=sum(r['physical_shortfall_kg'] > SHORTAGE_TOLERANCE_KG for r in samples),
            access_shortage_months=sum(r['access_shortfall_kg'] > SHORTAGE_TOLERANCE_KG for r in samples),
            access_shortfall_kg=total('access_shortfall_kg'),
            gpu_produced_kg=sum(r['gpu']['produced_kg'] for r in samples),
            gpu_production_need_ratio=sum(r['gpu']['produced_kg'] for r in samples) / need if need else None,
            recorded_spoilage_kg=(sum(r['whole_month']['recorded_spoilage_kg'] for r in samples)
                                  if complete else None),
            unclassified_net_kg=(sum(r['whole_month']['unclassified_net_kg'] for r in samples)
                                 if complete else None),
            closing_stock_kg=samples[-1]['closing_kg']))
    culture = history.get('culture') or {}
    receipt = culture.get('work_receipt')
    # Never relabel an old plan as current-month service. Empty/absent data are unknown.
    plan_month = max((p['month'] for p in culture.get('work_plans', [])), default=None)
    plans = [p for p in culture.get('work_plans', []) if p['month'] == plan_month]
    upkeep = [r for p in plans for r in p['upkeep']]
    return dict(seed=history['seed'], month=history['month'], sites=sites,
                cultural_work_receipt=receipt, latest_plan_month=plan_month,
                latest_plan_count=len(plans),
                upkeep_requested=sum(r['requested'] for r in upkeep) if plans else None,
                upkeep_granted=sum(r['granted'] for r in upkeep) if plans else None,
                upkeep_used=sum(r['used'] for r in upkeep) if plans else None,
                successor_knowledge_acquisitions=sum(
                    bool((p.get('successor_expectation') or {}).get('actual_acquisition'))
                    for p in plans) if plans else None)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('histories', nargs='+')
    for path in parser.parse_args().histories:
        with open(path) as source:
            print(json.dumps(dict(path=path, **summarize(json.load(source))), allow_nan=False))


if __name__ == '__main__':
    main()
