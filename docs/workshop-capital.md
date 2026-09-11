# Regional workshop capital

New towns must invest materials and labor to expand industrial processing capacity. This builds on demand orders and adaptive staffing. Workshops are communal regional assets, not yet individually owned businesses or placed buildings.

Each capacity unit contains 20 kg timber, 30 kg bricks and 2 kg tools, takes two worker-months to install, and supports four industrial worker-months per month. Construction can spend at most 10% of unreserved craft labor and cannot consume tools below the basic working reserve of 0.3 kg per resident. No money is created or charged to an imaginary contractor: this version uses the town's existing material inventory and allocated labor.

The target follows the smaller of outstanding industrial work and 125% of demonstrated work in the previous month, after subtracting household capacity. It is smoothed by 10% each month and capped at 0.03 units per resident. Construction materials enter the existing dependency planner and markets; actual GPU installation requires physical stock. The cap prevents construction demand from feeding unlimited reinvestment. There is no automatic grant of workshop materials at founding or when enabling the system in an old world.

Household equipment supplies a small industrial floor: the larger of one worker-month or 2.5% of available workers. Food-output recipes remain exempt from workshop capacity, but still require actual ingredients and labor. This allows survival and industrial bootstrapping without free workshops. Capacity is shared across industrial recipes, in the existing rotating work order. Research reservations remain protected.

Installed assets wear by 0.2% monthly while workshop production is enabled. Lost organic material enters detritus; recoverable tools enter the existing scrap loop. Repairs use the same material and labor path as construction. Buildings above current demand are retained and wear gradually rather than disappearing when the target falls. Installed material is excluded from warehouse stock but included in both goods and nutrient accounting.

## Controls and compatibility

`production.workshops = true` is enabled in the bundled economy catalog. Missing settings in older archives default to false; missing asset fields default to zero. Existing worlds can opt in using **Require workshop investment and upkeep** in the explorer or `configure_economy`. Disabling the control retains installed assets and suspends capacity limits, construction and wear, for comparisons.

Inspection shows capacity units, target, industrial work and cumulative wear. `History::production_summary` exposes installed materials and industrial worker-months. The history evaluator supports `--no-workshops` while preserving demand orders and adaptive staffing.

## Validation

GPU fixtures require construction to wait for missing materials, then install from declared inventories; verify wear and conservation; enforce the industrial capacity limit; and compare a twelve-month batch against monthly checkpoint continuation. Older catalog/state fixtures check default migration without granting inventories.

```sh
cargo test --tests --no-fail-fast -- --include-ignored --test-threads=1
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --output output/workshop-capital
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 100 --discoveries --living-world --save-worlds --no-workshops --output output/workshop-control
```

This page records the pooled-capacity increment. [Specialized workshops](specialized-workshops.md), [household payroll](household-economy.md), and [export contracts](export-contracts.md) are now implemented. Individually owned firms, general credit and paid carrier businesses remain outside this model. Investment follows bounded work and demand signals, rather than a complete firm profit forecast.

Calibration rejected a backlog-only investment rule: the first seed-17 pilot installed 87 workshop units (349 industrial worker-months/month) while using only 12.7 industrial worker-months in the final month. Seed 81 installed 151 units while using 80.1 worker-months. The pilot stayed financially and materially bounded, but grossly overestimated sustained demand. `output/workshop-capital-backlog-pilot.json` retains those results; the final rule limits expansion using demonstrated use.

## Final paired results

Quadro RTX 5000 Max-Q; diagnostic terrain/ecology edge 64; living environment, discoveries and scarce-inner-continent defaults; 100 years per seed. Both policies use demand orders and adaptive staffing.

| Seed | Population control → workshops | Shortage site-years control → workshops | Dry kg/person | Installed units | Tool sufficiency control → workshops |
|---|---:|---:|---:|---:|---:|
| 17 | 5402 → 5334 | 3 → 5 | 10.45 | 2.84 | 99.8% → 97.7% |
| 81 | 5699 → 5729 | 5 → 5 | 11.07 | 12.37 | 98.2% → 98.8% |
| 256 | 5593 → 5643 | 11 → 7 | 10.49 | 2.15 | 98.4% → 98.6% |

Inventories exclude installed workshop material, which is separately accounted. Raw edible stocks and prepared food are not dry goods. Final installed capital is small: most towns remain agricultural and have limited industrial work. The system constrains expansion without requiring every town to build a large workshop complex. Persistent export customers and specialized facilities remain necessary to create more substantial industrial centers.

Maximum observed relative accounting residual was below 2.20e-5 in all final runs. Dry stocks per resident remained close to their year-50 values. These are whole-history comparisons: changes to production affect later trade, people and politics, so small population differences should not be interpreted as isolated direct effects. No three-century or larger-resolution capital calibration was performed in this increment.

Final histories took 48–51 seconds each with overlapping test workloads; these are elapsed observations, not isolated performance benchmarks. Results: `output/workshop-capital.json`, `output/workshop-control.json`, `output/workshop-capital-analysis.md` and their saved worlds.

Checkpoint replay from seed 17 at year 100 passed: a 24-month batch exactly matched monthly advancement with save/reload at month 12. History, terrain and ecology matched. Report: `output/workshop-capital-replay.json`.

Final verification: all 93 tests passed, including hardware GPU tests (none left ignored); Clippy with warnings denied and formatting checks passed.

The following increment adds [funded repeat export contracts](export-contracts.md). Private firms remain separate work; specialized capacity is linked below.

Subsequent work adds [cost-aware supplier quotes](supplier-quotes.md) and [industry-specific workshop capacity](specialized-workshops.md). The measurements above describe the original pooled-capacity increment.
