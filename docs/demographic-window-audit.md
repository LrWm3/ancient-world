# Demographic exposure windows

`--demographic-audit=true` observes aggregate demographic execution without changing
allocation, production or health parameters. It stores up to 200 annual global
rows in the history export. `false` removes the audit; omission preserves an
archived audit. Old histories have none. Enabling mid-history starts observation
at that boundary; it cannot recover earlier causes. Individual demography and the aggregate/individual resolution framework are not
supported by this observer: both defer the actual demographic commit beyond the
GPU boundary.

## Boundary and interpretation

The observer reads existing town buffers immediately after GPU production readback,
before domestic care, demographic reconciliation, remedies, travel and response.
It uses opening cumulative birth/death counters to measure actual GPU changes.
Towns with zero opening population are excluded because their shader invocation
returns without refreshing demographic exposure. No additional GPU readback is
introduced. Execution and random streams do not depend on these observations.

Age-specific person-month exposure is recovered from that dispatch's ration need,
using the same age-specific ration units. This is preferable to using endpoint
population after births, deaths and migration. Rows also accumulate:

- Baseline, nutritional and illness mortality contributions from the model's
  completed exposure, alongside actual GPU birth/death counter deltas.
- Potential births at the current adult count; hunger suppression first, followed
  by illness suppression on the remaining births.
- Physical food deficits and additional access deficits, in kg calorie equivalent.
  Physical deficit is the shortfall between need and food available before
  consumption; additional unmet ration need is classified as access exclusion.

The birth decomposition is ordered arithmetic, not a claim that hunger and illness
are independent causes. Illness includes multiple upstream causes. Nutritional
stress uses stored memory plus the acute starvation response when enabled.
The current maximum sum of mortality rates is below the shader's mortality cap;
revisit decomposition if those constants change. Shared demographic constants are
used by the observer, so it is not an independent verification of their correctness.
Analytical fixtures and the difference from observed GPU counter deltas provide
separate numerical checks.

These are GPU demographic observations, not all historical deaths. War, travel,
disasters and later individual-resolution commits are outside this boundary.
Do not subtract these deaths from an endpoint population and call any difference
an error without accounting for those other transitions. Food access attribution
is contemporaneous: it does not establish why a town has insufficient stocks,
or whether changing that month's transfers will improve a century-long outcome.

Annual rows retain their actual number of observed months; the first or final row
may be partial. Duplicate/older-month calls do not append another observation.
Policy serialization preserves continuation. A switch into individual or resolution-framework mode through the API pauses this
observer rather than reporting deferred commits as zero mortality. The CLI rejects
explicitly enabling it in either mode.

## Commands

Run the same founding archive and settings in each arm, adding
`--demographic-audit=true`. For the current screen, use the command in
[food solidarity](food-solidarity.md), change history duration to 20 years, and
compare `--food-solidarity=false` with `true` for seeds 1024, 256 and 409.
Then summarize retained years:

```sh
python3 scripts/report_demographic_windows.py \
  output/demographic-window-screen/1024-off.json \
  output/demographic-window-screen/1024-on.json --years 1 5 20
PYTHONPATH=scripts python3 -m unittest scripts/test_report_demographic_windows.py
CARGO_INCREMENTAL=0 cargo test --lib
```

Raw exports, commands and logs stay under ignored `output/`. The reporter prints
structured local results and explicitly reports reconstruction differences rather
than silently relabeling projections as observed losses.

## Three-seed early-history comparison

Six 20-year runs used the same founding archives and settings as the prior
solidarity experiment; three controls and three contribution-enabled arms. Each
started with 600 people and five towns. The first-year result is effectively the
same in all arms: 17.30 births, 6.01 demographic deaths, no material food shortage.
The populations initially grow.

By five years, solidarity still has no material effect on these measurements:

| Seed | Births | Demographic deaths | Baseline contribution | Nutrition contribution | Illness contribution | Physical deficit / dietary need | Additional access deficit |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1024 | 63.97 | 214.63 | 27.23 | 174.88 | 12.52 | 15.38% | <0.001% |
| 256 | 74.60 | 130.42 | 29.36 | 93.51 | 7.55 | 7.68% | <0.001% |
| 409 | 74.84 | 130.99 | 29.52 | 94.17 | 7.30 | 7.86% | <0.001% |

Nutritional mortality supplies about 81%, 72% and 72% of the modeled early deaths.
The first-year lack of shortage does not persist. Physical deficits in years two
and three are respectively 17.33% and 29.17% of dietary need in seed 1024; 8.91%
and 19.95% in seed 256; 6.49% and 20.00% in seed 409. These are integrated
consumption-boundary shortages, not endpoint warehouse measurements.

| Seed | Solidarity | Population at year 20 | Births through year 20 | GPU demographic deaths | Physical deficit / need | Additional access deficit / need |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 1024 | Off | 321.81 | 204.36 | 482.56 | 10.894% | 0.082% |
| 1024 | On | 320.68 | 204.39 | 483.71 | 10.897% | 0.013% |
| 256 | Off | 491.38 | 270.33 | 378.95 | 5.774% | 0.306% |
| 256 | On | 491.63 | 270.48 | 378.85 | 5.804% | 0.171% |
| 409 | Off | 496.50 | 271.93 | 375.43 | 5.762% | 0.172% |
| 409 | On | 497.11 | 272.28 | 375.17 | 5.753% | 0.023% |

The controls' adult exposure fractions are 57–60% over the first twenty
years. At those same age exposures, potential births exceed baseline deaths:
238.37 versus 91.70 (1024), with nutritional and illness costs accounting for the
negative realized natural balance. This is a fixed-exposure comparison, not a
forecast of what a healthy population would become. Its age distribution and
resource demand would themselves change.

The contributions reduce cumulative access exclusion but do not remove the much
larger physical deficits. This explains why the affordability intervention alone
cannot prevent the initial collapse. It does not establish that total annual
harvest is too small: poorly timed harvests, spoilage, stores, geography or delivery
constraints could also leave food unavailable when needed. That distinction is
the next useful investigation. Avoid raising fertility or minting money in response
to an early physical shortage without tracing those quantities first.

### Verification

The final ordinary library suite passed 206 tests, with 156 extended/hardware
fixtures ignored. Two reporter tests check window selection, arithmetic
reconciliation and missing-data handling. All six native runs completed existing
monthly validation. Across years 1, 5 and 20, reconstructed birth totals differ
from GPU cumulative-counter deltas by less than 0.000056 people; reconstructed
deaths differ by less than 0.000093. The small discrepancy is retained in reports,
not repaired by modifying outcomes. These checks do not demonstrate exact
individual-mode compatibility or cross-hardware numerical equivalence.

Final-build observation control: seed 1024 was rerun for twenty years with the
audit disabled and enabled. Removing only the diagnostic field yields identical
full history exports. The final enabled export also exactly matches the earlier
screen. This supports the claim that the observer leaves this trajectory unchanged;
it is not a full-world checkpoint or cross-backend equivalence test.

Strict all-target Clippy also passes. Its first run identified an integration-test
`History` constructor missing the new optional field; the market fixture now
explicitly disables the audit. This was a fixture update, not a relaxed check or
an economic behavior change.
The market integration target passes 19 ordinary tests, with two extended tests
ignored. The combined Python reporter/circulation suite passes all 12 tests.

Follow-up: [early food supply and staged harbors](early-food-and-staged-harbors.md)
separates the aggregate supply gap from inaccessible town reserves and tests opening
small, materially funded harbors before full construction is complete.
