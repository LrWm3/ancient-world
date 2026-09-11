# History realism refinement

This pass addresses the first dependencies identified by [the seed audit](ideas-seed-audit.md): unsettled initial climate, effectively unlimited grain reserves, synchronized settlement expansion, and persistent flooding with no investment response.

## Rules

- Climate warm-up now allows 32 seasonal cycles with early exit. The existing annual-mean thresholds remain 0.25 °C and 10 mm/year. A cycle must also return to within 0.25 °C and 0.1 mm vapor of the preceding cycle's final seasonal phase. A configured limit that is too short still reports unresolved convergence; it does not silently pass.
- Granaries hold 12 demand months of grain, rising to 24 with sufficient pottery. Demand months use the existing market convention of 18 kg/person/month. Consumption precedes overflow loss, so an oversized harvest still feeds the town. Overflow enters managed detritus with its C/N/P intact and is included in the food-loss ledger. Ordinary spoilage retains its existing respiration/recycling accounting. This is an abstract storage rule, not a building-footprint simulation.
- Daughter villages consider expected harvest per hectare discounted by travel cost. Productive homelands retain settlers longer; attractive destination farmland lowers the population threshold. Parties contain 20% of residents, bounded to 40–90 people, with proportional cohorts and explicit food/cash transfers. Sites still require a year's provisions, remain on their original inner continent, and cannot expand during persistent inundation. These are game decision rules, not demographic estimates.
- After twelve wet months, towns can build raised granary foundations from finite brick stocks and part of the labor already diverted by flood disruption. Foundation inventory is saved and included in the goods ledger; monthly wear becomes declared material waste. Protection is limited to 1.5 m and applies only to stored grain. Fields, roads, ports, and physical water remain exposed. Foundation status appears in the inspector and investments produce causally linked `flood_adaptation` events. Automatic town evacuation is not implemented by this pass.
- Waterlogging now adds illness pressure to the GPU demographic model, in addition to hunger. Recovery follows environmental cleanup, and existing finite expedition remedies can treat the resulting illness. This creates a potential need, not a guaranteed discovery benefit or epidemic quota.

Existing archives default foundation inventory to zero. Their saved climate-cycle setting remains unchanged; regenerating with the new default produces the converged comparison baselines. Continuing an older archive uses the updated history rules, including accounting for excess stored food on its next monthly step.

## Evaluation protocol

Fresh seeds 17, 81 and 256 use matching 64-cell face grids, one geological epoch, ten initial ecological years, sixteen founders and 100 years of living history. Society, politics, governance, shipping, expeditions and discoveries are enabled. A separate seed-17 control changes only drought severity from 0.5 to 0.9. These are diagnostic worlds, not production-resolution benchmarks or mature geological calibrations. Jobs overlap, so elapsed times are not comparative performance measurements.

Seeds 17 and 81 can be compared with the earlier dense audit. Climate, food and settlement rules changed together, so differences cannot be attributed solely to one mechanism. The severe-drought control isolates that setting within the new rules.

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
target/release/examples/history_evaluate --seeds 17,81,256 \
  --resolution 64 --epochs 1 --years 100 --civilizations 16 \
  --discoveries --living-world --save-worlds \
  --output output/history-refinement --label history-refinement
# Repeat seed 17 with --drought-severity 0.9, output history-refinement-pressure.
/usr/bin/python3 scripts/analyze_ideas_audit.py output/history-refinement \
  output/history-refinement-pressure
```

## Remaining work

Political motives beyond recent hunger/raids, household interests in foreign policy, automatic evacuation from untenable sites, demand-scaled expedition applications, and longer-history geochemical hotspot calibration remain follow-ups. This pass does not claim parity with Dwarf Fortress history or completion of the full ecological calibration.

## Results

All four runs completed with climate convergence in nine cycles and passing annual history/ecological ledgers.

| Seed | Weather | Population after 100 years | Active towns | Food reserve months | Shortage site-years | Wars | Treaties |
|---|---|---:|---:|---:|---:|---:|---:|
| 17 | Default | 5,804 | 35 | 18.46 | 0 | 0 | 38 |
| 81 | Default | 5,793 | 32 | 17.67 | 0 | 0 | 14 |
| 256 | Default | 5,800 | 31 | 17.91 | 0 | 0 | 19 |
| 17 | Severe drought | 1,800 | 16 | 17.80 | 51 | 12 | 23 |

Ordinary-weather polities now have one to three towns, compared with nearly universal three-town polities in the prior dense audit. Individual town populations span roughly 56–368 residents. Population totals remain similar across ordinary seeds, so geographic variation in aggregate demographic growth is still limited. Bounded grain reserves correct the earlier 15–18-year accumulation without requiring ordinary worlds to experience famine.

The pressure control records 83 monthly food-crisis events, six governance crises, six secessions and 19.17 kg of remedy use. The smaller population and reduced expansion are substantial consequences of sustained severe drought; this is not a proposed default. Endpoint food stocks have recovered and do not describe the preceding shortages. Ordinary worlds still use no remedies; no persistent-inundation construction occurs in these fresh runs. Consequently, foundation effectiveness requires the controlled inundation test and archived-site continuation rather than an inference from these seeds.

Maximum managed-history relative residual across the four runs is 1.96e−5. Reports: [ordinary seeds](../output/history-refinement.md), [pressure control](../output/history-refinement-pressure.md). Matching JSON reports, full world archives and `*-ideas.json`/`*-analysis.json` diagnostics retain events, inventories, climate status and ecological comparisons.

Converged climates do not by themselves fix the weak second energy system: outer-continent geochemical production remains about 0.069–0.111% of total production in these ordinary-world endpoints. That needs a separate longer-geological-history calibration; no regional biomass multiplier was added here.

An additional ten-year continuation of the earlier dense seed-17 archive preserves its old geological climate and existing towns. Galan's previously persistent inundation recedes and completes twelve dry recovery months; it ends with about 108 residents. No foundations are built in that continuation, so it is evidence of recovery/old-save compatibility, not a construction success. The dedicated paired fixture supplies a declared brick inventory and holds a 0.75 m local flood: only the funded town can build protection, while both lose identical standing crops. Very deep river exposure exceeds the design's 1.5 m protection limit.

## Verification

The full release regression suite passed 60 distinct tests, including opt-in hardware GPU tests for conservation, drainage, ecology, social history and exact checkpoint continuation. After strengthening the paired foundation fixture, that test was rerun and passed. `cargo clippy --all-targets -- -D warnings` passed. The ordinary/pressure seed reports validate every annual boundary. Across those four runs, ecological C/N/P relative residuals remain below 1.68e−5 and ecological water residuals below 4.71e−5. No desktop interaction smoke test or production-resolution performance benchmark was performed in this pass.
