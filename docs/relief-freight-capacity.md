# Religious relief and shared freight capacity

Religious relief previously used a consignment-size/travel-time cap but did not
check the shared land-freight pool before buying food. Its resulting `Shipment`
already counted as occupied capacity. That allowed it to oversubscribe endpoints
and block subsequent commerce even when no freight capacity existed at dispatch.

Dispatch now caps the consignment by remaining capacity at both the donor and the
recipient, in addition to food surplus, institution funding, route eligibility,
room availability and the existing travel-time cap. It does so before computing
the representable payment and committing food, money, occupancy and the shipment.
Less than 18 kg remains an unusable request and spends nothing.

The shared query already subtracts active commercial cargo (including captured
intermediate-town footprints) and existing secular or religious shipments.
Appending the new shipment reserves its endpoints without a second mutable
capacity balance or a second copy of goods. Existing delivery/loss paths release
the capacity as the shipment leaves transit. No archive migration is needed.

Timing and priority are unchanged: secular relief claims are settled before
religious fallback in Open, followed by later market activity. Institutions still
compete through the existing deterministic selection order; this is a capacity
bound, not a new fair-sharing policy. Legacy configurations without bounded
freight retain their existing unlimited-capacity behavior.

The targeted room/relief fixture now also reserves capacity with commercial cargo
and an earlier relief consignment. Only 20 kg remains for religious dispatch;
reducing that to 17 kg prevents dispatch without changing history. Payment and
food removal match the accepted consignment, and a serialized opening state
commits identically. Processing commercial delivery credits its timber to the
recipient and frees only that cargo's capacity; pending relief still reserves its
own share. Existing relocation and intermediate-town freight fixtures
cover the surrounding transport behavior.

Unfinished: explicit loading participants, shared road-edge capacity, relief
footprints beyond the existing direct-route endpoints, and long-run balance.
Institution room occupancy remains the separately documented 0.1 room-month
handling assumption; this change does not add a crew or alter delivery speed.

Verification: all three targeted GPU fixtures passed (shared intermediate freight,
religious dispatch and relocation conservation); the strengthened real-delivery
check passed separately. All 127 regular tests passed (110 hardware tests ignored
in that run). All-target Clippy passed with warnings denied; full frozen monthly,
batched and checkpoint continuation passed on seeds 17/81/256. Raw logs remain
ignored under `output/relief-freight-*.log`. No long-run balance claim is made.
