# Bounded financed-purchase decisions

Implemented in `src/borrowing.rs`, opt-in through `credit::Config.purchase_policy`.
`Scripted` preserves prior controls, `Decline` supplies a no-purchase control, and
`Compare` chooses between accepting and declining the configured financed offer.
The fixture has one person and the existing state counterparty, one plot, a crop,
slow food gathering and a nutrition need. No new exchange market was introduced.

## Decision and monthly boundary

At the configured month's Acquire boundary, comparison starts from the same
[observed snapshot](FORECAST-CONTEXT.md) for both branches. Acceptance uses the
common financed-offer interface; decline commits Acquire without a purchase.
Hypothetical branches disable recursive comparison and execute the same ordinary
monthly production, consumption and credit settlement used in the live run.

Both branches continue through the configured horizon. It must cover all loan
installments, including the opening month, and is capped at 24 months. The fixture
uses 12 months, allowing the six-month crop and four-installment mortgage to be
observed. The context retains accepted schedules and current resources, but hides
future capacity overrides and discretionary cash transfers.

The rule is deliberately conservative:

1. Reject an infeasible initial purchase.
2. Reject any purchase projection with a missed installment or outstanding debt
   at the horizon. Even a same-boundary repossession that clears debt does not
   hide a missed installment.
3. Compare terminal state, accumulated need deficits ordered by need priority
   then resource ID, newly failed processes, and closing debt minus coins, in
   that order. Prefer decline on ties.

A receipt stores both projections, the selection and reason. Each monthly row
shows actual projected coins, debt, need deficits, labor consumed and recorded
arrears. Grain is not added to cash or priced as income. Labor totals count
productive/consumption use, not capacity reset effects. Comparison uses feasible
process execution rather than assuming all desired work can finish.

The selected purchase, payments, title and loan creation still commit atomically
through credit settlement. The existing validator recomputes the decision and
rejects missing or altered receipts. Decline creates no loan or reservation.
Outputs become visible before Productive as before; no phase was added or moved.
The decision runs once for this application date, not as a recurring loan search.

## CPU controls

Common terms: plot price 100 coins, downpayment 20, principal 80, 1% monthly
interest on outstanding principal and four principal installments. Paid on time,
the total cost is 102 coins, including two coins of interest. One coin is 100 ticks.
The person starts with five grain, one seed and two labor units per month, and
needs one nutrition unit monthly. A crop takes six months and returns eight grain
plus one seed. Fallback gathering produces one grain after three months, requiring
two labor units each month. Both branches use the existing need-driven production
policy; there is no scripted farming start.

| Control | Opening coins | Decline nutrition deficit | Purchase nutrition deficit | Purchase financing outcome | Choice |
| --- | ---: | ---: | ---: | --- | --- |
| Affordable productive plot | 103 | 3 | 0 | Repaid by month 5; 1 coin remains | Accept |
| Downpayment only | 20 | 3 | 4 | Missed month-2 installment; 21.60 deficiency after repossession | Decline |
| Low-yield plot (one grain per crop) | 103 | 3 | 4 | Repaid; 1 coin remains | Decline |

Deficits are summed unmet nutrition units over 12 months. The purchase column is
counterfactual for declined offers: the actual agent keeps its coins and does not
default. In the accepted control, live CPU results match the selected forecast.

The affordable control is **prefunded**, not a demonstration of production-funded
repayment or a need for leverage. It deliberately has sufficient cash for all
payments. There is no outright-cash purchase alternative yet. The downpayment-only
control demonstrates why enough cash to originate a loan is not enough to service
it, and extra grain cannot change that without a real exchange opportunity.

The first calibration allowed overlapping low-labor gathering processes to cover
all nutrition needs within the horizon. Declining then correctly won even for a
productive plot. Requiring the full monthly labor budget for gathering makes land
productivity useful in the retained comparison; no score or acceptance threshold
was adjusted to force a purchase.

## Limits and next integration points

This is a two-alternative policy over one configured applicant, offer, downpayment
and date. It does not search lenders, negotiate terms, refinance, or compose a
purchase with other acquisition drivers. Resale buyers and remaining-value work
policy are excluded from this comparison. The ordinary production policy is used
in both forecast and live execution. Existing unsupported-system guards remain.

Need priorities dominate money in this rule. There is no common utility scale,
time discount, priced labor disutility or valuation of the retained plot at the
horizon. Stock surplus and asset appreciation are not speculative coin income.
Current fixture nutrition deficits are measured; no additional deprivation rule
was added. Uncertain yields and shocks can invalidate a forecast after acceptance.
A horizon long enough to repay debt is not proof of lifetime sustainability.

A later increment could make an actual grain-sale opportunity available to both
branches, then assess production-funded installments using committed exchange
rules. Automatic borrowing must not treat a configured future gift or unsold
harvest as available money.

## Verification

From this directory:

```sh
cargo +1.92.0 run --locked --example borrowing
cargo +1.92.0 test --locked --test borrowing
```

Four focused tests cover the three choices, live versus selected projection,
resource limits, hidden gifts and shocks, abundant grain without coin income,
insufficient downpayment, short horizons, atomic rejection of forged decisions,
CPU/reference equality, monthly/batched execution and checkpoint/catalog-order
continuation. Credit, common-offer, resale, forecast-context and loan-view suites
bring the tested total to **31 passing tests**. All-target Clippy passed. Generated
CPU results remain under ignored `output/economics/`.
