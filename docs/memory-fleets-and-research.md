# Local memory, staffed fleets and research exchange

This increment connects five existing game systems. Coefficients remain game
rules, not historical measurements.

## Social memory and secular relief

Dated arrival testimony now influences relocation even when religious relief is
disabled. Actual relief outcomes build a directional recipient/donor memory:
delivered mass, successful and failed deliveries, latest event, and date.
Duplicate outcome events do not count twice. Repeated successful deliveries can
establish the existing mutual-aid custom.

A donor that previously received help from the requesting community receives a
bounded, decaying reciprocity contribution to willingness. This never overrides
war closures, safe food reserves, distance, or the requirement that a help-seeking
family actually arrived. Secular shipments reserve endpoint freight capacity
alongside commercial cargo; they can no longer bypass busy land logistics.
Memory remains stored in the existing cultural archive section for compatibility;
it requires cultural history, but not the religious-relief switch.

## Vessels and employment

New ports contain up to four named vessel records. They subdivide the existing
port timber/equipment inventory rather than minting additional hull material.
Installed materials and their existing wear/repair rules limit usable hulls.
Each vessel provides up to 250 kg of shared service capacity when funded with
0.25 worker-months from a resident household. Wages transfer actual town cash to
household accounts; reserved work competes with craft, research and enterprise
work and is excluded from municipal payroll. No cash or available work means
laid-up capacity. The shipping inspector shows identities, crews and wages.

These remain small, abstract fleet service units: there are no individual
ship movements, crew rosters, or cargo-to-hull manifests. Employment includes
standby and maintenance. Existing cargo obligations survive lost capacity;
new departures must fit remaining capacity. Older archives without vessel
records retain legacy pooled capacity.

## Discoveries

A workshop can copy an established specimen-processing method from another
living settlement over a passable, non-hostile direct route. Annual updates use
a snapshot, so knowledge crosses at most one edge per update. Copying costs
0.25 reserved worker-months and 0.05 kg writing material, transferred to the waste
and detritus ledgers. It records the source settlement and causal event.

Learned methods are separate from destructive-study mass. They permit processing
fresh specimens without repeating the original 1.5 kg study, but create no
specimens, remedies or phosphorus. Closed workshops cannot study or process.
This is direct-route research exchange, not a model of traveling researchers
or the shipping of documents.

## Archaeology and heritage

A recovered fragment can receive up to three institutional studies, at least
five years apart. A living leader of an operational local scholarly or religious
institution needs access to the physical object, 0.1 cultural worker-months and
0.05 kg writing material. Study competes with other cultural decisions.

Each reading becomes a separately attributed account and provenance event.
Co-located fragments permit a recorded comparison; similarity never establishes
common authorship or reveals the founding mystery. Earlier readings remain.
Lost, destroyed or inaccessible objects cannot be examined. No artifact material
is duplicated, and study supplies become local waste/detritus.

## Verification

Checked on the available NVIDIA Quadro RTX 5000 Max-Q, Vulkan, Rust 1.89.0.
Generated logs remain under ignored `output/`.

| Check | Outcome |
|---|---|
| Market routing and compatibility tests | 6 passed; 2 GPU tests not selected in this run |
| Ordinary library tests | 59 passed; 57 hardware tests remain explicitly ignored in this ordinary run |
| Crew payment fixture, seed 7, 64-cell faces | Passed: town/household money conserved, craft work reserved, port materials unchanged, unfunded fleet unavailable, fleet serialization preserved |
| Research exchange fixture, seed 7 | Passed: closed route and missing writing supplies block copying; successful copying consumes supplies/work without adding specimen mass or destructive-study credit |
| Heritage study fixture, seed 7 | Passed: no labor or lost object blocks study; paid readings preserve material and prior accounts; repeat study respects five-year spacing |
| Shipping integration, seed 7 | Passed: organic sea trade, reservations, closures and checkpoint continuation |
| Discovery integration, seed 7, four tests | Passed: finite mineral depletion, study before production, healthy-town treatment limits, rescue/loss transfers, checkpoint consumption |
| Heritage voyage integration, seed 7 | Passed: finite recovered fragment and checkpoint/batch equivalence |
| Relocation/relief integration, seed 17 | Passed after fixing delivery/custom event ordering; population, provisions, identities and reciprocal aid preserved |

The relief regression initially omitted the `mutual_aid_learned` event when the
new memory code registered the custom before the religious callback. Delivery
now completes its original callback first, and memory uses the explicit original
outcome ID. This preserves existing religious provenance and adds the equivalent
secular event.

An interrupted incremental build produced linker errors; rebuilding the library
tests with `CARGO_INCREMENTAL=0` resolved them. This was a build-cache failure,
not a passing simulation test.

These are accounting, intervention and continuation checks, not broad
multi-seed balance calibration. In particular, long-term affordability of
standing crews and the frequency of research transmission need wider playtesting.

Reproduce the focused additions with:

```sh
CARGO_INCREMENTAL=0 cargo test --lib crew_pay_conserves -- --ignored
CARGO_INCREMENTAL=0 cargo test --lib methods_need_contact -- --ignored
CARGO_INCREMENTAL=0 cargo test --lib heritage_study_requires -- --ignored
CARGO_INCREMENTAL=0 cargo test --lib relocation_conserves -- --ignored
cargo test --test shipping -- --ignored
cargo test --test discoveries -- --ignored
cargo test --test expeditions heritage_voyages -- --ignored
```
