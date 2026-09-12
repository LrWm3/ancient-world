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
