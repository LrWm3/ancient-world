# Maintained productive workshop

This extends the [voluntary-employment economy](VOLUNTARY-EMPLOYMENT-ACCEPTANCE.md)
with finite productive equipment. Its bounded question is whether an organization
can discover, authorize and pay for recurring upkeep while people, the household,
town and state continue meeting their obligations. It does not port the main
simulation's aggregate institutions or introduce a second maintenance scheduler.

## Supplied world and shared mechanisms

The nine agents, food distribution, initial coins, prices, legal institutions,
credit terms and employment offers are inherited from the voluntary-employment
fixture. The workshop receives one additional opening machine, with four productive
uses and an explicit accounting basis of four coins. This is an endowed asset,
not an endogenous investment or an unrecorded cash grant.

Refining requires that machine and consumes one use. Repair is an ordinary
one-month productive process: one ore plus one labor-hour restores four uses.
Repair and refining cannot use the same machine in the same month. Materials and
wages flow through existing markets, employment, settlement and separate financial
books. Use depreciates the original basis; repair expenses do not write the asset
back up. Exhaustion leaves a repairable asset, rather than silently deleting it.

The signed operating agreement permits refining and repair; state law must also
permit each action. Discovery does not create authority. It offers productive work
alone and productive work combined with an exhaustion-maintenance target. The
normal planner chooses among those alternatives and existing quote portfolios.
The existing sixteen-program ceiling still fails closed if enumeration exceeds it.

A new optional `DurableReserve { kind, target }` objective measures the shortfall
in remaining uses of owned live equipment. It is neither money nor a forecast of
output. The workshop's supplied preferences put completed production first, one
remaining service unit second, then cash. Production-first permits spending the
last use; once exhausted, the condition objective can justify paid repair.

This preference is explicit. Current-job hiring deliberately excludes later
unsigned jobs from both comparisons. Without a separate value for restored
condition, a repair whose benefit requires future hiring can be rejected. We have
not bypassed that consent rule or assumed that future workers will agree.

## Timing and dated decisions

Open applies previously accepted work choices and resets period capacities.
Acquire obtains real inputs and evaluates voluntary hiring against available
materials, capacity, payroll and both parties' objectives. Productive executes
accepted refining or repair. Consumption meets needs; Close settles wages and
records monthly outcomes. The financial audit reconciles every committed phase.
Repaired capacity becomes productive in a subsequent month; there is no same-month
reuse.

Discovery covers both recurring stock targets and one-shot upstream attempts.
When proposing upkeep alongside a one-shot attempt, it can retain the latest
accepted recurring target for the same productive route. It reads accepted work
from organization decisions and activated forward agreements **effective at the
dated request boundary**, never current mutable orders when regenerating an old
decision. An explicit empty work-target replacement clears prior work. Same-month
replacements follow actual Open ordering: ordinary decisions, then activated
forward programs in controller and agreement order. Pending, rejected
and future activations cannot supply a target. Founding orders are not inferred
as dated acceptances; current financed programs start processes without replacing
work targets.
Repair has priority zero; preserved productive work has priority at least one,
without increasing the priority on each review.

Two intermediate runs exposed gaps. Maintenance attached only to stock targets
could disappear when upstream demand offered one-shot work. Adding that path
allowed repairs, but replacing the existing recurring target with repair-only work
then stalled production. Preserving dated accepted targets addresses both cases.
Independent review also caught the risk of reconstructing historical catalogs
from live orders; the history regression exercises target installation and later
validation explicitly.

A subsequent 48-month trial still stalled: five refining jobs, one repair and 55
food-deficit units. Including forward-activated targets corrected reconstruction,
but did not cure that stall. Independent review traced the first divergence to a
month-seven ore forward. Its comparison credited the agreement for installing a
productive program, without comparing that same program with ordinary spot funding.
The advance spent working cash and changed operating choices; later, the town
refused mining wages because it projected no worthwhile sale. These failed trials
are implementation evidence, not a demonstrated physical limit of the economy.

### Compare financing with an independent operating choice

