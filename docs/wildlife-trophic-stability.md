# Trophic stability follow-up

The [first evaluation](wildlife-calibration.md) exposed century-scale animal losses.
This follow-up separates numerical feeding errors, intrinsically insufficient intake,
and predator–prey overshoot. These are game-model tests, not empirical rate fitting.

## Mechanisms

**Remove an implicit feeding threshold.** The weighted-food denominators previously
used a minimum of 1e-7 kg C/m². That changed per-capita intake at low biomass even
when catalog half-saturation was zero. The guard is now 1e-30, far below the existing
1e-12 numerical extinction threshold. A GPU fixture compares nutrient-limited growth
at ordinary biomass and at 1e-8 times that biomass against the same analytical answer.

**Make scarce animal prey harder to capture.** `animal_prey_refuge` is an optional
guild trait, measured in kg C/m². It defaults to zero for older catalogs. For animal
prey compartment `j`, the effective preference is:

```text
preference_j = catalog_weight_j × C_j / (C_j + refuge)
F = sum(C_j × preference_j × habitat_access_j)
capture = F / (F + food_half_saturation)
```

Plants and detritus retain their original preferences. A positive refuge requires a
positive food half-saturation: otherwise a sole prey preference would cancel out of
the allocation and fail to limit capture. The existing shared intake, per-prey removal
cap, assimilation, stoichiometry, respiration and nutrient-return rules still apply.

For a sole rare animal prey, effective food falls approximately quadratically with
density. For an omnivore, abundant alternative food receives more of its effort.
This is an abstract combination of refuge and changing capture opportunity, not
individual learning or a resolved cave/hiding-place inventory. It prevents neither
starvation nor extinction in every environment and never restores empty populations.

