# Governance and institutional participation audit

Inspected at `a35d491`. This is a code-boundary audit, not a claim that named
administrative work is implemented. It addresses the personal-duty review in the
integration worklist while retaining the implementation gaps below.

## Current authority and accounting

| Duty | Current cost and timing | Named participation gap |
|---|---|---|
| Local office coordination | `office_capacity` in `src/offices.rs` derives capacity from a valid holder and knowledge breadth. Vacancies retain 0.5 capacity. Governance observes it in Respond. | No personal commitment or completed-work receipt. A holder can be eligible while fully committed elsewhere. |
| Administrative funding and tax policy | `governance_month_observed` in `src/governance.rs` transfers existing council money to town finance, tracks funding, and changes loyalty/tax effects. | Payment does not establish an administrator's attendance; there is no corresponding personal-time reservation. Do not mistake money transferred to town finance for wages already delivered to a particular person. |
| Institutional upkeep and building work | `maintain_institutions` in `src/institution_capacity.rs` consumes the cultural labor budget, institution money and finite building inputs; staff must include present adult members. Readiness and physical condition affect later operation. | Staffing is a count, not the set of people doing the work. There is no maintenance-specific participant assignment per institution. |
| Institutional succession | `src/institution_succession.rs` uses present local representatives for a ballot and spends cultural work to convene it. | The ballot electorate is not a reserved work team. Political preference does not itself require labor, but organizing the election needs a specific accountable participant. |
| Institutional teaching | Source-specific cultural study requests retain teacher/institution IDs; reservation includes the teacher, and execution measures actual learning. | The wider cultural bundle still shares one team and grant rather than assigning each action its own workers. |
| Office campaigning | Existing cultural actions consume cultural work and money and produce political history. | Campaign participation does not cover subsequent routine office duties. |

`reserve_cultural_work_capped` in `src/culture.rs` builds one team from the
selected cultural actor, successor, institutional teacher and local recovery
workers. `settle_participation` charges the bundle's actual total once. This
bounds named cultural work overall, but does not guarantee that every upkeep
recipient supplied a member. Adding all institutional members to this same team
would make absent members block unrelated actions and is not an adequate repair.

## Integration boundaries to preserve

- Keep the five monthly phases. Opening policy/preparation remains separate from
  the later governance response. Do not retroactively apply a new officeholder's
  time to production already completed this month.
- Reserve routine administrative work before production if it is to compete with
  production. Both a personal grant and an existing town labor allowance must be
  debited; person capacity alone is not a second labor supply.
- Record which office and holder were reserved. Validate them at execution;
  succession or departure after reservation must not silently transfer work to
  the replacement. Completed work can inform the same month's later governance
  response; a new appointment can wait until the next reservation window.
- Keep existing money transfers single-owned. The administrator's assigned work
  must not create a second copy of funding, wages or tax proceeds.
- For institutions, split upkeep into institution-specific requests inside the
  existing cultural allocation. Reserve suitable members per request, retaining
  ordinary teacher/learner assignments. Charge the shared cultural allowance once,
  not once for the existing bundle and again for each new action.
- Keep aggregate mode and old-archive interpretation explicit. Old tenure or
  membership records are not evidence of historical hours worked.

## Next implementation and acceptance boundaries

Start with dated office service requests and grants, using an explicit policy for
competition with other labor. Capacity should read delivered service rather than
merely an office title, while preserving the separately declared unnamed-staff
baseline until that baseline is converted too. Compare aggregate requested work,
named attendance and actual administrative output separately. Follow with
institution-specific upkeep assignments rather than expanding the generic team.

Required checks include a fully committed holder, absent/dead holder, insufficient
shared labor, changed jurisdiction/holder after reservation, partial completion,
no duplicate funding or labor debit, and monthly/batch/checkpoint consistency.
For institutions also test two organizations sharing one member, an unrelated
absent teacher, and a funded building with no available maintenance member.
Scarcity comparisons must measure displacement of care, teaching and production;
reserving government work first is a policy choice, not automatically fair.

