# Monthly history schedule

The schedule is a game rule, not a claim about historical time. All stages execute
once per simulated month, independent of API batch size. No additional planetary
readbacks or whole-world observation copies are introduced.

## Audit and port

Previously, relief arrived before consumption but market cargo arrived afterward.
Preparation, resource registration and culture synchronization also ran once after
the API's monthly loop. Living ecology ran outside the history coordinator, with
returns applied after the timeline had already been recorded.

The port makes five executable stages in `civilization.rs`. Observe and decide
are explicit substeps of opening/reservation rather than a generic scheduler.
Reservations remain subsystem-specific; this is not a simultaneous global auction.

| Stage | Operations and visibility |
| --- | --- |
| Open | Prepare society/politics/governance; increment month; activate due policies; inspect environmental disruption; settle due market cargo, relief and relocating households; answer appeals; restore sites; prepare economy and claims; refresh extraction inputs; patron assistance and legacy cohort cleanup. Arrival price observations are returned to the response stage. |
| Reserve | Prepare discoveries, reserve cultural work, prepare fisheries, allocate extraction allowances, plan production, prepare enterprises and vessels, prepare household retail. The extraction and retail plans are explicitly passed to execution. Earlier reservations have priority over later claims. |
| Execute/settle | Upload committed inputs; claim, fish and run GPU production/consumption; read town results; settle extraction and enterprises; storage, housing and waterworks; settle retail. No production policy chosen later in the month can retroactively change this dispatch. |
| Respond | Quote markets and dispatch new cargo using opening delivery evidence; release vessel/cultural work; expeditions, relocation, site lifecycle, society, genealogy, culture, offices and governance. Annual politics/shipping/expeditions/governance run here when due. Events remain available to later consumers in this stage. |
| Close | Synchronize society, politics, governance, economy, resource claims, culture and offices; refresh social indicators; validate. Frozen history records its timeline here. Living history then commits environmental returns, reconciles land, records its timeline and validates the coupled boundary. |

## Observation and response rules

- Both market and relief cargo due this month are usable before consumption,
  subject to their existing flood/delay/loss rules. New cargo dispatched in the
  response stage does not get another arrival pass that month.
- Market delivery price evidence survives across production as an explicit local
  value; moving deliveries earlier does not discard price-learning evidence.
- Production plans use policies already established before execution. Later
  governance/annual decisions affect subsequent production. Opening local relief
  is an explicit same-month intervention; traveling relief still requires arrival.
- Relocation reads this month's consumption shortage, remembered hunger history,
  current floods, and the last completed social housing projection. These are
  intentionally different time scales; closing projections do not retroactively
  change departures. Migration remains sequential and bounded by existing capacity
  reservations, not a simultaneous population-flow solver.
- Events are committed alongside actions and can influence later stages. Closing
  records summaries; it does not defer provenance IDs or conceal same-month events.
- Economy migration and living activation explicitly call `initialize_history_boundary`.
  Zero-duration advancement no longer initializes or synchronizes economic claims.
  For positive durations, closing work runs per month, not per batch.
- Living failure handling retains the existing incomplete-boundary marker; this
  port does not promise rollback of already-submitted GPU ecological work.

## Compatibility and verification

Old archives need no new fields: observation and reservation values are transient.
Continuation uses the new timing rules, so old-version trajectories are not promised
identical. Frozen batch/single-step/checkpoint tests compare complete history,
terrain, ecology and clocks over annual boundaries on seeds 17, 81 and 256.
Living tests compare full/gathered observation paths and checkpoint continuation.

Independent ordering means only operations without causal or resource dependencies
may commute. Sorting actors is not a fairness model; arbitrarily shuffling stages
would change the rules rather than test their implementation.

## Verification run (2026-09-11)

Quadro RTX 5000 with Max-Q Design; Rust 1.89.0, development/test profile;
terrain 64 and ecology 16 cells per face for integrated schedule fixtures.

- `cargo test --lib`: 62 passed, 66 hardware tests ignored. Includes finite
  extraction allowance settlement, reversed claimant order, and need-weighted
  relief allocation under reversed request order.
- `cargo test --lib economy:: -- --ignored`: 5 passed. Exercises retail/common
  ration accounting, industry income/food access, adaptive cost quotes and
  competing inland/sea freight reservations.
- `cargo test --test history_environment -- --ignored`: three fixtures cover
  current-month food delivery, three-seed frozen batch/checkpoint equivalence,
  and three-seed living gathered/full/checkpoint equivalence.
- Frozen seeds 17, 81, 256 compare a 24-month batch with 24 single steps, then
  12 months against checkpoint-resumed 5+7 months. Comparisons include the entire
  serialized history, terrain/ecology bytes and ecology clock.
- Living seeds 17, 81, 256 compare 36 monthly steps with storm/drought forcing,
  followed by four months against resumed execution. Existing compact-readback
  volume and ecology-budget assertions remain enabled.
