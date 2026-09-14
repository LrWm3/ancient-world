#!/usr/bin/env python3
"""Summarize growth_ladder --food-diagnostics; JSONL input, JSON stdout.

All quantities are monthly boundary observations. Cross-town redistribution is
an instantaneous upper bound: no claim that routes can deliver it in time.
"""
import argparse
import collections
import json


def summarize(path):
    years = collections.defaultdict(lambda: collections.defaultdict(float))
    with open(path) as source:
        for line in source:
            month = json.loads(line)
            year = years[(month["month"] - 1) // 12 + 1]
            year["months"] += 1
            rows = month["food"]
            if not rows:
                raise ValueError(f"Missing production observations at month {month['month']}")
            physical = surplus = 0.0
            for row in rows:
                need, eaten, available = (row[k] for k in
                    ("need_kg", "eaten_kg", "physically_available_kg"))
                unmet = max(0, need - eaten)
                missing = min(unmet, max(0, need - available))
                physical += missing
                surplus += max(0, available - need)
                year["need_kg"] += need
                year["eaten_kg"] += eaten
                year["physical_gap_kg"] += missing
                year["access_gap_kg"] += unmet - missing
                year["site_months"] += 1
                year["illness_sum"] += row["illness"]
                year["cultivated_ha_months"] += row["cultivated_ha"]
                year["land_capacity_ha_months"] += row["land_capacity_ha"]
                year["farm_requested_work"] += row["farm_workers"][3]
                year["farm_granted_work"] += row["farm_workers"][1]
                demand, supply, output = row["crop_probe"]
                year["crop_demand_kg"] += demand[3]
                year["crop_growth_kg"] += output[3]
                for i, name in enumerate(("runoff", "geological_release", "mineralization", "food_return")):
                    if "phosphorus_probe" in row:
                        year["phosphorus_" + name + "_kg"] += row["phosphorus_probe"][i]
                if demand[3] > 0:
                    year["growing_site_months"] += 1
                    for i, name in enumerate(("nitrogen", "phosphorus", "water")):
                        if supply[i] < demand[i] * 0.999:
                            year[name + "_short_site_months"] += 1
                before, after = row["before_gpu"], row["after_gpu"]
                for i, name in enumerate(("produced_kg", "consumed_kg", "spoiled_kg")):
                    year[name] += after["ledger_kg"][i] - before["ledger_kg"][i]
                if row["opening"] is None or row["closing"] is None:
                    raise ValueError("Incomplete monthly boundary")
                year["opening_phase_net_food_kg"] += before["food_kg"] - row["opening"]["food_kg"]
                year["response_phase_net_food_kg"] += row["closing"]["food_kg"] - after["food_kg"]
            year["instant_redistribution_upper_bound_kg"] += min(physical, surplus)
            year["world_physical_deficit_kg"] += max(0, physical - surplus)
            year["ending_resident_population"] = sum(s["population"] for s in month["sites"])
    return {year: dict(values) for year, values in sorted(years.items())}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path")
    args = parser.parse_args()
    print(json.dumps(summarize(args.path), indent=2))
