# Seasonal crops, adaptive quotes and network trade revisit

This is a game-behavior regression study, not an agronomic or historical calibration.
The earlier failures were reproduced before changing code: all six seasonal cases
across seeds 17, 81 and 256 lost their populations within 50 years. Nonseasonal
adaptive wheat quotes ended at 706, 1,913 and 936 currency/kg. Networked trade and
shipping were already enabled in these cases.

## Corrections

The terrain survey supplies an annual **harvest** potential in kg/ha/year.
The seasonal kernel had treated one twelfth of that value as monthly whole-crop
biomass, multiplied it by a canopy calendar totaling only 4.1, then deducted
35% as residue. Before other limits, this retained only 22% of the annual budget.

Seasonal growth now distributes the annual harvest potential across the calendar
with `12 / 4.1`, and converts harvest potential to required whole-crop biomass using
`1 / harvest_index`. Residues still return to detritus; growing them consumes N/P,
water and the declared atmospheric carbon input. There is no nutrient refill, food
gift, survival floor or change to founding provisions. Higher resource demand can
prevent a low-harvest-index crop from reaching its potential. The canopy schedule,
seed abstraction, crop coefficients and annual potential remain game assumptions.

An intermediate test using calendar normalization alone retained eight towns and
886 people on seed 17 after 50 years (724 with adaptive prices), compared with zero
before. The harvest/biomass correction was then applied on units grounds, rather
than fitting a new yield multiplier to a desired population.

Adaptive prices now approach a supported target instead of compounding the same
scarcity pressure indefinitely:

- Independent replacement costs anchor producible goods. Primary goods without a
  modeled production cost use the catalog's nominal reference price.
- Scarcity shifts that anchor by a bounded log premium/discount.
- Paid delivery observations contribute 30% of the log target.
- Treasury and resident-household cash are shared across stock shortfalls rather
  than counted in full for every good. Reserved catalog slots receive no quote budget.
- Monthly adjustment remains rate-limited. Abandoned towns stop changing quotes.
- Holding an item no longer makes its own asking price an independent production
  cost estimate. This also removes that unsupported estimate from export-contract
  profitability checks, including with legacy prices.

A unit fixture holds scarcity constant for 1,200 months and checks convergence;
another starts with a 10,000-unit inherited quote and checks recovery. Actual recipe
input costs still influence the tool quote. Prices remain heuristic offers, not
market-clearing prices; wallets are stocks rather than measured expenditure flows.
The nominal anchor is an intentional stabilization assumption.

Networked trade needed no routing rewrite for these failures. Existing multi-hop,
hostile-transit, closure, payment and capacity fixtures passed. The multi-hop shipment
fixture now explicitly enables adaptive pricing and still checks a real paid delivery,
single reservation and conservation. The paired worlds also record actual market sales.

## Evaluation setup

NVIDIA Quadro RTX 5000 Max-Q, Vulkan; Rust 1.89 optimized test profile.
Terrain 64, ecology 32, one geological epoch with one ecological year,
eight founding groups (960 people), society, politics, governance and shipping.
Networked trade stays enabled. Default yield scale is 0.5.
The main factorial compares seasonal/nonseasonal crops and adaptive/legacy prices
for 600 monthly history steps with the planetary environment fixed.

Two held-out seeds, 409 and 1024, use the combined changes and a living environment
for 50 years. Longer combined checks use seeds 17 and 409 for 200 years with fixed
planetary conditions. These do not exercise every optional social system or GPU
backend. Small changes in legacy-price outcomes are possible because the unsupported
export cost fallback was corrected too.

### Results

Results below report simulated people, not a historical population target.
“Occupied” does not imply freedom from shortages. Wheat quotes are local offers;
they are not a price index or necessarily realized transactions.

