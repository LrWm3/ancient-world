# Managed farming, crafts and markets

New civilizations use economic history version two. Settlements still occupy only dry central-island cells. The social simulation now reserves actual ecological inventories, grows nutrient-limited crops, extracts finite resources, crafts goods and trades them through timed shipments. The original founding, population, expansion, leadership and history systems continue to operate.

This describes the base economy. [Demand planning](demand-economy.md), [diversified farming and culture](patron-foundings.md), [household wallets](household-economy.md), and optional environmental coupling extend it; see the [current civilization guide](civilizations.md).

## Run and inspect

```sh
cargo run --release -- --headless --load output/refined.world --epochs 0 \
  --civilizations 5 --history-years 100 --save output/economy.world \
  --history-export output/economy.history.json
cargo run --release -- --load output/economy.world

# Explicit migration of a saved first-beta history; does not replay previous years.
cargo run --release -- --headless --load output/civilization-beta.world --epochs 0 \
  --upgrade-economy --history-years 10 --save output/upgraded.world

# Override recipes and price definitions; the actual catalog is stored in the archive.
cargo run --release -- --headless --load output/economy.world --epochs 0 \
  --economy-catalog assets/economy.toml --history-years 10 --save output/custom.world
```

Open **Civilizations beta**, found societies and advance years. Select a settlement in the history window, then open **Farming and workshops** or **Market and stockpiles**. Inspect nutrient and water stocks, the strongest crop constraint, worker allocation, resource depletion, goods, prices, treasury and budget residuals. Fallow, manure retention, rainwater storage and market closure are recorded site policies. The food-spoilage scenario returns retained nutrients to compost instead of deleting them from the accounting.

Old first-beta histories retain their old behavior until explicitly upgraded through the CLI or **Enable farming, crafts and markets** button. The migration preserves existing people, food, shipments and events, records an ecological baseline, and reserves plots once. Save/resume includes recipes, policies, managed stocks, their exchange ledgers, money and cargo.

## Inventory ownership and ecological coupling

Before founding, ecology is initialized if necessary. Each settlement reserves a managed plot capped by its terrain-cell farmland allowance and two hectares per initial resident. The GPU transfers a proportional share of the associated ecology cell's soil, detritus, surface plant biomass, geological phosphorus and water into absolute site inventories. The source ecology buffers lose those inventories and record equal exports. A serial GPU reservation pass resolves sites sharing a coarse cell in stable order. Subsequent reservations take their share of the remaining stocks; the model does not pretend to resolve heterogeneous soils within a coarse cell.

Managed stocks are in kg C/N/P and m³ water, rather than kg/m². Their area comes from the cube-sphere grid. Subsequent farming and resource extraction run on GPU over these compact plots. Crop production respects the configured solar-energy scale (zero sunlight means zero harvest). Solar crop production imports atmospheric carbon, consumes soil N/P and stored water, and places the harvest into the food stockpile. Food carries 0.45 kg C, 0.02 kg N and 0.003 kg P per kg. Consumption and spoilage respire carbon; the retained fraction of N/P becomes compost and the rest is an explicit export. This represents aggregate food processing and metabolism, not human body composition.

Compost decomposition returns nutrients to soil. Fallow fixation imports nitrogen, charging an energy proxy against crop production potential. Geological phosphorus release consumes a finite source inventory. Monthly rain enters the water ledger, crop water use leaves through evapotranspiration, and excess water leaves as recorded runoff. Policy changes alter storage capacity; they currently represent scenarios, not construction orders. Phosphorus, nitrogen and water each limit crop yield. Tools improve production and pottery reduces food spoilage. Fifty percent of the population is available as workers; farming, forestry, extraction and crafts share that labor budget.

Without [living history](living-history.md), the surrounding ecosystem remains paused while managed plots evolve. Living history advances seasonal ecology, weather and water alongside society; tectonics and terrain geometry remain fixed. [Diversified farming](patron-foundings.md) adds species-specific crops and livestock. Optional [environmental returns](environmental-returns.md) sends bounded farm runoff to the actual river network and releases abandoned plots back into ecology. When disabled, these exports retain the legacy sink behavior. On-demand regional terrain is not a canonical farm map.

## Materials and recipes

The editable, validated `assets/economy.toml` defines eight stable goods and five recipes. Quantities are kilograms, prices are abstract currency per kg, and recipe labor is worker-months per batch. This is the first material chain, not the roadmap's full crop/livestock/commodity catalog.

