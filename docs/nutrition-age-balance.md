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
passed Clippy with warnings denied. Results are pending; no defaults have changed.

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

## First completed pair: seed 409 (ensemble still running)

The runs use diagnostic commit `574abd5`; subsequent mortality-only controls do
not alter the already-running processes. Both seed-409 centuries completed;
seed 1024 is pending in each ensemble. Do not interpret this as a completed
held-out ensemble.

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
remain unchanged pending the second seed and further evidence.
