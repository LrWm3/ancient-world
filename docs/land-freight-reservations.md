# Land freight reservations

Planned economies now treat `production.land_freight_kg_per_person` as simultaneous
land cargo carrying capacity, rather than a fresh allowance at each quarterly
market. The bundled value remains 20 kg per resident. Long journeys therefore tie
up capacity across market quarters. Capacity is an abstract shared transport service,
not an additional inventory of goods or an explicit population of carriers.

For each settlement:

```text
installed capacity = population * land_freight_kg_per_person
reserved = sum(remaining cargo kg using this origin, destination or sea-port approach)
free capacity = max(installed capacity - reserved, 0)
```

Arrivals and weather losses settle before new market orders. Their removal releases
reservations; delayed cargo continues to reserve its remaining mass. Each new land
shipment reserves both endpoints, shared across goods and both directions. Sea cargo
also reserves its origin, destination and two port approaches, counting each distinct
site once; see [inland sea freight](inland-sea-freight.md). Shrinking
population can leave a town over capacity: existing contracts finish, but new orders
wait. Unplanned legacy towns keep their unlimited allowance. Existing archives need
no new fields: reservations are reconstructed from persistent cargo records. Planned
worlds resumed under this version intentionally receive the new capacity semantics.

Supplier eligibility now excludes land suppliers with less than 1 kg of free
endpoint capacity before price ranking. An exhausted cheap supplier can no longer
prevent an available alternative from serving the buyer. Existing contract preference,
price ordering, inventory reserves, buyer budgets, storage limits and stable ordering
remain in force. This is not a multi-round allocation optimizer: partial orders do
not automatically fill their remainder from a second supplier in the same quarter.

`History::land_freight_capacity(site)` exposes available capacity, and the selected
town inspector displays it. Unknown site IDs return zero. The query measures
capacity, not route accessibility; route, policy, war and flood eligibility remain
separate checks.

## Scope and limits

Sea shipments reserve both shared harbor capacity and inland endpoint/port approach
services. There are still no
intermediate-road bottlenecks, explicit wagons/crews, freight wages or return trips
in this change. Capacity scales with remaining cargo mass, so spoilage releases some
of it; it does not track a fixed vessel or wagon occupied by a ruined load.

The update intentionally makes sustained long-distance planned trade more constrained.
The factor of 20 is a game parameter; this pass does not claim calibrated historical
freight rates or measured multi-seed scarcity outcomes.

## Verification

A GPU-initialized, CPU market fixture holds production and geography outside the
comparison. A cheaper supplier's existing shipment consumes its entire allowance.
The buyer orders from the more expensive available supplier. A matched branch first
delivers the old shipment and correctly selects the cheaper supplier instead.

Two further quarterly markets cannot renew the occupied allowance. Serialized
continuation matches exactly. Population decline clamps free capacity without
deleting cargo; arrival restores availability. Tools are conserved exactly and total
private cash is conserved within 0.01 abstract currency. The fixture also verifies
legacy unlimited capacity and invalid-site handling. Its directly controlled arrival
date isolates reservation duration rather than testing the route travel-time model.

Existing market/shipping integration tests additionally exercise multi-hop routes,
closure, harbor reservations, actual production, conservation, and checkpoint
continuation. The inspector is compiled but not manually exercised.

```sh
mise exec rust@1.89.0 -- cargo test --lib freight_tests -- --ignored
mise exec rust@1.89.0 -- cargo test --test markets -- --include-ignored
mise exec rust@1.89.0 -- cargo test --test shipping -- --ignored
mise exec rust@1.89.0 -- cargo test --all-targets
mise exec rust@1.89.0 -- cargo clippy --all-targets -- -D warnings
```

Logs and hashes: [Artifact retention policy](evidence/README.md).
