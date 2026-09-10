# Floods during living history

Living history now connects monthly GPU storms and river overflow to damaged crops, spoiled stores, disrupted transport, expedition delays and recovery. Enable **Living world**, then use **Step** or **Evolve**. Town inspection displays local exposure, cleanup time and cumulative losses. Historical events record flooding, recession, recovery and transport interruptions, with causal links where available.

## Physical model

Storm occurrence is deterministic by region and social month. The archived economy catalog's `[weather]` settings default to `storm_probability = 0.04` and `storm_multiplier = 3.0`. Both farming and planetary water use the same rainfall forcing. Rain enters the water ledger; excess rain does not multiply photosynthetic potential without limit.

GPU river routing retains finite water and C/N/P payloads. Bankfull capacity is twice surveyed mean discharge integrated over a month. Excess routed water can enter local surface storage; its proportional nutrient payload enters regional ecological soil. Coarse ecological collection sums fine-cell transfers by area. Water above natural depression storage drains halfway back to runoff each month, in addition to existing evaporation.

For rivers above 1 m³/s whose spill surface is within 1 cm of terrain, a river corridor occupies 5% of the cell. Local exposure is areal-average surface water divided by that fraction. Overflow storage is capped at 1.5 m within the corridor; all remaining water continues routing. Other cells use their ordinary surface depth. This subgrid approximation lets rivers flood their surroundings without requiring an entire planetary cell to fill deeply. Actual water inventories remain area averaged.

## Historical effects

At 0.25 m local exposure a town floods. Severity rises to one at 1.5 m. Each wet month destroys up to 6% of stored food and 35% of standing crops, scaled by severity. Lost biomass transfers into managed detritus with its nutrients; it is also recorded as food spoilage. Existing hunger, migration and mortality systems respond to shortages. There is no additional arbitrary flood death roll.

Temporary flood cleanup takes three dry monthly steps and reduces available work and crop output during recovery. Another flood restarts cleanup. After twelve consecutive wet months, the town instead enters **persistent inundation**: its cleanup timer clears, waterlogging still constrains work and growth, and it cannot sponsor new daughter villages. Twelve consecutive dry months clear this classification. This preserves physical water rather than forcing perennial wetlands to drain. Food crises and harvest events can refer to the flood or transport interruption that preceded them.

Initial GPU settlement surveys and monthly expansion screening use the same local exposure as flood damage. A candidate observed at or above 0.25 m is removed from the historical expansion list, even if it later dries. This conservative remembered-risk rule prevents expansion back onto known flooded sites; it is not yet a probabilistic flood-return-period model. Existing towns are not automatically relocated.

Roads close when their surveyed path floods. Ports close for flooded approaches or insufficient water depth; sea lanes close for insufficient depth. Recovery clears the environmental closure independently of the owner's open/closed policy. Alternative available roads can still carry trade.

Due cargo waits while its physical connection is blocked. Each blocked monthly step spoils 20% of remaining food; other goods remain intact while waiting. At six blocked months the journey terminates and remaining cargo is written off. Spoilage and write-offs are declared exports from managed nutrient inventories; goods enter the used/waste ledger and food enters the spoilage ledger. Nothing is credited to inaccessible destination soils. Purchase money was transferred at dispatch: the buyer bears the loss, with no automatic refund or minted compensation. Events distinguish spoilage, successful recovery and terminal loss. Legacy relief shipments use the same food spoilage and six-month termination rules.

Inland approaches to shipping ports are checked too. Returning expeditions wait offshore during harbor closures and continue consuming provisions and wages. Flooded outer camps face greater hazards and suspend specimen collection.

## Persistence and evaluation

Flood state, wet/dry streaks, persistent inundation, recovery timers and cargo delays are serialized with history. Missing fields default for older archives. The first resumed step reconstructs uninterrupted exposure from an existing flood event when streak counters are absent; an already overdue blocked contract terminates immediately instead of receiving another six months. Validation checks finite losses, valid historical references and bounded timers. Checkpoints retain the existing complete-month requirement.

```sh
mise exec rust@1.89.0 -- cargo run --release --example history_evaluate -- \
  --seeds 0,7,42 --resolution 64 --epochs 1 --years 100 \
  --civilizations 5 --discoveries --living-world \
  --output output/flood-complete --save-worlds
```

For controlled comparisons, add `--storm-probability 0.2`. Reports include flooded site-months, losses, closures and delayed deliveries. See [measured results](flood-history-results.md).

The subsequent [audit fixes and verification](flood-fixes-results.md) cover settlement screening, persistent inundation and bounded cargo losses.

## Limits

This is monthly regional overflow, not a daily hydraulic wave simulation. Surveyed mean discharge remains the reference capacity. Geological terrain, river topology, shorelines and shared lake surfaces remain fixed during history; floods do not yet move sediment elevations or channels. River nutrient deposition benefits regional ecology, not managed farm inventories directly. Managed farm runoff remains a declared export.

Cargo write-offs abstract abandonment or loss outside managed settlements; they do not simulate salvage, insurance or a return voyage. The 20% monthly food decay and six-month deadline are game rules, not calibrated preservation models. Levees, drainage works, relocation and abandoned-farm succession remain future work. Existing historical roads, ports and claims remain valid records when inundated; fresh expansion uses local exposure and remembered exclusions.
