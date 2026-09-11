# Household nutrition: scarcity comparison

This is a toy-game balance and integration check, not a physiological or historical
calibration. The matched control keeps individual residents and family-weighted
retail, but disables household mortality exposure and the personal hunger work
penalty. Both sides keep the same starting seed, crop yield, food accounting,
municipal income policy and single demographic authority.

## Protocol

- Seeds 17, 81 and 256; yields 0.33 and 0.15; nutrition feedback off/on.
- Fifty years sampled monthly, five requested civilizations, terrain/ecology 32.
- One geological epoch then frozen environment; society, politics, governance,
  offices and shipping enabled. Individual demography and observational comparison
  enabled. Individual workshop refinement remains off, so these runs chiefly test
  mortality and the smaller named-service workforce, not a fully personal economy.
- Common food share 0.5 and other household policy defaults retained.
- Quadro RTX 5000 Max-Q, Vulkan; other builds/tests overlapped. Times are not
  performance benchmarks. No generated trajectories are committed.

Reproduce with:

```sh
cargo run --example nutrition_evaluate -- --years 50 --output output/nutrition-final-fixed.jsonl
python3 scripts/summarize_nutrition.py output/nutrition-final-fixed.jsonl
```

The summarizer rejects incomplete paired suites and missing monthly samples.
Hunger is weighted by account need within each month; work is abstract personal
capacity, not the entire settlement workforce. Averages include later empty-world
months (zero hunger and work), so low average hunger can coexist with collapse.
Use the monthly trajectories to distinguish recovery from disappearance.

## Failure found and corrected

The preliminary twenty-year suite stopped in seed 256, yield 0.33, feedback on,
at month 47. Workshop 6 at site 3 reported -1.1175871e-8 completed work against
0.049999997 committed work. The validator correctly rejected the negative value.
This was exhausted floating-point labor leaking into a negative recipe batch,
not an overproductive household or an insufficiently lenient validator.

The GPU now bounds final recipe batches and remaining recipe labor at zero before
allowing further transfers. It does not clamp the reported outcome afterwards.
A 48-month regression repeats the actual failing world and checks nonnegative
production/consumption counters, workshop execution, and population accounting.
Firm-level diagnostics now identify work, allowance, date and closure state.
The initial fifty-year attempt was stopped to rerun against this correction.

## Interpretation limits

Opening household shortage is about 47% in both yield scenarios: purchasing power
is already restrictive before later harvest differences accumulate. These are
joint food-access and production pressures. The first-month household mortality
adjustment is tiny; larger population differences develop through subsequent
household allocation, surviving identities and economic feedback. Final population
alone cannot identify which pathway dominated.

The 35% maximum personal work penalty and mortality coefficients were not retuned.
A future experiment should vary common entitlements and household payroll
separately, then add living ecology and individual workshop operation. Increasing
food supply indiscriminately would obscure the distinction between unavailable
food and food households cannot afford.

## Completed fifty-year results

All twelve corrected runs completed (7,200 simulated months). Maximum absolute
population residual was zero; maximum relative food residual was 1.153e-6.

| Seed | Yield | Final population, control | Final population, nutrition | Mean hunger, control | Mean hunger, nutrition |
|---|---:|---:|---:|---:|---:|
| 17 | 0.33 | 74 | 180 | 0.106 | 0.068 |
| 81 | 0.33 | 52 | 188 | 0.121 | 0.066 |
| 256 | 0.33 | 65 | 219 | 0.120 | 0.061 |
| 17 | 0.15 | 0 | 1 | 0.269 | 0.335 |
| 81 | 0.15 | 0 | 0 | 0.252 | 0.372 |
| 256 | 0.15 | 1 | 1 | 0.417 | 0.420 |

Nutrition-enabled runs at yield 0.33 have higher mean personal capacity because
more people survive, despite the per-person hunger penalty. Actual committed named
work remains small (about 0.75–0.94 units/month in those runs), so these results
must not be presented as evidence of a fully individual labor economy.

Personal capacity first differs in month 2 in all six pairs, matching the previous-
month observation contract. Population first differs in months 6/13/8 at yield
0.33 and 6/10/8 at yield 0.15 for seeds 17/81/256. The first-month adjustment to
summed expected deaths is only -0.001179 in each world, and initial household
hunger is identical between controls. This separates immediate input changes from
later path-dependent population differences.

The low-yield regime produces collapse under both interpretations. At moderate
yield, replacing uniform age-band exposure with family exposure substantially
changes long-term survival. That is a meaningful behavioral difference, not
proof that either model is more realistic. The next balance question is who can
access existing food, rather than automatically increasing the amount produced.

## Verification

Eight household tests passed with hardware cases enabled, including the original
failing seed and the ablation checks. All 23 economy integration tests passed,
including the existing seasonal agriculture/adaptive-price seed comparison,
resource ledgers and checkpoint cases. The ordinary library suite passed 106 tests
(95 hardware tests ignored by that command). Clippy with warnings denied and the
repository artifact check passed. These checks support implementation and bounded
accounting; this small, low-resolution frozen ensemble does not establish balance
for larger or living worlds.
