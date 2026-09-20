# Production surplus and scheduling audit

Historical diagnostic before the candidate-generation fix. The same audit source
now runs the revised planner; current results are in [DATED-CANDIDATES](DATED-CANDIDATES.md).

The tested economy has enough productive capacity to cover food, warmth and
annual rent. The autonomous planner leaves feasible work out of its candidates.
An explicit feasible work calendar demonstrates that capacity; it is a diagnostic,
not a replacement agent policy.

## Controlled comparison

All eight main runs start with identical state: five grain, one seed, one fuel,
forty raw wood, two labor per month, one state-owned plot with accepted annual
access at one grain, and one six-use tool owned by the state. The offer, when
enabled, costs three grain. Experience earns one point per harvest and unlocks
a one-labor manual harvest after four points.

Vary tool availability and learning under both payment policies. All runs use
eighteen-month decision forecasts and the unchanged six-month production-candidate
window. Each runs sixty months on CubeCL CPU and independently on the Rust
reference; states, complete ledgers and monthly reports agree exactly.

| Policy | Variant | Harvests | Food deficits | Crop labor | Fuel labor | Unused labor | Grain at end |
| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Debt first | Baseline | 7 | 4 | 61 | 32 | 27 | 1 |
| Debt first | Tool offered | 7 | 4 | 61 | 32 | 27 | 1 |
| Debt first | Experience | 7 | 4 | 58 | 32 | 30 | 1 |
| Debt first | Both | 7 | 4 | 58 | 32 | 30 | 1 |
| Protect essentials | Baseline | 8 | 2 | 64 | 32 | 24 | 7 |
| Protect essentials | Tool offered | 8 | 2 | 64 | 32 | 24 | 7 |
| Protect essentials | Experience | 8 | 2 | 60 | 32 | 28 | 7 |
| Protect essentials | Both | 8 | 2 | 60 | 32 | 28 | 7 |

Every run survives, meets warmth throughout, pays all four rent installments and
has zero planting requests blocked by arrears. The protective runs have one
active closing month in arrears; debt-first has none. None purchases the tool.
Experience reduces labor requirements but the saved labor ends up unused; fuel
work and harvest counts remain unchanged within each policy.

The previous payment comparison used six-month decision forecasts: its normal
protected run had seven harvests and four food deficits. Here both policies
receive eighteen-month forecasts. Do not attribute that improvement to tools
or experience.

## Why the plot waits

Debt-first plants in months 1, 9, 17, 25, 33, 41, 49 and 57, harvesting in
6, 14, 22, 30, 38, 46 and 54. The eighth crop is still underway at the cutoff.
At the productive boundary of month 7:

- The plot is free and access is valid.
- One seed and two labor units are available.
- Grain stock is seven.
- The nutrition planning receipt is NoDeficit.

Month 8 repeats this with six grain. Only month 9, with five grain, produces
an autonomous planting request. The two-month idle gap repeats each cycle.

[Candidate generation](src/simulation.rs) calls projected_shortfall over
World.horizon, six months. That projection considers consumption and expected
process outputs but omits rent and other committed stock outflows. With six or
seven grain it generates no crop candidate. Extending World.decision_horizon
does not extend this window.

[The forecasting portfolio](src/planning.rs) evaluates priorities and optional
deferral of generated work. It correctly simulates rent later, but cannot select
an earlier planting action that never entered its candidates. This differs from
failing to see an annual payment: the evaluator can see it, but its available
actions are restricted.

Protected runs plant in months 1, 9, 16, 24, 32, 39, 47 and 55, harvesting in
6, 14, 21, 29, 37, 44, 52 and 60. Idle-boundary receipts also say NoDeficit.
Payment/consumption timing changes when stock crosses the same threshold.

## Where saved labor goes

All main runs have 120 labor available over sixty months. The audit reconciles
actual crop work, fuel preparation and unused capacity against that total.