- Food timing intervention transfers existing stocks into identical cargo in both
  worlds, changing only its due date. Due-now cargo increases the current consumed
  quantity and reduces the current shortage; future cargo stays in transit.
- Strict library Clippy and repository artifact checks are part of the commit gate.

Initial failures were useful: zero-duration enable calls still needed economic
initialization, so that path was retained explicitly. The food fixture initially
used unpaid cargo and deleted starting food; archive accounting rejected both.
The corrected fixture uses valid paid cargo and transfers, rather than deletes,
starting stocks. These were fixture/port errors, not relaxed budget tolerances.

These tests establish schedule and continuation behavior on this backend, not
cross-hardware equality, long-run balance, or universal actor-order independence.
The port deliberately changes market arrival timing. It does not establish that
new trajectories match the previous release, nor make every subsystem synchronous.


## Explicit initialization and relocation observations (2026-09-11)

Economy upgrade and living-history activation now invoke a named initialization
entry point; public zero-month advancement no longer doubles as setup. Initialization
still uses the common boundary transaction and accounting checks.

Immediately after execution/settlement, before markets and expeditions respond,
relocation captures a compact vector of shortage, production, crowding, migration
pressure and flood flags. It records the current history month and the social
projection's observation month. Departure processing rejects stale snapshots,
future-dated social evidence and mismatched site identities before modifying state.
This makes the existing social-projection lag explicit without a terrain readback.

These are pressure observations, not a copy of all decision state. Provisions,
people, routes, political eligibility, household hunger, relationships and destination
capacity still use response-stage state. Capacity and possession checks intentionally
remain live, so sequential departures cannot spend or reserve the same stock twice.
Relief and governance pressure observations are described in the next increment below.

The relocation fixture changes live shortage and production after capture and checks
that remembered pressure uses the captured evidence. A stale observation is rejected
without changing history. Existing relocation conservation/admission checks and
three-seed frozen/living continuation fixtures cover the integration.

Follow-up verification on the same Quadro backend: the GPU relocation fixture
passed, as did all three history-environment fixtures (25.91 seconds excluding
compilation), including seeds 17, 81 and 256 and exact checkpoint/batch comparisons.
Strict library Clippy passed. No new persistent fields or GPU readbacks are needed.


## Relief and governance evidence (2026-09-11)

Opening secular appeal decisions now capture pending reports, affinity, hostility
and host shortage in a dated observation. Before processing, every captured appeal
must still match the original report and remain unanswered. Replaying evidence
from answered appeals or using evidence in another month fails before any transfer.
Food surplus, freight capacity, route availability and host survival remain live
checks: an earlier commitment can consume stock but cannot make a stale observation
spend unavailable food. Religious sponsorship fallback retains its existing live
eligibility and reservation rules; it is not yet a captured proposal system.

Governance captures current shortage combined with the last completed social
projection, office capacity and occupation pressure immediately before its monthly
response. Observations carry both history and social-projection dates and site IDs.
Payroll demand, available treasury, tax rate, administration and committed events
remain response-stage inputs. Delivery and war events still update trust in the same
month; a snapshot of pressures does not defer those causal events.

Controlled tests mutate live pressure after capture and compare with unmodified
controls. Governance additionally compares a fresh observation of the changed
pressure, demonstrating that the observation boundary matters. Relief tests check
that depleted actual food stocks block spending even with favorable evidence and
that replaying an answered appeal cannot send a duplicate shipment.

Policy audit: annual faction tax assignment occurs in `politics_year`, after monthly
governance payroll/loyalty accounting and production. This port preserves that lag.
The tax policy timing increment below makes annual faction rates explicitly pending.
There is still no general queue for direct configuration changes or civic concessions.
These observation records are transient and add neither archive fields nor GPU reads.

Verification: 62 ordinary library tests, two GPU governance fixtures, the GPU
relocation/relief fixture, and all three history-environment fixtures passed on the
Quadro RTX 5000 backend. Seeds 17, 81 and 256 retained exact frozen/living batch and
checkpoint comparisons. Strict library Clippy, formatting and artifact checks passed.
This is evidence for these timing and accounting contracts, not a new balance study.


## Tax policy activation (2026-09-11)

Councils retain an active `tax_rate` and may have a `pending_tax` containing the new
rate, decision month, effective month and decision event. Annual faction decisions
schedule the next month's opening. They no longer immediately replace the active
rate in the closing-year record. The opening stage activates due policy before
relief, production, social taxation and governance consume state.

Scheduling or activating a rate creates no money. Existing social taxation transfers
and governance effective-tax pressure read only the active rate. Repeating an
identical proposal does not delay activation; a revised proposal supersedes the
pending rate. Returning to the active rate cancels the pending change. Decision,
activation and cancellation events preserve links to preceding decisions.

