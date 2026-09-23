# Two-person calibration and planning variants

This fixture establishes a feasible exchange opportunity before asking autonomous
planning to discover it. It also provides two independent, opt-in planning axes:
expectations about counterparties and persistence of a selected work/purchase plan.
The existing planner defaults remain ordinary counterparty behavior and monthly
replanning. Buying and selling both retain the six-month horizon.

## Calibration setup

`calibration::scenario` starts from the reciprocal-market model, keeps persons 88
and 91, and gives each six grain and six fuel units as an initial runway. It removes
the other two persons and their plots, rights, balances, registrations and practice.
Each person still has two labor units/month, 24 coins, one seed, storage capacity
32 and a state-granted plot right. Both need one nutrition and one warmth unit per
month. There are no payment obligations in this fixture. Neither person has an
exclusive occupation or exclusive access to a process.

Person 88 starts skilled at crops: eight grain per three-month crop, with one labor
unit required each month. Person 91 starts skilled at wood: two fuel per monthly
process at one labor unit. The other techniques remain available at their existing
higher costs. These are process-level opportunities, not guaranteed monthly output;
resource limits, scheduling, consumption and storage still apply. Trade uses the
existing fixed quotes: two grain for four coins or one fuel for two coins.

The fixture changes population and initial buffers from the previous experiment;
it does not change prices, skills, yields or market rules between its own controls.
It is therefore a new calibration fixture, not a causal test of initial buffers
alone. Finite stocks provide time to establish production, not recurring subsidies.

## Controls and 24-month CPU results

| Case | Food deficit | Warmth deficit | Grain traded | Fuel traded | Ending coins 88 / 91 | Productive labor debits |
| --- | ---: | ---: | ---: | ---: | --- | ---: |
| Autonomous, no exchange (`autarky`) | 0 | 8 | 0 | 0 | 24 / 24 | 74 |
| Fixed complementary preferences, no exchange (`isolated-specialists`) | 18 | 18 | 0 | 0 | 24 / 24 | 52 |
| Fixed complementary preferences, exchange (`directed`) | 0 | 0 | 24 | 23 | 26 / 22 | 48 |
| Autonomous baseline | 0 | 0 | 8 | 0 | 40 / 8 | 69 |
| Prior published counterparty plans | 0 | 0 | 8 | 0 | 40 / 8 | 69 |
| Three-month plan persistence | 0 | 0 | 8 | 0 | 40 / 8 | 69 |
| Both variants | 0 | 0 | 8 | 0 | 40 / 8 | 69 |

Productive labor here counts negative labor effects in Productive-phase transactions,
not all account debits including Open expiry. These totals are not labor needed for
identical outputs: the directed case ends with more grain. The no-exchange cases
contain two independent persons with no exchanges between them, not an assertion
that an optimal autarkic schedule has been found. Fixed preferences still allow
ordinary fallback work and continuation of existing processes; they are not a
prohibition on doing other work.

The directed case ends at `(grain, fuel, coins)` = `(22, 5, 26)` for 88 and
`(6, 7, 22)` for 91. A 72-month CPU continuation control reaches those same ending
balances, trades 72 grain and 71 fuel, and has no deficits or aborted processes.
This establishes repeated exchange under finite money, storage and actual process
execution well beyond the initial six-month buffer. The no-trade specialization
control cannot maintain both needs; its longer run can also abort work as storage
fills, so absence of process failure is asserted only for the viable trade control.

Autonomous trade meets needs over 24 months, but its grain-only exchange shifts
coins toward 88. It does **not** discover the directed fuel-selling arrangement.
The calibration therefore separates operational feasibility from discovery and
longer-term funding. It is not proof of autonomous sustainable specialization.

A 48-month baseline extension confirms the concern: grain trade totals 12 units,
fuel trade remains zero, and closing coins are 48 / 0. Food deficits remain zero,
but warmth deficits total two units. Closing `(grain, fuel)` balances are `(10, 2)`
and `(6, 0)`. Thus a 24-month needs-only check would miss a developing liquidity
imbalance. Only the baseline and directed controls were extended; these results do
not establish the long-run outcomes of the experimental planning variants.

## Counterparty expectation policy

`production_market::Config.counterparties` selects:

- `Ordinary` (default): other people use ordinary need-directed work and allow all
  purchases in the private forecast, as before.