A separate diagnostic starts with the tool already owned by the person. This
changes endowment and is not part of the controlled purchase comparison.
It uses all six charges, reducing crop labor from 61 to 55 and raising unused
labor from 27 to 33. Harvest dates, food deficits and rent remain unchanged.
This isolates tool efficacy from the decision not to spend grain buying it.

Tools and experience work: harvest labor falls. They do not shorten crop
duration, raise yield or change candidate admission. Saved labor cannot
automatically create an extra crop.

## Can existing resources support a surplus?

A manual crop occupies the plot for six months and needs eight labor:
two planting, four growth and two harvest. It yields eight grain and replenishes
one seed. Food needs one grain per month; warmth needs one fuel per month,
with one labor and one wood producing two fuel.

Two back-to-back crops can produce sixteen grain per twelve months. Food plus
recurring rent needs thirteen. Crop labor is sixteen per year and warmth
preparation needs six: twenty-two against twenty-four available. These rates
suggest capacity, but do not prove that monthly indivisible tasks fit.

Three further runs check execution:

| Diagnostic | Harvests | Food / warmth deficits | Rent | Crop / fuel / unused labor | Ending grain |
| --- | ---: | --- | ---: | --- | ---: |
| Earlier dated crop starts, debt first | 8 | 2 / 0 | 4 | 64 / 32 / 24 | 7 |
| Earlier dated crop starts, protected | 8 | 2 / 0 | 4 | 64 / 32 / 24 | 7 |
| Explicit crop/fuel calendar, debt first | 10 | 0 / 0 | 4 | 80 / 30 / 10 | 21 |

The first pair adds crop intents at months 7, 13, 19, etc. without changing
resources or autonomous planning. Month 7's intent completes a month-12 harvest,
proving that an original idle boundary supports earlier production.
Later intentions still compete for labor and access. Scheduled starts carry no
nutrition goal, so their priority differs from a nutrition-driven autonomous
crop. This is a diagnostic limitation, not evidence that an improved planner
must use that ordering.

The final witness supplies a complete crop/fuel calendar through the existing
dated-intent API. It uses ContinuingFirst and a one-month candidate window to
suppress unrelated future-buffer requests. Crops start every six months from
month 1; fuel preparation is requested in the three months after each planting.
No yields, capacities, inputs, rights, obligations or payment timing change.
Actual work and accounting still pass through ordinary reservation and settlement.

It completes harvests in months 6, 12, 18, 24, 30, 36, 42, 48, 54 and 60, with
no arrears, blocked starts or unmet needs:

    5 initial grain + 80 produced - 60 consumed - 4 rent = 21 ending grain

Only four payments fall within this run because the first anniversary is month 13.
The recurring annual-rate comparison assumes one per paying year. Initial food
covers the first harvest wait; this does not establish viability without that food.

The calendar changes planning and allocation. It is a feasibility witness, not a
claim that one isolated heuristic change will reproduce its results.

## Recommended next change

Make candidate generation aware of dated obligations and production lead times.
Include an early-start alternative when it can avoid a future shortfall; let the
existing consequence forecast compare it with waiting. Bind that candidate to
the need/commitment it serves so competing work receives explicit priority.

Keep the physical catalog and current allocator initially. Test the month-7/8
boundary, seasonal food timing and completed work, rather than merely adding a
larger buffer or changing scheduler order. Additional land, yields or agent types
are not required to make this particular economy viable.

## Reproduce and validation

From the repository root:

    cd exp/economics
    cargo +1.92.0 run --locked --example production_audit > ../../output/economics/production-audit.md

Use -- --calendar-only for the fast witness, or -- --starts-only for earlier-start
probes plus the witness. The [example](examples/production_audit.rs) is editable
source; generated traces remain under ignored output/.

All twelve cases executed on CPU and reference, with exact state, ledger and
report equality. Each replays its ledger and checks grain accounting and all
120 labor units. Formatting and strict all-target Clippy pass. Core simulation
behavior was not changed.
