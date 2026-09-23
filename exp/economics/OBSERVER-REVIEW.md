# Observer review: missing autonomous wood trades

The observers are useful: they distinguish a missing counterparty from failed
settlement and expose selected forecasts which anticipate sales that never occur.
This review changes no economic behavior. It narrows the next experiment instead
of tuning prices or work quantities without identifying the failure boundary.

## Reproduction and controls

Run from `exp/economics`, using a fresh ignored output directory:

```sh
MONTHS=12 CASE=both TELEMETRY_PLANNING=alternatives TELEMETRY_SETTLEMENT=true \
  TELEMETRY_DIR=../../output/economics/observer-review-12 \
  cargo +1.92.0 run --locked --example reciprocal_market
MONTHS=12 CASE=directed TELEMETRY_PLANNING=alternatives TELEMETRY_SETTLEMENT=true \
  TELEMETRY_DIR=../../output/economics/observer-review-12 \
  cargo +1.92.0 run --locked --example reciprocal_market
```

Both use CubeCL CPU, four people, the same opening balances, skills, resources,
fixed quotes, six-month forecasting horizon, and `IncludeUnfilledBids` demand
signal. These fixtures are deterministic and have no seed selection. `both`
replans autonomously. `directed` holds complementary work/purchase policies fixed:
88/89 prefer crops and buy wood; 91/92 prefer wood and buy grain. This is a
feasibility control, not evidence of autonomous specialization. The wood listing
trades prepared fuel (resource 7), not raw wood (resource 6).

| Twelve-month result | Autonomous | Directed |
| --- | ---: | ---: |
| Grain traded, units | 16 | 28 |
| Wood/fuel traded, units | 0 | 22 |
| Nutrition deficit, summed units | 0 | 0 |
| Warmth deficit, summed units | 2 | 0 |
| Aborted process transitions | 0 | 0 |
| Capacity debits, units | 94 | 92 |
| Ending coins, agents 88/89/91/92 | 32 / 32 / 16 / 16 | 30 / 30 / 18 / 18 |
| Minimum individual month-end coins | 16 | 18 |
| Rejected match attempts, either market | 0 | 0 |

Capacity debits retain the existing scenario measurement; they include any capacity
expiry and should not be relabeled as exclusively productive labor. These are
process/trade diagnostics, not a comprehensive loan/default or tax-obligation test.

## Where exchange stops

The autonomous wood book never contains both a buyer and seller in the same month:

| Month | Wood buyers | Wood sellers |
| --- | --- | --- |
| 1 | 88, 89 | None |
| 2 | None | 88, 89 |
| 3 | 88, 89 | None |
| 4 | None | 88, 89 |
| 5–6 | None | None |
| 7 | None | 91, 92 |
| 8 | 88, 89, 91, 92 | None |
| 9 | None | 91, 92 |
| 10–12 | None | None |

Every submitted wood quote is two coins. There are **zero wood match attempts**,
not failed wood settlements. Grain has eight successful attempts and no rejected
attempts. In the directed control, buyers 88/89 and sellers 91/92 coexist every
month from month two onward, giving 22 successful wood matches at the same price.

Thus price crossing, funding rejection and storage rejection do not explain an
observed wood-match failure: no such match is attempted. Month-end liquidity also
has not been exhausted. This does not prove that every suppressed hypothetical
order could have been funded; closing balances are not opening available budgets.

## What planning reveals

In month two, **all four people select candidate 14**: prefer wood production and
allow purchases in the grain market only. Each independently predicts future wood
sales, while none permits a current wood purchase. Their forecasts simulate other
people doing ordinary need-directed work with purchases allowed, rather than the
other people's simultaneously selected plans. This assumption is visible in the
plan records and confirmed in `production_market::forecast`.

For agent 88, compare two month-two alternatives over months two through seven:

| Forecast field | Candidate 13: wood work, buy any good | Selected 14: wood work, buy grain only |
| --- | ---: | ---: |
| Nutrition/warmth deficits | 0 / 0 | 0 / 0 |
| Buffer gap | 0 | 0 |
| Closing coins | 18 | 24 |
| Bounded closing stock value | 2 | 2 |
| Wood sales, units | 3 | 6 |
| Grain purchases, units | 4 | 4 |

The alternatives make the local ranking understandable: with earlier score
components tied, candidate 14 predicts greater closing wealth. However, the
completed `plan_outcome` for that selection records **zero wood sales, zero grain
purchases, and four grain sales**. Actual agents replan during the horizon, so this
is evidence of a conditional forecast mismatch, not a claim that settlement lost
a transaction or that a fixed plan was violated.

Of the 28 selected forecasts whose full six-month horizons finish within this
run, 22 predict positive wood sales; all 22 realize zero wood sales. These windows
overlap, so they are not 22 independent experiments and their predicted volumes
must not be added into a run-wide expected sales total. Twenty later forecasts
remain incomplete at month twelve and are explicitly reported as pending.

This supports investigating counterparty assumptions and synchronized changes of
work/purchase policy. It does not yet isolate which change would fix autonomous
coordination. In particular, the buy/sell choice also interacts with the town
book's buy-first, at-most-one-side-per-good rule; a purchase permission can affect
whether an agent posts a sell order. The selected/alternative traces expose the
result, but not all intermediate order-generation decisions.