- `LastPublishedPlan`: take choices from the latest prior recorded decision and
  hold those choices in the forecast. The evaluating agent's own candidate overrides
  its prior choice. If no prior plan is available, ordinary behavior is the fallback.

This is an **experimental public prior-plan signal**, not an inference from orders
or a model of private intent discovery. It assumes those previous choices are
observable. It never reads other agents' current-month selected candidates or
future state, and it does not solve for a mutually consistent set of current plans.
Source months and actual assumed choices are retained in the decision and exported
by the planning observer. Prior choices can be stale and need not predict current
work; the experiment measures that limitation rather than granting foresight.

On 38 completed, overlapping six-month forecast windows, summed absolute errors
across forecast/realized sales and purchases were:

| Planner | Grain quantity error | Fuel quantity error | Actual selected-choice changes |
| --- | ---: | ---: | ---: |
| Baseline | 30 | 0 | 19 |
| Prior published plans | 40 | 0 | 19 |
| Persistence | 32 | 2 | 7 |
| Combined | 46 | 0 | 7 |

Errors are per-good diagnostic sums across overlapping windows, not independent
samples or missing run-wide transactions. This counterparty hypothesis worsened
aggregate forecast error in this fixture, even though all four variants produced
the same reported aggregate economic outcomes. It remains an option for comparison,
not a recommended replacement for the default.

## Plan persistence policy

`production_market::Config.persistence` selects `Monthly` (default) or
`Hold { months }`, bounded to 1–12 months. The example uses three months, matching a
crop's duration without altering the production scheduler.

A new selection gets an inclusive `retain_through` date. Within that period, the
agent keeps its selected **preferred work and purchase permission** unless a newly
forecast candidate improves the safety tuple: terminal outcome, priority-ordered
need deficits, then aborted-process count. Wealth, stock-value, buffer or labor
improvements alone do not break the hold. Expiration, an unavailable old candidate,
or a safety override permits a new selection and starts a new period.

All candidates are still evaluated each month so the safety comparison is current.
This is a preference-persistence policy, not computational caching, a contractual
labor reservation, or a promise to counterparties. Existing process commitments
still execute through the ordinary resolver. The guard uses the full forecast
horizon, not a newly invented immediate-need test.

The two persistence cases each recorded 16 scheduled selections, 31 retained
selections and one safety override over 48 person-months. Selected choices changed
seven times instead of nineteen. That demonstrates the mechanism, but did not
improve trade composition or liquidity in this calibration. A preferred-plan change
is not necessarily a change in actual work because continuation and fallback work
remain available.

## Run and inspect

From `exp/economics`, use a fresh directory (files are not overwritten):

```sh
CASE=baseline MONTHS=24 TELEMETRY_DIR=../../output/economics/calibration-demo \
  cargo +1.92.0 run --locked --example calibration
```

Available cases: `autarky`, `isolated-specialists`, `directed`, `baseline`,
`expectations`, `persistent`, `combined`. Run each case into that directory; its
case name identifies the JSONL file. Use a different directory for a longer repeat.
All use CubeCL CPU and deterministic initial conditions; there is no seed selection.
The example exports selected plans, settlement details and metrics automatically.
Defaults are 24 months, baseline policy. General users can configure the two policy
axes independently on any compatible production-market world.

The observer now exports `selection_reason`, `retain_through`, persistence policy,
and dated counterparty choices. It labels assumptions instead of inventing a
reasoning narrative. Decisions and their persistence state live in existing dated
market history, so checkpoint resumption retains them; private rollout histories
still omit bulky alternative transcripts after the assumed choices are captured.

## Validation and scope

Focused tests cover the 72-month finite-budget control, invalid hold durations,
wealth-only retention versus safety override, expiration, prior-month information
boundaries, and combined-policy CPU/reference equality with reordered catalogs and
Acquire checkpoint reconstruction. Existing production, reciprocal-market and
telemetry regressions cover unchanged defaults and ordinary settlement behavior.
All 32 focused tests passed (one persistence unit test, three calibration tests,
nine production-market, nine reciprocal-market and ten telemetry tests), as did
strict all-target Clippy.
No scheduler or allocation priority changed. The full crate suite is not part of
this bounded experiment; raw results and logs stay under ignored `output/`.

The next useful question is how to discover mutually compatible plans. Publishing
prior choices alone did not do that here, and persistence alone did not solve it.
Keep the calibrated directed control as the feasibility check before adding more
resources or claiming broader autonomous coordination.
