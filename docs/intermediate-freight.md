# Intermediate towns constrain freight

Networked market cargo now reserves inland carrying services at every settlement
on its selected road path, including both sea approaches. A small junction can
limit through-trade even when seller, buyer and harbors have spare capacity.
This extends the existing finite freight allowance; it creates no extra goods,
carrier population, currency or material stock.

## Rules

The quarterly market constructs reservation footprints from stable commercial
predecessor trees. These use the same road costs, closures, abandoned-town checks
and wartime transit rules as commercial distance quotes. Trees are cached per
origin and origin administration during that pass, then reused across buyer/good
orders. Inland travel after a sea leg retains the original seller's transit
permissions. This is CPU work over the bounded settlement graph, not planetary
terrain pathfinding.

Every supplier must have at least one kg of free service at all required stops.
Accepted quantity is bounded by the smallest remaining stop allowance as well
as the existing inventory, money, storage and vessel limits. Allocation immediately
reduces working allowances, preventing later orders from reusing capacity.
Repeated stops and ports reserve once.

`Cargo.freight_stops` stores the sorted, unique **membership set**, not an ordered
geographic itinerary. It includes seller, buyer and relevant ports. Reservations
persist across market quarters, road closures, changed shortest paths and
checkpoint continuation. Spoilage reduces the reservation with actual cargo mass;
delivery or cargo loss releases it. Recomputing a route cannot silently move
already committed carriers to a different town.

Old cargo without this field retains its original endpoint/port footprint. We do
not fabricate the historical road path of a shipment already in transit. Newly
dispatched cargo gets the expanded footprint. Legacy unplanned towns continue to
have unlimited service capacity under their existing rules. Archive validation
rejects duplicate, invalid or missing endpoint/port entries in nonempty footprints.

## Limits

This reserves settlement transport services, **not individual road edges**.
Reservations conservatively cover the whole journey until delivery; there is no
per-leg release, cargo position, intermediate toll payment or carrier employment
market. Existing delivery/closure behavior remains unchanged. Capacity does not
yet trigger automatic shortest-path rerouting through a second-best junction or
harbor within the same order. An unavailable route can therefore leave usable
capacity elsewhere idle; another eligible supplier can still win the order.
Relief shipments retain their existing endpoint reservations.

## Checks

A controlled five-town fixture starts from seeds 17, 81 and 256. All receive the
same prescribed commercial topology and goods for an isolated mechanism test:

| Intervention | Tools dispatched | Observed bottleneck |
|---|---:|---|
| Seller → 3 kg junction → buyer | 3 kg | Intermediate service |
| Seller → 10 kg bypass → buyer | 10 kg | Endpoint/service allowance |

The fixture checks money and material totals, unrelated-town capacity, capacity
held across a second market quarter, and serialized continuation. Roads are
changed while cargo is in transit: the old junction retains its reservation until
arrival, then recovers its full three kg allowance. These are controlled
cause-and-effect checks, not evidence that natural seed economies are balanced.

Verification on the Quadro RTX 5000 Max-Q / Vulkan passed: all three freight
fixtures, eight market tests, the shipping conservation/closure/checkpoint test,
and 61 ordinary library tests. The ordinary run skipped 66 hardware fixtures;
the relevant fixtures above were run explicitly. All-target clippy and formatting
checks passed. This pass did not run a natural century-scale balance ensemble.
Raw outputs belong under ignored `output/`.

```sh
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --lib freight -- --include-ignored --nocapture --test-threads=1
CARGO_INCREMENTAL=0 mise exec rust@1.89.0 -- cargo test --test markets --test shipping -- --include-ignored --test-threads=1
```
