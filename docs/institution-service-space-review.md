# Institutional space and service review

This records the current implementation and the next integration boundary. It is
not a claim that shared room scheduling is already implemented.

## Current coupling

`facilities::demand` targets space from all locally present adult members, with
kind-specific factors and a 2–64 capacity bound. `maintain_institutions` takes the
minimum of work, fee payment, local staffing and usable-space coverage as quarterly
support. Readiness changes by `0.20 * support - 0.08 - 0.12 * disruption` before
clamping. Thus space coverage below 0.4 cannot maintain readiness even with full
staffing, work, payment and no disruption. Adequate funding and space jointly
restore operation in the existing controlled fixture.

That rule conflates two different demands: the minimum working organization and
the capacity to serve its entire membership. A room suitable for a teacher and one
student can still lose all operational readiness when many other members join.
Reducing the membership-space denominator alone would have the opposite risk:
every consumer below could use the smaller room without accounting for competing
service demand.

`Institution::operational` currently combines active status, a held mandate when
one exists, readiness >= 0.25, and cached building condition/construction status.
It does not reserve rooms or consume service capacity. Most consumers impose other
work, money, object or route limits, but they do not share a room ledger.

| Consumer | Existing consequential limits | Space integration needed |
|---|---|---|
| Institution lessons (`culture.rs`) | Captured topic/teacher/institution, present named participants, 0.1 cultural work, gradual learning | Capture a teaching-space assignment alongside the work plan; execution must use both |
| Heritage studies (`expedition_heritage.rs`) | Accessible find, study interval and count, writing material, cultural work, living author | Reserve study space with the actual source and participating institution |
| Petition hearings (`civic_petitions.rs`) | Local grievances and representation, living leader, 0.1 cultural work, petition cooldown | Capture hearing space for the selected institution; do not reserve rooms for every candidate |
| Religious relief (`religious_relief.rs`) | Institution treasury, real surplus food, witness/route/hostility conditions, one dispatch per institution per month | Distinguish administrative dispatch work from transport and food; room capacity must not mint either |
| Heritage visitors (`heritage_renown.rs`) | Observed renown, accessible custody, route and traveler work | A destination's capacity is a quote; arrival must reconcile capacity against other visitors/services |
| Expedition sponsorship (`expeditions.rs`) | Real sponsor treasury plus existing population, equipment, food and voyage conditions | Institutional authorization should use organizational capacity; funding must retain its existing single payer |
| Religious influence (`culture/dynamics.rs`) | Operational local order, readiness, supporters, doctrine/contact and pressure | Separate remembered service outcomes from a fresh unlimited service grant; preserve lagged social effects |

## Integration direction

Keep political mandate and organizational continuity separate from service volume.
A small maintained institution can organize limited activity; serving more people
should require more time, rooms, personnel or funds. Large membership should create
requests and competition rather than automatically disabling all small services.

Before changing the shared operational gate:

1. Add a dated institution service plan under the existing Reserve/Respond/Close
   contract. Express capacity in the current abstract room-capacity units and
   explicit duration, not invented square metres. State whether the request is a
   working group, storage/access, a public gathering or an organizational decision.
2. Pilot captured teaching and heritage-study requests. Match eligible work and
   available space jointly; never reserve a room for an action known to lack its
   minimum useful work or actual artifact. Preserve aggregate/older-plan behavior.
3. Execute the granted action once, record used/released capacity, and expose its
   shortfall alongside the existing work receipt. Lost premises and absent actors
   must invalidate only the affected assignment. Unused capacity cannot fund past
   production or advance a future visitor arrival.
4. Bring hearings, visitors and dispatch duties into the same scope before broadly
   relaxing the old operational gate. Authorization and remembered religious
   influence require distinct treatment; they are not interchangeable room users.
5. Only then compare a small working-core readiness requirement with the current
   whole-membership requirement. Hold room stocks, labor and money fixed in the
   controlled comparison; keep buildings and their repair/expansion costs finite.

Required evidence includes a small school teaching a bounded group despite large
membership, two incompatible simultaneous requests sharing one room, larger
premises serving more requests, and an unused room failing to compensate for an
absent teacher. Check funds/materials/work once-only settlement, destruction,
stale plans, membership changes, and batch/checkpoint continuation. Matched
histories should report actual lessons, studies, hearings and visitors as well as
operational counts. A higher count alone cannot establish better service.

## Measured remaining constraints

The completed essential-first 30-year runs (`21b617e`, seeds 17/81) show the
following end-boundary counts. These overlap; they are not mutually exclusive
causes of failure.

| Seed | Operational / active | Readiness below 0.25 | Building condition below 0.25 | Vacant mandate | Fewer than two local adults | Space coverage below 0.4 with at least two local adults | Treasury below 0.5 |
|---|---:|---:|---:|---:|---:|---:|---:|
| 17 | 6 / 31 | 24 | 14 | 5 | 3 | 22 | 0 |
| 81 | 9 / 32 | 21 | 13 | 9 | 4 | 17 | 0 |

The current snapshot therefore cannot blame every remaining failure on empty
cash. Readiness also remembers earlier conditions, and the analytic space limit
does not by itself attribute every historical failure. These observations justify
reviewing service scope and damaged premises without removing leadership or
staffing requirements.

One timing exception matters for the proposed service ledger: religious relief is
selected through `answer_appeals_observed` in Open, before the main Reserve window.
It cannot consume a Respond grant retroactively. Its integration needs a declared
opening allowance or a previously prepared dispatch authorization, with a matched
comparison for any change in aid timing. The teaching/study pilot can stay within
the existing cultural Reserve/Respond window without that additional timing change.
