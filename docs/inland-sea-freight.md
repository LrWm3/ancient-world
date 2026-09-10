# Sea cargo also occupies inland transport services

A harbor's available boat capacity no longer lets goods bypass a town's inland
freight limits. Planned cargo reserves the origin and destination services. Sea
cargo additionally reserves services at both ports, including a port used by another
inland settlement. Each distinct site is counted once even if it is both the seller
and the port. Ordinary land cargo and sea approaches compete for the same existing
`production.land_freight_kg_per_person` allowance.

Supplier selection rejects unavailable origin, destination or approach capacity
before comparing prices. Sea quotes can choose another initially available harbor.
Commitment takes the minimum of stock, money, storage, harbor capacity and all
participating inland services. Each accepted shipment reduces the quarterly working
allowances immediately; later orders cannot reuse those carriers. Continuing cargo
keeps its reservations across market quarters and checkpoint continuation. Delivery
or loss releases the reservation through removal of the actual cargo.

The calculation derives service membership from the shipment's archived origin,
destination and sea-lane ports. No second goods stock or new cargo schema is needed.
Old planned histories intentionally acquire these additional approach reservations
on continuation. Legacy unplanned towns retain unlimited inland service. Harbor
assets, upkeep, weather closures and their existing capacity still apply separately.

Approach availability is computed once per quarterly quote pass, not by repeatedly
scanning all cargo inside the buyer/seller/lane loops. Monthly commitment uses the
updated working allowances. Quotes are not a multi-round routing optimizer: a route
which fills during allocation is not automatically replaced by the next-best harbor
within the same buyer/good order.

## Checks

A controlled market fixture uses a 1,000 kg harbor and a 3 kg inland service. Existing
land freight can occupy that service and block sea trade despite empty harbor space.
With the service free, a rural seller ships only three kg to the other island and
reserves capacity at all four participating sites. Later quarters cannot renew it;
arrival releases it and goods/cash remain accounted for. Serialized continuation
matches exactly. A direct port-to-port fixture fills the same three kg allowance
with two different goods, proving that duplicate site roles are charged once.

The ordinary shipping integration test still surveys connected water paths, funds
real port assets, delivers cargo, closes lanes, and verifies checkpoint continuation
and the full ledgers. The controlled reservation fixture deliberately prescribes
its road/sea geometry to isolate arbitration.

## Boundaries

This is a shared inland service allowance, not a population of named carriers.
Reservations last for the entire shipment, conservatively coupling approach and sea
travel; crews cannot yet become free after handing cargo to a boat. Road segments
between sites, cart/animal assets, wages, freight pricing and explicit return legs
remain open work. Cargo mass lost to spoilage releases proportional capacity under
the existing convention. The historical calibration ensembles published before
this change do not include these new approach reservations.

The updated seed-409 century run completed with 2,791 people, 23 active sites,
31,719 deliveries and maximum relative ledger residual 1.95e-5. The previous model
had 2,758 people and 31,469 deliveries under the same scenario. These are nonlinear
trajectory differences, not a claim that constraining transport improves productivity.
Only one saturated annual observation was recorded in each default-capacity run;
the controlled three-kg fixture demonstrates the immediate bottleneck explicitly.
Social/validation wall time was 48.5 seconds versus 47.2 previously, and production/
readback was 25.9 versus 26.4 seconds. Shared workloads preclude treating those small
differences as measured regressions or improvements.

[Artifact retention policy](evidence/README.md).
