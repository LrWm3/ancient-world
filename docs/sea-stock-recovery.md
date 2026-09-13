# Buyer-funded recovery across the great lake

Stored-goods recovery previously required a direct road on the same inner
continent. At year 50 of the workshop-repair screen, seed 256 and seed 409 have no
land routes between their five towns. Seed 1024 has just one route, between sites
0 and 4; site 4 is abandoned. Its other declining towns are on separate inner
continents. Road recovery therefore cannot address much of the potential stock
retention. This does not imply those stocks all have useful buyers.

## Mechanism

The existing abandoned-stock-recovery option now permits one surveyed sea leg
between the buyer's own port and the abandoned source's own port. Both must have
surviving commissioned harbors; the buyer must have a funded vessel fleet. No
beach landing, new route survey, intermediate port or source-side population is
invented. The existing direct-road behavior remains for same-continent journeys.

At the existing quarterly Respond boundary, after ordinary procurement:

- Check living buyer, empty abandoned source, trade permission, war and siege.
- Require an open surveyed lane and usable harbors. Choose the shortest eligible
  journey, breaking ties by stable lane index.
- Bound collection by source stock, useful demand, money, storage, pending cargo,
  both harbors' free handling capacity, buyer-funded vessel service and the buyer's
  remaining freight allowance. No borrowing or new issuance is introduced.
- Transfer actual payment to the source estate and remove actual source stock
  into one cargo record. The buyer gets nothing until arrival.
- Reserve the round trip, including surveyed shore access. The maximum remains
  six months each way by road; sea collection permits twelve months each way.
  These are explicit game range limits, not free travel. A captured path records outward and return travel.
- Fund vessel service at the buyer only. The source harbor handles the load, but
  its absent inhabitants contribute no crew. The collecting crew is an abstraction
  of the existing paid fleet service, not a new individual expedition population.

Opening uses the previous month's funded service to advance the voyage, exactly
as ordinary vessel cargo does. Insufficient service slows or stops the clock.
The lane and harbor checks hold delivery if access is closed or flooded. Existing
cargo delay, spoilage, siege and arrival rules still apply. Reopening does not
reserve a different route, duplicate cargo or backdate staffing.

The cargo's existing recovery marker distinguishes crew responsibility; no archive
field or version migration is needed. Ordinary commercial cargo still reserves
crew service at both endpoints. Both cargo types reserve harbor handling capacity and record both physical
endpoints in their freight footprint. The footprint does not imply that the
abandoned endpoint supplies crew.

## Limits

This recovers already stored fungible goods, not unmined deposits, installed
buildings, private wallets or unique artifacts. Failed ports cannot be rebuilt
for free. Payments remain in estate ownership and may themselves be inactive
money, so material recovery is not proof of repaired monetary circulation.
Abandonment can eliminate demand as well as leave stock; no recovery count is
required when no useful funded voyage exists.

## Comparison and tuning

Baseline is workshop-utilization repair, commit 79a5ca2. All worlds use the same
32/32 founding archives and circulation controls described in
`workshop-utilization-repair.md`, with abandoned-stock recovery enabled and credit
and issuance disabled. These are tuning runs, not held-out validation or isolated
performance benchmarks.

The initial sea pilot inherited the road's six-month one-way limit. Three
fifty-year runs (1024, 256, 409) and century extensions for 1024 and 409 produced
no sea collections. Inspection found seed 409's nearer useful source lane is
4,057 km before shore approaches: more than six months even at sea speed. The
retained sea limit is twelve months one way; the direct-road limit stays six.
Existing funding and freight constraints remain in force for the longer trip.

The first twelve-month century attempt stopped on `invalid market cargo`.
The new cargo recorded only the buyer as a freight stop, confusing crew
responsibility with physical endpoints. The corrected footprint records both
endpoints while the fleet workload still belongs only to the buyer. The corrected
runs restarted from the same founding archives. This failed attempt is not
counted as a successful calibration run.

| Seed, year | Arm | Population | Ending hunger | Operator work | Operating margin |
| --- | --- | ---: | ---: | ---: | ---: |
| 1024, 100 | Baseline | 117.336 | .01979 | 12.680 | 50.79 |
| 1024, 100 | Sea recovery | 117.300 | .01995 | 12.680 | 50.79 |
| 409, 100 | Baseline | 242.021 | .01830 | 381.455 | 2111.57 |
| 409, 100 | Sea recovery | 239.260 | .02302 | 375.919 | 2085.11 |

Hunger is weighted by current household food need; work and operating margin are
cumulative. These endpoint comparisons do not establish lifetime welfare or a
unique causal explanation for later population differences.

Seed 1024 makes five sea dispatches from abandoned site 1 to surviving site 0.
Tools (about 5.2 kg) arrive in month 1186 and bricks (about 1.3 kg) in month 1189.
Three smaller loads remain in transit at month 1200: 2.138 kg bricks, 1.430 kg
wooden containers and 1.026 kg bricks. They are not counted as delivered stock.
There are 27 total recovery dispatches versus baseline's 25, because sea and road
recovery share buyer resources; five sea orders do not imply five net extra orders.

Seed 409 makes one sea collection: about 2.1 kg flax arrives in month 975. There
is no pending recovery cargo at year 100. The small physical recovery does not
justify describing the slightly worse population, hunger or industrial outcome
as a success. Retain the capability because it enables a funded use of existing
resources, not because additional transactions inherently improve the economy.

## Verification

The hardware-backed recovery fixture covers road and sea behavior, unchanged
estate ownership, finite cash/stock/capacity, no early delivery, duplicated orders,
no funded buyer vessel, closed lane, ruined source harbor, excessive distance,
insufficient buyer freight, removal of crew funding after dispatch, and serialized
continuation. Its freight-footprint assertion covers the integration error found
by the first longer run.

The ordinary cargo-load test checks that recovery charges only the collecting
port's crew. Ordinary cargo continues charging both endpoints. Existing vessel
funding, finite-work and completed-interval/continuation fixtures remain in use.
Raw exports, logs and commands remain ignored under `output/sea-recovery-screen/`
and `output/sea-recovery-longer-screen/`.

Final verification: 196 ordinary library tests passed (152 ignored), the expanded
hardware recovery fixture passed, and all four vessel tests passed including the
hardware-backed funding case. Native build and strict library Clippy pass.

The final twelve-month candidate also reran all three fifty-year seeds. Reported
population, cumulative operator work and operating margin exactly match the
workshop-repair baseline; no sea collections occur in those intervals. Across the
five final candidate runs the largest absolute audited relative cash residual is
2.65e-7. Eleven comparison runs completed across the two range settings, plus the
one failed longer-range attempt described above. Runs overlapped compilation or
GPU fixtures, so their elapsed times are not isolated benchmarks.
