# Demand-driven town production

New worlds enable `[production]` in `assets/economy.toml`. Missing settings in old archives retain continuous production. An existing world can opt in through the existing `configure_economy` API or economy-catalog CLI option, without deleting or reinitializing stocks.

The monthly CPU town planner makes bounded recipe orders from useful reserves, six months of herd feed, seeds, active specimen-research supplies, and ingredient dependencies. Stocks and incoming cargo satisfy requests before new work is scheduled. Knowledge gates apply before ordering upstream ingredients. Scrap recipes require available scrap. The GPU performs all actual farming, extraction, crafting, wear, and nutrient transfers. Recipe scheduling rotates monthly to avoid permanent catalog-order priority; available labor and materials still bound completion.

Wood, ore and clay extraction now stop at outstanding demand. Industrial crops stop growing when standing biomass and stores cover their orders. Food cultivation remains governed by existing food, water and nutrient constraints; scarce-inner-continent yield settings are unchanged.

A shared dry-goods capacity defaults to 100 kg per resident, independently of food granaries. Cargo reserves its destination space. Processing that reduces occupied space can continue in an overfull yard. Existing timber, mineral and industrial-crop stocks survive capacity reductions and imported legacy overstock; the limit throttles new production rather than deleting it. Incidental hides and wool beyond local needs enter recorded compost. This is regional storage capacity, not yet a system of individually constructed warehouses.

Low ore stocks no longer trigger the legacy mining rush in planned economies: lean inventories must not divert farmers from subsistence. Crop yield and ecological limits are unchanged.

Useful quantities of tools, pottery, cloth, leather and military goods undergo use-related wear. Organic wear enters detritus; recoverable metal enters scrap with losses. Bricks in storage no longer decay. Road investment stops at 1,000 kg, the existing maximum useful improvement. Equipment-in-use is a regional population allowance, not yet individual ownership or squad equipment assignment.

Planned markets buy food first, use scarcity prices for food, and buy inputs and useful reserves rather than a quota of every good. Land shipments share [finite in-transit capacity](land-freight-reservations.md) at both endpoints (20 kg per resident), reserved across quarters until arrival or loss; sea shipping retains port capacity. Markets reserve cash, stock and incoming cargo. Freight is capacity-constrained but is not yet a paid carrier industry.

The inspector shows targets, storage capacity and unused craft labor. `History::production_summary` and annual history evaluation samples expose inventories and capacity. The planner deliberately reports spare labor rather than pretending all allocated workers performed useful work. New worlds also use [adaptive industrial staffing](adaptive-industries.md) to release workers from blocked or satisfied industries. [Regional workshop capital](workshop-capital.md) now constrains industrial work through finite assets and upkeep. [Funded repeat export contracts](export-contracts.md) now connect customer demand to production. Private firms and profit-driven investment remain future work.

## Validation

`cargo test --tests -- --include-ignored --test-threads=1` includes GPU conservation, markets, archive continuation, a controlled overfull-warehouse fixture, and sparse planner tests. Run history comparisons with:

```
cargo run --release --example history_evaluate -- --seeds 17,81,256 --years 200 --discoveries --living-world --save-worlds --output output/demand-economy
python3 scripts/economy_report.py output/demand-economy.json --baseline output/patron-complete.json
```

Add `--legacy-production` for the continuous-production control. Existing baseline archives retain the old setting. Do not interpret bounded stock alone as success: inspect shortages, population, tool sufficiency, trade and conservation together.

Completed multi-seed comparisons, five-century inventory measurements, calibration fixes and validation details are in [the results report](demand-economy-results.md).
