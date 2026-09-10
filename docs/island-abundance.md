# Island abundance calibration

The previous realism pass bounded storage, but most islands still produced enough food to fill it. This pass separates finite natural nutrient supply, accessible settlement land, and attainable crop harvests.

## Controls

New-world configuration and the explorer's **Island abundance** panel expose:

- `island_phosphorus_scale`: scales initial island soil/detritus phosphorus and the phosphorus concentration used for island geological sources and subsequent geological imports. Rock, soil, water and climate distributions still vary geographically. Mixed ecological cells aggregate the scaled chemistry by actual land area. Outer land and aquatic starting inventories retain their previous rules. Nutrients arriving later through transport remain real transfers.
- `settlement_plot_hectares`: caps each town's accessible managed footprint, formerly 5,000 hectares. The smaller reservation contains proportionally less woodland, ore, clay, soil nutrients and water. Unclaimed material stays in the surrounding ecology; founding does not delete it. The existing labor limit still controls cultivated area. This is an accessible local catchment, not a new bound on the island's total land area.
- `crop_yield_scale`: attainable managed crop production relative to the previous potential. It reduces production before nutrient/water uptake and ledger accounting. It does not remove harvested stocks, reduce incoming sunlight, or change the wild vegetation shader. Climate, soil, tools, labor, crop seasons and drought still determine actual yields.

Archives save these settings. Missing fields in older archives restore the old values (1, 5,000 ha, 1), preserving their existing rules and inventory. The explorer applies abundance changes when generating a new planet; it does not retroactively drain existing stocks. The evaluator accepts matching `--island-phosphorus-scale`, `--settlement-plot-hectares` and `--crop-yield-scale` overrides.

## Experiments

Each trial uses seeds 17, 81 and 256, 64 cells per face for terrain and ecology, one geological epoch, sixteen founders and 100 years of living history with politics, shipping, expeditions and discoveries enabled. Drought and storm settings remain unchanged. This gives twelve trial histories, compared with the preceding three-seed baseline. All are diagnostic-resolution simulations; overlapping jobs are not performance benchmarks.

The first trial reduced phosphorus to 0.25 and plots to 160 hectares. It reduced managed phosphorus stocks substantially, but food reserves remained full and no shortages occurred. A 60-hectare trial introduced some scarcity (four food crises on seed 81, one on seed 256), but constraining land alone was insufficient across all seeds. The next two trials retained 160-hectare plots and compared attainable crop yields of 0.65 and 0.50.

Lower phosphorus by itself barely changes standing wild plant carbon in this interval: phosphorus still exceeds plant demand in many places. The change reduces source inventories; it does not establish a phosphorus-limited wild ecosystem or create barren islands. Longer ecological histories and geological supply calibration remain necessary for that separate outcome.

## Verification

GPU checks compare initial inventories at matching and coarse ecology resolutions: island phosphorus decreases, C/N are unchanged, and cells containing no island land retain identical starting inventories. Budget checks cover subsequent transfers. Configuration tests reject out-of-range/nonfinite settings and verify old-archive defaults. Seed histories validate their ledgers annually.

The existing plot-withdrawal fixture explicitly uses its former 5,000-hectare diagnostic withdrawal: subtracting two whole-planet float32 inventories cannot accurately measure a tiny local nitrogen withdrawal to within 0.5% of that withdrawal. No solver budget tolerance was relaxed. The smaller real-world plots are covered by annual history and ecology ledgers in the seed trials.

## Selected defaults and results

```toml
island_phosphorus_scale = 0.25
settlement_plot_hectares = 160.0
crop_yield_scale = 0.50
```

The selected trial halves century-total crop production (49.1–50.1% of the prior baseline), reduces food losses to 26.3–27.6% of baseline, and leaves 21.3–21.8% as much managed soil phosphorus. Population is modestly lower, with multiple towns, commerce and expeditions still active. This is reduced surplus with occasional scarcity, not a permanently starving archipelago.

| Seed | Population | Active towns | Food crises | Annual shortage site-years | Wars | Expeditions |
|---|---:|---:|---:|---:|---:|---:|
| 17 | 5,672 | 33 | 3 | 3 | 0 | 19 |
| 81 | 5,613 | 32 | 9 | 5 | 4 | 26 |
| 256 | 5,631 | 31 | 3 | 0 | 0 | 46 |

Year-100 reserves remain around 17–18 demand months: recovery and storage caps make the endpoint a poor measure of historical overproduction. Monthly crisis events can occur between annual samples, explaining seed 256's crises without an annual shortage observation. Wars are outcomes of existing grievances and logistics, not a quota introduced by this change.

Artifacts: [160-ha resource trial](../output/island-abundance-trial.md), [60-ha resource trial](../output/island-abundance-small.md), [65% harvest trial](../output/island-abundance-yield65.md), [selected 50% harvest trial](../output/island-abundance-yield50.md). Each prefix has JSON, full world archives, and `*-ideas.json` analysis. The previous baseline is [history refinement](history-refinement.md).

Reproduce a selected run:

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
target/release/examples/history_evaluate --seeds 17,81,256 --resolution 64 \
  --years 100 --civilizations 16 --island-phosphorus-scale 0.25 \
  --settlement-plot-hectares 160 --crop-yield-scale 0.5 \
  --discoveries --living-world --save-worlds \
  --output output/island-abundance-yield50 --label island-abundance-yield50
```

Use crop yield 1 for the first two resource-only trials, with plots 160/60 ha respectively; use yield 0.65 for the milder harvest trial. Those alternatives are retained rather than overwritten by the chosen defaults.

Two older non-seasonal civilization fixtures explicitly keep abundant settings because their purpose requires daughter towns and surplus relief donors. They start history directly from minimally initialized terrain; spontaneous expansion is no longer guaranteed under the new scarcity settings. Full generated, seasonal histories with the selected settings are evaluated separately above. Their conservation/checkpoint assertions were not relaxed.

The final settings were also tested with five founders (one per island): seed 17 ends with 1,497 residents and eight towns, seed 81 with 1,508 and eleven, and seed 256 with 1,499 and ten. They record six, six and five food crises respectively, with no wars. All three retain active trade and viable populations. These additional runs bring validation to fifteen century-long trial histories. [Sparse-founder report](../output/island-abundance-sparse.md).

The specimen-accounting fixtures also retain deliberately abundant supply settings so they can launch repeated voyages to the same finite outcrop. With the new defaults, one sponsor legitimately cannot afford that second expedition; changing the simulation to guarantee it would defeat the scarcity change. Its source-depletion, nutrient accounting and checkpoint assertions remain intact. New-default expedition affordability is exercised by the fifteen trial histories.

Final verification: all 62 distinct tests passed across the regression run and the remaining suites rerun after the explicit fixture calibration, including hardware GPU tests. Clippy with warnings denied and formatting checks passed. All fifteen trial histories completed with converged initial climate and passing annual budget checks. Maximum relative residuals were 2.29e−5 for managed history, 1.64e−5 for ecological C/N/P, and 4.71e−5 for ecological water. These results do not substitute for production-resolution or long-geological-history calibration. The new explorer controls were compile-checked, not exercised in a desktop interaction smoke test.
