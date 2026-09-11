# Inter-island shipping

See [the latest memory, fleet and research extension](memory-fleets-and-research.md) for subsequent changes.

Shipping connects the central-island economies through actual great-lake routes. Enable **Lake shipping** in the civilization controls after enabling social history, or explicitly opt in from the CLI:

```sh
cargo run --release -- --headless --load output/civilization-mild.world --epochs 0 \
  --shipping --history-years 100 --save output/civilization-shipping.world
# Fresh paired experiments, with the same weather and market rules as the previous controls:
cargo run --release --example history_evaluate -- --shipping --civilizations 5 \
  --output output/history-shipping-five --label shipping \
  --compare output/history-tuned-five.json --save-worlds
```

The reusable API provides `enable_shipping`, `set_sea_lane_open`, `History::sea_quotes`, and `History::sea_capacity`. The history inspector lists harbors, construction inventories, free transport capacity and lane closure controls. Shipping state, paths, material stocks, cargo reservations and dated events are archived. Existing worlds stay unchanged until explicitly enabled; archives without the optional shipping section load with shipping disabled.

## Geography and assets

Each island receives one regional harbor. Among newly surveyed settlements, those with the strongest available construction reserves are considered first; a candidate must also have reachable coastal access. This lets mature worlds use viable daughter towns instead of automatically choosing an exhausted founding settlement. Access follows dry central land to the nearest reachable great-lake cell within 2,000 weighted travel km. Pairwise sea surveys use [GPU frontier searches](gpu-navigation.md) over adjacent great-lake cells, including cube-face seams, with a 20,000 km physical limit. They cannot cross land, the enclosing continent or exterior ocean. Surveying runs when shipping is enabled and for newly founded settlements each year. Completed routes remain sparse CPU history records; pathfinding and current-water route checks run on GPU. CPU Dijkstra remains an explicit diagnostic option.

A commissioned harbor holds 200 kg timber, 10 kg tools and 100 kg masonry. Materials transfer from its settlement's real stockpiles above civilian reserves; commissioning waits until all are available. Annual wear consumes 2% of held materials, records disposal in the conservation ledgers, and attempts replacement from available goods. A harbor supports up to 1,000 kg of cargo in transit, scaled by its weakest material reserve. All lanes sharing a harbor compete for that capacity. Both endpoint harbors reserve the shipment until arrival; new contracts cannot overbook them. Maintenance can reduce capacity while grandfathered contracts finish.

These are deliberately small regional harbor and pooled merchant assets, not full-scale ocean ports or individually simulated ships. There are no separate crews, vessel locations, return voyages, piracy, wrecks, tariffs or freight wages. Maintenance is a material cost, not a newly invented monetary payment. One harbor per island is an initial constraint: an abandoned host loses service and currently has no automatic replacement.

## Commerce and diplomacy

A shipment uses surveyed inland roads, the harbor approach, one sea lane and roads at its destination. Travel is 150 weighted inland km/month and 600 physical sea km/month. Market range uses total land-equivalent distance, subject to the archived market setting (bundled default 3,000 km). Supplier selection still uses real surplus, prices, buyer money and competing demand. Shipping adds no productivity or population multiplier.

Closed markets, abandoned ports and closed lanes stop new contracts. Wartime restrictions apply to endpoints and inland transit, including the original merchant administration's enemies after landing. Cargo already underway completes its paid contract after closure, consistent with existing land-market semantics. Reservations and actual cargo inventories survive save/resume; payment happens once at dispatch.

Successful sea deliveries produce `sea_arrival` and ordinary `market_arrival` records. Existing diplomacy therefore learns from actual trade; reachable sea contact can support non-aggression treaties. Trade does not establish an invasion or settlement route: warfare and relief remain subject to their existing land-route rules, and settlement geography remains confined to the central islands.

## Validation and interpretation

Tests cover shared port capacity, closed lanes and markets, worn-out assets, hostile inland transit after landing, actual GPU-produced sea shipments, arrivals after closure, restored service, invalid archived paths/assets/cargo, conservation, and exact same-backend checkpoint continuation with different dispatch batches. All generated route cells are validated against the world and cross-face adjacency.

The five-seed century comparison is in [the generated shipping report](../output/history-shipping-five.md). JSON additionally retains annual ready-port counts and sea-delivery events. Ports may take decades to accumulate materials, and some never commission during the interval. Shipping creates an opportunity for contact; it does not impose treaty, population, trade-volume or war targets. Higher resolutions and denser founding populations are separate comparisons, not assumed equivalent worlds.
