# Institutional space and service review

This records the current implementation and the next integration boundary. It is
a partial service-capacity integration; consumers beyond lessons and heritage study remain unconverted.

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

## Reservation ledger foundation

`institution_services::Plan` now supplies a standalone, serializable ledger keyed
by month, site and institution. Typed lesson and heritage-study requests carry
their actual subject IDs. An indivisible working group must fit the opening usable
space, and its occupants multiplied by its duration must fit remaining monthly
room capacity. Request order is explicit; denied requests leave space for smaller
ones. This is abstract capacity, not an assertion about floor area or daily room
schedules.

Execution checks the captured boundary, live space and caller-provided eligibility.
It records used and released grants once. The first settlement ends reservation;
failed work cannot reopen allocation retroactively. Increased space cannot enlarge
old grants, while damaged space can invalidate unexecuted groups. Closing releases
unexecuted grants. The ledger does not supply actors, labor, money or materials.

New cultural work plans now capture both service types. Heritage selection records
institution, author and artifact at the opening boundary; the author joins focused
identity checks and named participant reservations. The common cultural work grant
limits room grants to tasks with at least 0.1 available worker-months. These remain
conditional task allowances: earlier cultural consumers may still exhaust work,
and execution rechecks work and writing materials before consuming room capacity.

Heritage study executes before lessons, preserving the existing Respond order.
A lesson occupies two room units for 0.1 month; heritage study occupies one for
0.1 month. These are explicit toy scheduling parameters, not measured class sizes
or working hours. Legacy meeting places provide a two-person group and room-time scaled by condition;
component facilities use their usable room-time and largest usable nominal room. Lost, destroyed or remote
premises provide no capacity. Original material inventories remain unchanged.

Receipt use precedes actual study, with cultural settlement in Respond releasing
unused grants. The existing
`service_work_report` query and archive carry the service receipts. Old archived
plans with no service field retain their former execution path; newly reserved
plans use the new rules in both aggregate and individual modes. Manuscript-only
learning does not acquire an institutional room requirement.

Other service consumers and small-working-core readiness are still unfinished.
The shared operational/readiness gate remains unchanged until those consumers
have explicit capacity contracts. The full frozen scheduler monthly/batched/checkpoint comparison passes on seeds
17/81/256. Matched-history calibration of the new service behavior remains
required before drawing balance conclusions.

Verification for the consumer integration: all 121 regular library tests pass,
as do Clippy across all targets and the two hardware-backed learning/heritage
fixtures. The learning fixture verifies that loss of its declared finite room
prevents progress and spending, then releases the unused grant at settlement.
The heritage fixture opens named participation before reservation and compares
intact/lost premises with the same writing stock and work; duplicate execution
cannot consume supplies twice. Its legacy no-plan path retains prior readings.
The full frozen monthly/batched/checkpoint comparison also passes on seeds
17/81/256.

An existing limitation remains: generic cultural identity validation can cancel
a whole bundle when local objects change. The new live room check does not remove
that broader cancellation rule. Narrowing cancellation to the affected service remains follow-up integration work.

Archive validation now checks finite room arithmetic, indivisible grants,
once-only settlement state, duplicate service identities, shared opening capacity,
source references, captured site/month/institution, configured task dimensions,
and the cultural work backing both grants and completed service. Old plans without
service receipts retain their compatibility path. Opening room stocks are historical
observations: validation does not require a subsequently damaged building or dead
author to remain usable. Execution still performs the live checks.

The validation fixtures reject twelve malformed ledger variants (including overbooked
space and partial indivisible grants), plus six serialized cultural-plan mutations
covering wrong boundary, missing source IDs and absent work backing. The real
heritage consumer passes these checks with both intact and destroyed premises;
unused capacity remains a legitimate historical receipt.

With archive validation enabled, all 122 regular library tests, the heritage GPU
fixture, Clippy across all targets, and the repeated three-seed frozen
monthly/batched/checkpoint comparison pass. These establish consistency and
continuation, not balanced institutional service volumes.

## Group size versus available room-time

