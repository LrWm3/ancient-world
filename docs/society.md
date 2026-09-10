# Social history: seasons, households, routes and councils

This is an opt-in extension to economic history, available through **Enable social history** in the civilization sidebar or `--society` in headless mode. Existing histories retain their behavior until explicitly enabled. Enabling preserves food, population, materials and money, partitions residents into age cohorts, reserves planting seed from stored food, and records the new baseline. It does not reconstruct earlier family history.

```sh
mise exec rust@1.89.0 -- cargo run -- --headless --load output/refined.world --epochs 0 \
  --civilizations 8 --society --history-years 100 \
  --save output/civilization-social.world --history-export output/civilization-social.history.json
mise exec rust@1.89.0 -- cargo run -- --load output/civilization-social.world
```

To extend an existing economic history, omit `--civilizations` and load its archive. To resume a history that already has this layer, omit `--society` too.

## Model

- **Demography on GPU:** children, working-age adults and elders age between cohorts. Age-specific food needs, mortality, shortage-driven disease and reduced births affect actual population. Adults supply farm and workshop labor. Cohorts are continuous estimates, not individually simulated residents.
- **Seasonal farming on GPU:** production enters standing grain before an annual harvest. Hemispheres have opposite harvest calendars, production follows a smoothed seasonal curve, and sowing consumes reserved seed. Standing grain and seeds carry the same C/N/P as stored food and are included in conservation checks. Missing seeds limit subsequent production. This is one generic grain crop, not a crop-species or daily phenology model.
- **Household ownership:** named houses partition private site inventories through shares that sum to one. Their heads are representative historical figures drawn from aggregate residents; inheritance changes the head while preserving assets. Named deaths consume accumulated cohort mortality rather than killing a second resident. Governing houses pass their office to a successor. Heads promoted from cohorts have estimated birth dates. This is an asset-lineage foundation, not biological genealogy, marriages or independent household decisions; shares currently remain fixed.
- **Terrain routes:** sparse CPU Dijkstra searches connect each new site to up to three nearby prior sites on the same island. Paths follow adjacent dry central-land cells, with slope and river-crossing costs. Markets and relief use these routes when the layer is enabled. Route closure prevents new departures; cargo already traveling continues. Only direct surveyed links are used, without network itineraries, ports or inter-island shipping. Founding migration remains the older annual abstraction.
- **Councils:** annual taxes transfer private money to a finite public treasury. Councils return relief payments and buy actual bricks for roads. Road assets enter goods accounting and reduce travel cost; there is no automatic money or material creation. Councils are the first institution, without factions, elections or diplomacy.
- **Provisioned raids:** a food crisis in the preceding two years can motivate a raid against a richer, reachable foreign settlement. Actual adults and food leave the origin. Marching consumes provisions; starvation and fighting reduce real manpower. Loot transfers existing food. Survivors must travel home before their population and remaining supplies rejoin the settlement. This is a sparse raid model, not territorial warfare, tactical combat or an equipment-based army system.
- **Causal records:** food crises retain relevant prior local harvest/spoilage/closure references; raid departures reference crises, outcomes reference departures, and returns reference outcomes. These are selected modeled antecedents rather than a complete attribution of every event. Cause buttons follow references in the event browser; enter `#ID` to find an event or clear the search to return to the timeline. IDs also persist in exported JSON; validation rejects forward references and cycles.

The inspector exposes age groups, standing crops, seed reserves, hunger, household shares, successors, council funds, route controls and expeditions. Planetary ecology remains paused during social history; reserved managed plots continue their nutrient, water, farming and extraction cycles. Continuous landscape feedback, irrigation, crop/livestock diversity, full genealogy, political factions and wars remain further milestones.

## Persistence and verification

Social schema version one is embedded in the existing version-two world archive. All demographic buffers, household shares, councils, route paths/assets, expeditions and event references persist. Exact floating-point JSON roundtrips preserve shares and treasuries across save/resume. Invalid cohorts, calendars, ownership sums, paths, expedition IDs and causal references are rejected.

The GPU uses an additional 48 bytes per settlement for demography. Dense production and demographic updates remain compute shaders; sparse households, route search, fiscal transfers and expeditions run on CPU. Current limits remain 256 settlements, 16 civilizations and 200,000 events. Long advancement calls still block while monthly compact state is read back.

Tests exercise a century of seasonal production and inheritance, population/food/C/N/P/water/money/goods accounting, contiguous routes and closures, and controlled raids with real outbound and return legs. Checkpoint continuation matches both uninterrupted history and different advancement batch sizes, including a save with soldiers in transit.

## Representative run

On 2026-09-06, the optimized development build on the Quadro RTX 5000 Max-Q completed 100 social years from `output/refined.world` (seed 42, terrain 256², eight founding civilizations) in 15.63 seconds, including loading and output. The result had 24 settlements, 2,943 people, 248 households, 42 routes, 304 inheritances and 16 leadership successions. Maximum managed-budget relative residual was 8.58e-6; food and population residuals were below 5e-7. This well-provisioned sample had no food crises or raids; the controlled shortage fixture tests conflict separately. This is one timing observation, not a performance guarantee or a multi-seed calibration.

The optional [political extension](politics.md) now adds parent/child records, marriages, child inheritance, council factions, central-island territorial administration and campaigns. The limitations above describe the base social layer when that extension is disabled.
