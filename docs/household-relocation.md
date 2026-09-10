# Household relocation v1

New histories with society enabled allow hardship-driven household relocation. The History window exposes an on/off policy and active journeys. `History::set_household_relocation` changes departure policy; `household_relocations` exposes pressure memory, journeys and retired lost-family records. Disabling departures does not cancel funded journeys. Older archives default to disabled, with empty pressure history, and can be enabled explicitly.

## Admission and travel

Quarterly decisions consider six shortage months in the last two years, or persistent inundation. An origin can send at most one household per year, must retain another resident household, and cannot send the current civilization leader. A journey carries 1–8 population equivalents, capped at a quarter of origin population, using proportional age cohorts. Sparse household/genealogy records are not a literal individual census.

Destinations must be existing active communities on the same central island, reached by a direct open road within ten months at 150 km/month. They require a year's observed food production, almost no recent shortage, spare productive capacity with a 5% margin, sufficient agricultural land, and six months of food reserves after admitting pending travelers. Hostile controllers and flooded destinations are excluded. Production capacity uses actual cohort dietary needs; using adult requirements for everyone prevented otherwise viable moves in the first calibration.

The origin supplies the entire journey plus three reserve months and retains one month of food for residents. Food, cash and tools move out of existing inventories. Travelers produce nothing, consume age-specific rations, and incur mortality if provisions run out. Closed roads delay arrivals. A failed destination triggers a return trip, with continued travel costs. Completely lost parties become inactive family records; their food/equipment losses and surviving financial estate are accounted for once.

Arrival preserves the household ID, genealogy links, original ancestry and faith. Local political faction membership follows the destination. Population and remaining supplies enter destination inventories. Beneficial private-stock shares are relinquished at origin and reassigned at destination; these shares are ownership claims, not additional goods. Buildings, unique objects and their physical custody do not automatically move. Departure, arrival, blockage, return and loss events reference the household; subsequent journey events link to the departure.

This is a limited voluntary escape path, not whole-town evacuation. There is no sea relocation, new settlement founding, route search through intermediate towns, moving government, or guaranteed rescue. Destitute communities may lack provisions even when conditions justify leaving. Town abandonment remains governed by the settlement lifecycle.

## Accounting and persistence

Traveling age cohorts, food, money and tools participate in the existing population, food, C/N/P, financial and material residuals. Journey consumption is charged to the originating community's consumption/external-exchange ledger. Arrival does not count as a new birth or production. Pressure memory, departure policy, journeys and lost-family records serialize at normal history boundaries. Cultural actions and household political participation exclude travelers and lost families.

## Paired evaluation

Hardware GPU runs: seeds 17, 81 and 256, 200 years each, yield **0.33**, living environment and expedition discoveries enabled. Terrain and ecology resolution 64. Other evaluator defaults are recorded in the JSON reports. Figures below compare relocation disabled → enabled; population excludes travelers (none remained in transit at the endpoints).

| Seed | Arrived households | Population at year 200 | Food-crisis episodes | Abandonments |
|---|---:|---:|---:|---:|
| 17 | 7 | 3,654 → 3,705 | 91 → 81 | 7 → 7 |
| 81 | 12 | 2,651 → 2,678 | 114 → 83 | 8 → 7 |
| 256 | 18 | 5,046 → 4,994 | 70 → 76 | 1 → 1 |

All 37 departures arrived on their original island. One journey was delayed by a closed route. There were no terminal losses or failed-destination returns in these ordinary runs; controlled fixtures cover those branches. The later terminal-loss retirement safeguard does not affect these runs. No post-abandonment local activity notices appeared in the lifecycle audit. The largest normalized accounting residual was 4.19e-5 (0.0042%); the population residual stayed below 4.4e-7 at the endpoints. Seed 17 includes a household leaving Iriwick for Farholm before Iriwick later becomes ruins.

Relocation helps some communities without guaranteeing higher population or fewer crises: seed 256 ends slightly worse. Changes to population and ownership also influence later trade, politics and history, so the endpoint differences are not a direct count of lives saved. Keep admission conservative for now; these results do not justify making every struggling town send refugees or weakening destination capacity checks.

Reports and saved worlds: `output/relocation-final/enabled.{json,md}`, `disabled.{json,md}`, and `{enabled,disabled}.SEED.world`. `lifecycle.json` records the abandonment audit. Reproduce with:

```sh
mise exec rust@1.89.0 -- cargo build --release --example history_evaluate
# Historical v1 settings: add --legacy-relief; also add --no-relocation for the paired control.
target/release/examples/history_evaluate --seeds 17,81,256 --years 200 \
  --crop-yield-scale 0.33 --discoveries --living-world --save-worlds \
  --label relocation --output output/relocation-final/enabled
```

The ignored hardware-GPU test `relocation_conserves_and_reserves_capacity_and_preserves_identity` covers insufficient destination production, capacity reserved for incoming households, insufficient provisions, funded movement, food/cohort/material balances, route closure and starvation, terminal loss, failed-destination return, cultural identity, and serialized in-transit continuation. The evaluator validates integrated history during long runs. `history_replay` separately compares a mature saved world's 24-month continuation against monthly execution with a checkpoint halfway through, including bitwise terrain/ecology equality.

Mature seed 17 replay passed: 24 months, checkpoint at month 12, identical history and bitwise-identical terrain/ecology (`output/relocation-final/replay.json`).

Validation: the full suite passed 102 tests including hardware-GPU tests; the expanded relocation capacity fixture also passed after its final addition. Clippy with warnings denied and formatting checks passed.