The audit identifies actual gaps; it does not close named administration,
institutional action assignments, multi-site branches or multiple officer roles.

## Opt-in office service pilot

`History::set_office_service(true)` enables monthly holder attendance, requiring
personal participation. Existing histories and defaults retain the preceding
capacity behavior. The pilot requests 0.1 worker-month per eligible office,
reserving after care, committed crews and learning, before later production
reservations. This ordering is an explicit initial priority, not a general claim
that administrative work outranks production.

The same grant consumes personal availability and the shared service allowance.
GPU production sees the reservation; settlement removes it only after production
has honored it. Unused time expires. Service settles before demographic losses in
Execute, and the existing later governance response reads the completed result.
A different holder/controller cannot inherit the old grant. A newly appointed
holder waits for a subsequent monthly reservation.

Only the holder's increment above the existing 0.5 unnamed-staff capacity scales
with delivered/requested work. That residual staffing remains aggregate. Office
service does not repeat governance funding, household wages or tax transfers; it
adds an attendance requirement to existing office duties. It currently has no
separate salary contract. Records persist under `Offices.service` and appear in
`service_work_report`. Disabling participation requires disabling office service
first at a completed boundary.

Institution-specific upkeep assignments, governance outcome comparisons and broader
allocation calibration remain open. This is an initial bounded participation
path, not a completed conversion of government staffing.

Verification: the GPU office fixture passes occupied personal capacity, shared
labor exhaustion, partial attendance, changed tenure, duplicate reservation and
settlement rejection, unchanged finance and serialized continuation. The regular
library suite passes 114 tests (104 hardware tests skipped). The full frozen
history fixture passes monthly/batched/checkpoint continuation on three seeds,
with the pilot enabled for 17/256 and a legacy arm for 81. All-target Clippy is
clean. These boundary checks do not establish scarcity balance. The subsequent
work-comparison extension is described below.

## Office comparison boundary

The opt-in office pilot captures the town's labor allowance during Reserve, before
matching its holder. Close records three shared resolution metrics: request versus
town allowance, allowance versus named reservation, and reservation versus used
work. The receipt identifies the opening holder and controller. It observes the
settled commitment rather than rereading a successor's capacity or applying service
a second time. These are conditional work comparisons, not a replay of an alternate
aggregate government or a forecast of tax revenue.

Comparison can be disabled without changing execution. Plans from older archives
without a captured allowance do not manufacture an opening expectation. Institutional
upkeep teams and broader governance outcome comparisons remain unfinished.

Verification of the comparison extension: the GPU office fixture checks full,
partial, unavailable, town-starved and ended-tenure plans; comparison-on/off leaves
all non-resolution history identical, and duplicate receipt rejection leaves the
whole history unchanged. Legacy plans without expectations emit no comparison.
All 114 regular library tests pass (104 hardware tests skipped); the three-seed
frozen monthly/batched/checkpoint fixture also passes with the new receipts.

## Institution-specific upkeep teams

In named-participation mode, quarterly cultural plans now capture each due
institution's local members and upkeep request. Within the granted cultural
allowance, planned elections reserve first, then upkeep, each in stable institution
order; each institution
selects its available member with the most remaining personal capacity, using ID
order to break ties. This is a bounded initial priority, not fair allocation across
institutions. Other cultural actions reserve the remainder with their existing team.

The institution executes only its own live member grant, rechecking membership and
presence. Its work is recorded inside total cultural completion but is subtracted
from the generic team's completion before personal settlement. Money and building
materials still transfer through the original upkeep code once. Cancellation of
an unrelated cultural actor does not erase the institution's separate reservation.
Unused time expires; later production cannot reuse it. Aggregate mode and older
plans without separate upkeep assignments retain the bundled behavior.

This supplies named attendance for upkeep and election convening; it does not yet
split other administrative actions into their own teams, introduce staff salaries, or
replace member counts used to estimate an institution's organizational support.

