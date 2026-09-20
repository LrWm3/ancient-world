# Additional plots based on productive capacity

Implemented as `trading-32-plots`, with `trading-32-no-plots` as its matched
control. The existing `trading-32` scenario is unchanged. This extends generic
rights, processes, access agreements and annual obligations; there is no farmer
class or special farming scheduler.

```sh
cd exp/economics
cargo +1.92.0 run --locked -- trading-32-plots
cargo +1.92.0 run --locked --config 'profile.dev.package.economics-compute-smoke.opt-level=2' --example plots_audit -- 72 cpu
cargo +1.92.0 test --locked --test plots
```

## Request and allocation

The state has eight additional plots. Every person can request a use right;
ownership remains with the state. Each accepted plot adds an annual obligation
of two grain, payable in grain or two coins. Its first bill is due 12 months
after acceptance, then annually. Native grain tax receipts use the existing
one-token-per-two-grain issuance rule; coin payments do not issue new tokens.

Starting in month 13, one person's application is reviewed each month, rotating
through stable person IDs. A person can hold at most two additional plots.
The lowest-ID available parcel is considered. These are explicit bounded
allocation policies, not a claim that the most productive person globally gets
the land. Finite land can run out before later applicants receive a turn.

An applicant with unpaid annual obligations, an outstanding production forward,
or a terminal lifecycle state is deferred. Otherwise, two local 13-month
forecasts compare keeping current rights with accepting the offered plot. The
expanded forecast must:

- complete productive work yielding the configured commodity on the extra plot;
- finish with more of that commodity than the baseline, after inputs, consumption
  and the first added annual bill;
- meet all modeled needs, pay every due annual obligation, and remain active.

Rights must last through the first tax date. Actual labor, seed, tools, wear,
experience, storage and existing commitments constrain the forecast. An idle
parcel that merely creates another bill is rejected. The stored request records
both forecasts and the decision, so an acceptance is inspectable and replayable.

This is a first-year feasibility screen. Forecasts exclude future shocks, market
receipts and other people's future competition; they do not guarantee permanent
solvency. Fixed production stock targets still apply, so work can pause once
stocks are sufficient. There is no automatic surrender, auction, rent negotiation
or repricing of existing rights.

## Execution and accounting

Acquire first resolves the existing market transfers. The plot review reads that
reserved result, including tools purchased and coins spent this month, and then
grants a right at the same commit barrier. The right is usable by subsequent
Productive work. Settlement canonically recomputes the request and rejects
altered forecast receipts or acceptance IDs. A parcel cannot be granted twice.
No production is retroactively changed and no new monthly phase is introduced.

Work orders can now request another instance of an exclusive-site process when
a separate eligible site is free. Continuing instances and the new request still
share the existing labor, input and equipment reservations. A single tool cannot
be used twice in the same month. Shared-site processes and processes without
sites retain their previous concurrency rule. At most one new instance of a
particular work-order process is requested per person per month.

The local forecast helper now preserves accepted plot agreements and their real
activation dates, and excludes unaccepted offered rights. This also ensures
future tool-purchase projections see the person's actual additional rights and
taxes without treating dormant catalog offers as usable land.

## Seed and controlled comparison

Both treatments start each person with two seed units instead of the previous
scenario's one, plus identical stocks, people and tools. This is a declared
initial endowment in both controls, not seed created by a plot grant. Additional
land does not provide free labor, seed, tools or storage. Seed acquisition and
multiplication are not added in this change.

One reusable seed cannot finance two concurrent crops. However, a second plot
can sometimes help even with one seed, by allowing crop work while the original
plot is occupied by construction. Eligibility is therefore based on completed
forecast work rather than a hard-coded seed threshold.

## CPU observations

These results predate the [state pricing and tool-productivity change](STATE-PRICING.md);
the request mechanism and tax terms remain current.

Both the 36-month and 72-month runs matched exactly between CubeCL CPU and the
reference backend for final state, ledger and monthly reports. The comparison uses 32 people
and three tool providers with the existing upfront coin/forward market.

| Observation | No extra rights | Additional plots enabled |
| --- | ---: | ---: |
| Extra plots accepted by month 36 | 0 | 8 |
| Completed grain output through month 36 | 520 | 624 |
| Extra-plot tax paid through month 36 | 0 | 6 |
| Completed grain output through month 72 | 1,416 | 1,608 |
| Extra-plot tax paid through month 72 | 0 | 54 |
| Unpaid tax at month 72 | 0 | 0 |
| Total modeled need deficits through month 72 | 64 | 64 |
| Terminal people at month 72 | 0 | 0 |

Amounts are physical grain units, converted from hundredth-unit ledger ticks.
The eight accepted people are 88, 92, 96, 100, 102, 104, 106 and 108; grants occur
between months 13 and 33. The baseline includes the same spare parcels and dormant
offers but disables the review policy. The equal nonzero deficit totals should
not be described as universally satisfied needs. This is a deterministic fixture,
not a balance result across seeds or allocation policies.

Focused checks cover productive acceptance, low labor and absent seed controls,
no double-spending of seed across simultaneous crops, the first annual tax,
insufficient right duration, existing arrears, holding limits, exclusive grants,
receipt tampering, actor reordering, CPU replay and midmonth continuation. Raw
logs remain under ignored `output/economics/`.