| Chain | Inputs | Outputs |
| --- | --- | --- |
| Charcoal kiln | 4 kg wood | 1 kg charcoal; declared carbon/nutrient losses |
| Smelter | 2 kg ore + 1 kg charcoal | 1 kg metal; processing waste and combustion |
| Smith | 1 kg metal | 1 kg tools |
| Brick kiln | 2 kg clay + 0.2 kg charcoal | 2 kg bricks |
| Potter | 1 kg clay + 0.1 kg charcoal | 1 kg pottery |

Timber comes from reserved ecological biomass with finite C/N/P. Ore potential becomes a bounded extractable deposit at reservation time; extraction depletes it. Clay reserves derive from plot area and weathered soil thickness. The first metal chain is a generic regional resource abstraction rather than mineral-specific assays or underground veins. These mineral/soil extraction stocks are distinct from ecological N/P. Workshops consume inputs once, share remaining labor, and stop stockpiling outputs above their target. Manufactured goods wear into recorded material sinks. Goods accounting reconciles initial stocks plus extraction/crafting against recipe consumption, wear, stores and in-transit cargo; recipe mass differences accumulate as processing waste.

Recipes cannot create primary resources, increase total mass or create carbon. Their order is a persisted dependency order: downstream workshops can use upstream output from the same month, but there is no repeated same-month feedback loop. Tools and money supplied at the initial baseline are declared imports. Daughter settlements receive existing money from their parent and do not mint new capital.

## Markets and history

Each quarter, open settlements post scarcity-based commodity quotes. Buyers reserve purchases from eligible same-island sellers within the archived market range (3,000 weighted km with the new network rules; 1,500 km in legacy catalogs). Buyers pay from finite treasuries at departure, sellers receive payment once, and goods enter cargo rather than the buyer's stockpile. Shipment capacity is bounded by the buyer's population. Arrival takes at least one month, using distance at 150 km/month. Incoming cargo is counted when calculating unmet demand. Food relief remains available separately from paid trade.

Without the optional [social history layer](society.md), markets use aggregate site treasuries and straight-line same-island distances. That layer introduces household shares, public treasuries, terrain routes and roads. Household income, institutions, roads, port trade and [inter-island shipping](shipping.md) are implemented by the linked extensions. General debt markets, transport fuel and blockades along individual edges remain outside this base model. Closing a site's market stops new contracts; already-departed cargo still arrives. Price changes, scarcity and resource depletion produce uneven local economies, while sales and arrivals enter the dated history. Optional social history, politics and governance provide families, factions, territorial wars and diplomacy; [Patron foundings](patron-foundings.md) provide religions, institutions and artifacts.

## Validation and limits

History validation includes food, population, managed C/N/P, water, money and commodity ledgers, with a 0.1% relative error threshold. Ecological source exports are independently checked by the existing planet budget report. GPU history work uses scratch ecological state; the history and modified ecology commit only after validation. Different dispatch batching and checkpoints must produce identical continuation on the same backend.

Tests cover:

- Area-weighted plot reservations, including settlements sharing a coarse ecology cell.
- A GPU recipe compared with a fixed input/output fixture and invalid catalogs.
- Zero sunlight stopping harvest despite stored nutrients; zero accessible phosphorus stopping crops, followed by manure-supported recovery.
- Market closure, real payment, cargo remaining unavailable until arrival, and continuation across transit checkpoints.
- Century-scale founding, expansion, famine, succession and the previous ecological/hydrology regression suite.

The beta retains the 256-site and bounded-history limits. One advancement call allocates an additional scratch ecology buffer and runs blocking monthly GPU readbacks; long runs can pause desktop interaction. The next scaling pass should keep this engine resident and schedule bounded monthly batches. No CPU fallback is used for farming or production.

## Integrated sample

On the Quadro RTX 5000, a 256²-per-face planet evolved five founders for 250 social years into 77 settlements and approximately 18,327 people. The run recorded 5,929 paid sales, 5,924 arrivals, 127 relief shipments and 28 successions; all managed ledgers passed validation. A separate original-beta save with 250 settlements upgraded and continued ten years successfully. These are functional calibration samples, not fixed historical growth or performance targets.

## Regional weather and commercial networks

The bundled catalog now includes archived `[weather]` and `[market]` settings. New markets can traverse intermediate open settlements, retain commodity-specific reserves and select suppliers using price and travel distance. Regional weather alters actual GPU rainfall and crop potential. See [History evaluation](history-evaluation.md) for rules, controlled comparisons and limitations. Catalogs without these sections keep the original settings.