The mechanism is motivated by context-dependent functional responses described in
[the marine predator review](https://pmc.ncbi.nlm.nih.gov/articles/PMC7013479/) and
[optimal prey switching research](https://pubmed.ncbi.nlm.nih.gov/36416056/). Their
results do not justify applying one response to every animal or claiming a fitted
value for this fictional setting.

No ecological buffer grew. The GPU catalog stores the refuge float in a previously
redundant guild word; prey IDs already occupy the four diet slots. Old archives rebuild
tables from their serialized catalogs with refuge zero. No archive version change is
needed for the optional catalog field.

## Diagnostics and reproduction

`WildlifeReport` now includes area-weighted C/N/P for compartments 0–25 in each of
the four geographic regions. This exposes producers, dissolved nutrients, detritus,
deep water, sediment and burial alongside animal stocks. Regional allocation remains
an approximation within mixed ecological cells; it is not a new fine-scale census.

The evaluator records ten-year snapshots, full guild definitions and before/after
states. `WILDLIFE_CATALOG` selects an editable experimental TOML catalog, and
`WILDLIFE_OUTPUT` prevents trial files from overwriting one another. Existing positional
rate overrides still apply after reading that catalog.
`WILDLIFE_CLOSED_ONLY=1` omits the open-barrier ablation when evaluating an additional
long-run seed. The default still executes both barrier settings.

```sh
WILDLIFE_CATALOG=path/to/catalog.toml WILDLIFE_OUTPUT=output/wildlife/trial.json \
  mise exec rust@1.89.0 -- cargo run --release --example wildlife_evaluate -- 1200 32
mise exec rust@1.89.0 -- cargo test --lib wildlife_tests -- --ignored --test-threads=1
```

The controlled capture fixture accounts for prey mortality before feeding and checks
the exact expected predator growth at abundant and scarce prey densities, with refuge
enabled and disabled. It also verifies C/N/P conservation. Catalog tests check legacy
defaults, invalid values and the packed GPU representation.

## Adopted new-world settings

| Guilds | Annual maximum intake | Animal-prey refuge | Food half-saturation |
|---|---:|---:|---:|
| Terrestrial herbivores 0–4 | 2.0 | 0 | unchanged |
| Small predators 5 | 0.8 | 5e-6 | 5e-6 |
| Apex predators 6 | 1.2 | 5e-6 | 5e-6 |
| Aquatic grazers 8 | 2.4 | 0 | 0.001 |
| Aquatic predators 9 | 0.8 | 5e-6 | 5e-6 |
| Migratory river animals 10 | 2.4 | 5e-6 | 0.001 |
| Waterbirds 11 | 0.8 | 5e-6 | 5e-6 |

Intake is kg food C/kg consumer C/year; the two density columns are kg C/m².
Detritivores and all maintenance, assimilation, mortality and stoichiometric settings
remain unchanged. These rates allow growth when resources permit; they do not supply
food or bypass phosphorus requirements. The catalog sanity test rejects a guild whose
optimistic nutrient-sufficient intake cannot even cover maintenance and mortality.

New worlds use these settings. Existing archives retain their serialized guild rates
and default missing refuge fields to zero. Their numerical denominator fix still
applies, but their saved catalogs are not silently recalibrated.

## Results and limitations

[Artifact retention policy](evidence/README.md)
and [all trial summary rows](evidence/wildlife-stability/tables.md) preserve 23 world
runs, including failed parameter-only adjustments. Regenerate the tables with:

```sh
python3 scripts/wildlife_report.py docs/evidence/wildlife-stability/*.json.gz
```

| Intact-barrier result | Without refuge | With refuge |
|---|---:|---:|
| Seed 17, 500 years: central terrestrial occupied guilds | 2/7 | 7/7 |
| Seed 17, 500 years: lake predator share of aquatic consumer C | 0% | 4.15% |
| Seed 17, 500 years: lake predator occupied area | 0% | 40.7% |
| Seed 81, 500 years: central terrestrial occupied guilds | not run | 7/7 |
| Seed 81, 500 years: lake predator share / occupied area | not run | 22.83% / 94.0% |

The comparison uses the same intake settings and seed. Trait differences are active
during the initial ecological year, so these are whole-run comparisons, not identical
post-spinup checkpoint interventions. The analytical fixture provides the independent
test of the immediate capture mechanism.

At 100 years, seeds 17, 81 and 256 retain roughly 50–52 million kg of great-lake
grazer carbon with refuges, versus zero or almost zero without them. Predator shares
are 1.74–2.32%. Held-out seed 409 at terrain/ecology resolutions 64/32 gives 2.40%.
The two 500-year closed-barrier runs use 32/16. Both retain all seven terrestrial guilds
on central and outer land, but this means **some occupied area**, not equal abundance
or species richness. These guilds include fictional giant animals; body-size realism
has not been established by this test.

Lowering aquatic predator/bird half-saturation from 5e-6 to 5e-7 without refuges still
lost the predators by 500 years. Increasing intake alone also failed. The retained
time series shows prey collapses followed by predator losses, with later prey recovery.
Refuges preserve enough prey and predator populations for later recovery instead of
requiring spontaneous recruitment.

The new compartment diagnostics show persistent algal carbon and increasing dissolved
and deep phosphorus in the refuge runs while animal stocks cycle. This argues against
simple exhaustion of the entire primary food stock as the explanation for these
particular animal troughs; it does not establish local access to every nutrient stock.

Maximum C/N/P relative error across the retained runs is below 1.26e-4 (0.0126%), within
the existing budget tolerance. All 47 ordinary tests, six focused GPU wildlife tests,
14 GPU ecology tests, 18 GPU economy tests, and Clippy passed with the adopted defaults.

**Remaining limits:** aquatic boom-and-bust cycles are still large, and the second
long-run seed's predator share is high. This is improved persistence, not equilibrium
calibration or ecological realism proven over arbitrarily long histories. Coarse-grid
refuges are not resolved habitat structures, and per-month dispersal still needs a
resolution study. The next useful work is to measure cycle amplitudes, relate refuge
capacity to actual habitat, and separate persistent populations within guilds before
adding species identities.
