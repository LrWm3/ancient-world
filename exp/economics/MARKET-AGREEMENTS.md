# Production agreements, common offers and commitment planning

Implemented for the single-person, citizenship/land/farming/wood experiment.
The broader specialist and household fixtures retain their existing policies.

## Farming is a production agreement

`agreements::process(world, instance)` exposes a process through the same accepted
agreement structure as citizenship and land. The counterparty is explicitly the
**environment**, not a fabricated agent with a balance sheet. The process instance
remains the authoritative execution receipt; this does not introduce a second
mutable process or contract table.

Its production terms describe:

- the productive asset kind, with the accepted instance identifying its specific
  plot, use right, operator and output beneficiary;
- stage-entry inputs, including seed;
- minimum services each month, including labor;
- stage durations and a dated requirements schedule;
- conditional outputs, including grain and replacement seed;
- the explicit `AbortWithoutRefund` consequence if required performance fails.

Terms are derived from the same process definition used by execution. In the
current farming offer the six-month labor schedule is `2, 1, 1, 1, 1, 2`, with
one seed consumed at planting and eight grain plus one seed produced on completion.
No output is issued at acceptance. Missed performance aborts the process without
refund of spent inputs or production of promised outputs. The shared agreement
reports Completed or Failed from the committed process receipt, including the
failure consequence. Land arrears remain a separate scoped restriction: an
existing crop can finish even when its land agreement blocks new starts.

The generic production view also works for wood collection and other processes.
Process catalogs are treated as fixed during an accepted production agreement;
term versioning and amendments are not implemented. Declared schedules are recipe requirements; specialized tool/experience modifiers
still resolve in the existing execution engine. Completion outputs and storage
remain subject to its validated transactions.

## One offer interface

`offers` provides namespaced IDs (`Membership`, `Land`, `Process`), a common request
record, and these operations:

1. `discover`: visible terms, including opportunities with obtainable prerequisites.
2. `prepare` / `feasible`: read-only validation of a proposed acceptance bundle.
3. `accept`: prepare and publish through ordinary atomic settlement.

Visibility does not imply authorization. Opening feasibility does not promise
long-term success. The planner separately forecasts continued performance.

For example, a person can propose an ordered bundle:

```text
Membership(state citizenship offer)
Land(state plot offer)
Process(cultivation offer)
Process(wood collection offer)
```

The same resolver is used for planner prerequisite acquisitions and ordinary
process allocation. Specialized resolvers still enforce membership eligibility,
land rules and physical production; callers do not duplicate those checks.
Equipment and stock exchange keep their existing paths for now.

In Acquire, prerequisites are accepted and the process portion becomes a dated
production plan. The seed and labor are consumed only when that plan executes at
Productive. All requested processes draw on one opening budget and reservation
map; neither fresh outputs nor an already-reserved plot can fund another start.
Explicit bundles require every new process to be accepted, otherwise nothing is
published. They include existing active work first. Automatic planning retains
its selectable work priorities and can record rejected individual requests.

Prerequisites must precede dependent processes. This first interface retains the
existing limit of one membership and one land acceptance per boundary. Process
execution stays in Productive or Consumption as appropriate; acceptance does not
add a new scheduler phase. A prepared proposal is not a live reservation until
committed, and stale replay is rejected by existing batch validation.

## Planning across commitments

The shared evaluator continues to run actual settlement rules on sanitized copies
of state. It cannot see future scripted capacity shocks. In governed scenarios its
horizon covers the full duration of initial productive commitments, plus the
existing land-payment forecast window.

Each candidate now carries a `CommitmentAssessment`:

- current active processes plus processes started by this decision, with forecast
  completion/failure status;
- dated capacity claims summed across those commitments, with capacity estimated
  from currently known conditions;
- monthly work receipts for the whole forecast, including competing essential
  activities and requested, allocated and completed work.

Capacity claims are informational forecasts, not reservations of future labor.
They use the declared recipe; the rollout is authoritative for tools, changing
conditions, storage, available stocks and actual allocation. Future optional
activities appear in work receipts rather than being misrepresented as already
accepted promises.

A governed candidate that starts a process now and forecasts its failure is
rejected with a reason. Existing commitments can still be sacrificed for urgent
needs: scoring compares terminal outcomes, impairment and deprivation before
broken commitments, then stock coverage and work. This is deliberately bounded
foresight, not an optimal scheduling algorithm or a guarantee against shocks.
Earlier ungoverned scenarios retain their selection policy.

## Validation and limits

Focused tests cover common discovery, prerequisites, atomic bundles, dated
execution, shared labor, duplicate starts, seed consumption, successful harvest,
missed-work consequences, currently feasible but unsustainable offers, and two
activities that individually fit capacity but conflict at a later harvest.
The latter test keeps an existing crop, then introduces a four-month collection
commitment: current use is `1 + 2 <= 3`; harvest-month demand is `2 + 2 > 3`.

There is still only one person in the governed scenario. Multiple independent
planners competing for scarce offers, negotiated prices, flexible deadlines,
renegotiation, insurance and remedies other than the existing production abort
are future work. The current architecture is ready for a controlled two-person
scenario next; it does not yet claim fair multi-agent market allocation.

## CPU scenario result

The 36-month `opportunity-farming` run accepts citizenship and land in month 1,
completes harvests in months 6, 12, 19, 25 and 33, and has another crop active at
the end. Annual grain payments of two units settle in months 13 and 25. Every
month meets both nutrition and warmth; there are no arrears or aborted crops.
Ending personal stocks are five grain, zero uncommitted seed and five fuel.

Four 18-month regression comparisons (`opportunity-farming`, `tool-beneficial`,
`offer-long`, `storage-exchange`) retain identical state, reports and committed
batches after excluding planner diagnostics. These controls establish behavioral
continuity for the tested fixtures, not economic balance for multiple people.

Full crate suite: **165 tests passed**, including CPU/reference equality, replay,
checkpoint continuation, atomic rejection and the six new process-offer controls.
Formatting, Clippy with warnings denied and repository artifact checks passed.
Raw runs and build artifacts remain in ignored `output/economics/`.
