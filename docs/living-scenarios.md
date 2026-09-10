# Ecological interventions during living history

`Generator::scenario(region, intervention)` now accepts civilizations when living
history is enabled and both simulations are at a completed shared monthly boundary.
It validates the living clock offset, unfinished-month marker, and pending managed
returns before applying a change. Frozen histories must first call
`enable_living_history()`; invalid requests leave ecological buffers and event logs
unchanged.

Interventions apply immediately at that boundary, before the next monthly update.
They do not advance either clock. This is not a future-event queue: requests made
during an incomplete month are rejected. The existing geochemical-supply, guild
removal/restoration, and lake-mixing controls use this same path in the explorer.

Each ecological scenario records its ecological month, optional history month and
history event ID. A corresponding `ecological_intervention` history event describes
the change. Archives validate the reference and clock offset. Older scenario records
retain absent history references; no historical events are fabricated on import.

Removing a guild transfers its C/N/P into detritus and suppresses recolonization.
Aquatic guilds in water-dominated cells enter aquatic organic matter (pool 22);
other cases enter terrestrial detritus (pool 18). Restoration clears the suppression
mask; it does not create replacement animals.

## Fishery control and experiment

`AgricultureCatalog::fisheries_enabled` defaults to true, including old archives.
Disabling it stops managed harvest while wildlife continues evolving. The explorer
exposes this under economic settings. Disabled or disconnected fisheries clear their
receiving-water address and area before production; fishery labor reservations are
then released. Thus this is a harvest-system ablation, not a claim of identical
labor allocation across enabled and disabled worlds.

Run:

```sh
mise exec rust@1.89.0 -- cargo test --test living_scenarios -- --include-ignored --test-threads=1
```

The experiment writes `output/living-scenarios/fishery.json`. It uses terrain 64,
ecology 16, sixteen founding groups and seeds 17, 81, 256. A declared diagnostic
inventory replaces animal stocks with guild 10 C/N/P at 0.1/0.012/0.0015 kg/m²
in every cell. This artificial fixture isolates one food source; it is not a
naturally assembled ecosystem or an empirical calibration.

After two frozen history months, each world enables living history, checkpoints,
and branches into baseline, guild removal, an absent-terrestrial-guild negative
control, closed fisheries, and removal with closed fisheries. All branches advance
one shared month. Catch is the difference in cumulative counters, not lifetime catch.

| Seed | Baseline new catch, kg | Catch after removal, kg | Baseline minus removal food reserve, equivalent kg |
|---|---:|---:|---:|
| 17 | 13.2343 | 0 | 5.8867 |
| 81 | 18.0468 | 0 | 8.0273 |
| 256 | 18.0468 | 0 | 8.0273 |

Negative controls match baseline catch and food reserves exactly. Both closed-fishery
branches have zero new catch and identical food reserves. Population is unchanged
across branches after one month. Fish contributes to the food inventory, but these
provisioned towns do not become hungry from losing this small contribution. Prices,
migration, famine and long-term nutritional consequences remain unproven by this test.

Across the fifteen branches, maximum absolute relative residuals were 1.14e-7
for the economic ledger, 1.69e-8 for ecological C/N/P, and 1.21e-8 for ecological
water. Planet-scale relative errors can conceal small local mistakes, so they
supplement the cell-level transfer assertions rather than replace them.

Tests also check removal C/N/P transfers cell by cell, aquatic waste destination,
ecological and economic budget tolerances, invalid-request atomicity, absent old
archive fields, and byte-identical GPU ecological continuation after checkpointing
and advancing three months in one call versus three individual calls. Restoring an
extinct guild without a surviving source does not generate biomass.

Recorded raw results and provenance are in [evidence/living-scenarios](evidence/living-scenarios/).
These are controlled integration results on Vulkan/Quadro RTX 5000 Max-Q, not a
cross-hardware portability claim. A [longer natural-stock experiment](natural-fishery-trajectories.md) now measures
catch, consumption, household access and recovery after regional removal. Local
runoff response and higher-resolution spatial controls remain evidence gaps.
