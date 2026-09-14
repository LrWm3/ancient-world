# Farm nutrient replenishment and retention

This follows the [growth reversal diagnosis](growth-food-diagnostics.md).
The question is whether retaining finite nutrients can support crops without
increasing the geological release rate or adding nutrients from nowhere.

## Existing stock and flow rules

Managed soil receives phosphorus from a finite geological reserve and from
detrital mineralization. Geological release defaults to `1e-7` of remaining
reserve per month. Detritus mineralizes at 8% per month. Harvest residues,
spoilage and animal processes already return nutrients through their respective
ledgers. Consumed food returns the town's configured fraction of N/P to detritus
(initially 85%); the remainder is an external loss from the managed farm domain.
Recycling is a transfer, not another geological source or energy input.

Rain exceeding storage exports soil nutrients downstream before release,
mineralization and crop uptake. Previously C, N and P all had the same mobility:
up to 5% of the entire available soil inventory could leave in a month. The prior
diagnosis measured a large initial P flush followed by insufficient replenishment.
Exports enter the ecological return path, but do not automatically become nutrients
available to upstream managed plots. Global conservation alone cannot establish
local fertility.

## Scoped control

`ProductionSettings.phosphorus_runoff_mobility` scales only the phosphorus part
of dissolved runoff. Allowed values are 0–1; the default and old-archive default
are 1, preserving the previous rule. At 0.1, the same water exports one tenth as
much P. The withheld amount stays in available soil. Nitrogen, carbon, water,
release and crop nutrient requirements are unchanged. Downstream return and the
managed exchange ledger use the actual reduced flux.

The control uses an existing spare GPU uniform component, with no extra buffer
or readback. It does not implement sorption pools, erosion-bound phosphorus,
mineral-specific dissolution or treatment infrastructure. It is an experimental
mobility coefficient for this toy model, not a measured soil parameter.

`growth_ladder --manure-retained 0.98` separately changes the initial towns'
consumed-food nutrient return policy. It records ordinary policy events and
does not override later changes or impose the policy on daughter towns. This
tests the existing return mechanism; it does not charge additional collection
labor or model the practicality of 98% recovery.

## Reproduction

Build with `cargo build --release --example growth_ladder`. Use separate output
directories and copy the same earlier `seed-17/base.world` into each beforehand
to reproduce a matched comparison. Without that copy, the runner generates a base.

```sh
target/release/examples/growth_ladder --seeds 17 --modes combined --gates 50,100 --continue-on-decline --food-diagnostics --phosphorus-mobility 0.1 --output output/nutrient-retention
target/release/examples/growth_ladder --seeds 17 --modes combined --gates 50,100 --continue-on-decline --food-diagnostics --phosphorus-mobility 0.1 --manure-retained 0.98 --output output/nutrient-recycling
python3 scripts/summarize_growth_food.py output/nutrient-retention/seed-17/combined/food-monthly.jsonl
cargo test --test nutrient_retention -- --include-ignored
```

Both arms keep geological release at its default. Their specification records
the interventions; checkpoints retain the actual production settings and town
policies. Continue-on-decline deliberately examines whether an early gain lasts.

## Verification

The GPU fixture checks the immediate mediator against matched opening stocks:
90% less runoff P, the same withheld quantity added to crop-available soil,
unchanged N/water availability and unchanged geological release. Runoff must be
nonzero. It also checks managed conservation residuals and exact checkpoint plus
monthly/batched continuation. CPU validation rejects invalid mobility values and
checks that missing settings retain legacy mobility. These checks passed on the
Quadro RTX 5000 with Max-Q Design.

## Century comparison — 2026-09-14

Seed 17, identical original base archive, terrain/ecology 256, one geological
epoch, 16 groups totaling 1,920 residents, individual demography. Both controls
use the previous `combined` arm's modest mortality, food-access, yield and
founding changes. The reference is that earlier combined run, not its baseline
arm. No default balance setting changed.

