# Diagnosing the growth reversal

Annual ending inventory and household gaps cannot explain seasonal shortages.
`growth_ladder --food-diagnostics` retains monthly, per-site boundary evidence
without changing allocation or balance.

## Reproduction

```sh
cargo build --release --example growth_ladder
target/release/examples/growth_ladder --seeds 17 --modes baseline,combined --gates 50 --food-diagnostics --output output/growth-food
python3 scripts/summarize_growth_food.py output/growth-food/seed-17/combined/food-monthly.jsonl > output/growth-food/combined-food-summary.json
cargo test --lib demographic_audit
cargo test --test food_diagnostics -- --ignored
```

For an exact comparison, copy the earlier suite's `seed-17/base.world` into the
new output directory before running. Otherwise a fresh base is generated. Never
overwrite the earlier suite. The diagnostic runner advances monthly, keeps the
annual output, and truncates uncheckpointed monthly rows on resume. Checkpoints
remain stage gates.

## Observation boundaries

The food audit now covers individual/aggregate resolution modes as well as legacy
demography. Previously its production trace was restricted to legacy demography.
Its old annual demographic expectation counters remain legacy-only; they do not
attribute individual deaths.

| Observation | Meaning |
| --- | --- |
| Opening food | Before Open deliveries and relief |
| Before/after GPU food and ledgers | Reserve-completed versus production/consumption-completed stock |
| Closing food | After Respond, including new shipments and other historical actions |
| Need, available food, funded entitlement, eaten | Actual consumption boundary, before demographic losses |
| Crop demand/supply | GPU N/P kg and water m³ before shared crop uptake; requested and actual crop growth kg |
| Farm workers, cultivated area, land capacity, tools | Dispatch allowances; zero named-worker requests may mean aggregate labor is active |
| Household cash, allocation, price, town finance | Closing observations, not opening purchasing power |
| Illness, waterworks | Exposure/infrastructure observations, not causes assigned to each death |

For each town-month, split unmet food into physical shortfall
`min(unmet, max(need - available, 0))`, then access shortfall `unmet - physical`.
They sum to unmet food. This is an ordered accounting decomposition, not independent
intervention effects: removing both constraints may be necessary. Food quantities
use the model's edible-equivalent kg.

Compare local physical shortages with other towns' simultaneous surpluses. Their
minimum bounds what instantaneous, costless redistribution could fix. The remainder
cannot be fixed by moving that month's available stock among these towns. Neither
figure asserts that real routes can deliver that amount in time. Transit cargo
and future harvests are not available food at this boundary.

Opening/response net stock changes expose flows outside production, but do not
classify every transfer: relief, trade, voyages, military consumption and relocation
can contribute. These are not independent conservation residuals.

Probes add 64 bytes per settlement to the existing GPU economy layout and use its
existing town readback, with no planetary snapshot. Old archive metadata defaults
them to zero. They are cleared for non-producing towns and never enter decisions.
The rolling archive trace retains 60 months; the runner streams all months under
ignored `output/`.

The phosphorus probe additionally records actual runoff export, geological
release, detrital mineralization and consumed-food returns, all in kg P. Market
request counters are cumulative observations; take differences between months.
Their nine constraints are: no eligible surplus, no usable route, no free freight,
then need, storage, freight, seller surplus, purchase batch ceiling, and money.
Requests can repeat the same unsatisfied need; their sum is not unique food demand.

## Diagnostic controls

Keep seed, base archive and mode fixed and use separate output directories:

- `--needs-based-food`: remove the household purchasing requirement for existing
  local food. It supplies no new food and does not eliminate inter-town barriers.
- `--phosphorus-release 0.000001`: increase the existing finite geological source
  release from its default `0.0000001` per month (tenfold). Soil additions debit
  the geological reserve. This tests a mechanism, not a recommended balance value.

These controls persist in checkpoints and the experiment specification. Defaults
are unchanged. Long-run feedback can alter wages, migration, nutrient returns and
other outcomes, so interpret the immediate food/crop mediators before population.

## Verification

An analytical fixture separates physical, access and mixed shortages. A hardware
fixture compares observed monthly steps with an unobserved batch across a
checkpoint, checks individual-mode food boundaries, and compares GPU fulfillment
against independently calculated resource ratios.

## Results — 2026-09-14

Seed 17, the original long-suite base archive, terrain/ecology 256, one geological
epoch, 16 founding groups and complete individual demography. Quadro RTX 5000
with Max-Q Design. Four 50-year arms; defaults were not retuned.

| Arm | Year-50 population | Town records | Trailing 20-year slope |
| --- | ---: | ---: | ---: |
| Baseline replay | 1,406 | 16 | −13.55/year |
| Combined replay | 2,134 | 16 | −44.38/year |
| Combined + needs-based local food | 2,122 | 16 | −49.06/year |
| Combined + tenfold finite P release | 3,828 | 22 | +38.85/year |

