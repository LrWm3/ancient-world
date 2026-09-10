# Geochemical habitats and underground growth

This increment preserves geological habitat patches through ecology aggregation and makes underground growth limits inspectable. It does not implement individual cave systems, new animal guilds, or a continent-wide productivity quota.

## Model

Terrain already provides plate-boundary activity with rejuvenated outer belts and old interiors. Ecology now classifies nested habitats on each **terrain** cell before aggregation:

- Enriched: activity ≥ 0.20.
- Active province: activity ≥ 0.55.
- Vent: activity ≥ 0.82.

All require groundwater access (`clamp(groundwater_m * 2, 0, 1) > 0.05`) and reactive rock fraction > 0.001. These are game habitat thresholds, not measured geological classifications. They apply to both continents; the existing producer catalog still restricts its twelve underground species to the outer continent.

Coarse cells retain habitat area fractions, peak activity for inspection, and separate conditional temperature, rainfall, groundwater, fertility, substrate mix, trace minerals, activity and regional affiliation for enriched and active habitats. Shallow underground producers select from the enriched environment; deep producers select from the active environment. Ordinary surface species continue using the general land environment. Peak activity never multiplies whole-cell production.

For local activity `a`, wetness `w` and groundwater access `g`, the exposed reactive rock fraction consumed per year is:

```
w * (0.02*a + 0.5*a^4) * (0.2 + 0.8*g)
```

Aggregation stores the area-weighted product of that rate and reactive rock fraction. Dividing by mean reactive fraction yields a substrate-weighted rate. This preserves the initial integrated reaction flux, including a narrow patch in otherwise inactive rock. Habitat uptake uses the habitat's share of reactive flux. Each monthly reaction remains capped by the existing source inventory and respects the supply-disable intervention. Source stocks and starting biomass have not been increased.

The requested fictional H₂/CO₂ symbiosis now has its own carbon-fixation pathway. Hydrogen energy draws from the H₂ reserve, replenished by consuming reactive rock. Secondary oxidative chemistry separately requires oxidant and consumes only the usable share of its energy pool. Both pathways require water, trace minerals and nutrients, pay the existing nitrogen-fixation cost, and record imported fixed carbon in the C/N/P ledger. Oxidant used for nitrogen fixation is charged as well as oxidant used for biomass growth.

There is no underground seed injection: production already establishes from zero. Mortality and maintenance remain active. Existing underground litter enters detritus and recycling, providing an existing connection to surface soil. Dedicated underground consumers and more explicit vertical nutrient pathways remain future work.

## Inspection and archives

`Ecology::inspect_environment` and the cell inspector expose nested habitat fractions and each underground layer's latest growth, loss and limiting resource. Limits are energy, N, P, habitat, groundwater, trace minerals and oxidant. They identify the implemented production ceiling, not a complete sensitivity analysis. Loss includes producer mortality and maintenance, not animal grazing. Rates are whole-cell kg C/m²/month; habitat fractions in the viewer are normalized by land area.

Regional budget reports add habitat fractions, layer growth and loss, resource-limit area fractions, reactive rock consumption and diagnostic coverage. Last-month rates are annualized in `budget`; `examples/calibrate.rs` integrates twelve actual monthly samples. Habitat absence is included in the regional constraint fractions. Diagnostics are invalidated by fresh aggregation until biology runs again, rather than displaying stale results.

Archive version 4 retains `ANCIENT2` framing and extends environment records from 128 to 352 bytes. Versions 2–3 load their original physical environment fields, inventories and clocks, with new derived fields initially unavailable. They are reconstructed at the next monthly step. This is stock-preserving compatibility, not a claim that the revised ecological equations reproduce the older executable. Version-one terrain import remains explicit. At ecology resolution 256, the new fields add 84 MiB; memory estimates use the actual environment struct size.

## Reproduction

```sh
# epochs, terrain edge, ecology edge, comma-separated seeds, report path,
# optional ecology-only warmup months before the sampled year
cargo run --release --example calibrate -- 5 64 64 0,7,42,99,999 output/hotspot-matched.json
cargo run --release --example calibrate -- 5 64 32 0,7,42,99,999 output/hotspot-coarse.json
cargo run --release --example calibrate -- 5 64 64 7,42 output/hotspot-long-ecology.json 1800
cargo test -- --include-ignored --test-threads=1
```

## Initial calibration

The baseline and revised matching-grid runs use five geological epochs (50 ecological years), followed by twelve sampled months. Each final revised world reports converged climate. These are 64²-per-face diagnostic worlds, not production-resolution benchmarks.

