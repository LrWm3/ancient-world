# Shared road-corridor freight capacity

Networked market dispatch now reserves undirected settlement-to-settlement road
corridors as well as town carrying services. Opposing shipments and different
goods compete for the same corridor. This is a carrying-capacity proxy, not a
simulation of traffic, road width or individual wagons.

## Monthly boundary and finite claims

The quarterly market reads completed road condition and existing cargo. It builds
road footprints from the same predecessor trees as town stops, including inland
approaches on either side of a sea voyage. Joining the approaches does not invent
a road across the lake. Each unique corridor's free capacity is calculated once
per market window. Supplier eligibility checks both road and town availability;
accepted orders immediately subtract from both working allowances before later
orders compete. Existing buyer/good priority remains unchanged.

For roads with maintenance state, the initial capacity rule is:

```
kg in transit = min(endpoint gross carrying allowances)
                × (0.5 + 0.5 × clamp(road_bricks / 1000, 0, 1))
```

This is an explicit game-balance assumption. An unimproved road retains half the
endpoint carrying ceiling; a fully improved road reaches it. Road weathering and
repair therefore affect later cargo allocations, in addition to their existing
travel-cost effect. Town allowances still enforce actual carrier availability.
A closed or flooded corridor admits no new cargo. Legacy roads without upkeep
keep unlimited corridor capacity, while existing town limits remain in force.
Parallel roads between the same settlements share one corridor, using the best
passable surface rather than summing capacity: route trees identify predecessors,
not individual parallel-road identities.

`Cargo.freight_edges` stores sorted unique endpoint pairs, independently of route
array positions and the unordered town-stop set. Capacity claims persist until
cargo delivery/loss, and shrink with cargo mass after spoilage. Road replacement,
closure or a new shortest path cannot silently relocate an existing claim.
Capacity decline can leave existing cargo overcommitted; free capacity is then
zero, without deleting goods. Old cargo without an edge footprint is not assigned
an invented historical road route. Archive validation checks unique canonical
pairs, valid settlements and membership in the captured stop footprint.

## Remaining limits

Reservations cover the whole journey, not only its current leg. Captured-corridor flooding now delays due delivery, as described below. Relief,
armies and expeditions do not yet compete in this road ledger. Capacity exhaustion
does not search a second-best path to the same supplier. Corridor capacity has not
yet been evaluated in a natural long-run balance ensemble. Local road geometry,
spatial maintenance and year-round works scheduling remain open.

## Verification

The two focused tests pass, including the GPU fixture on seeds 17, 81 and 256.
With identical carriers and demand, unimproved roads dispatch 1.5 kg versus 3 kg
on fully improved roads. The controlled freight fixture compares identical
carrier budgets with unimproved versus fully improved roads, checks competing
reverse-direction orders, closure and surface-loss overcommitment, and saved
continuation through delivery. A separate footprint fixture checks that sea
approaches retain road edges without constructing a lake-crossing road. These
fixtures isolate mechanisms; they do not establish natural economic balance.

All eight market tests, shipping conservation/closure/continuation, and all three
history-environment tests pass, including gathered/full readback and
monthly/batched/checkpoint equivalence. The ordinary library suite passes 131 tests
with 113 hardware fixtures ignored; the relevant hardware tests above were
explicitly run. The road inspector displays finite remaining corridor capacity;
no visual interaction test is claimed.

Reproduce the mechanism and integration checks:

```sh
cargo test --lib freight::tests -- --include-ignored --nocapture
cargo test --test markets --test shipping --test history_environment -- --include-ignored
```

All-target Clippy passes with warnings denied. Repository artifact and diff checks
pass. Generated outputs stay under ignored `output/`.


## Captured-corridor flooding follow-up

Due market cargo now also checks its captured road edges. If all currently open
parallel roads on one captured corridor are flooded, the cargo remains in transit
under the existing monthly delay/spoilage/termination rules. An open route through
a different town does not silently replace its reservation. Reopening permits
delivery and releases the original footprint. This observation does not reserve
extra capacity or debit a second purchase; repeated observation within the same
month cannot add a second delay.

The existing endpoint/lane flood checks remain. Empty legacy footprints and removed
historical edges do not invent flood observations. This does not implement political
seizure, leg-by-leg vehicle position or capacity-aware rerouting. Future rerouting
must move the actual reservations before using the alternative corridor.

Both focused freight tests pass, including three seeds. The new scenario closes
a reserved edge while another route remains open, verifies that tools remain held
with unchanged goods/money and unchanged footprints, then reopens and delivers.
Serialized continuation is identical. All eight market tests, shipping conservation/closure/continuation, and all three
history-environment tests also pass, including monthly/batched/checkpoint and
gathered/full observation equivalence.
