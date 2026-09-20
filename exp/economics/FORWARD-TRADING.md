# Upfront tool prices and prepaid production forwards

Historical first calibration. Current cash scenarios use the [state price spread
and revised tool benefits](STATE-PRICING.md): spot purchases at 0.75, state resale
at 1.50 and forwards at 0.50. The original one-to-one descriptions and results
below document the earlier experiment.

Implemented in the standalone CPU experiment. `trading-32` now uses upfront coin
payments; `trading-32-no-forward` disables state financing under the same prices
and opening stocks. `trading-32-share` preserves the earlier output-share control.
There are still 32 people, including three providers, plus the state. Three was
selected under the old royalty model; it is held fixed here, not claimed to be
optimal under the new payment system.

From `exp/economics`:

```sh
cargo +1.92.0 run --locked -- trading-32
cargo +1.92.0 run --locked --example forward_audit -- 72 trading-32 cpu
cargo +1.92.0 run --locked --example forward_audit -- 72 trading-32-no-forward
cargo +1.92.0 test --locked --test forward
```

## Price and funding

Providers quote coins upfront at **25% of the state-priced value of projected
output assisted by this tool over the next 12 months**. This carries forward the
previous capture setting as a purchase quote. It measures output of assisted
processes, not the counterfactual productivity increment. Recycled inputs of the
same commodity are excluded. An idle tool with no projected valued output does
not get purchased. The buyer owns the delivered tool and owes no later royalty;
repair does not create a new purchase, while a replacement needs a new quote.

Buyers spend their opening coins first. For the shortfall, they can sell future
production to the state through a prepaid commodity forward. The contract records
buyer/debtor, state/creditor, commodity quantity, immutable spot-price ratio,
advance amount, issue month, due month and delivered quantity. A stored projection
explains the purchase quote and forecast surplus supporting the promise.

The state pays the provider directly on the buyer's behalf. Funding, tool transfer
and forward creation form one atomic transaction. The provider gets the entire
coin price now. No separate same-batch incoming payment is spendable by another
purchase. The state uses existing treasury coins: forwards do not mint money or
alter annual tax-based issuance.

Each forward has exactly the same commodity/coin ratio as a state spot purchase
bid, with no interest or discount. This fixture posts one coin per unit for grain,
fish, milk, wool, hay, stone, clay, ores and refined metals. These are explicit
experimental prices, not inferred relative values. Contracts require an exactly
representable quantity at the existing hundredth-unit precision. Providers also
buy materials with coins, and people can buy posted grain with coins, so provider
income can fund both inputs and food.

## Projection and limits

The buyer runs a bounded, deterministic 12-month local reference rollout using
the hypothetical tool. It retains current rights, needs, taxes, inputs, equipment,
processes, experience and work policy. Forecast surplus is positive stock growth
after this work, consumption and taxes; opening stocks do not count as promised
future production. The forward must fit a single commodity's forecast surplus.
Future scripted starts/capacity shocks and other agents' future actions are not
visible. Currently observed shared deposits remain available in the local
forecast, so competition can make it optimistic. This is an expectation, not a
guarantee or a search for an optimal financing strategy.

There is at most one outstanding forward per buyer and one tool purchase per
buyer per Acquire phase. Matching retains stable buyer/kind/provider priority and
fixed provider assignments. The state checks finite coins and receiving storage,
including outstanding forward quantities, when underwriting. Later spot purchases
can use that space; delivery always rechecks physical space. Unsold tools remain
with the provider. Rejection events distinguish no coin funding, insufficient
treasury, disabled advances, existing forward, no projected use and insufficient
eligible surplus (including a receiving-space failure).

## Timing and settlement

The existing experiment schedule is unchanged:

1. Open regenerates capacities and observations; Due settles annual taxes.
2. Acquire first delivers due forward commodities, then resolves tool purchases,
   then posted stock trades, all against bounded opening budgets.
3. Productive work uses tools delivered at that barrier. Later phases settle
   taxes in arrears, consumption and closing conditions as before.

A forward issued in month `m` forecasts `m` through `m+11` and is due in month
`m+12`. Forward claims feed the existing production-planning claim forecast.
Outstanding promised stock is protected against discretionary spot sale,
including a forward created in the same Acquire batch. Collection preserves six
units of grain for the debtor; annual taxes settle first. This protection is an
explicit policy, not universal creditor priority or a guarantee of nutrition.

At maturity, transfer only actual available goods that fit the state's store.
Partial deliveries reduce the balance; the rest stays overdue and is retried at
later Acquire boundaries. Failed harvests do not create goods, coins or fictitious
repayment. There is no automatic write-off, collateral seizure, resale, lender
loss provisioning, negotiated term or secondary contract market yet.

Every accepted transfer has balanced ledger effects. Origination and repayment
records are canonically rebuilt during settlement, so tampered terms cannot be
committed independently of their actual transfers. Forecasting and matching run
on the host; the CubeCL CPU backend gathers and validates account deltas. This
does not establish GPU execution of the planner.

## Validation

Focused tests cover cash-only and financed purchases, exact spot valuation,
finite treasury reservations, disabled financing, no ongoing royalties, maturity,
short/blocked deliveries, tampered origination and repayment, local forecast
information limits, CPU/reference equality, actor reordering, ledger replay and
midmonth continuation. Longer-run observations follow below.

## 72-month comparison

Both cases use 32 people, three providers, the same opening endowments, spot
prices and 25% quote setting. Conditions are deterministic; this is one fixture,
not a multi-seed economic balance study. Quantities below are physical units or
coins, converted from hundredth-unit ledger ticks.

| Observation | State forwards enabled | Cash only |
| --- | ---: | ---: |
| Tools delivered by month 18 | 29 | 26 |
| Tools delivered by month 72 | 45 | 45 |
| Last delivery month | 35 | 42 |
| Total upfront tool sales, coins | 160.00 | 143.25 |
| Forward contracts | 15 | 0 |
| State advances, coins | 23.25 | 0 |
| Outstanding/overdue forward goods at end | 0 / 0 | 0 / 0 |
| Nutrition / warmth deficits | 0 / 0 | 0 / 0 |
| Unpaid annual tax | 0 | 0 |
| Terminal people | 0 | 0 |

All 15 advances delivered their promised commodity by the end of the run. The
first six financed purchases occur in month 13, after the first annual revenue
boundary. The state starts without treasury coins, so it cannot solve an initial
coin shortage by creating an advance. The final three providers hold respectively
8.50, 14.00 and 17.50 coins with financing, versus 3.75, 6.25 and 15.25 without.
This is an ending-stock observation, not a recurring profitability assessment.

Financing accelerates some purchases but does not increase the final delivery
count here. Timing changes future production projections and quotes, so total
sales values differ. The financed run logs 44 no-funding attempts, 201 treasury
shortfalls, three existing-forward rejections and 1,506 no-projected-use attempts.
These are repeated monthly attempts, not counts of distinct excluded people.
A standing tool request alone does not imply productive use in the next year.
Neither financing nor the old three-provider calibration establishes that all
tool requests should clear under this new pricing rule.

The financed 72-month CubeCL CPU run matches the reference run exactly in final
state, every ledger batch and all monthly reports. The full regression suite
passes 109 tests, including six forward tests. Strict
Clippy, formatting and the repository artifact check pass. The long audit can be
run with `--config 'profile.dev.package.economics-compute-smoke.opt-level=2'`
before `--example` to speed up the many host-side forecast rollouts without
changing the model. Raw audit logs stay under ignored `output/economics/`.