## Observer usefulness and remaining gaps

Metrics reproduced the scenario totals. Settlement records identified the failure
boundary before matching. Plan alternatives explained the selected policy locally.
Full-horizon comparisons exposed unrealized sales without comparing a six-month
forecast to one month's receipts. No changes inside economic subsystems were
needed to obtain these findings.

The remaining diagnostic gap is **why an order was absent**. Existing records
export submitted orders and attempts, but omit some decisions about ineligibility,
purchase-policy exclusion, lack of need, protected stock and insufficient surplus.
The town order generator computes useful demand/reserve data then discards it for
non-posted orders. A missing order alone cannot distinguish those cases. A compact
order-generation receipt would be more useful than more general logging here.

The next bounded behavioral comparison should hold everything else constant and
vary the forecast of other agents: today's ordinary-work assumption versus their
last observed selected policies. This is a hypothesis to test, not a committed
solution: last-month intentions can also be stale, and everyone holding the same
wood-selling intention could remain uncoordinated. Track same-month bid/ask
overlap, forecast errors, actual need satisfaction and liquidity; do not use trade
volume alone as success. A compact exclusion receipt would help interpret it.

## Verification and limits

Both JSONL streams finished with zero omitted logs and 60 unique committed batch
IDs. The autonomous run exported 48 selected plans, 768 alternatives, 64 orders,
eight attempts and 28 full-horizon comparisons. The directed policy has no search
transcript, correctly producing no plan records; it exported 86 orders and 36
successful attempts. Matched quantities reconcile exactly to per-market volume,
and every recorded coin-transfer transaction sums to zero. These checks were
performed against the generated JSONL, not inferred from console summaries.

The CPU scenario console independently agrees on deficits, volumes, capacity
debits, balances and process failures. Raw logs remain under ignored `output/`.
This is an observational comparison, not a parameter sweep or intervention; the
full Rust suite was not rerun because no simulation or observer code changed.

## Order-generation follow-up

Implemented the missing domain receipts and reran the same twelve-month autonomous
CPU case with `TELEMETRY_PLANNING=selected` and `TELEMETRY_SETTLEMENT=true` into
`output/economics/order-generation-review/`. No economic policy was changed.
The new output exactly matches the prior run's selected plans, full-horizon
outcomes, committed transactions, market observations, balances and need metrics.
There are 192 new receipts across the two goods, with zero omitted log records.

For the wood book's 48 buy-side and 48 sell-side opportunities:

| Side | First decisive outcome | Count |
| --- | --- | ---: |
| Buy | Submitted | 8 |
| Buy | Purchase policy excludes wood | 40 |
| Sell | Submitted | 8 |
| Sell | Stock protected for needs/commitments | 26 |
| Sell | Less than one lot in opening stock | 6 |
| Sell | Buy already selected | 8 |

There were no eligibility exclusions or no-need-improvement rejections in this
particular wood sample. This is not a claim that all policy-blocked buyers would
otherwise have needed, afforded or stored a purchase; those later gates were not
reached. Submitted buy/sell counts still never overlap in the same month.

The receipts explain concrete boundaries:

- Month two: all four buy sides are policy-blocked. Agents 88/89 have three units
  with two protected, so each submits one sell lot. Agents 91/92 have one unit,
  all protected, so neither submits a sell order.
- Month seven: buyers are again policy-blocked; 91/92 have three units with two
  protected and post sells, while 88/89 protect both of their two units.
- Month eight: all four submit buys, and all four sell sides are skipped solely
  because the buy side was selected first. The buy evaluations show 88/89 holding
  three units with two protected and 91/92 holding four with two protected. Thus
  everybody has at least one lot above the protection threshold, yet nobody is
  permitted to submit a sell side at this boundary.

Month eight exposes an additional structural mismatch: the buy test examines a
six-month horizon, while the seller protects two months. An agent can therefore
have both a long-horizon demand for another unit and stock above its shorter
protection threshold. Each hypothetical purchase reduces that agent's warmth
shortfall by one. The hardcoded buy-first, one-side rule turns all four into
buyers. This evidence establishes why the sell sides were skipped, not that
executing every possible sale would improve their actual long-term outcomes.

This makes **explicit order-side selection** a useful next bounded policy test,
with current buy-first as the control. Evaluate buy and sell candidates against a
consistent planning basis, then choose a side according to a named policy. Simply
switching everybody to sell-first could create the reverse imbalance; permitting
both sides would also need self-trade prevention and shared resource budgets.
The counterparty-forecast mismatch remains relevant; changing side selection alone
is not yet demonstrated to solve it. No such behavioral change is included here.

Verification covers forged-receipt rejection, absent-admission and missing-stock
reasons, policy exclusions, protected stock, buy-first suppression, submitted-order
correspondence, and telemetry distinguishing submission from subsequent funding or
storage failure. Town, reciprocal-market and telemetry suites pass (33 tests),
including CPU/catalog-reordering/checkpoint controls; strict Clippy passes. An
initial ordering test caught receipts following trader registration order; canonical
side/agent sorting corrected the diagnostic ordering without changing matching.
Raw runs and test output remain ignored under `output/`.
