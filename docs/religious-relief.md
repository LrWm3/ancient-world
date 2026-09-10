# Funded religious relief

Religious orders can now respond materially to witnessed refugee appeals. This is
an opt-in first integration, not a calibrated replacement for public relief.
Enable **Allow funded religious relief** in History → Household relocation, or call
`History::set_religious_relief(true)`. Household departures and witnessed appeals
must also be enabled for new requests to arrive. Existing shipments finish even
if sponsorship is subsequently disabled.

## Causal path and finite transfers

1. A household actually arrives, carrying its dated request for people left behind.
2. Existing public relief gets first opportunity. If it cannot offer 18 kg, an
   operational local religious institution can fund an alternative response.
3. Hospitality-themed orders can assist across traditions. Other orders require
   the witness household to share their tradition. No conversion is requested.
4. The institution pays the host town for existing provisions: treasury decreases,
   municipal cash increases, host food decreases, and shipment food increases.
5. The existing relief transport handles travel, blocked routes, spoilage, receipt,
   and write-off. Expenses and external nutrient losses use existing ledgers.
6. Only delivered food creates recipient-local evidence of assistance. It adds a
   bounded term to existing inter-town relief affinity, which also informs
   evacuation destinations. A lost shipment earns no remote reputation.

Orders must be operational. The host must retain six months of food and have no
current shortage. Responses reject war, closed or flooded routes, journeys longer
than twelve months, abandoned destinations, and reports older than eighteen months.
A hospitality order can spend up to 25% of its treasury per response; others 10%.
Each order can dispatch once per month. Quantity is bounded by affordable stock,
three months of the reported population's needs, and `3000 / travel_months` kg.
These are explicit game parameters, not historically fitted constants.

## Identity, evidence, and persistence

Mission records link the institution, host, recipient, payment, promised quantity,
dispatch event, delivered quantity, and outcome event. Dispatch cites the witnessed
appeal; arrival or loss cites dispatch. The viewer exposes recent missions and
recipient-local institutional trust. Facts remain separate from religious accounts.

For received quantity `R` and current recipient population `P`, displayed trust is
`0.5 + 0.5 R / (R + 18 max(P,1))`. The host affinity contribution uses the same
fraction capped at 0.15. Neither changes faith, doctrine, or political control.
The affinity contribution is active only while this experiment is enabled.

The cultural archive contains policy and mission records. Older archives missing
this field load with sponsorship disabled and no fabricated missions. Validation
checks mission references and pending mission/shipment correspondence.

## Verification

The hardware-backed relocation fixture exercises actual arrival-triggered appeals,
paid sponsorship, disabled/poor/inactive/closed-route/stale-report controls,
no duplicate payment, unchanged faith, actual delivery, six blocked months followed
by loss, and unchanged food/economic residuals (relative tolerance `1e-6`). It
compares serialized history continuation through the same shipment-arrival method
used in monthly simulation, including a save point while cargo is in transit.

```sh
mise exec rust@1.89.0 -- cargo test --lib relocation_conserves_and_reserves_capacity_and_preserves_identity -- --ignored
mise exec rust@1.89.0 -- cargo test --test culture -- --ignored --test-threads=1
```

The cultural continuation test also exercises full-world save/load with the policy
enabled, monthly versus batched advancement, and missing-field migration. The
controlled pending-mission fixture tests history serialization, not a full GPU
world checkpoint with a mission in flight.

## Limits and next experiments

This inherits relief's abstract transport capacity; it does not reserve capacity
against commercial shipments. Institution readiness gates service, with no new
per-mission labor ledger. Trust evidence is cumulative and local, without decay, rumors,
or religious authority effects. Dated destination reports below have separate aging. Orders do not yet solicit earmarked donations or
operate branches. Funding is only a fallback, so it will not activate in every seed.

Next evaluate matched harsh-world seeds with sponsorship enabled/disabled, measuring
eligible appeals, payments, delivered food, shortages, institution survival, and
subsequent relocation. Broader religious legitimacy, charitable teaching, and disputes
should build on witnessed outcomes once this response is calibrated. The extensions
below add reciprocity and one outcome-based practical transmission rule.

