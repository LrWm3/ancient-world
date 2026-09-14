# Early food supply and staged harbor experiment

The [demographic window audit](demographic-window-audit.md) located heavy physical
food shortages in years two and three. This follow-up examines supply, storage and
routes, then tests a bounded harbor commissioning change. These are toy-world
balance experiments, not historical calibration.

## Five-year food trace

The same seeds 1024, 256 and 409 start with 600 people, five towns and 129,600 kg
calorie-equivalent provisions. Settings match the prior audit: frozen 32/32 world,
gradual nutrition, ordinary purchased food, council welfare reserves, wealth tax,
clothing and practical research; credit, issuance and solidarity disabled.

| Seed | Cumulative food production | Dietary need | Food eaten | Recorded spoilage | Ending town food | Commissioned / surveyed ports | Food cargo dispatched |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| 1024 | 341,101 | 494,134 | 418,287 | 13,452 | 37,743 | 2 / 4 | 356 |
| 256 | 476,882 | 527,293 | 486,794 | 24,033 | 95,055 | 0 / 5 | 0 |
| 409 | 449,209 | 529,874 | 488,234 | 21,341 | 68,634 | 2 / 5 | 0 |

Food figures are kg calorie equivalents. Managed food production is the edible
output entering the aggregate food ledger, not raw crop mass; adding raw tuber,
grain and flax harvest would be misleading. Dietary need comes from the integrated
GPU exposure audit. Dispatch is departure, not arrival. These town summaries
exclude food traveling or consumed by other systems and are not a complete closed
food balance; native validation retains the broader ledgers.

Seed 1024's production plus initial provisions does not cover dietary need even
before storage losses. Yet it also retains food in better-supplied towns. In seeds
256 and 409, aggregate production plus provisions exceeds need, but poor towns
still cannot consume all required food while better-supplied towns retain reserves.
Spoilage is smaller than the unmet dietary need in each seed. Eliminating it alone
would not resolve these runs' shortages, and it would not move food between towns.

Route evidence is material. In seed 256, all five ports are surveyed but none has
commissioned at year five; some already hold almost all required timber and masonry
but insufficient tools. Seed 409 has two commissioned ports, but buyers still
repeatedly record unusable routes. Seed 1024 makes only two food dispatches totaling
356 kg. The request counters are observations of constraints at attempted purchases,
not independent probabilities or proof that money never limits other purchases.

## Optional staged commissioning

`--staged-harbors=true` permits commissioning when **each** installed material
category reaches 25% of its existing target: 50 kg timber, 2.5 kg tools and 25 kg
masonry. The policy is stored in `Shipping`; old archives default to false.
`--staged-harbors=false` restores the full-material rule for future openings.
Omission preserves the archived policy. Ports already opened are not decommissioned
when changing the rule.

Targets remain 200/10/100 kg; construction can continue toward them. Annual
construction timing, material debits, scarce-tool protection, labor limits and wear
are unchanged. The new rule changes the minimum useful opening size, not investment
budgets or construction speed.

Existing handling capacity already scales with the least-complete material
category. A quarter-built harbor has 250 kg installed handling capacity; this is
not free shipping capacity. New ports still need real vessels and funded crew
work. The existing fleet/material limits remain in force, and flooding still
blocks service. Staged opening events report actual installed materials and
handling capacity rather than claiming a full harbor exists.

This does not create routes, remove distance limits, relax seller reserves, waive
prices, deliver cargo instantly or increase food output. It targets one observed
commissioning bottleneck. Better shipping can still fail if a source lacks surplus,
a buyer lacks funds, vessels are unavailable or the voyage takes too long.

## Reproduction and scope

Run the same command and archives as [the demographic screen](demographic-window-audit.md),
using 5 and 20 years. Compare only `--staged-harbors=false` and `true`, keeping
`--food-solidarity=false`. Raw results are under ignored `output/early-food-screen/`.

