# Family support: century and held-out comparison

This evaluation extends the earlier [thirty-year family-support pilot](household-family-support.md).
Seeds 256 and 409 were not in that pilot's two-seed comparison (17 and 81); they
are held out for this intervention, not previously unseen by all project tests.
The aim is to distinguish sustained purchasing access from physical production
and to check whether the late population trajectory stabilizes.

Both arms use simulation commit `0a6a03d`, terrain/ecology resolution 32/16, one
epoch, sixteen founders and 100 years of living history. Individual demography,
workshop, agriculture, extraction and construction participation, comparison
receipts and monthly household diagnostics are enabled. Crop yield scale is 0.5;
common-access, nutrition, council relief, catalogs and production settings match.
Only `--family-support` differs. Production parameters are fixed, not physical
output: changed workers and demand can affect later production endogenously.
Both arms include the recently committed road-capacity and informal-learning
changes, so their results must not be substituted into the old thirty-year table.

The two processes run concurrently on the same Quadro RTX 5000/Vulkan backend.
Elapsed times are not estimates of isolated policy overhead. Generated reports
and logs stay under ignored `output/`; no raw results are committed.

```sh
cargo build --example cultural_work_calibrate
target/debug/examples/cultural_work_calibrate \
  --seeds 256,409 --years 100 --resolution 32 --crop-yield-scale 0.5 \
  --individual-demography --workshop-refinement --agriculture-refinement \
  --extraction-refinement --construction-refinement --compare-resolution \
  --household-diagnostics --output output/family-century-base.json
```

Repeat with `--family-support --output output/family-century-support.json`.

```sh
python3 scripts/compare_food_access.py output/family-century-base.json \
  output/family-century-support.json --allow-difference family_support
```

## Interval diagnostics

The comparison script now reports both cumulative outcomes and the latest observed
interval. It subtracts actual cumulative observations at the two recorded dates,
never an invented zero baseline. Negative cumulative increments, inconsistent food
gap partitions and mismatched interval dates are rejected. A single observation
has no interval report; zero demand has an undefined gap percentage, not a claim
of complete food security. Four Python checks cover exact partitions, hidden late
decline, invalid reports and zero-demand cases.

## Results

All four century histories complete. Both reports pass the strict comparison:
only `family_support` differs in metadata, monthly food partitions balance, and
both declared seeds reach year 100. All sixteen sites remain active in every arm.

| Seed | Population control → support | Cumulative purchasing gap, % need | Physical gap, % need | Family cash received | Recipient households | Household council relief control → support |
|---|---:|---:|---:|---:|---:|---:|
| 256 | 708 → 1,338 | 3.6217 → 2.9343 | 0 → 0 | 246,843.31 | 235 | 18,211.79 → 15,550.31 |
| 409 | 712 → 1,322 | 3.7086 → 3.0461 | 0 → 0 | 267,681.77 | 234 | 17,596.43 → 17,368.76 |

Cash received is counted once; sent and received totals differ by less than
3e-10. Maximum monthly population residual is zero and maximum food residual
is below 4.4e-7. Maximum absolute final economy residual is below 1.2e-5.
Canonical rosters retain no unresolved population discrepancy. This verifies
accounting, not desirable population behavior.

| Seed | Years | Population change control → support | Interval purchasing gap, % need |
|---|---|---:|---:|
| 256 | 90–100 | −31 → −24 | 2.9927 → 2.9330 |
| 409 | 90–100 | −119 → −40 | 4.7365 → 3.2622 |

Family support improves retained population in both held-out seeds, but neither
support arm stabilizes during the final decade. No physical food shortage occurs
in any arm. This comparison supports purchasing access as a contributor to the
population loss; it does not imply that all of the improvement comes from one
immediate mechanism after a century of feedbacks. The earlier controlled fixture
isolates the direct cash-to-funded-food effect.

## What still needs explanation

Code inspection identifies several coupled demographic responses. In
`individual_demography::DemographicProjection::from_exposure` and household personal mortality,
monthly baseline death risks are augmented by food shortage times age-specific
coefficients `[0.06, 0.025, 0.05]`. A persistent 0.05 household shortage therefore
adds 0.003 to monthly child mortality before disease. These are toy coefficients,
not physiological estimates. Hunger also lowers subsequent personal work capacity.
Expected births are reduced by adult hunger and disease, then refined using the
represented share of adults aged 18–44.

The current comparison does not isolate those responses from one another. A next
population experiment should independently vary hunger-related mortality, disease
and birth-age refinement while holding food-access policy fixed. Improving access
by itself has not closed the long-run decline gate. Do not increase crop yield to
address the measured purchasing gap, or claim the existing mortality coefficients
are vindicated by conservation tests. Defaults remain unchanged; family support
stays opt-in.

Four Python comparison tests pass. This increment changes report analysis and
records completed experiments; it makes no Rust simulation change.

A subsequent [political distribution comparison](distribution-policy-century.md) retains family support in both arms and reverses final-decade decline in these seeds by allowing council distribution shares to change. It also exposes thinner council reserves; this does not invalidate the fixed-policy results above.