Verification: all three hardware upkeep fixtures pass. The new fixture checks
occupied members, membership revoked after reservation, unrelated actor cancellation,
unchanged total money, no duplicate upkeep, zero upkeep work attributed to the
generic team, and serialization of reserved plans. The three-seed service-allocation
fixture and frozen batch/single/checkpoint fixture pass. The regular suite passes
114 tests with 105 hardware tests skipped. These checks establish accounting and
continuation behavior, not long-run balance of the new upkeep priority.

## Election convening

Quarterly named plans now capture an election request only for an already vacant
mandate with eligible local members. One of those members reserves the 0.05
worker-month convening task before upkeep and the generic cultural team. The
ballot still represents all eligible local members; it does not simulate each
voter's time or imply the convener controls their votes.

New election requests declare their complete 0.05-worker-month minimum. Reservation
checks both remaining town allowance and convener availability before committing
time; an infeasible ballot leaves the allowance available for upkeep or other work.
Execution still requires the full grant and a still-eligible, present convener. Membership or faith changes can invalidate that assignment.
Vacancies discovered during response do not obtain retroactive work; they wait
for a subsequent quarter's request. Contested ballots retain the existing rule
requiring a second paid-work deliberation. Old plans without election assignments
retain the bundled path, and aggregate mode remains available.

Election and upkeep execution also reject stale dated assignments before looking
up a personal commitment. Commitment slots are reused each month; an old plan must
not accidentally consume a new month's grant at the same numeric slot.

Verification of election assignments: all four hardware institution fixtures pass.
They cover funded convening, insufficient grants, loss of convener eligibility,
unrelated cultural cancellation, stale plans referencing otherwise valid slots,
duplicate action assignment rejection, single settlement and saved-plan equality.
The legacy ballot/runoff fixture remains enabled. The matched three-seed
learning-allocation fixture also passes. No long-run claim about election frequency
or upkeep starvation follows from these boundary tests.

The integrated frozen schedule also passes full monthly/batched/checkpoint equality
on seeds 17, 81 and 256 after the election extension. The regular library suite
passes 114 tests (105 hardware tests skipped).

## Minimum useful institutional grants

The initial election-first pilot could reserve 0.04 worker-months for a ballot that
required 0.05, leaving no allowance for useful upkeep. Institutional work requests
now carry a minimum: 0.05 for elections and zero for divisible upkeep. This is a
feasibility check inside the existing cultural allocation window, not a change to
monthly execution order or a new global allocation policy. Old captured plans
default to zero minimum and retain their already-made reservations.

A controlled school-vacancy fixture on the Quadro RTX 5000 tested these cultural
allowances (worker-months per quarter). These are completed institution-specific
actions; any generic cultural grant is separate.

| Allowance | Election completed | Upkeep completed |
|---:|---:|---:|
| 0 | 0 | 0 |
| 0.010 | 0 | 0.010 |
| 0.025 | 0 | 0.025 |
| 0.040 | 0 | 0.025 |
| 0.050 | 0.050 | 0 |
| 0.075 | 0.050 | 0.025 |
| 0.100 | 0.050 | 0.025 |
| 0.500 | 0.050 | 0.025 |

The personal-availability intervention also leaves upkeep funded when every
eligible convener has less than the ballot minimum, even with ample town allowance.
The 0.05 threshold is intentionally discontinuous: a completed election takes
priority over upkeep that quarter. This fixes wasted partial ballots; it does not
establish fairness across many institutions or long-run demographic balance.

With the 0.05 allowance held fixed, the next-quarter fixture completes upkeep after
the mandate is filled and no further election is requested. All four institution
hardware fixtures and the three-seed learning-allocation comparison pass; the
regular library suite passes 114 tests (105 hardware tests skipped).

The full frozen-history fixture also retains monthly/batched/checkpoint equality
on seeds 17, 81 and 256 with minimum useful grants enabled.
