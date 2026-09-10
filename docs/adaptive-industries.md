# Adaptive industrial staffing

Towns now reassign workers toward feasible work instead of reserving fixed industrial shares after those industries have filled their orders. This extends demand-driven production; it does not change crop yields, island fertility, market funding, or storage limits.

The GPU forecasts forestry, shared ore/clay extraction and recipes from orders, known techniques, existing materials, finite reserves and available storage. Shared ingredients are reserved once in the forecast. Feasible intermediate products can support subsequent jobs. Actual production still checks every material and labor limit; forecasts do not create inventory. Monthly rotation and gradual reassignment avoid permanent recipe priority and abrupt staffing changes.

Staffing moves one quarter of the way toward the desired allocation each month. Farming retains at least 62% of workers. Unneeded industrial workers return to farming, which remains limited by cultivable land, climate, water and nutrients. Previously reserved research and cultural labor is protected. This is regional staffing, not individual occupational retraining or wage competition.

Ore and clay previously could each consume an entire month's mining labor. Both now draw from a single extraction budget, including under the legacy controls.

New worlds enable `production.adaptive_labor`. Missing settings in archived catalogs preserve fixed staffing. The explorer's **Adapt workers to available jobs** control enables it for existing worlds without resetting inventories. Evaluation supports `--fixed-labor` to isolate staffing while retaining demand orders. Both policies include the corrected mining budget.

## Paired histories

Quadro RTX 5000 Max-Q, diagnostic terrain/ecology edge 64, seeds 17/81/256, 100 years, living environment, discoveries, scarce-island defaults. These are whole histories: changing employment also changes later demography and political opportunities.

| Seed | Population fixed → adaptive | Shortage site-years fixed → adaptive | Dry kg/person adaptive | Unused craft share fixed → adaptive |
|---|---:|---:|---:|---:|
| 17 | 2,589 → 5,390 | 68 → 5 | 10.48 | 92% → 42% |
| 81 | 2,928 → 5,702 | 1 → 5 | 11.10 | 77% → 46% |
| 256 | 4,428 → 5,600 | 7 → 12 | 10.56 | 94% → 44% |

Craft shares are final-month unused/allocated craft labor, not annual unemployment. Farming occupies 91–97% of final-month allocated labor in these adaptive histories. Raw edible inventories are excluded from dry storage figures; prepared food has its own ledger. Between years 50 and 100, dry inventories per resident rose only from 10.41 to 10.48, 10.94 to 11.10, and 10.37 to 10.56 kg. Maximum observed relative accounting residual across both policies was below 2.45e-5.

Adaptive staffing improves utilization and supports more people in these seeds, but does not eliminate scarcity. Shortages increased in two seeds, and tool sufficiency in seed 256 fell from 100% to 98.4%. The outcomes do not justify increasing island abundance. Spare craft capacity remains because staffing changes gradually, seasonal needs vary and the forecast can differ from actual completion.

Artifacts: `output/adaptive-staffing.json`, `output/fixed-staffing.json`, their saved worlds, and `output/adaptive-staffing-analysis.md`. Each adaptive run took 60–74 seconds; controls took 51–71 seconds. Runs overlapped other workloads, so these are elapsed observations, not isolated GPU benchmarks. Recorded adaptive history-stage totals were 19.7–22.1 seconds for social processing/validation and 13.2–14.9 seconds for production/readback. Social processing remains the largest of those measured history stages; these timings do not isolate the staffing shader cost.

Reproduce:

```sh
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --output output/adaptive-staffing
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --fixed-labor --output output/fixed-staffing
cargo test --tests -- --include-ignored --test-threads=1
cargo run --release --example history_replay -- output/adaptive-staffing.17.world output/adaptive-staffing-replay.json
```

Regression fixtures cover shared mining labor and idle industries releasing workers while preserving overstock and accounting. Nonnegative labor is now part of state validation; the idle-industry fixture caught a floating-point normalization edge case near complete farming employment. The specimen and survivor rescue fixtures explicitly transfer existing tools between towns when needed, instead of assuming that a sponsor can immediately afford a second, larger expedition.

[Workshop construction and maintenance](workshop-capital.md) is implemented in the following increment; export investment responding to persistent customer demand and paid freight businesses remain future work. This increment makes worker allocation respond to real jobs; it does not yet represent those capital and ownership decisions.

Checkpoint replay from the adaptive year-100 seed-17 archive passed: a 24-month batch exactly matched monthly advancement with a checkpoint at month 12. Historical JSON, terrain and ecology all matched; report: `output/adaptive-staffing-replay.json`.

Full validation passed: 91 tests, including hardware GPU tests, with no ignored tests left unrun; `cargo clippy --all-targets -- -D warnings` and formatting checks passed.

## Three-century check

Seed 17 completed 300 years in 364.7 seconds with 23,318 people across 118 active towns, 62 cumulative shortage site-years and 130,237 market deliveries. Dry goods were 10.62 kg/person (10.46 at year 250), while population-weighted tool sufficiency was 95.3%. Maximum observed relative accounting residual remained 2.18e-5 or less. Files: `output/adaptive-staffing-long.json`, its saved world, and `output/adaptive-staffing-long-analysis.md`.

Population and settlement count continued growing: 33 active towns at year 100, 70 at year 200 and 118 at year 300. This is frontier expansion, not a demonstrated carrying-capacity equilibrium. The long run supports bounded inventory and finite accounting under growth; it does not establish stable population or successful balance across all seeds. No paired 300-year fixed-staffing control was run.
