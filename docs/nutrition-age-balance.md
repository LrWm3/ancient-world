# Household nutrition and age-structure comparison

## Question

The held-out construction histories retain only 615–735 residents after a century.
Aggregate food-access gaps do not show which age groups bear deprivation. Before
changing crop output or demographic rates, compare the existing personal-nutrition
mechanism with its existing ablation.

The personal mortality adapter replaces each represented resident's age-band food
exposure with household food exposure; it does not add both hunger penalties.
The ablation also removes the following month's personal hunger work penalty, so
this experiment estimates their combined effect, not mortality alone. Both runs
retain actual residents, household retail, birth-age structure and the same food
production rules.

## Protocol

Use seeds 409 and 1024 for 100 years, terrain resolution 32, ecology 16, one
geological epoch, 16 founding civilizations, crop yield scale 0.5 and living
history. Enable individual demography, workshop, agriculture, extraction and
construction refinement, comparison receipts and household diagnostics. The
control adds `--no-individual-nutrition`.

The example now accumulates expected deaths by child/adult/elder band from every
completed demographic snapshot, separately for age-band exposure and household
exposure. Anonymous residents retain age-band exposure. The report also includes
current age populations and the count of observed site-months. These are summed
conditional expectations, not observed deaths or an independent demographic model.
Read them alongside actual birth/death resolution summaries, food-access gaps,
household observations and population residuals.

Commands use `target/debug/examples/cultural_work_calibrate` with:

```sh
--seeds 409,1024 --years 100 --resolution 32 --crop-yield-scale 0.5 \
--individual-demography --workshop-refinement --agriculture-refinement \
--extraction-refinement --construction-refinement --compare-resolution \
--household-diagnostics
```

Outputs are ignored `output/nutrition-age-personal.json` and
`output/nutrition-age-cohort.json`; corresponding logs remain local.
Both ensembles were launched after the diagnostic example built successfully and
passed Clippy with warnings denied. The completed results and follow-ups are below; no defaults have changed.

## Independent follow-up control

The runner also accepts `--no-household-mortality`, which retains household retail
and the personal hunger work penalty but uses age-band mortality. It conflicts
with `--no-individual-nutrition` to keep experimental intent explicit.
`HouseholdEconomy.household_mortality` is persisted and defaults true for existing
archives; the original combined nutrition switch remains the master switch.
Neither control changes default behavior. This allows a follow-up to distinguish
redistributed mortality exposure from the work/income feedback.

Verification: all three nutrition tests pass, including the GPU funded-food
fixture. The mortality-only control suppresses personal mortality entries while
preserving the measured hunger work factor; its disabled state survives archive
round-trip and older archives default to enabled. The fixture retains food-wallet
and exposure checks. All-target Clippy passes with warnings denied.

## Seed 409 century comparison

The runs use diagnostic commit `574abd5`; subsequent mortality-only controls do
not alter the already-running processes. Both seed-409 centuries completed first; the completed seed-1024 pair is
reported below.

| Measure at 100 years | Personal nutrition | Combined nutrition ablation |
|---|---:|---:|
| Residents | 735 | 568 |
| Children / adults / elders | 201 / 429 / 105 | 175 / 314 / 79 |
| Committed demographic births | 3,513 | 3,216 |
| Committed demographic deaths | 4,698 | 4,568 |
| Cumulative physical food gap | 0% | 0% |
| Cumulative access gap | 3.705% | 5.001% |
| Maximum monthly population residual | 0 | 0 |

Both retain sixteen sites. Food residuals stay below 4e-7. Personal nutrition
reproduces the preceding construction run's 735 residents, supporting that the
added diagnostics did not change this trajectory. Neither trajectory stabilizes.

Within the personal run's completed snapshots, summed expected deaths from
age-band exposure are 1,321.93 children, 1,505.71 adults and 1,221.33 elders.
Substituting actual household exposure yields 1,951.86, 1,249.96 and 1,504.01.
Thus unequal access shifts conditional mortality toward dependents and away from
adults. These are same-snapshot calculations; the separate ablation world has
already diverged and cannot isolate that immediate mediator on its own.

At the final observation, 37 of 227 households with food needs have more than
10% unmet need. Together they have 2,184 kg monthly need, about 479.62 currency
units of current sector wages, and only 0.0021 currency units left after retail.
This is an endpoint observation, not proof that these particular households were
hungry throughout the century. It supports investigating dependent-household
entitlements and income rather than increasing physical food abundance.