```sh
python3 scripts/report_early_food.py output/early-food-screen/256.json
python3 scripts/report_demographic_windows.py \
  output/early-food-screen/256-staged-20.json --years 1 5 20
PYTHONPATH=scripts python3 -m unittest scripts/test_report_early_food.py
```

The food reporter rejects legacy non-managed ledgers, distinguishes missing cargo
observations from zero, and labels retained timeline coverage. It does not infer
historical shortage duration from missing samples. Unit fixtures check commissioning
thresholds for every material, proportional handling capacity, empty fleets,
flooding and serialization. Native comparisons test integration with actual
construction and trade; no additional GPU readbacks are needed.

## Matched outcomes

| Years | Seed | Staged | Population | Commissioned ports | Vessels | Food dispatched kg | Integrated physical deficit / need |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |
| 5 | 1024 | Off | 445.92 | 2 | 6 | 356.1 | 15.384% |
| 5 | 1024 | On | 445.92 | 2 | 6 | 356.1 | 15.384% |
| 5 | 256 | Off | 544.19 | 0 | 0 | 0.0 | 7.680% |
| 5 | 256 | On | 544.49 | 2 | 5 | 256.4 | 7.663% |
| 5 | 409 | Off | 543.85 | 2 | 4 | 0.0 | 7.858% |
| 5 | 409 | On | 543.62 | 3 | 8 | 134.8 | 7.868% |
| 20 | 1024 | Off | 321.81 | 3 | 10 | 9,752.2 | 10.894% |
| 20 | 1024 | On | 329.65 | 3 | 11 | 9,986.3 | 10.736% |
| 20 | 256 | Off | 491.38 | 2 | 8 | 6,745.4 | 5.774% |
| 20 | 256 | On | 491.98 | 2 | 7 | 7,248.2 | 5.740% |
| 20 | 409 | Off | 496.50 | 3 | 12 | 4,879.2 | 5.762% |
| 20 | 409 | On | 497.24 | 3 | 11 | 5,725.9 | 5.783% |

The immediate mechanism works: some previously unusable harbors open with paid,
installed materials and acquire vessels through the existing system. In seed 256,
sites 0 and 3 open at month 36 with only 4.2 and 4.3 kg of installed tools,
providing about 422 and 432 kg of handling capacity. They could not meet the old
10 kg tool threshold. This also explains a limit: opening in year three cannot
prevent the shortages already observed in year two. No new food
source was added. Dispatched volumes remain small relative to cumulative demand,
so the early crisis persists. The five-year population response is mixed; at
twenty years the differences are +7.84, +0.60 and +0.74 people. These diverging
runs do not isolate every secondary effect on labor, prices or institutions.

Physical shortage is not the only outcome. Twenty-year additional access deficits
increase from 0.082% to 0.417%, 0.306% to 0.379%, and 0.172% to 0.195% respectively.
Opening trade does not automatically make food affordable to every household.
Seed 409 even has a slightly higher integrated physical deficit despite ending
with slightly more people. Keep the policy optional rather than treating endpoint
population as sufficient evidence for a new default.

All six staged runs pass existing monthly conservation validation. Independent
endpoint money residuals are below 3.8e-8 relative. The three fresh five-year
controls also pass. No conservation tolerance or crop/health coefficient changed.

The next useful experiment is local subsistence production in years one through
three: cultivated area, tool service, crop habitat and finite N/P/water constraints.
A separate supply test can examine reserve rules and vessel throughput, but those
should remain distinct interventions. Increasing harbor size or reducing spoilage
alone would not address all three seeds' observed gaps.

Verification: 207 ordinary library tests pass (156 extended/hardware tests ignored),
14 Python audit/reporter tests pass, and strict all-target Clippy passes. A fresh
seed-1024 five-year run with staged harbors disabled matches the entire original
control export after removing only the new default-false shipping policy field.
This is a same-backend regression and finite-budget check, not a full-world
checkpoint equivalence or cross-hardware study. No century-scale staged-harbor
balance claim is made by this short screen.
The ordinary market integration target also passes 19 tests, with two extended
cases ignored.
