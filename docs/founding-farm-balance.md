# Founding farms: years one through three

This follows the [early food and harbor screen](early-food-and-staged-harbors.md).
The purpose is to locate the next intervention, not to raise yields until all towns
survive. No simulation rules or defaults changed in this investigation.

## Runs and observations

Nine fresh continuations of the same founding archives: seeds 1024, 256 and 409,
each stopped at years 1, 2 and 3. Simulation revision: `faa1c5e`. Each starts with
600 people, five towns and 129,600 kg of calorie-equivalent provisions. Frozen
32/32 grids on the Quadro RTX 5000; gradual nutrition, council welfare reserves,
wealth tax, practical research and clothing enabled; needs-based food, solidarity,
credit, issuance and staged harbors disabled. These archives use managed crops
without the optional seasonal canopy model. Default-resolution and living-world
behavior have not been established by this screen.

| Seed | Year | Food produced during year (kg equivalent) | Dietary need during year | Ending town food | Population |
| --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | 1 | 31,404 | 110,255 | 43,624 | 611.3 |
| 1024 | 2 | 64,955 | 110,110 | 15,932 | 575.5 |
| 1024 | 3 | 78,666 | 98,876 | 23,251 | 506.9 |
| 256 | 1 | 43,735 | 110,255 | 55,921 | 611.3 |
| 256 | 2 | 92,536 | 111,234 | 44,191 | 599.2 |
| 256 | 3 | 118,099 | 105,252 | 74,508 | 552.4 |
| 409 | 1 | 43,223 | 110,255 | 55,324 | 611.3 |
| 409 | 2 | 99,433 | 111,447 | 47,145 | 604.7 |
| 409 | 3 | 109,153 | 106,089 | 67,914 | 556.8 |

Annual production is the difference in cumulative managed edible-food ledger
entries between independently replayed endpoints. Need is the corresponding
annual GPU demographic observation. These are not a closed food balance: supplies
also spoil, travel and serve other activities. In particular, year-three production
exceeding need globally does not imply that every town had food throughout that year.

## What the measurements narrow down

- Starting provisions hide weak initial production: the first year's edible output
  covers only 28–40% of that year's need. Production rises in subsequent years.
- At the 45 town/year endpoints, the last tool multiplier is approximately 0.91–1.00,
  and cultivated area is approximately 47–86 ha against 160 ha available per town.
  More land alone would not change the binding labor allowance at these boundaries.
- Every sampled final production step reports no N/P/water constraint. Soil N/P
  and water remain positive at these endpoints. This **does not** establish that
  all intervening months were unconstrained; endpoint stocks are not monthly fluxes.
  Soil nutrient inventories are declining and require longer-term attention.
- Crop outcomes vary sharply within a world. Seed 409's site 0 has harvested only
  about 2 kg of tubers cumulatively after three years, while site 1 has harvested
  about 30,948 kg. The shared 8% tuber land allocation does not respond to that
  difference. These observations do not independently isolate temperature, weather,
  calendar and resource effects.
- Food processing preserves C/N/P as well as nominal energy. Raw crop mass,
  nominal harvest calories and the actual edible-food ledger must remain separate.
  A high raw tuber yield is not an equal quantity of grain-equivalent food.

## Next policy experiment

`production::plan` currently derives food staffing pressure from recent shortage
and cooked-food reserves below **three months**. It does not forecast survival to
the next local harvest. `food_worker_shares` responds gradually and protects
previously reserved services and feasible tool maintenance. Early provisions can
therefore keep the food-pressure signal low even while the settlement has yet to
establish replacement harvests.

The next bounded experiment should compare a harvest-aware staffing signal with
this reserve-only signal. It should use observed local production, known harvest
months and actual stored food; standing crops must be discounted rather than
counted as guaranteed supplies. Keep total work fixed and retain committed duties,
tool maintenance, seed, soil and water limits. Measure cultivation and crop growth
*before* the shortage, completed competing work, harvest timing, food deficits and
later population. Include a well-provisioned productive town as a negative control.
A crop-mix adaptation policy is a separate experiment, not part of the same change.

These are hypotheses for intervention. The current screen does not prove that
anticipatory staffing will suffice, and additional farm work could damage useful
industrial capacity. Neither mortality nor the global yield multiplier was tuned.

## Reproduction and checks

Use the founding archives and full settings described in the preceding screen,
with `--history-years 1`, `2`, or `3`, `--demographic-audit=true`, and
`--staged-harbors=false`. Generated histories, commands and native logs from this
screen reside locally under ignored `output/founding-farm-screen/`.

```
python3 scripts/report_founding_farms.py output/founding-farm-screen/*-[123].json
python3 -m unittest discover -s scripts -p 'test_report_*.py'
```

The reporter resolves crops by the archived catalog and labels cumulative quantities
and last-step probes explicitly. It flags potentially stale probes on abandoned
or empty sites. It does not infer annual constraint frequencies from endpoints.
Four new fixtures cover mass/energy separation, archived catalog identity,
stale probes and incompatible input. Eight reporter tests passed altogether.
All nine native runs exited successfully with their conservation validation enabled.
No new GPU layout, readback or simulation randomness was introduced; the existing
binary ran these comparisons. Full Rust tests were not rerun for this Python/docs
change. Founding archives and raw experiment artifacts are intentionally not
committed, so reproducing these exact numbers requires the retained local inputs.
