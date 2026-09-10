# History experiments and regional weather

`examples/history_evaluate.rs` runs reproducible, paired-seed experiments through the same generator API as the desktop. Each run creates a fresh planet, evolves its geology, founds civilizations, enables social history, politics and governance, then samples every social year. Runs are sequential on the selected GPU.

```sh
# Original market/weather rules as a control, with the current numerical fixes.
cargo run --example history_evaluate -- \
  --civilizations 5 --legacy-markets --legacy-weather \
  --label control-five --output output/history-control-five

# New rules, identical seeds/geography/history, with paired deltas and inspectable worlds.
cargo run --example history_evaluate -- \
  --civilizations 5 --label improved-five --output output/history-improved-five \
  --compare output/history-control-five.json --save-worlds

# Denser contact experiment: repeat both commands with --civilizations 16.
# Production-grid check: repeat both with --seeds 42 --resolution 256.
cargo run -- --load output/history-improved-five.42.world
```

Defaults are seeds `0,7,42,99,999`, terrain/ecology edge 64, one geological epoch, 16 founding civilizations and 100 social years. Resolution 64 is a diagnostic world, not a substitute for production-resolution evaluation. `--epochs`, `--years`, `--resolution`, `--seeds`, and `--civilizations` control the experiment. A comparison with different seeds, resolutions, geological history, social duration or founding counts is rejected. Either legacy flag can be used independently for an ablation experiment. `--drought-severity 0.9` reproduces the stronger stress setting; the actual setting is always included in the report.

`--shipping` explicitly enables the new harbor/sea-transport baseline. JSON records this flag, annual port commissioning counts and `sea_arrival` events. Commercial connectivity includes available sea quotes when enabled.

`--expeditions` enables shipping and finite outer-continent research/rescue voyages. `--expedition-hazard 0` gives a field-hazard control; values up to 5 permit stress tests. Actual expedition rules are recorded. Annual samples include crew away, knowledge, active voyages and outcomes; the Markdown expedition table separates returned vessels from rescued parties. Resident population excludes expedition crews, which remain included in conservation checks.

JSON includes archived market/weather rules, GPU name, experiment configuration, annual population, occupied sites, events, cross-border deliveries, changes of administration, unrest, tool sufficiency, commercial connectivity, drought exposure, all eight conservation residuals and elapsed time. Markdown provides per-seed summaries and paired changes. A partially completed seed suite is marked `complete: false`; conservation or validation failures stop the command with an error instead of being accepted as history. Times include planet generation, annual sampling/readbacks and optional archive output; they are end-to-end observations, not kernel benchmarks.

Deliveries are classified using administrators at annual observation boundaries. Changes of ownership within the year can affect attribution. Territorial transitions are also annual observations and can miss a reversal within a year. Shortage site-years and recovery count annual observations; recovery means a previously hungry site is no longer hungry and remains occupied. A settlement disappearing is not counted as a recovery. Event counts additionally retain monthly food-crisis records, migration, abandonment, faction changes, wars and treaties. Resident population excludes soldiers away on campaign; the population conservation check includes them. Household wealth inequality remains outside these reports because shares of pooled inventory are not independently evolving household economies.

## Changes being evaluated

New bundled catalogs enable commercial paths through intermediate settlements, using only existing surveyed roads. Every transit settlement must be occupied and have an open market. Hostile road edges and entry into an administration at war with the merchant's origin are unavailable. Roads shorten travel time. Buyers discount distant quotes when selecting a supplier, but pay the actual quoted price; no fictional shipping fee is added to the money ledger. Commercial distance is capped at 3,000 weighted km. Military and relief routes retain their direct-path rules.

Tool sellers now retain 0.75 kg/person, above the buyer's 0.5 kg/person target, instead of the original 3 kg/person. Other commodity reserves stay at 3 kg/person and food at twelve months. Dispatch reserves actual cargo and pays once; transit inventory remains in the conservation ledger and arrival uses the accumulated path length. Existing contracts finish after closure, matching the previous shipment semantics. Land trade remains aggregate commercial transport without intermediate handling fees or interception. Optional [inter-island shipping](shipping.md) adds surveyed harbors and finite shared sea capacity.

New bundled catalogs also enable multi-year regional drought forcing during social time. A deterministic hash of seed, absolute month and a spatial region selects dry regimes. Regions derive from normalized spherical coordinates in a three-dimensional lattice, so they can span cube-face seams. Nearby sites within a region share a regime; boundaries are discrete. The GPU multiplies both rainfall and crop potential by the regional factor before finite soil/water constraints. Ordinary monthly variation remains active.

The bundled default has a 20% dry-regime probability, a 50% reduction and 48-month regimes. It was reduced from the initial 90% stress experiment after testing continuation of a mature world. Consecutive dry regimes can prolong a drought. These are configurable game-history parameters, not an empirical climate calibration. They expose food storage, trade, relief and existing political responses without prescribing a number of crises or wars. Global planet climate, hydrology and ecology remain paused while managed plots evolve.

`regional_drought` and `weather_recovery` events describe forcing changes. Harvest and food-crisis events link to their antecedents, and existing grievances can then link to wars. The People and institutions inspector shows the latest GPU rainfall/crop multiplier. Social age cohorts are now authoritative for resident totals; this prevents separate floating-point integrations from diverging after population collapse. The fourth cohort component holds the weather diagnostic and is not redistributed as population during migration.

All market and weather parameters are editable in `assets/economy.toml`, validated, and archived. Missing sections deserialize to the original rules. Existing archives therefore keep their saved rules until the catalog is explicitly changed:

```sh
cargo run -- --headless --load output/civilization-governance.world --epochs 0 \
  --economy-catalog assets/economy.toml --history-years 100 \
  --save output/updated-history.world
```

Same-backend checkpoint and dispatch-batch continuation tests cover droughts. A zero-rain/zero-growth controlled fixture checks that drought affects actual GPU budgets. Commercial fixtures check intermediate paths, road improvements, closed markets, abandonment, hostile transit, a real multi-hop shipment and arrival after closure. The existing century political fixture also exercises severe population decline and cohort consistency.

## Interpretation

The first experiment already corrected the earlier single-world diagnosis: original rules produce substantial cross-border trade and treaties when sixteen founding civilizations share islands. With five founders, the selected worlds put civilizations on different islands and produce no international road trade. More road reach does not bridge the great lake. Trade counts can decrease under the new supplier selection because nearby stock is preferred; more transactions are not itself a quality target.

The weather experiment tests whether finite reserves produce varied stress, recovery and political consequences. It does not establish realism from the presence of wars. With optional shipping now implemented, long-distance relocation/refugees, independently changing household wealth, global environmental feedback and occupation reconstruction remain substantive next steps.

Measured results and interpretation are in [History evaluation results](history-evaluation-results.md).

`--discoveries` implies expeditions and adds the finite specimen baseline. Annual JSON records collection, study, processing, remedy use/spoilage, farm phosphorus, workshop labor and three additional conservation residuals. The Markdown report adds a specimen-application table. Compare against an expedition-only report with the same seed/geography/history settings to measure the incremental effects.
