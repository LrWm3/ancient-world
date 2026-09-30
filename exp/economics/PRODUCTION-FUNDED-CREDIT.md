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
| Food-tight, reserve disabled | 6 coins | 120 coins | Decline: need limit | Would repay, but 9 nutrition deficits |

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
loan payable at the cost of nine unmet nutrition units. The original comparative borrowing
score accepted against its own worse decline branch under that same selling policy.
The fixture now sets an absolute zero-deficit nutrition cap and rejects this loan.
Both branches remain visible in the receipt. This does not repair harmful selling
or guarantee food when declining; see [absolute need limits](BORROWING-DECISIONS.md#absolute-need-limits).

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

## Optional forecast-based sales

[Bounded sale planning](SALE-PLANNING.md) adds an opt-in quantity comparison through
ordinary production and consumption. The fixed-reserve fixtures above remain
controls. The new policy can sell safely before an imminent harvest, but its
short horizon can worsen later scarcity; it is not the default.

## Authorized recovery and ordinary stock sales

The fixed-reserve sale policy now composes with authorized mortgage liquidation.
At Acquire, it reads the current loan/recovery book and suppresses ordinary sales
when either seller or buyer has an active proceeding. Its receipt and settlement
log identify the stayed party; actual quantity remains zero even with surplus
stock and funded demand. Estate sales still use authorized bids and custody.
Closure at the existing Due boundary restores eligibility for subsequent ordinary
exchange; sale receipts cannot be reused for another outgoing action at Acquire.

The continuing crop/mortgage test compares funded and unfunded property sales.
Before the stay, grain sales pay two of six principal units. A real four-coin
property sale later clears the balance and releases the stay; an unfunded bid
leaves four due and ordinary sales paused. Crop control follows the property sale.
Separate books, CPU/reference execution, reconstructed checkpoints and tampered
receipts are checked, along with a separate insolvent-buyer control. Custodians
remain ineligible traders. Joint work/sale planning is now checked separately below.

Fixed-reserve member sales also compose with household pooling. Two one-coin sales
pool one actual coin and leave one private coin for the member's mortgage. The
household is not a co-borrower: its balance remains outside the member's estate.
The continuing test therefore needs five property-sale coins to clear the loan,
compared with four without pooling. Later sales pool once after closure. The
funded/unfunded matrix checks separate statements, physical output, custody,
CPU/reference and reconstructed continuation. This enables a participant who is
a household member; a household itself is not a stock-sale participant in this
specialized driver. Household joint work/sale policy remains guarded.

The bounded sale forecast now shares that recovery path. Active stays cap its
candidate quantity at zero, while ordinary production/consumption continues in
the projection. The test adds a real monthly nutrition need and verifies both
funded closure and ongoing insolvency without food deficits. A zero-sale candidate
can be feasible without repairing the debt or authorizing forbidden exchange.
Forecasts remain conditional on configured future recovery bids and real bidder
funds; this is not autonomous liquidation-price discovery.

Member sale forecasts now run through the household wrapper too. They preserve
private food, contribution carry, separate collective cash and the member's
estate. The same nutrition-constrained test projects both an actually funded
property sale and a continuing stay; it does not assume that collective balances
can pay a private debt. Forecasts remain read-only, and reconstructed CPU and
reference continuations agree with the double-entry statements.

The single-participant joint work/sale planner now composes with recovery too.
Its stay-constrained alternatives continue already committed cultivation and
produce the usual dated Productive batch. A funded sale after harvest releases
property and later closes the estate; an unfunded bid preserves the original
claim. Forged plan dates fail atomically. Both executions retain food constraints,
separate books and CPU/checkpoint equality. Household joint labor allocation,
multiple producers and negotiated joint plans keep their existing scope limits.

Stock-sale reservation now inherits the household contribution budget from earlier
estate purchases and financing. Cumulative fractional shares constrain both actual
lots and forecast candidates. `contribution_limit` records that final feasible
candidate ceiling alongside raw storage and funding limits. The regression in
`tests/inventory_liquidation.rs` reproduces the former household-storage failure
and verifies three/two/zero fills under identical physical constraints.

Fixed-reserve and bounded-forecast mortgage sales now compose with independent
land leases (`tests/mortgage_lease.rs`). Sale proceeds become spendable at the
later Due boundary. Stable and proportional collection preserve their allocation
rules for scarce cash; accepted native rent remains a goods obligation. Household
members pool half their actual sale income, and the ordinary household rent rule
can explicitly transfer that collective coin back to support a later bill. This
is a recorded contribution, not an assumption of the member's debt. Separate
books, forged receipts and CPU/reconstructed continuation agree. A lease on the
financed parcel itself is still rejected. Joint work/sale planning now has the
lease/prepayment verification described below.

## Continuing person/household farm finance

`tests/household_farm_finance.rs` composes repeated cultivation and seed return,
a financed parcel, a separate native-grain lease, two prepaid grain deliveries,
posted stock sales, nutrition and optional household membership for 24 months.
The calibrated crop yields 20 grain per harvest. The loan is explicitly accepted;
prepayments and finite buyer allowance are supplied terms, not autonomous demand.

The bounded six-month sale forecast protects nutrition while ordinary work and
accepted claims execute. Household production and sale income pool under the
existing agreement, and private loan payments remain private: this fixture does
not enable member-loan support. Financial statements reconcile separate member,
household and state positions. Reference, CPU and a reconstructed continuation
cross the first promised delivery and the annual rent date.

Controls matter here. Removing the stock-sale allowance leads to repossession.
A six-month fixed stock buffer can instead repay the mortgage, pay rent and
complete both deliveries while still missing a meal. Financial claim performance
alone therefore does not establish that this economic loop meets essential needs.
The forecast result is bounded by these yields, terms, observations and horizon;
it is not proof of indefinite sustainability or reliable underwriting.

The single-participant joint work/sale planner now includes independent leases
and direct prepayments in the same continuation. Its dated Productive batch is
built after ordinary acquisition, including accepted advance receipts and actual
forward deliveries. A grain-forward case remains feasible. A control that delivers
the only seed correctly reports an infeasible continuation and never schedules
planting from that spent seed. Both retain real performance, separate books,
CPU/checkpoint equality and atomic rejection of altered plan dates. Household
joint allocation and negotiated joint plans retain their separate guards.