The consumer comparison exposed a discontinuity: using condition-scaled space for
both time and group size prevented a two-person lesson in a two-unit room after
even minimal wear. New plans now capture separate `group_space` and opening
room-months. A usable room retains its nominal group size, while wear reduces
available room-time. Component rooms below 25% condition or still under construction
contribute neither. Time adds across usable rooms, but a working group must fit the
largest individual room. The institution's existing operational gate still applies.
Old plans lacking `group_space` retain the earlier continuous-size interpretation.

This is a toy capacity rule, not an estimate of real classroom sizes or opening
hours. It keeps repairs useful without treating slight wear as immediate closure.
It also makes the two limits independently testable. At healthy capacity, the
0.5 cultural work ceiling usually binds before room-time; room-time becomes a real
constraint under damage and can support more consumers as they are integrated.

The GPU heritage fixture captures four finite finds and one actual teacher/student
lesson, reserves through the ordinary participant/work path, executes heritage
then teaching, and compares the same opening requests and resources:

| Room condition | Group capacity | Available room-months | Finds studied | Lesson progress | Room-months used | Writing material consumed |
|---|---:|---:|---:|---|---:|---:|
| 0.25 | 2 | 0.50 | 4 | None | 0.40 | 0.20 |
| 0.99 | 2 | 1.98 | 4 | Positive | 0.60 | 0.20 |

Both stay within 0.5 worker-months. Under damage, heritage-first allocation leaves
0.1 room-month, below the lesson's indivisible 0.2 requirement. With adequate
maintenance, the lesson also completes. The separate learning fixture now uses
0.99 condition and still teaches, while destruction prevents progress.

Room-denied requests are now filtered before shared work allocation, as described
below. No global work ceiling or service duration was increased to force these
outcomes.

Both competing-consumer cases reproduce identical culture state, event records
and town goods after serializing their captured plans and continuing execution.
All 122 regular tests and Clippy across all targets pass with the separated
capacity fields; the focused hardware suite includes both actual service consumers.
The repeated full frozen monthly/batched/checkpoint comparison also passes on
seeds 17/81/256 with this change.


## Feasible work before allocation (2026-09-12)

Captured room grants now filter cultural demand before research and culture share
work. Receipts retain both original demand and the amount remaining after known
room exclusions. The dated work plan stores the uncapped filtered amount so other
legitimate actions can still fill the existing 0.5 worker-month ceiling. Older
plans without this field retain their previous interpretation.

Denied lessons and heritage studies cannot recruit service-only teachers/authors.
If a named institutional duty cannot find a member, its allowance cannot leak into
a room-denied generic task. Personal matching and execution still enforce their
own live constraints; this is not a forecast of guaranteed completion.

Controlled evidence:

- Four heritage studies plus one lesson request 0.5 worker-months. Damaged space
  permits 0.4 and maintained space permits 0.5; actual reservations match those
  amounts while preserving the previous outputs and material use.
- Equal-weight research/culture claims of 1.0/0.5 against one worker-month become
  allocations of 0.6/0.4 when only 0.4 cultural work has space, instead of 0.5/0.5.
  This is an allocation fixture, not a measured long-run research gain.
- An unavailable-member upkeep request plus a room-denied lesson reserves zero
  generic work. Manuscript study and unrelated actions retain their demands.
- New receipt fields serialize, missing fields retain legacy defaults, and invalid
  allocations are rejected at boundary validation.

Remaining work includes narrower cancellation when an unrelated object changes,
other service consumers (hearings, visitors, relief), and sustained balance tests.
This change does not remove the whole-institution operational gate or introduce
same-month recycling of work released after execution.


Verification on the Quadro RTX 5000 Max-Q / Vulkan:

- `cargo test --lib`: 124 passed, 108 hardware tests ignored.
- Targeted library run with `--include-ignored --test-threads=1`: nine passed,
  covering room receipts, both actual service consumers, feasible allocation,
  and the matched research/culture fixture on seeds 17/81/256.
- `cargo clippy --all-targets -- -D warnings`: passed.
- `cargo test --test history_environment frozen_schedule_batch_and_checkpoint_equivalence -- --ignored --nocapture`:
  passed on seeds 17/81/256 (11.52 seconds test execution). This compares monthly,
  batched and resumed complete histories; it is not a new long-run balance study.

Raw local logs are under ignored `output/service-feasibility-*`; no generated
results are committed. The focused consumer fixtures also compare serialized
captured plans and continued state.