Pending policy is archived. Old councils without these fields keep their active rate
and an unknown activation date; no historical decision is invented. Archive checks
require finite bounded rates, past/current decision dates and the next-month pending
deadline. `tax_effective_since` cannot be in the future. Initialization and zero-time
calls do not activate pending decisions.

This is an explicit schedule for annual faction tax policy, not a universal policy
engine. Autonomy concessions and direct configuration edits retain their existing
semantics. Shared spending priorities and broader policy categories remain separate
work; they are not silently moved into the tax queue.

Verification fixtures cover pre-boundary inactivity, activation once, unchanged money
at activation, revised/cancelled proposals, old-council import and serialized pending
continuation. The multi-seed history fixtures cross annual decisions and resume at month 24,
including pending rates where decisions changed, checking exact batch/checkpoint
outcomes. The small serialized fixture guarantees a pending rate is exercised.

Tax timing verification on the Quadro backend: 64 ordinary library tests passed;
the GPU expanded-faction fixture passed; all three history-environment fixtures
passed (25.78 seconds excluding compilation), retaining exact seed 17/81/256
batch/checkpoint comparisons. Strict Clippy passed after a boolean simplification;
formatting and artifact checks passed. No new balance claims are made.


## Shared secular relief allocation (2026-09-11)

Secular appeals no longer consume a host's entire available budget in appeal-array
order. Captured eligible reports produce bounded requests, then a pure allocation
pass gathers total demand on each donor's food surplus and each endpoint's freight
capacity. Every request is scaled by the tightest proportional budget factor.
Incoming and outgoing relief share the same endpoint capacity; existing journeys
are deducted before new requests are considered. The donor retains the existing
12-month food reserve.

The commit order is canonical by host, origin, cause and household rather than
appeal storage position. Funded secular shipments commit before religious fallback
may reserve remaining resources. Live food/freight checks remain at commit to bound
float rounding. A funded grant rounded below the shipment minimum is declined rather
than allowing fallback to consume another secular grant's capacity.

This is deliberately conservative: grants below 18 kg are released and overlapping
constraints can leave capacity unused. There is no refill pass or maximum-flow claim.
The priority is explicit (existing commitments, secular allocation, religious
fallback); it is not an assertion that this is the only reasonable social policy.
Markets later in the month see these committed reservations through existing freight
accounting. Regional/intermediate relief stops still use the existing endpoint model.

Analytical fixtures cover 120:60 requests sharing 90 kg as 60:30, reversed request
order, incoming/outgoing freight competition, zero budgets and combined constraints.
An integrated fixture reverses a two-origin appeal batch, obtains two 45 kg shipments
from a 90 kg surplus, preserves the donor reserve and checks the food ledger. It uses
synthetic endpoint connectivity to isolate allocation, not to test route geometry.
Broader cross-system labor and treasury arbitration remains outside this increment.

Relief allocation verification on the Quadro backend: 67 ordinary library tests
passed, the GPU relocation/relief fixture passed, and all three history-environment
fixtures passed with exact seed 17/81/256 batch/checkpoint comparisons. Strict Clippy,
formatting and repository artifact checks passed. These are timing/allocation checks,
not long-run relief-policy calibration.

## Shared workshop labor allocation (2026-09-11)

Enterprise preparation now gathers affordable shifts before paying wages. Existing
rent transfers happen first; each active operator requests work bounded by its
leased equipment, observed demand, remaining working capital and the town's
existing craft labor allowance. A proportional allocation shares that allowance
across workshop families. Payroll commits only the granted work. Fixed family
slots give the demand sum a stable order, and GPU allowances round downward so
rounding cannot authorize unpaid work. Cash-poor operators cannot reserve shifts
they cannot finance.

Previously the firm loop consumed a declining allowance, privileging the first
operator. Under contention multiple operators can now receive smaller shifts;
existing distress rules can consequently affect several firms instead of only
late-listed ones. This is a game allocation policy, not a labor market equilibrium.

The change preserves the current craft allowance and cross-system scheduling.
Culture still reserves before enterprise preparation, and vessel preparation uses
remaining capacity after enterprise plans. It does not unify those capacity formulas
or provide a global labor/treasury auction. Rent deposits and household wage
transfers retain their existing commit order; only labor allocation is independent
of firm storage order. No archive fields or terrain readbacks were added.

Analytical tests cover proportional contention, permuted requests, cash-limited
requests, empty/zero budgets and rounding bounds.

Verification: 68 ordinary library tests passed; both GPU enterprise fixtures passed
(accounting/failure and prepaid work/checkpoint continuation). All three
history-environment fixtures passed in 24.72 seconds excluding compilation,
including exact seed 17/81/256 batching and checkpoint comparisons. Strict Clippy,
formatting and the repository artifact check passed. These checks establish bounded
allocation and continuation, not long-run economic balance under labor scarcity.
