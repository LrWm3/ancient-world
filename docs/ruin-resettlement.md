# Restoring abandoned settlements

`ruin-resettlement` is a default-on registered policy requiring society and
politics. It operates on the existing managed economy and crop inventories, with
one household per successful foothold. Saves with an explicit disabled setting
preserve it; missing settings use the enabled default.
Disabling proposals does not cancel funded journeys or erase occupation claims.

## Admission and scheduling

Annual Respond reviews consider known, existing land routes on the same inner
continent, at most 600 km long. Origins and households are inspected in stable ID
order; destinations with more surviving shelter are tried first. At most one
successful departure per origin per year is proposed automatically. Ordinary
relocation has the preceding allocation window. This is explicit sequential
admission, not a simultaneous auction.

A public `Generator::request_resettlement(household, site)` uses the same checks.
It cannot compel an unavailable household or bypass funding. Household ambition,
loyalty, caution and distance affect willingness; rulers and people with conflicting
travel/service commitments remain unavailable. Individual mode moves a whole
resident roster. Aggregate mode supplements known passengers only from unassigned
cohorts, using the existing relocation representation.

Admission requires:

- An abandoned, empty site with dry current terrain and no hostile war between
  origin and destination administrations; the connecting route must be open, dry
  and confined to inner land.
- No other incoming household already reserving the destination.
- At least two settlers including an adult, no more than 20% of origin residents,
  and at least forty residents and another household remaining at home.
- Surviving shelter for the party. Surveyed agricultural potential, discounted to
  33%, must cover their annual food requirement. This is a prospect estimate,
  not guaranteed harvests or renewed soil stocks.
- Food for travel plus twelve months after arrival; the origin retains twelve
  months of food for those staying.
- 0.5 kg tools and 10 money per settler. Repair supplies target shelter for twice
  the party size, subtracting surviving capacity: 2 kg wood and 3 kg bricks per
  missing place. The origin retains at least as much of each resource as it sends.
- Existing crop seed (up to 1 kg per crop), taken from the source seed inventory.

Travel reuses conserved household journeys, provision consumption, attrition,
route closures, infection and population-resolution receipts. Construction goods
and seed are now included in traveling C/N/P and material accounting. Goods and
seeds return with survivors if the attempt fails; total party loss releases their
C/N/P to regional detritus at the origin accounting cell, following the coarse
journey-loss abstraction rather than a generated route burial site.

Open rechecks shelter, live flood exposure, political access, remaining food and
whether someone has already occupied the site. Failure sends the party back with
its remaining inventories and an explicit travel time. A successful arrival
preserves the site's ID, original founding date, cultural provenance, damaged
infrastructure, depleted resources, soil, artifacts and stored inventories.
Administration passes to the sponsor; household faith and ancestry remain their
own. The existing Open reoccupation step activates the site, and the normal
production/repair and land-return coupling handle subsequent work. No asset is
instantly repaired, crop seed multiplied or land stock reset on arrival.

The initial party receives a provisional 5% private-stock interest where old
household interests remain. Old interests share the remainder. With no old
interests, the arriving household receives the full interest. Public service and
repair supplies remain in the ordinary town economy, not a new parallel stockpile.

## Dormant ownership notices

Each arrival records an occupation and notices for prior household interests and
local artifact owners/competing claims. Original artifact title and custody are
not changed at arrival. Objects still require actual recovery work if lost.

Pending notices expire after **120 occupied months** without a represented
claimant appearing. The clock starts at occupation, pauses while the place is
abandoned, and advances once per monthly Open. Flooding alone does not pause it
while residents still occupy the site. This is a toy legal policy, not a claim
about historical property law or automatic worldwide notification.

A present household member, actual owner or institution member can represent
its claim. A community claim can be represented by its controlling civilization's
leader. `contest_resettlement_claim(occupation, claim, person)` verifies presence
and representation; remote declarations cannot fabricate an arrival. The monthly
review also detects these appearances and existing artifact petitions. Once
contested, a claim does not expire merely because the representative later leaves.
A later occupation preserves recorded contests rather than clearing them.

Unchallenged expiration transfers a dormant household's remaining private-stock
interest to the restoration household, provided that household still resides at
the site. It does not transfer the former household's wallet. An absent artifact
owner's expired title passes to the local community; expired competing claims are
removed from the active claim list. Artifacts moved elsewhere cannot be seized by
the timer. Provenance and expired notices remain; loss, physical custody, materials
and creator identity do not change. Existing ownership settlement can release a
notice early. Active artifact disputes still use the existing petition mechanism;
contested household interests have no new general adjudication model in this pass.

## Scope and review

This first pass restores a foothold with surviving shelter, not a building-free
camp. It does not add sea colonization, multi-hop expedition planning, forced
population recruitment, independent institutional sponsors, or new ruin surveys.
Ordinary relocation may bring later households once the place is active and meets
its usual food/housing conditions. Existing regional farm and resource limits
continue to constrain recovery.

New histories enable restoration when society and politics are enabled. Use
`--disable-system ruin-resettlement` to stop new proposals, or
`--enable-system ruin-resettlement` to enable it explicitly on a compatible history.
The explorer exposes the same registered policy. Events link the sponsor,
origin, destination, household, journey and title notices; occupation records live
under `society.relocation.resettlement` in exported history.

## Verification

Hardware fixtures use seed 17, terrain resolution 64, ecology resolution 16, and
24 months of existing history. They deliberately abandon a connected town and
provide the origin with recorded finite test endowments. These are controlled
mechanism checks, not evidence that ordinary towns commonly afford restoration.

The six GPU tests cover admission failures without partial debits, competing
households, annual policy gating, destination reservation, roster conservation,
retained site identity and damage, unsafe-arrival return journeys, total-party
loss, and C/N/P, goods, money, food and population residuals. Claim tests cover the
120-occupied-month boundary, abandonment pauses, repeated monthly calls,
represented versus remote owners, preserved provenance and lost-object state.
A full archive continuation runs six production months both batched and singly
and compares the complete resulting histories. Scalar residual changes must stay
below `1e-5`; continuation comparisons require exact serialized equality on this
backend.

Reproduce with:

```sh
CARGO_INCREMENTAL=0 cargo test --lib relocation::resettlement::tests -- --ignored --nocapture
```

Long-run restoration frequency, viability of small successor communities and the
balance of provisional ownership remain uncalibrated. The ten-year claim period
is an explicit starting game rule. Ordinary CPU tests do not run these GPU cases.

Verified on the available Quadro RTX 5000: **219 ordinary library tests passed**
(165 hardware/extended cases skipped by that command), followed by **10 explicitly
selected GPU regressions passed**. The latter include all six restoration tests,
ordinary relocation conservation, decline/reoccupation, flooded-candidate recovery,
and registry enable/disable/archive continuation. The selected GPU run took 8.44 s
excluding compilation. No long seed ensemble was run for this change.

Default-on follow-up: 220 ordinary library tests and seven selected GPU tests
passed (the six restoration cases plus registry application/archive continuation).
Checks include default enablement, explicit saved disable settings, prerequisite
suppression and once-per-year proposal timing. This changes activation defaults,
not funding, survival or ownership thresholds.
