# Shared buying and selling horizons

Implemented an opt-in shared order horizon and compared twelve-month CPU runs.
Matching buy-demand and sell-protection windows eliminates the observed contradictory
stock signals, but neither tested alignment improves the overall outcome. The
existing default remains unchanged pending further experiments.

## Configuration and boundaries

`town_market::Config.order_horizon` selects `Legacy` or `Aligned(months)`.
Legacy preserves the production planner's horizon for buying (one month without
that planner) and `reserve.reserve_months` for selling. Aligned uses one value for
both calculations, validated against the existing 1–24-month reserve bound. It
applies to all listings in the venue and survives cloning into forecast worlds.
The production forecast horizon stays six months in this experiment.

The order generator still observes opening Acquire balances/admission, computes
needs and protected stock, and applies the unchanged buy-first, one-side-per-good
rule. Matching still enforces money, goods and storage against a shared budget.
Neither horizon variant reserves hypothetical production or changes the scheduler.
The comparison changes the window for existing commitment claims as well as
consumption protection; this fixture contains no payment obligations.

Each `order_generation` receipt now includes `buy_months` and `reserve_months`,
including for skipped evaluations. This makes the actual windows visible without
inferring them from the name of an experiment.

From `exp/economics`, use a fresh output directory for each case:

```sh
MONTHS=12 CASE=both ORDER_HORIZON=legacy TELEMETRY_PLANNING=selected \
  TELEMETRY_SETTLEMENT=true TELEMETRY_DIR=../../output/economics/horizons-legacy \
  cargo +1.92.0 run --locked --example reciprocal_market
MONTHS=12 CASE=both ORDER_HORIZON=2 TELEMETRY_PLANNING=selected \
  TELEMETRY_SETTLEMENT=true TELEMETRY_DIR=../../output/economics/horizons-2 \
  cargo +1.92.0 run --locked --example reciprocal_market
MONTHS=12 CASE=both ORDER_HORIZON=6 TELEMETRY_PLANNING=selected \
  TELEMETRY_SETTLEMENT=true TELEMETRY_DIR=../../output/economics/horizons-6 \
  cargo +1.92.0 run --locked --example reciprocal_market
```

Omitting `ORDER_HORIZON` preserves legacy behavior. Same four persons, initial
stocks/coins/skills, fixed prices, demand signal, policies and backend in all three
runs; no seed or randomness controls differ.

## Results

| Twelve-month observation | Legacy 6/2 | Aligned 2/2 | Aligned 6/6 |
| --- | ---: | ---: | ---: |
| Grain traded | 16 | 0 | 8 |
| Wood/fuel traded | 0 | 2 | 0 |
| Nutrition deficit | 0 | 0 | 0 |
| Warmth deficit | 2 | 12 | 4 |
| Aborted process transitions | 0 | 0 | 0 |
| Months with both wood buyers and sellers | 0 | 1 | 0 |
| Submitted buys also above sell-protection threshold, both goods | 18 | 0 | 0 |
| Ending coins: 88 / 89 / 91 / 92 | 32 / 32 / 16 / 16 | 26 / 26 / 22 / 22 | 32 / 32 / 16 / 16 |
| Completed forecast windows predicting wood sales but realizing none | 22 | 0 | 4 |

No attempted match fails in these runs. Each configuration produces 28 completed
six-month forecast comparisons; later windows are pending, and these rolling
windows overlap. Only two completed 2/2 windows predict positive wood sales, so
its zero missed-sales count does not establish general forecast accuracy. All
trade totals and deficits use native resource units, not a combined utility score.

The stock-signal check examines evaluated, submitted buy receipts whose opening
available quantity minus protected quantity is at least one market lot. Aligned
windows reduce that count from 18 to zero in this fixture. The controlled boundary
test isolates why: with three wood units, 6/2 submits a buy despite protecting only
two; 2/2 finds no purchase improvement and offers a sale; 6/6 protects all three
units and submits a buy. This is not a universal theorem for complex substitutions
or indivisible recipes, whose demand/protection semantics can still differ.

### Two months: some wood trade, but worse need satisfaction

The two wood units trade only in month eight: 88 sells to 91 and 89 sells to 92.
This is opposite to the skill-based roles of the directed control, and not sustained
specialization. There are no grain buy orders: 44 opportunities are excluded by the
selected purchase policy and four fail the need-improvement test. There are 34 grain
sell orders, but no buyers. The reduction in grain exchange accompanies twelve
warmth-deficit units, mostly among 91/92, while nutrition stays satisfied.

Thus shortening the horizon removes the conflicting signal and enables one wood
exchange boundary, but short-window demand and the resulting selected plans do
not support the desired complementary production. This observational comparison
does not isolate the individual contribution of each changed plan.

### Six months: stronger protection reduces offered supply

Only four grain sell orders are submitted; 26 sell-side opportunities are rejected
as protected stock and eighteen are skipped after a buy is selected. Grain trades
in months four and ten only. Wood has no sell orders: fourteen evaluations protect
all available stock, six lack one lot, and 28 sell sides are skipped after buying
is selected. Alignment removes the surplus/buy overlap, but does not establish a
seller population; all four persons have a warmth shortfall in month three.

## Interpretation and next step

A common horizon is a useful explicit modeling option, and the observers confirm
that it removes this specific inconsistency. It is not sufficient to make the
market coordinate complementary work. Changing the default solely because one
variant generates a wood trade would obscure its much worse need satisfaction.

Keep the three configurations available. The remaining problem is whether agents
can form mutually supportable production and purchasing plans, using realistic
expectations of counterparties. Investigating those expectations remains more
informative than adding goods or tuning prices to force volume. No order-side
policy or counterparty prediction change is bundled with this experiment.

## Verification

The fresh legacy run exactly reproduces the prior run's plans, forecast comparisons,
transactions, balances, needs and market observations. All three streams finish
without omitted logs. Match quantities reconcile to per-market volume; each coin
transaction conserves coins. Focused tests cover aligned versus mixed horizon
signals, invalid durations, unchanged planner horizon, and CPU/reference equivalence
under reordered participants and checkpoint continuation. All 36 need-order, town-market and production-planning tests passed, along with
formatting and strict all-target Clippy checks. Raw logs and outputs
remain under ignored `output/`; the full crate suite was not rerun.
