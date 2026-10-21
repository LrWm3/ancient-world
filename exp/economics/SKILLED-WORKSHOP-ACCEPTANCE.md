# Personal skill in organizational production

This extends the [maintained workshop](MAINTAINED-WORKSHOP-ACCEPTANCE.md) with a
named person performing an organization's work. The question is whether completed
paid work can improve that person's productivity while the organization retains
its assets, contracts, outputs and financial book. It is a bounded learning-by-doing
extension; it does not add teaching or recruitment of replacements.

## Shared mechanism

`Activities.performers` optionally maps a process definition and stage to its
personal labor resource. For such a stage, the productive resolver selects a
worker from actual accepted employment deliveries. It records the agreement,
worker, month, technique and hours on the process transition and in employment
history. For the selected technique, selection among eligible workers follows employment
priority and then stable agreement ID.
The stage's labor must come from one worker's delivery; teams are not supported.

The organization remains the process operator. Inputs, outputs, equipment,
revenues, wages and depreciation remain on its ordinary accounts. Only competency
eligibility and practice attribution use the named worker. Equipment ownership is
still checked against the operator. A person does not acquire an organization's
machine or products by working on them, and an organization does not acquire
personal skill by paying for work.

A successful opted-in stage awards configured practice to its performer. Requests,
reservations, wages, failed stages and aborts award nothing. Eligibility reads the
opening committed state, so learning from one transaction cannot qualify another
transaction in the same batch. Future decisions and full-world forecasts observe
the resulting personal competency through the existing forecast machinery.

The resolver reserves hours across competing stage requests, separately from
scheduling. Settlement rechecks the actual delivery and remaining hours, exact
labor consumption and technique, then commits participation and resource changes
atomically. Extra anonymous capacity does not permit using the same delivered
hours twice. History validation rejects missing participation rows, altered hours,
unknown techniques and deliveries without a personal source. Labor relayed through
another employer is explicitly unsupported, rather than attributed to its reseller.
This traces accepted employment deliveries under the existing phase rules; it is
not universal provenance tracking for every capacity transfer or legacy operator job.

Normal Acquire still decides and accepts employment, Productive executes the
accepted work, and Close settles wages. Hiring projections count hours against
the specific proposed employment agreement. Lower labor requirements can therefore
reduce the accepted job and wage; they do not merely leave paid capacity unused.
Unopted processes keep their existing operator-based practice behavior.

## Calibrated scenario

The fixture retains the maintained economy's nine agents, prices, opening cash,
food, machine, legal permissions and financial obligations. The deliberate changes
are:

- Novice refining requires two hours instead of one.
- The worker has three hours each month instead of two, allowing a novice workshop
  job and one municipal job. The workshop's maximum employment and payroll become
  two hours/two coins.
- Two completed refining stages qualify the worker for a one-hour refining
  technique. The qualifying second job still costs two hours.
- Refining and repair require a named performer. Repair still takes one hour and
  awards no refining practice.

No practice is endowed, and no recurring subsidy or extra opening money is added.
The worker's extra capacity is an explicit scenario calibration, not an emergent
benefit. The `--no-benefit` control keeps every calibration above but makes the
qualified technique require two hours too. Both variants use the same machinery,
practice awards and consent rules.

## Acceptance and measured comparison

The positive acceptance runs 48 reference months, retaining the maintained
scenario's mature-year readiness gate: recurring output and repair, replenished
materials, positive organizational income, sufficient operating cash, timely
obligations, no food deficits or wage arrears, and nine reconciled books. It also
checks that paid workshop hours equal named completed work and all refining
practice belongs to the worker.

The first twelve months additionally use CubeCL CPU, reconstructing the simulation
after every phase and requiring exact world, state, ledger and audit equality.
Independent ledger replay covers those same twelve months. This is in-memory
reconstruction, not serialized recovery or evidence of GPU speedup.

Measured twelve-month reference results:

| Treatment | Refining jobs | Repairs | Workshop paid hours | Mints | Workshop net income | Food deficit units | Wage arrears |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Personal productivity benefit | 8 | 1 | 11 | 7 | -2 | 0 | 0 |
| No productivity benefit | 6 | 1 | 13 | 6 | -6 | 0 | 0 |

Both repay the startup loan and reconcile nine books. These are startup results:
zero food deficits over twelve months do not establish viability indefinitely,
and lower unit labor cost does not imply that every participant earns more cash.
The control establishes an actual production and paid-hours difference; it does
not measure an optimal training investment or the long-run distribution of gains.