Forward assessment now compares a potentially acceptable deal with each party's
own signed operating program funded from existing resources and spot trades. It
tests both the next ordinary Open and the forward's later activation date. It
leaves counterparties' programs unchanged: a private projection cannot assume
that someone else consents to a new operating plan. Each branch uses the same
observations, objectives, physical execution and conditional-labor rules.

A deal must be no worse for either party than retaining current choices **and**
that party's best independently acceptable tested option, with a strict benefit
for at least one party. This extends the existing loan outside-option principle
to forwards. It does not force the independently projected program into the live
world. Normal decision-making still chooses subsequent actions.

Receipts retain the outside option's date, losses, own completed work and
conditional labor. Proposals already rejected by the original comparison skip the
extra rollouts; an absent outside-option receipt can therefore mean “not evaluated,”
not “no independent choice exists.” These are projected preferences, not guarantees
of future employee consent or a new universal solvency test. Ordinary settlement
still enforces real budgets and obligations. The acceptance scenario checks actual
food, wages, debt, delivery and operating outcomes separately.

## Acceptance and controls

The original economy's readiness gate remains unchanged. This variant has a
separate physical-throughput gate because four uses followed by one repair consume
five months. Each of months 13–24, 25–36 and 37–48 must demonstrate at least two
cycles’ worth of activity: eight refining completions, eight metal sales and two repairs.
Individual cycles can straddle these reporting windows.
It also requires recurring municipal output, paid hours matching completed work,
replacement and repair ore, positive organizational income, nondeclining cash and
enough cash for the next operating payment. Across months 13–48, delivered ore
must cover **actual** refining and repair consumption, not just the minimum activity
gate. Whole-lot inventory can fluctuate between individual annual windows.

Shared gates retain no food deficits, no wage or loan arrears, timely household
forward delivery, annual land dues, repaid startup borrowing, authorized work and
nine reconciled books. Balanced books alone cannot pass this test.

The reference test runs 48 months. Its first twelve months also run on CubeCL CPU
with reconstruction after every phase and exact world, state, ledger and audit
comparison. Ledger replay independently reproduces those first twelve months.
This is in-memory reconstruction, not serialized recovery or a GPU performance
claim. These are expensive correctness tests with repeated private forecast
rollouts and boundary reconstruction, not a population-scaling benchmark. The
original unmaintained economy retains its full 48-month CPU/reference regression.

Interruption controls remove repair authority, worker capacity or the repair
recipe at the exhausted boundary, then restore it. They check that no repair or
phantom wage occurs while blocked, then require actual repair and resumed output
after restoration. A dated one-month capacity override is tested separately from
changing the worker’s ongoing capacity. Forecasts do not assume foreknowledge of
future capacity overrides; after observing a period override, future base capacity
remains available, whereas a structural capacity change persists in projections.
Focused controls additionally cover missing materials, insufficient payroll,
foreign-owned equipment and already-full equipment.

All four interruption runs pass through month sixteen: no repair or workshop wage
in blocked month eight; repair resumes in month nine, refining in month ten,
another repair in month fourteen and refining again in months fifteen and sixteen.
Each finishes with ten refining completions, two repairs, zero food deficits and
no unpaid wages. This tests recovery from one restored interruption, not permanent
loss of capacity or long-term insolvency.

The example offers permanent `--no-authority`, `--no-repair` and
`--no-condition-goal` controls. `--supplied` freezes a founding static program menu;
the ordinary variant discovers alternatives from dated requests. This is a
comparison of different discovery policies, not an assertion that their later
candidate sets are identical.

## Measured results

The discovered-upkeep reference run passes all readiness gates at month 48:
36 refining completions, nine repairs, 36 metal sales, 36 mints and 42 municipal
mining completions. The workshop pays 45 labor-hours and the town pays 42. There
are no food deficits, unpaid wages or overdue loans; startup borrowing is repaid,
both household deliveries are timely, and annual land dues are paid at each due
boundary. All nine financial books reconcile.

