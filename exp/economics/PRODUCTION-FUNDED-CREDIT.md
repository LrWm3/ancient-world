# Production-funded credit pilot

Implemented as an opt-in extension of the bounded borrowing experiment. A state
posts an existing `currency::Bid` for grain. A scoped `stock_sale::Policy` sells
surplus against it, using the same transactions, budgets and storage checks in
reference forecasts and live CPU execution. No speculative harvest valuation pays
a loan: only completed grain-for-coin transfers provide spendable proceeds.

## Boundaries and limits

Due installments still precede Acquire. Acquire resolves the financed purchase,
then bounded stock sales; production and consumption follow. A month-6 harvest
can first sell in month 7, after that month's installment. Sale proceeds cannot
retroactively pay it or fund another outgoing action within the same batch.

Before selling, retain six months of grain required by direct consumption recipes,
plus grain inputs for one productive process cycle. Separate seed is never offered.
The reserve estimator uses the cheapest enabled single-stage, single-stock-input
recipe for each need and the largest productive input requirement across enabled
recipes. It is a scoped direct-recipe heuristic, not a substitution-network or
multi-crop commitment planner; six months of stock is not a guarantee of sustained
nutrition. Zero reserve is permitted for an explicit diagnostic control.

Sell at most two one-grain lots per month, further bounded by available surplus,
the state's opening coins, its remaining posted purchase allowance, and receiving
storage. Both parties must be active. Receipts retain demand, reserve, quantity,
funding and storage limits, and actual goods/coins. The cumulative allowance is
persisted and validated with the credit boundary; it is not escrow or issuance.
The existing stock exchange supplies conserved accounting legs. Forged receipts
and repeated batches are rejected atomically.

This integration supports one configured seller/bid in the credit pilot. It does
not establish a shared multi-seller allocation mechanism, negotiate price, or
compose arbitrary acquisition drivers. Other credit isolation guards remain.

## Controlled CPU experiment

The 18-month fixture uses a 100-coin plot, 20-coin downpayment, 80-coin loan,
1% monthly outstanding-principal interest and twelve installments. The person
starts with 65 coins, five grain and one seed. A six-month crop produces twelve
grain and one seed. Ownership-following cultivation rights run through month 120,
independently of the observation window. Nutrition demand is one unit/month. Grain and seed occupy
storage; coins do not. Each party has 100 storage units. State opening coins are
1,000; its separate posted purchase allowance is 120 coins unless limited below.

Compared with the earlier borrowing fixture, term increases from four to twelve
months, crop yield from eight to twelve grain, and savings change from 103 to 65
coins. These explicit fixture changes let harvest proceeds matter while retaining
bridge funding. Prices are experimental calibrations, not historical estimates.

| Case | Bid price / grain | Purchase allowance | Borrowing decision | Purchase projection |
| --- | ---: | ---: | --- | --- |
| Funded | 12 coins | 120 coins | Accept | Repays; 0 nutrition deficits |
| Limited allowance | 12 coins | 12 coins | Decline | Misses month-9 installment |
| Food-tight price | 6 coins | 120 coins | Decline | Misses month-11 installment |
| Food-tight, reserve disabled | 6 coins | 120 coins | Accept | Repays, but 9 nutrition deficits |

Funded execution sells two grain in each of months 7, 8, 15 and 16: eight grain
for 96 coins over 18 months. The loan repays in month 13, with 5.20 coins total
interest and 7.80 coins remaining then; later sales leave 55.80 coins at month 18.
Declining under this policy produces seven nutrition deficits versus zero when
borrowing. Monthly sales, closing money and deficits match the selected
forecast. With only downpayment savings, the same funded bid cannot prevent a
month-2 payment shortfall: future harvests do not bridge earlier obligations.

The limited case constrains the posted allowance, not the state's entire treasury;
separate boundary tests constrain actual coins and storage. Rejected branches
remain counterfactual. Collateral enforcement can clear their debt, so zero ending
debt alone is insufficient evidence of successful repayment.

Disabling the food reserve sells opening food immediately and makes the cheap-price
loan payable at the cost of nine unmet nutrition units. The comparative borrowing
score still accepts against its own decline branch under that same selling policy.
It does **not** impose an absolute acceptable-food threshold. This control shows
why a repayment check cannot replace protection of essential stocks.

The ordinary production policy is unchanged. The original fixture inherited a
cultivation right ending in month 9 from the short baseline experiment, which
prevented a second crop and caused two late food deficits. Extending this fixture's
right to month 120 removes those deficits without a planner change. The
[repeated-credit audit](REPEATED-CREDIT.md) preserves the old right as a control and
checks six harvests with no nutrition deficits over 60 months. This is a finite,
deterministic run, not evidence of resilience to shocks or indefinite permission.
Known posted bids survive forecast cloning, while future
scripted gifts and shocks remain hidden; deterministic projections are not promises
of future access or yields.

## Verification

From `exp/economics`:

```sh
cargo +1.92.0 run --locked --example stock_sale
cargo +1.92.0 test --locked --test stock_sale --test borrowing --test credit --test credit_offers --test resale --test forecast_context --test storage_currency
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Seven stock-sale tests cover the controls, bridge timing, conservation, reserve/quantity/
funding/storage bounds, allowance exhaustion, forged receipts, replay rejection,
and CPU/reference, monthly/batched and checkpoint continuation equality. The repeated-credit change reran stock-sale, borrowing, credit and credit-offer
suites: 24 tests passed, along with all-target Clippy. The earlier integration
also checked resale, forecast-context and storage-currency suites.
Generated run output stays under ignored `output/economics/`.