The 48-month reference run passes the unchanged maintained-workshop readiness
gate: 36 refining completions, nine repairs, 36 metal sales and mints, 42 municipal
mining completions, 47 paid workshop hours and 42 municipal hours. The worker has
36 practice points; the workshop has none. Startup borrowing is repaid, household
deliveries and annual dues are timely, there are no food deficits or wage arrears,
and all nine books reconcile.

| Months | Refining | Repairs | Ore delivered | Workshop income | Workshop cash, opening → closing | Town income | Town cash, opening → closing |
| --- | ---: | ---: | ---: | ---: | --- | ---: | --- |
| 13–24 | 9 | 3 | 20 | 3 | 7 → 11 | 10 | 19 → 29 |
| 25–36 | 10 | 2 | 22 | 6 | 11 → 17 | 12 | 29 → 41 |
| 37–48 | 9 | 3 | 22 | 5 | 17 → 23 | 12 | 41 → 53 |

The mature years replace all 64 ore consumed by refining and repair. They each
pay twelve workshop hours, so the two extra novice hours remain a startup cost
rather than a recurring subsidy. Cumulative workshop income is twelve coins.
The grower still has eleven storage-blocked attempts, as in the maintained
baseline; passing readiness does not mean every plan is efficient.

Verification completed on 2026-10-21. The 48-month reference test and exact
twelve-month CubeCL CPU comparison pass, including reconstruction after every
phase and independent ledger replay. Both runs reconcile every committed phase.
Generated logs remain under ignored `output/`.

## Focused controls

Focused tests exercise qualification across months, voluntary job sizing and pay,
missing or unwilling workers, unavailable payroll, a skilled organization with an
unskilled worker, failed work, competing same-batch work, forged or duplicate
participation, and unsupported stage shapes. Replacing the worker preserves the
first person's skill and requires the replacement to learn independently. A
replacement run also compares CPU/reference state and ledger. Historical work
remains attached to its original worker and agreement.

The maintained scenario's four interruption controls now explicitly assert zero
food deficits during recovery, alongside actual repair, resumed production and
paid work. This strengthens the prior report's recovery claim into a test gate.
All five maintained-workshop regressions pass: the four interruptions and the
48-month reference run with its twelve-month CPU/reconstruction/replay comparison.

The focused verification comprises eleven named-work tests and 76 existing tests
covering employment, member employment, voluntary hiring, equipment and its
accounting, experience, competition, intermediary allocation, composition,
continuation and work choice. All pass, as do formatting and strict all-target
Clippy. These are targeted regressions, not a claim that the repository's entire
test suite was rerun.

## Supported scope and remaining work

This first adapter supports one worker per single-month productive stage, ordinary
employment delivery, existing technique alternatives and full-world organizational
forecasts. Its labor resource must not also be a stage entry input, process output
or externally routed pool input. A worker receiving resold hours of the same
resource that month cannot supply named personal work under this conservative rule.

Older person-local planning adapters prune other participants and historical
processes. Configurations combining named work with competition, production-market,
pool-market, work-choice or legacy cash/plot policies are rejected explicitly.
The corresponding public competition, intermediary and composition/continuation
entrypoints also reject before advancing the simulation. Ordinary agency startup
forwards and full-world forecasts remain supported. This is a compatibility limit,
not a claim that named work composes with every experimental planner.

Still outstanding: multi-worker teams, attribution across multi-month stages,
teaching contracts and their costs, skill-sensitive wage negotiation, recruitment
of replacements, migration and succession under ongoing obligations, and bounded
local forecasts that preserve worker context. The organization does not yet choose
between hiring an expert and investing in a novice's training. Supplied institutions,
wages and market terms remain supplied.

## Reproduction

Run from `exp/economics`; generated outputs belong under ignored `output/`:

```sh
cargo +1.92.0 test --locked --release -j8 --test performers
cargo +1.92.0 test --locked --release -j8 --test skilled_workshop -- --nocapture
cargo +1.92.0 test --locked --release -j8 --test maintained_workshop -- --nocapture
cargo +1.92.0 run --locked --release -j8 --example skilled_workshop -- 48
cargo +1.92.0 run --locked --release -j8 --example skilled_workshop -- 12 --cpu
cargo +1.92.0 run --locked --release -j8 --example skilled_workshop -- 12 --no-benefit
```

`--details` prints the current month's named participation records. The tests are
correctness checks with repeated private forecasts, not population-scaling benchmarks.
