# Civilization beta

> **Archived design / legacy reference — 2026-09-09.** This preserves the original proposal and its historical limitations; it is not the current implementation status or an active task list. See the [current civilization guide](../civilizations.md) and [documentation index](../README.md).

**Current status:** new societies now use [managed farming, crafts and markets](../economy.md). This page describes the original version-one beta, which old saves can still resume until explicitly upgraded. The new model supersedes the food-proxy and frozen-farm limitations below.

Central-island societies now have a runnable history loop: found villages, grow food, consume stores, grow or lose population, send relief, establish daughter settlements, replace leaders, and record dated events. The enclosing continent remains uninhabited.

## Try it

In the desktop sidebar, open **Civilizations beta** at a completed epoch boundary and choose **Found five civilizations**. Advance one or ten social years at a time. The history window lists societies, leaders, settlements and searchable events. Selecting a settlement focuses both maps; amber atlas dots mark settlements and gray dots mark ruins. Ordinary world saves include civilization history. **Export history JSON** writes an inspectable companion file beside the configured world path.

```sh
cargo run --release -- --headless --resolution 64 --epochs 1 \
  --civilizations 5 --history-years 100 \
  --history-export output/history.json --save output/civilizations.world

# Continue an existing history without advancing the frozen environment.
cargo run --release -- --headless --load output/civilizations.world \
  --epochs 0 --history-years 10 --save output/continued.world

cargo run --release -- --load output/continued.world
```

`--civilizations` accepts 1–16 founders and is only used once per world. Founders preferentially occupy separate connected central islands, subject to suitable dry agricultural land. Failure to find enough sites is reported. Regenerating a world starts a new environmental and social baseline.

## Model and accounting

A GPU survey evaluates central land, climate, soil, water and elevation. GPU monthly kernels compute cultivated area, production, food consumption, spoilage, births and deaths. Sparse CPU graphs handle connected island identities, candidate selection, shipments, founding, leadership and events. The GPU does not run the historical record graph. Every new social step uploads and reads back the compact settlement table; terrain remains on GPU except for explicit eligibility validation snapshots.

Population is a continuous cohort measured in population equivalents, not discrete individuals. Each founder starts with 120 people and twelve months of food. Monthly requirements are 18 kg food per person; cultivation is limited by labor and available farmland. Climate-derived crop yield receives reproducible monthly variation. Food spoils at 1% per month. Shortages reduce births and increase mortality. Settlements below one population equivalent become ruins.

Annual decisions can send surplus food to needy settlements on the same island, with distance-based arrival delays and food reserved in transit. These are relief shipments, not priced market trade. Prosperous settlements can transfer 60 people and starting food into a nearby daughter settlement. Founding transfers happen at the annual boundary; migrant travel is not simulated yet. Named leadership records retain predecessor and death links, but stand for offices within the aggregate population rather than additional residents.

The food ledger reconciles initial stocks plus production against consumption, spoilage, stored food and cargo in transit. The population ledger reconciles initial cohorts, births and deaths; founding migration transfers existing population. Relative residuals are displayed and validation rejects errors above 0.1%. Stocks must remain finite and nonnegative. Transactional history advancement commits only after validation, and seeded decisions depend on simulated time rather than execution speed.

## Deliberate beta limits

The environmental snapshot freezes when civilizations are founded. Geological advancement, ecology advancement and ecological scenario changes are rejected thereafter. Social years have their own monthly clock. Farm production is an agricultural proxy with a food budget; it does **not** yet withdraw carbon, nitrogen, phosphorus or water from planetary ecology. Monthly yield variation is not a live weather or crop-season simulation.

Island connectivity derives from the terrain grid; this is not yet a canonical cross-resolution island identity. Routes use same-island membership and spherical distance, not roads or terrain pathfinding. Settlements are regional records, not generated buildings or excavatable local sites. There are no families, occupations, manufactured goods, money, inter-island shipping, diplomacy, warfare, religions or artifacts yet. These are tracked in the [civilization roadmap](civilizations.md).

Current bounds are 256 settlements, 2,048 separated founding candidates, 16 initial societies, 200,000 events, 1,000 years per advancement call and 10,000 years total. GPU allocation and individual buffer limits are checked before the survey. Sparse history uses host memory. Version-two world archives gain an optional version-one civilization section; older version-two worlds load without societies. The checksum includes history, and the maximum JSON header is 64 MiB. Invalid references, stocks, geography and provenance are rejected on load.

## Verification

Hardware tests cover central-only settlement eligibility, expansion and succession over a century, famine and actual shipment arrivals, food/population conservation, and exact same-backend continuation after checkpointing versus single-month dispatches. Run:

```sh
cargo test --test civilization -- --include-ignored --test-threads=1
```

A 256²-per-face terrain run on the Quadro RTX 5000 evolved five founders for 250 years into 250 settlements and approximately 29,566 people, with 245 founding migrations and 28 leadership successions. The food residual was below 0.0001%. This fertile baseline produced no relief shipments; the separate famine fixture exercises those. The run approaches the current settlement cap, so it is a beta scaling check rather than a calibrated historical population forecast. Desktop loading and rendering of the resulting archive also passed a screenshot smoke check.
