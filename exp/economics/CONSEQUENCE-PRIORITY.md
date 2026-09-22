# Consequence-based allocation: warmth versus investment

Implemented opt-in policy for the two-person intermediary experiment. The policy
uses projected consequences to rank otherwise feasible input bundles. It does
not change process execution order, stock supply, tool terms or legal access.

Run from `exp/economics`:

```sh
cargo +1.92.0 run --locked --example consequence_priority
```

## What is ranked

`consequence_priority::Ranking` implements the existing `RankingPolicy`
interface. It accepts claim IDs and comparable dated reports for acceptance
versus denial. It has no farmer, tool, fuel, person-type or resource-ID branches.
The intermediary adapter supplies these reports from the forecasts already used
to select an action:

- Acceptance means this candidate receives its inputs and executes.
- Denial means no new action at this boundary; existing commitments continue.
- Each month reports terminal state, the count of impaired conditions, and summed
  deprivation normalized by each condition's terminal threshold (scale 1000).

Compare net reductions across the forecast horizon in severity order: terminal
months, impaired-condition months, normalized deprivation. At the first severity
with positive net reduction, rank earlier avoided harm first, then larger total
reduction. An increase in a higher-severity category prevents a lower-severity
benefit from earning harm-reduction priority. If there is no positive benefit,
the claim receives no special priority. Equal assessments use the configured
existing policy, including its stable identity or seeded lottery tie-break.

This ordering is a policy choice, not an assertion of a universal welfare
function. Severe future consequences can outrank milder immediate consequences.
Pure work savings and surplus-stock benefits do not receive special harm priority;
when health consequences are tied, ordinary ranking still permits investment.

The implementation validates comparable ordered months and missing/duplicate
claims. Diagnostic rounds retain the allocation mode, original forecast reports,
selected requests, allocation receipts and any fallback choices. Reports are
computed from the simulation model, not accepted as arbitrary external claims.
Strategic exaggeration and verification of agent-submitted reports are not modeled.

## Integration and timing

Set `Experiment.allocation_mode` to `Mode::AvoidHarm`. `Mode::Existing` remains
the default. `prepare_with_policy` permits comparing both modes from exactly the
same immutable boundary. The existing access-expectation mode is independent;
these comparisons leave it optimistic to isolate ranking.

First collect independent choices and feasibility checks, then rank the same
claims using either existing priority or avoided harm. The conditional-bundle
resolver still reserves all required inputs or nothing. Rejected applicants can
make the existing one-shot fallback decision against retained work. Acquire
stores the resulting dated production plan; Productive executes it unchanged.
Deprivation and capacity consequences retain their existing monthly timing.
No opportunity can obtain scarce inputs merely by reporting serious harm.

The denial projection does not search every possible fallback in advance. Actual
fallback occurs after allocation, so the marginal value report can overstate the
harm of rejection. Forecasts also omit other agents' unknown future demand. The
first cold applicant reports a modest immediate deprivation reduction; it does
not foresee the full damage from repeatedly losing under stable priority.

## Controlled setup

Both people already have citizenship, plot agreements and growing crops. Compare
months 2–13 using CubeCL CPU settlement. Shared wood capacity and opening stock
are two; regeneration is one per month. A tool requires two wood and two labor.
Fuel collection requires one wood and one labor, and produces two fuel. There
is enough wood for either initial request but not both. Existing crop attendance
is reserved first. Person 88 starts with two fuel; person 89 starts with zero in
the cold case, or two in the covered-warmth control.

For each initial state, tests compare identical proposed actions, forecast
alternatives and consequence reports between ranking modes. Only the ranking
changes. Later live requests may diverge as outcomes diverge; the full trajectories
are policy experiments, not a claim that every later demand vector is identical.

## CPU results

In the cold case both modes see person 88 request a tool and person 89 request
fuel. Existing stable priority admits the tool first, leaving no wood for fuel.
Avoided-harm priority admits fuel first. The rejected tool applicant then chooses
fuel as a fallback using the remaining wood. Both crops continue that month;
there are no tool benefits created from unsuccessful projections.

Totals over both people for the 12 comparison months:

| Initial warmth | Ranking | Unmet warmth | Unmet nutrition | Completed crops | Aborted crops | Harvest grain | Tools | Terminal agents |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Person 89 cold | Existing stable priority | 6 | 1 | 1 | 1 | 16 | 1 | 1 |
| Person 89 cold | Avoided harm | 0 | 1 | 3 | 0 | 24 | 0 | 0 |
| Both covered | Existing stable priority | 4 | 6 | 1 | 1 | 16 | 1 | 1 |
| Both covered | Avoided harm | 0 | 1 | 2 | 0 | 24 | 1 | 0 |

The cold person's month-2 warmth deficit falls from one to zero. Over the run,
the new policy prevents the crop abort and terminal outcome, at the cost of no
tool investment. The extra total grain comes from preserving crops, not inventing
a productivity boost. Nutrition is not completely solved: one unit remains unmet.

When both start with fuel, both initially request tools and have no projected
health advantage over one another. Avoided-harm priority therefore preserves the
original first tool award. It later prioritizes urgent need claims as buffers
run down. This control shows investment remains possible; it does not establish
that long-lived scarcity can never crowd investment out.

## Validation and scope

All 16 focused tests across `consequence_priority`, `intermediary`,
`access_expectations` and `resolution` pass. The four comparisons above ran on
CubeCL CPU. Formatting, Clippy with warnings denied and repository artifact
checks pass.

Focused tests cover severity, timing, magnitude, seeded tie-breaks, reordered
claims, malformed reports and higher-severity worsening. Integration checks use
identical opening requests, verify changed recipients and actual CPU stock/labor
use, reject modified dated work, and compare CPU/reference results. Monthly,
batched and checkpoint-resumed execution agree. Inactive warmth demand and an
applicant without spare labor do not block the other person's investment.

The comparison is deliberately small: two people, one shared material, a fixed
12-month forecast and one fallback attempt. It does not normalize benefits by
resource cost, optimize total welfare, compare all possible joint bundles, or
establish fair long-run investment access. The ranking adapter is generic; the
experiment remains scoped to new intermediary-driver actions. Existing monthly
phases, ongoing commitments and other markets keep their current priorities.