The combined ablation worsens the endpoint and access gap. Disabling personal
nutrition is therefore not supported as a repair. Mortality-only and bounded
common-entitlement comparisons are the next useful controls; current defaults
remain unchanged pending the food-access comparison and further evidence.

## Follow-up comparisons

A 30-year seed-409 run using `d659261` with `--no-household-mortality` completed
against the existing 30-year samples of the two century histories.
The default-enabled new control changes no other behavior. Its local output is
`output/nutrition-age-mortality-only.json`.

The entitlement comparison now runs seeds 409/1024 for 30 years with the existing
`--common-share 0.65` control, retaining crop yield 0.5 and all personal
production/nutrition systems. Its local output is `output/nutrition-access-65.json`.
This changes the distribution of existing food, not the ecological or production
inputs. The 0.65 value is a game-balance probe between the current 0.5 and the
previous full-common-food ablation, not a proposed default or historical estimate.


| Seed 409 at 30 years | Personal nutrition | Both effects disabled | Mortality only disabled |
|---|---:|---:|---:|
| Residents | 1,609 | 1,529 | 1,438 |
| Children / adults / elders | 463 / 833 / 313 | 462 / 788 / 279 | 424 / 748 / 266 |
| Cumulative food access gap | 3.325% | 4.469% | 4.785% |
| Physical food gap | 0% | 0% | 0% |

All three have zero monthly population residual and food residual below 2.6e-7
at this horizon. Removing the work penalty improves the mortality-ablation
endpoint by 91 residents, but the full personal model still performs best in
this seed. These interacting trajectories do not justify replacing household
mortality or removing work feedback as the population repair. The next comparison
therefore targets access to existing food. Longer and additional-seed evidence
remains necessary; no default has changed.


## Completed second century pair: seed 1024

Both ensemble reports now declare complete.

| Measure at 100 years | Personal nutrition | Combined nutrition ablation |
|---|---:|---:|
| Residents | 615 | 452 |
| Children / adults / elders | 172 / 345 / 98 | 137 / 252 / 63 |
| Committed births / deaths | 3,167 / 4,471 | 2,830 / 4,298 |
| Cumulative physical food gap | 0.0666% | 0.0602% |
| Cumulative access gap | 3.752% | 5.106% |

Both retain sixteen sites and zero monthly population residual; maximum food
residual remains below 3.1e-7. Personal nutrition reproduces the earlier
construction century's 615 residents. Its age-band expected deaths are
1,218.44 / 1,405.65 / 1,210.52; household exposure changes these to
1,785.10 / 1,155.34 / 1,516.87. The dependent/adult exposure difference repeats.

Across these two seeds, removing the combined nutrition effects worsens both
remaining population and food access. This is evidence against that proposed
repair, not evidence that the personal model is well balanced. The continuing
0.65 common-share comparison tests an actual entitlement adjustment; population
stability and the wider integration worklist remain open.


## Completed 0.65 entitlement probe, 30 years

Both seeds completed using `d659261`. The intervention changes only the configured
long-run common share, retaining the founding transition and finite food stocks.

| Seed | Population, 0.5 → 0.65 | Access gap %, 0.5 → 0.65 | Physical gap %, 0.5 → 0.65 |
|---|---:|---:|---:|
| 409 | 1,609 → 2,018 | 3.325 → 1.936 | 0 → 0 |
| 1024 | 1,529 → 1,926 | 3.173 → 2.042 | 0.1489 → 0.1525 |

All retain sixteen sites. Maximum population residual is zero; food residuals
stay below 2.2e-7. The intervention does not increase crop potential, yet materially
improves access and population. Physical shortages remain nearly unchanged.

This is not stability: 0.65 populations fall from 2,083/2,038 at year 10 to
2,018/1,926 at year 30. A 100-year extension on the same seeds is now running in
`output/nutrition-access-65-century.json`; it must reproduce the first 30 years
before its later outcomes are used. No default share has changed.

## Reproducing matched tables

```sh
python3 scripts/compare_food_access.py output/nutrition-age-personal.json \
  output/nutrition-age-cohort.json --allow-difference individual_nutrition
python3 -m unittest discover -s scripts -p test_compare_food_access.py
```

The comparison command requires complete monthly reports, matching seeds and
explicitly acknowledged metadata differences; it checks endpoint horizons, finite
values and the food-gap partition. It does not infer causality or accept a partial
run as completed. Two focused tests cover known percentages and rejection of
partial, duplicate, missing, nonfinite and inconsistent results. The command
reproduces both completed nutrition-century rows above.
