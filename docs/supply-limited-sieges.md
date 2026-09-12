# Defenses, sieges and military freight

This first land-siege mechanism reuses the existing army identities, carried
provisions, monthly ration debit, casualty resolution and return journeys.

## Construction and persistent evidence

`commission_defenses(site, bricks)` transfers 100–100,000 kg of existing bricks
into a canonical, stationary fortification artifact. No defensive effect appears
until construction work completes. Reserve requests at most half a worker-month
per site after office service and before production planning. Where participation
is enabled, available named residents must grant that same work; their attendance
does not add a second workforce. Execute settles the dated receipt and releases
unused work. The toy construction rate is 500 kg per worker-month.

Walls persist through abandonment and political control changes. Artifact
destruction disables them. Expanding a surviving structure needs additional real
materials and work; damage does not regenerate automatically. Destroyed structure
recovery is not implemented. These are initial brick defenses, not a full military
architecture/material-selection catalog.

## Monthly siege and supply

An authorized campaign reaching a site with completed defenses starts one siege.
The existing army ration and shortage losses run first in Respond. Supplied armies
then wear down defensive integrity over successive months instead of resolving
an immediate battle. Once breached, the ordinary battle model resumes. A force
withdraws when continuing would consume its return provision margin, or political
access is lost. A second simultaneous besieger is not supported.

Active encirclement blocks land freight through the town, including due
commercial cargo whose captured itinerary uses it as an intermediate stop and
relief at either endpoint. Those deliveries retain their inventories and wait; this first
siege delay does not add spoilage beyond existing weather delay rules. Direct sea cargo
can still arrive; captured inland approaches remain subject to encirclement. There is no naval blockade or added civilian casualty roll:
existing food access and demographic accounting remain responsible for shortages.

`send_military_supply(army, kg)` dispatches actual food to an established siege or
occupation camp, keeping two months of the origin's civilian food reserve. It
requires a passable direct corridor and reserves the existing land freight pool
and road capacity, with an additional 10 kg/person dispatch ceiling for legacy
unplanned transport. Commercial and relief capacity therefore see the reservation.
Food is counted once in transit and moves into the army only on arrival in Open,
before that month's ration debit. If the army has withdrawn, freight returns by
the recorded journey duration; a closed corridor holds it in transit. Acceptance
of peace ends encirclement immediately and orders actual troop withdrawal, without
teleporting those stores home.

Commissioning and resupply are explicit scenario/API decisions in this first
version. There is no autonomous siege investment planner, general carrier crew
model, naval interception or indefinite-shipment expiry. Constants describe game
mechanics and have not been calibrated against historical sieges.

## Verification

The controlled hardware fixture imports declared construction materials, runs
actual monthly work, starts a real mobilized campaign, checks delayed battle and
conserved shared-freight dispatch, and compares a depleted army's withdrawal with
the funded baseline. It then accepts peace, verifies returning supply, and compares
18 months batched with checkpoint-resumed monthly execution. All standard food,
population and economic ledger validation remains enabled.

A controlled access fixture checks an encircled intermediate junction, an
unrelated road, a direct sea delivery and a sea journey with an inland approach.
Blocked cargo retains its quantity and payment; lifting the siege permits one
delivery, with matching serialized continuation and food accounting. The fixture
isolates route access; it does not model physical progress along individual road
segments. Existing journeys reserve their entire captured itinerary until arrival.