| Arm | Year-50 population | Year-100 population | Active towns / records at 100 | Trailing 20-year population slope |
| --- | ---: | ---: | ---: | ---: |
| Previous combined reference | 2,134 | 1,548 | 16 / 16 | −2.58/year |
| P mobility 0.1 | 3,828 | 4,155 | 54 / 56 | +13.96/year |
| P mobility 0.1 + initial-town return 0.98 | 3,828 | 4,925 | 63 / 66 | +27.20/year |

Both controls match the earlier tenfold-release experiment's population and town
count at year 50 without increasing the release fraction. Recycling produces
more detrital returns immediately, but does not improve population then because
crop P is still sufficient in both controls. By year 60, retention alone fulfills
89.37% of requested crop growth; stronger recycling fulfills 100%. This immediate
crop mediator supports the later population difference; late differences also
include changing migration, town founding and other feedbacks.

| Year-100 measurement | Retention | Retention + recycling |
| --- | ---: | ---: |
| Actual / requested crop growth | 81.39% | 98.77% |
| P-short town-months / observed town-months | 137 / 648 | 9 / 757 |
| Physical food gap, edible-equivalent kg/year | 6,360 | 13 |
| Access food gap, edible-equivalent kg/year | 29,260 | 19,856 |
| Runoff export, kg P/year | 27,107 | 39,624 |
| Geological release, kg P/year | 311 | 339 |
| Detrital mineralization, kg P/year | 2,526 | 3,772 |
| Consumed-food return, kg P/year | 1,965 | 2,671 |

Mineralization and food return are different boundaries of a recycling loop;
**do not add them as independent new P sources**. More settlements claim more
finite land inventories, which explains why total geological release increases
despite an unchanged rate. Larger available stocks can also cause larger actual
exports despite stronger retention. The table is a set of observed fluxes, not
a complete P budget by itself.

### Expansion hides depletion of established farms

Available soil P in the original 16 towns falls from about 443,973 kg after month
one to 186 kg at year 100 with retention alone, or 2,475 kg with stronger
recycling. Yet all-town available soil P at year 100 is 437,281 or 638,072 kg,
respectively: newly claimed managed land changes the scope of that total. Those
claims transfer existing ecological stocks; they are not proof that older farms
have a replenishment equilibrium. The retention-only trajectory also declines
for part of the century before renewed growth. An ending positive slope does
not establish sustained growth on a fixed land footprint.

### A separate crop-establishment gap

At year 70, essentially all local physical shortage is concentrated in a daughter
town: site 41 in retention, site 43 in recycling. In the latter, all six crop
slots at month 840 have zero seeds, standing crop and cumulative harvest. It has
positive farm labor and potential, about 55,986 kg available soil P and 171,998 m³
crop-available water, but requests no crop growth. The existing shader requires
seed or established biomass and can establish crops from purchased seed goods.
More soil P cannot fix this particular missing input. Investigate seed delivery
and provisioning of daughter settlements separately rather than relaxing their
nutrient requirements.

At year 100, other towns' simultaneous edible surpluses could cover every physical
gap under the diagnostic's instantaneous redistribution bound in both arms.
That is not a claim that actual routes, reserves or purchasing power can deliver
those goods. Food access remains a distinct limitation.

### Checks, interpretation and next steps

Both 100-year arms completed without errors. Annual closing population residuals
were zero; maximum absolute normalized economy residuals were `1.305e-5` and
`1.239e-5`. Two retention tests (including the GPU test), three runner tests and
two Python summary fixtures passed. Generated monthly observations and archives
remain under ignored `output/`; only this summary is committed.

These controls make a useful improvement in this seed, but are not multi-seed
calibration or a complete factorial experiment: recycling alone was not tested.
Keep the current defaults until comparisons include other seeds and established
farms on a fixed footprint. The most useful follow-ups are:

- Separate locally available, retained and slowly releasable P rather than
  exporting a fixed fraction of the whole available stock regardless of soil.
- Make improved nutrient recovery require actual collection/composting work,
  with recorded losses and finite material transfers.
- Review daughter-town seed establishment and food delivery independently.

The present controls provide measurable interventions for that work without
changing the monthly schedule or weakening conservation.