| Seed | Seasonal | Adaptive | People at year 50 | Occupied towns | First-town wheat quote | Market sales |
|---|---|---|---:|---:|---:|---:|
| 17 | false | false | 1309.8 | 8 | 8.000 | 2,628 |
| 17 | true | false | 1297.0 | 8 | 8.000 | 2,808 |
| 17 | false | true | 961.4 | 8 | 3.193 | 1,515 |
| 17 | true | true | 904.7 | 8 | 3.192 | 1,507 |
| 81 | false | false | 1406.8 | 8 | 0.800 | 4,058 |
| 81 | true | false | 1404.7 | 8 | 0.800 | 3,068 |
| 81 | false | true | 1009.6 | 8 | 3.747 | 3,433 |
| 81 | true | true | 1001.6 | 8 | 3.432 | 1,986 |
| 256 | false | false | 973.7 | 8 | 8.000 | 2,302 |
| 256 | true | false | 902.4 | 8 | 8.000 | 1,574 |
| 256 | false | true | 713.2 | 8 | 3.192 | 1,359 |
| 256 | true | true | 645.8 | 8 | 3.192 | 961 |

With both changes, active wheat quotes across all towns in these three worlds
range from 2.54 to 4.45. No world loses all settlements.
The maximum absolute normalized economy residual is 1.37e-5, below the existing
0.001 tolerance.

| Additional run | People | Occupied towns | Active wheat quote range | Market sales |
|---|---:|---:|---:|---:|
| Seed 409, living environment, 50 years | 823.3 | 9 | 2.17–4.14 | 3,635 |
| Seed 1024, living environment, 50 years | 773.9 | 6 | 2.48–3.87 | 2,148 |
| Seed 17, fixed environment, 200 years | 322.5 | 8 | 3.19–4.08 | 2,810 |
| Seed 409, fixed environment, 200 years | 688.7 | 7 | 1.72–4.23 | 9,255 |

The long runs remain viable but show substantial population decline, especially
seed 17. Fixing annual yield units and quote drift does not establish a stable
long-run demographic equilibrium or eliminate poverty, shortages, depletion or
town failure. Maximum normalized residual across the long checks is 2.36e-5.
No claim is made about 500-year outcomes or other GPU backends.

## Verification and reproduction

The seed fixture now explicitly fails if no occupied settlement retains at least
20 people, or if an adaptive active-town wheat quote reaches 100 currency/kg.
These deliberately broad game-regression thresholds catch the reported failures;
they are not empirical calibration ranges and do not require every town to survive.

All 23 economy integration tests passed, including the full 12-case factorial
comparison (296.77 s with other checks running concurrently). The final combined
three-seed acceptance rerun passed after adding explicit viability and price bounds.
Clippy passed for all targets with warnings denied.

The library suite passed 58 tests (54 extended/hardware tests excluded in that
command). Both adaptive-price fixtures passed, including actual GPU history quotes.
All eight market tests passed; the changed adaptive multi-hop fixture passed
separately. All six export-contract tests passed after removing the unsupported
cost fallback. The seasonal fixture checks stored-seed dormancy, full-season
accounting, checkpoint continuation and batch/monthly equivalence.

Run from the repository root, with generated logs under ignored `output/`:

```sh
mkdir -p output
mise exec rust@1.89.0 -- cargo test --test economy -- --include-ignored --test-threads=1 --nocapture > output/crop-price-suite.log 2>&1
mise exec rust@1.89.0 -- cargo test --test markets -- --include-ignored --test-threads=1
mise exec rust@1.89.0 -- cargo test --lib adaptive_price_tests -- --include-ignored
mise exec rust@1.89.0 -- cargo test --lib export_contracts -- --include-ignored
CROP_PRICE_SEEDS=409,1024 CROP_PRICE_COMBINED_ONLY=1 CROP_PRICE_LIVING=1 mise exec rust@1.89.0 -- cargo test --test economy seasonal_crops_and_adaptive_prices_seed_comparison -- --ignored --nocapture
CROP_PRICE_SEEDS=17,409 CROP_PRICE_MONTHS=2400 CROP_PRICE_COMBINED_ONLY=1 mise exec rust@1.89.0 -- cargo test --test economy seasonal_crops_and_adaptive_prices_seed_comparison -- --ignored --nocapture
```

`CROP_PRICE_MONTHLY=1` prints the first 36 months and five-year observations for
diagnosis. All numeric runs are source-reproducible; raw logs are not committed.
Seasonal selection remains explicit through the agriculture catalog described in
[crop and price experiments](crop-and-price-experiments.md). Existing saves retain
their inventories and catalog choices; resuming uses the corrected equations rather
than reproducing the old bugs bit-for-bit.
