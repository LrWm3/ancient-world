# Nutrients without settlement expansion

Follow-up to [farm retention](farm-nutrient-retention.md): the earlier comparison
allowed daughter towns to claim additional finite soil inventories. This control
limits all three arms to the original 16 settlement records. It isolates that
expansion channel without turning off food purchasing, migration, trade, weather,
living ecology or the rest of history.

## Settings and reproduction

Seed 17, terrain/ecology 256, one geological epoch, 1,920 initial residents,
individual demography, the same original `growth-long/seed-17/base.world` used
in the previous comparisons. All arms use `combined` and its existing modest
health, relief, yield and founding settings. Only phosphorus mobility and initial
town recycling differ. Geological release remains `1e-7` of the remaining reserve
per month in every arm. No default balance settings changed.

Copy that base archive into each output directory's `seed-17/base.world` before
running. For a fresh reproduction without the local archive, generate one base
once and reuse it for every arm; do not silently compare differently generated
bases. Raw evidence stays under ignored `output/`.

```sh
cargo build --release --example growth_ladder
target/release/examples/growth_ladder --seeds 17 --modes combined --settlement-cap 16 --gates 50,100 --continue-on-decline --food-diagnostics --output output/fixed-nutrients-reference
target/release/examples/growth_ladder --seeds 17 --modes combined --settlement-cap 16 --gates 50,100 --continue-on-decline --food-diagnostics --phosphorus-mobility 0.1 --output output/fixed-nutrients-retention
target/release/examples/growth_ladder --seeds 17 --modes combined --settlement-cap 16 --gates 50,100 --continue-on-decline --food-diagnostics --phosphorus-mobility 0.1 --manure-retained 0.98 --output output/fixed-nutrients-recycling
```

These runs used the preceding retention executable (`1c0bfb7`). The accompanying
daughter-seed fix cannot affect these arms: the existing cap rejects founding
before its new seed admission check; no new economy requires seed initialization.
This is a nutrient comparison, not a long-run calibration of the seed fix.

The control freezes the set of settlement locations, not all activity on them.
Cultivation, labor, diet, reserves and environmental exchanges still change.
Consequently this is not a closed laboratory nutrient budget or a claim that
all differences downstream are direct effects of phosphorus.

## Results — 2026-09-17

All three runs completed 100 years on the Quadro RTX 5000 with Max-Q Design.
Every annual observation has exactly 16 settlement records; all 16 are active
at the end and there are no resettlements.

| Arm | Population at 50 | Population at 100 | Trailing 20-year slope |
| --- | ---: | ---: | ---: |
| Matched reference | 2,134 | 1,559 | −3.11/year |
| Retention (P mobility 0.1) | 3,852 | 2,711 | −93.62/year |
| Retention + recycling (return 0.98) | 3,852 | 5,672 | +13.80/year |

Use the freshly run reference here, not the earlier uncapped reference's 1,548
ending population. The old and new reference trajectories are not bitwise
identical; their first population difference occurs in year 29 (two people).
These controls share the same executable and base archive.

### Retention alone postpones the constraint

The retention arm peaks at 4,633 residents in year 70, then falls to 2,711.
Available soil P falls from 18,728 kg at year 50 to 947 kg at year 80 and only
2.33 kg at year 100. Year-100 crop growth fulfills 56.03% of requested growth;
P is short in 180 of 192 town-months. Nitrogen and water do not constrain the
recorded crop requests then. Lower mobility helped retain the initial inventory,
but did not balance its continuing losses and use.

This differs substantially from the earlier expanding retention arm, which ended
with 4,155 people and a positive trailing slope. Expansion was masking an ongoing
problem in established farmland, not merely making an already stable system larger.

### Stronger recycling helps on the same land

The recycling arm still fulfills 100% of requested crop growth at year 100, with
no N/P/water-short town-months. Its available soil P is 22,605 kg at year 50,
4,927 kg at year 80, and 2,684 kg at year 100. Thus its improved production does
not depend on claiming additional sites. However, soil stock is still declining:
this is evidence for an improved century outcome, not a demonstrated equilibrium.

Year-100 measured P flows, kg/year:

| Flow | Reference | Retention | Retention + recycling |
| --- | ---: | ---: | ---: |
| Geological release | 125.24 | 125.24 | 125.24 |
| Dissolved runoff | 1.31 | 0.09 | 160.88 |
| Food-consumption return to detritus | 705.33 | 1,177.56 | 3,047.98 |
| Mineralization from detritus | 737.57 | 1,281.01 | 3,214.37 |

Very low runoff in the depleted arms is not successful nutrient retention: little
available P remains to export. Recycling and mineralization are successive stages
of the same loop, not independent imports. The return arm also feeds more people;
its larger absolute returns are not a direct estimate of the policy effect at
equal food consumption. Its initial return fraction rises from 85% to 98% and
does not yet require additional collection labor or infrastructure.

### Remaining food limits

At year 100, edible-equivalent kg of unmet food:

| Gap | Reference | Retention | Retention + recycling |
| --- | ---: | ---: | ---: |
| Physical local shortage | 7,556 | 53,706 | 13,908 |
| Household access shortage | 558 | 844 | 18,895 |
| Remaining physical deficit after ideal instantaneous redistribution | 0 | 11,432 | 0 |

The recycling arm's requested crop growth is fully supplied, but that does not
mean local production meets every town's food needs in every month. Other towns'
simultaneous surpluses could cover its physical gaps under the diagnostic upper
bound. Real routes, seller reserve policies and household purchasing power still
matter. Nutrient availability and distribution must remain separate diagnoses.

## Verification and limits

All annual closing population residuals are zero. Maximum absolute normalized
economy residuals are below `2.932e-5` in all arms. These are float32 simulation
checks, not exact elemental balances. Monthly observations retain the actual
production and consumption boundaries; summaries can be regenerated with
`scripts/summarize_growth_food.py`.

The independent daughter-founding change passed all 21 civilization tests,
including planting-stock conservation, seedless rejection, actual harvest and
checkpoint/batch continuation. See its separate report for timing and scope.

This remains one seed and two nutrient interventions, not multi-seed calibration.
There is no recycling-only arm, no extra recovery-work cost, and no equilibrium
claim. Keep default parameters unchanged. The useful next comparisons are
longer fixed-site runs, other seeds, and recovery constrained by paid collection
work. For food deficits that remain while crop resource requests are fulfilled,
investigate access and transport before increasing nutrient supply again.