Every annual field in both replays matches the original suite through year 50.
Thus switching from annual batches to instrumented monthly advancement did not
alter the previously measured trajectories. The final phosphorus-flux probe was
added before the two interventions; the replay files lack that extra probe.
Absent flux observations must not be interpreted as zero.

### Different constraints at different stages

The baseline has **zero physical food shortage** in years 10, 20 and 30, but
12,047, 11,502 and 12,877 edible-equivalent kg/year of access shortfall. Early
decline despite edible stocks is therefore materially different from the later
shortage after improved access allows more people to survive.

At year 40:

| Measurement | Combined | + needs-based food | + finite P release |
| --- | ---: | ---: | ---: |
| Physical food shortfall, kg/year | 39,420 | 43,901 | 0 |
| Access shortfall, kg/year | 1,476 | 0 | 2,464 |
| Managed crop growth / requested growth | 57.19% | 58.65% | 100% |
| P-constrained town-months | 180/192 | 180/192 | 0 |

Nitrogen and water do not bind crop growth in these year-40 comparisons. In the
combined replay no town reaches its land ceiling at years 20, 40 or 50; average
cultivation is about 62% of available managed land in year 40. Tool multipliers
average 0.869 then. Labor allocation and tools affect potential, but neither a
land cap nor a missing-water constraint explains the recorded 43% loss *after*
crop growth requests are calculated. This run uses aggregate farm allowances;
complete demographic rosters do not imply that named farming is enabled.

Food production here means conversion into edible-equivalent stock, not raw crop
harvest or standing biomass. Combined year-40 edible production is 456,878 kg,
against 486,588 kg of need; stored-food spoilage adds 14,521 kg of loss. The crop
probe independently identifies phosphorus as the binding resource upstream.

### Why the phosphorus constraint develops

The initial available-soil inventory is not a sustainable monthly source. In the
needs-based control, measured runoff exports 205,548 kg P in year 1, 17,909 kg in
year 5 and 734 kg in year 10. By year 20 the soil is depleted enough that runoff
exports only 4 kg P/year. Geological release is about 125 kg/year. Detrital
recycling continues, but is insufficient to fulfill crop demand. These runoff
exports leave managed plots through the existing ecological return path; they
are not missing ledger entries or proof that global P has disappeared.

The finite-release control supplies about 1,252 kg P/year from geological reserves
in years 20 and 40, rather than importing it or bypassing nutrient accounting.
It removes the measured P constraint, prevents local physical food shortage at
year 40, maintains positive growth through year 50 and enables six daughter towns.
Needs-based allocation alone removes access shortage but does not prevent the
later reversal. This supports a causal role for available P in this seed; it does
not identify tenfold release as an optimal or realistic default.

### Why trade does not cover local shortages

At year 40, other towns' simultaneous edible surpluses could cover all local
physical gaps under instantaneous unrestricted redistribution. Actual export
rules protect **12 months** of seller reserves, and routes remain constrained.
The combined checkpoint's cumulative food-request counters through year 50 show:

- 916 decisions with no eligible surplus seller (after market-open/reserve filters).
- 202 with no usable route; this category includes missing usable sea capacity.
- One freight-bound and four batch-ceiling-bound decisions.
- Only 3,679 kg of commercial food dispatched across those recorded requests.

The 9.19 million kg requested is repeated reserve-replenishment demand, not
9.19 million kg of unique hunger. Absence of a money-limited selected offer does
not prove every town could afford every hypothetical shipment. Relief is separate
from these commercial counters. By year 50 the combined run also has 6,439 kg of
monthly world physical deficits summed across the year, so redistribution alone
would no longer cover everything even at this idealized bound.

### Verification and limits

- Five demographic-audit CPU tests and three growth-runner tests passed.
- Two Python summary fixtures passed, including mixed physical/access shortages,
  cross-town redistribution bounds and missing-boundary rejection.
- The GPU fixture passed with the final probes: observed monthly/checkpoint
  continuation equals unobserved batching, and resource ratios match the CPU check.
- The existing GPU food-request fixture passed, checking the market constraint
  classifications against actual supplier, route and budget gates.
- All four 50-year arms completed without simulation errors. Maximum annual
  normalized economy residual was below `1.465e-5`; closing population residual
  was zero in each. Raw outputs/checkpoints occupy about 4 GB under `output/`.

This is one instrumented seed with two mechanism interventions, not multi-seed
calibration. The findings justify reviewing farm phosphorus replenishment,
retention and loss rates, alongside emergency export-reserve policy and usable
routes. They do not justify globally multiplying food output or claiming every
future decline has the same cause. The new tools make those follow-up comparisons
measurable; no default production or distribution policy changed here.