| Seed | Baseline annual outer chemo share | Revised share | Underground biomass increase |
|---|---:|---:|---:|
| 0 | 0.220% | 0.392% | 2.70× |
| 7 | 0.193% | 0.649% | 3.65× |
| 42 | 0.146% | 0.484% | 3.52× |
| 99 | 0.297% | 0.998% | 3.57× |
| 999 | 0.310% | 0.996% | 3.42× |

Outer underground carbon is 0.00412–0.00775 kg/m² averaged over all outer land. Central production remains at least 99.85% solar. Enriched habitats occupy 11.52–18.18% of outer land, active provinces 2.56–5.08%, and vents 0.57–1.31%. The upper tail is above the suggested <1% vent target; thresholds have not been adjusted solely to force the percentages. These habitat areas are not claims of lush or chemosynthesis-dominated ecosystems.

The first implementation retained the shared oxidant ceiling. Despite faster reactions it reduced underground biomass; `output/hotspot-oxidant-capped.json` records that unsuccessful iteration. Separating the requested hydrogen pathway from secondary oxidative chemistry produced the improvement above. Most eligible underground habitat is now energy-limited rather than nutrient-limited in these particular worlds. Outside it, habitat remains the reason for no production. This does not justify increasing nutrients globally.

Maximum C/N/P residual across the matching and coarse five-seed suites is 1.05×10⁻⁵ relative (0.00105%). Stocks remain finite and nonnegative. The controlled hydrogen test removes new supply and sunlight: reserve-driven underground growth occurs despite no initial oxidant stock, then declines by over half after depletion; unreacted geological stock stays untouched while supply is disabled.

### Resolution and performance limits

The narrow-patch fixture compares terrain 16 with ecology 16 and 8, including patches on all six faces. Habitat area and initial integrated reaction flux agree within 2×10⁻⁵ relative; warm conditional habitat survives inside a cold coarse cell, and both underground layers establish without seeding.

This is **not full ecological resolution convergence**. Coarse ecology 32 on terrain 64 retains the habitat belts, but produces 28–46% more mean underground biomass than ecology 64. Inventories and competitive nutrient pools remain shared within a coarse cell; later depletion is not tracked separately for each constituent terrain cell. Conditional means also cannot preserve every species tolerance distribution. Existing aggregate photo/chemo reporting has approximate attribution in mixed land/water cells. Separate subcell reserve cohorts and improved coastal production attribution are appropriate next steps before claiming matching results across resolutions.

A transport experiment replaced full environment reads with selected physical fields. Seed 7's budget and annual results were exactly unchanged, but timing did not improve consistently (2.31 versus 2.53 ms/month in those samples), so the change was discarded. Transport remains the slowest measured ecological kernel at this diagnostic size. These samples are not a controlled hardware benchmark.

A seed-7 20-geological-epoch attempt stopped during epoch 12 (Ecology stage) with `secondary lake surface flow unresolved after 4096 iterations`. It is excluded from successful calibration; the solver limit was not bypassed. Longer fixed-geology ecological checks are recorded separately below.

### Longer ecology-only checks

Seeds 7 and 42 completed 50 ecological years of geological epochs, another 150 years with terrain geology held fixed, and a sampled year (201 ecological years total). Weather, water and ecology continued throughout the extra interval. Both retained finite, nonnegative inventories. Maximum relative C/N/P residual was 4.10×10⁻⁵ (0.00410%).

| Seed | Outer annual chemo share at year 201 | Underground C, kg/m² | Remaining reactive rock, kg/m² |
|---|---:|---:|---:|
| 7 | 0.0828% | 0.000712 | 260.85 |
| 42 | 0.0865% | 0.000878 | 267.11 |

Thus the early biomass increase is **not sustained at the same level without geological renewal**. Much remaining rock is in less accessible or less active locations. Increasing access accelerates consumption; it does not supply an enduring second energy source by itself. The next source-model improvement should distinguish deeper finite reserves, exposure into reactive zones and depleted material, rather than repeatedly resetting available rock. Full subcell reserve cohorts would also address the coarse-grid discrepancy. Persistent spectacular ecosystems and dedicated underground food webs are not yet demonstrated by this increment.

### Verification

All 89 tests passed with hardware-GPU tests enabled. This includes conservation, absent-energy/phosphorus fixtures, lake exchange, ecological and living-history checkpoint continuation, version-three environment migration, archive exports, and the new narrow-habitat and finite-hydrogen fixtures. `cargo clippy --all-targets -- -D warnings` passed.