| Months | Refining | Repairs | Metal sales | Ore delivered | Workshop income | Workshop cash, opening → closing | Town income | Town cash, opening → closing |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: | --- |
| 13–24 | 9 | 3 | 9 | 20 | 3 | 9 → 13 | 10 | 19 → 29 |
| 25–36 | 10 | 2 | 10 | 22 | 6 | 13 → 19 | 12 | 29 → 41 |
| 37–48 | 9 | 3 | 10 | 22 | 5 | 19 → 25 | 12 | 41 → 53 |

The three mature years consume 64 ore: 56 for refining and eight for repairs.
Deliveries replace all 64. Annual inventory fluctuations therefore do not hide
depletion of startup materials. Income and cash differ because financial reporting
also recognizes inventory, depreciation and the timing of settlement.

The grower still records eleven storage-blocked production attempts over 48 months.
These are rejected attempts, not eleven units of destroyed grain. The loop meets
its bounded obligations despite this remaining planning inefficiency; it is not
an optimal allocation result.

Verification completed on 2026-10-21. The 48-month reference run and exact
twelve-month CubeCL CPU comparison pass, including phase reconstruction and ledger
replay. The original economy's full 48-month CPU/reference, reconstruction and
catalog-order regression also passes. All four interruption tests and 38 focused
discovery, hiring and forward tests pass. Formatting, strict all-target Clippy and
repository artifact checks pass. Generated logs remain under ignored `output/`.

### Twelve-month controls

These twelve-month reference controls use the same initial resources and terms.
The founding-menu control changes discovery policy; the other three remove one
maintenance prerequisite.

| Control | Refining | Repairs | Mints | Food deficit units |
| --- | ---: | ---: | ---: | ---: |
| Discovered upkeep | 8 | 1 | 7 | 0 |
| No legal repair permission | 4 | 0 | 4 | 0 |
| Repair recipe disabled | 4 | 0 | 4 | 0 |
| No condition-reserve preference | 4 | 0 | 4 | 0 |
| Static founding program menu | 0 | 0 | 0 | 13 |

The three missing-prerequisite controls exhaust the original machine and stop.
No repair appears merely because it is economically useful; lawful feasibility and
the supplied valuation both matter. Food stocks bridge this short window, so zero food deficits here do
not establish long-term viability. All five runs retain nine reconciled books
and no wage arrears, illustrating why financial consistency alone is insufficient.
The static menu fails startup entirely; it is not evidence that manual program
provision inherently cannot work, only that this particular frozen menu is
insufficient for this demand-driven startup.

## Reproduction

Run from `exp/economics`; keep generated logs under ignored `output/`:

```sh
cargo +1.92.0 test --locked --release -j8 --test maintenance_discovery --test voluntary_hiring
cargo +1.92.0 test --locked --release -j8 --test maintained_workshop -- --nocapture
cargo +1.92.0 run --locked --release -j8 --example maintained_workshop -- 48
cargo +1.92.0 run --locked --release -j8 --example maintained_workshop -- 12 --cpu
cargo +1.92.0 run --locked --release -j8 --example maintained_workshop -- 12 --no-condition-goal
```

`--details` prints the existing organization and hiring decision receipts. The
experiment uses the same observers, execution and financial reporting as ordinary
work; it does not introduce special upkeep transfers.

## Limits and next boundary

This is one machine using an already traded input, one repair recipe and supplied
preferences. It does not discover new maintenance-material supply chains, choose
between repair and replacement investment, optimize fleets or negotiate wages.
The condition metric does not certify legal access or immediate availability;
execution still checks both. Existing multi-asset binding selects the first
damaged matching asset, so this fixture does not establish optimal threshold
selection across heterogeneous equipment.

Broader forecasts still use explicitly conditional labor; actual Acquire obtains
live consent. Initial assets, organizational membership and financial terms remain
supplied. The [personal-skill follow-up](SKILLED-WORKSHOP-ACCEPTANCE.md) now binds
productive work to an actual employee and exercises learning-by-doing with lower
later labor requirements. Teaching investments and endogenous succession or member
replacement under operating obligations remain outstanding.