## Reciprocity and locally learned practice

The enabled relief experiment now distinguishes gifts from reciprocal aid. A
reciprocity-themed sponsoring order declares an expectation of future assistance.
The expectation belongs to the recipient/host community pair, with the original
institution and mission retained as provenance. It is measured in kg of delivered
food, not cash, interest, or a spendable asset. Gifts create no new obligation.

Actual reverse religious-relief deliveries settle older obligations first, in
mission order. Only the surplus of a reciprocal return can create a new obligation.
Undelivered food creates none; dispatch alone repays nothing. Each mission stores
its received food, repayment portion, and later returns credited against it.
Validation reconciles total repayment with total credited returns. Obligation
settlement continues for already-dispatched shipments if the experiment is disabled.

An order can honor its community's outstanding obligation across faith boundaries.
It still needs a witnessed request, operating capacity, a treasury, safe host food,
and an open route. There is no automatic repayment extraction, interest, default
penalty, or obligation-driven war. Public relief remains the first response path;
only religious-relief deliveries currently settle these expectations.

Two separate receipts of at least 18 kg establish a local mutual-aid practice. This
allows local orders to consider outsiders even without a hospitality doctrine.
It does not change affiliation, patron ancestry, or doctrine. A learned practice
uses the ordinary 10% funding budget; a hospitality doctrine retains its 25% budget.
`mutual_aid_learned` cites the delivery that established the practice. This simple
threshold is an experimental transmission rule, not empirical calibration.

## Dated local knowledge

Departing households carry a snapshot of their origin's food cover. Only successful
arrival transmits that report to the destination. Religious relief shipments carry
the host's post-dispatch food-cover snapshot; it becomes known only on receipt.
There is no implicit remote refresh. Old journeys/missions lacking a snapshot
transmit no invented historical observation.

A report records observer, destination, observation month, receipt month, food cover
(capped at 24 months), and the receiving event. At most one report per ordered site
pair is retained; older testimony cannot replace a newer observation. Events retain
the underlying arrivals. Age weight is `1 / (1 + age_months / 12)`. Destination
preference is multiplied by
`1 + 0.25 * age_weight * clamp((food_months - 6) / 6, -1, 1)`.
Unknown destinations have a neutral multiplier. Aging returns preference toward
neutral, rather than asserting that an old observation becomes false.

Existing route, provisions, housing, land and production gates still apply. This is
imperfect information in destination ranking, **not** a full replacement of remote
admission checks. No report creates housing, supplies, or a route. Departure events
record the testimony multiplier. Reports and learned practices persist in archives
and appear as counts alongside the mission records in the viewer.

## Mechanism-level inspiration and checks

The distinctions between pooling, gifts and reciprocity, remembered places, and
cultural transmission were informed by the ABMA teaching-model collection:

- [Network mechanisms](https://github.com/SantaFeInstitute/ABMA/blob/master/ch8/ch8_networks.nlogo)
- [Remembered landscapes](https://github.com/SantaFeInstitute/ABMA/blob/master/ch4/ch4_remembered_landscape.nlogo)
- [Cultural transmission](https://github.com/SantaFeInstitute/ABMA/blob/master/ch5/ch5_cultrans.nlogo)

No example code was copied or translated. These implementations use the existing
world's stocks, shipments, institutions, and event records. Pooled funds, contractual
loans, rumor relays, and archaeological deposition are not implemented by this step.

CPU fixtures check local-only knowledge, aging, stale-report rejection, serialization,
partial receipts, gifts, and debt-cycle prevention. The hardware relocation fixture
checks paid reverse shipments, no credit before receipt, once-only repayment,
conservation, learned practice, and unchanged faith. The existing cultural suite
checks monthly/batched and archive continuation. Ensemble calibration remains future
work; these tests demonstrate the causal connections, not their historical accuracy.
