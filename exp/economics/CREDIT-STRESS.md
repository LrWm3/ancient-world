# Harvest-funded loan stress controls

A borrower buys one productive plot from the state, which also lends and buys
surplus grain. Three 24-month CPU controls vary only unanticipated labor outages.
They exercise repayment, recovery from arrears, and repossession without adding a
new credit mechanism or changing default lending policy.

## Settings and information boundary

The fixture builds on [production-funded credit](PRODUCTION-FUNDED-CREDIT.md):
a 100-coin plot, 20 down, 80 borrowed, 1% monthly interest on outstanding principal,
and twelve principal installments. One coin is 100 accounting ticks. The borrower
starts with 65 coins (including bridge savings), five grain and **three seed**.
The lender starts with 1,000 coins. No subsequent cash gifts or minting occur.
Starting with only the downpayment would fail before any harvest could fund debt.

A completed six-month crop produces twelve grain and one seed. The existing
posted bid pays twelve coins per grain, with a cumulative 120-coin purchase
allowance. Sales protect six months of food and are limited to two grain per
month, available stocks, buyer funding and storage. Cultivation rights follow
title through month 120. Both parties have 100 storage units; coins need no space.
The ordinary need-directed production policy remains in use; acceptance of the
mortgage is scripted to isolate servicing after purchase.

Compared with that base fixture, all three controls get three starting seed,
an **eight-month arrears grace period**, and a lender with two labor units/month.
Spare seed permits replanting after crop failure. Grace gives the replacement crop
time to produce saleable goods. Lender labor is an explicit service endowment,
not a consequence of receiving title. It enables maintenance after repossession.

- **Normal:** no capacity outage.
- **Temporary:** borrower labor is zero in month 6, so the first crop aborts at
  harvest. This is a lost harvest from missed work, not a stochastic yield model.
- **Persistent:** the same outage in months 6 and 12 destroys two crops. There
  are no further outages; repeated losses are enough to trigger enforcement.

Outages use the existing Open capacity override. Private forecasts cannot read
future overrides. Seeds, labor and failed crop work pass through ordinary process
transactions; the runner never patches completed outputs or loan balances.

## CPU outcomes

Amounts below are coins except grain and unmet nutrition units.

| Control | First arrears | Debt cleared | Final plot owner | Cash interest paid | Grain sold | Final borrower coins | Unmet nutrition |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: |
| Normal | None | Month 13, cash | Borrower | 5.20 | 10 | 79.80 | 0 |
| Temporary | Month 8 | Month 15, cash | Borrower | 6.72 | 8 | 54.28 | 6 |
| Persistent | Month 8 | Month 16, collateral | Lender | 4.20 | 0 | 17.67 | 16 |

Financial recovery is not a claim that the temporary shock was harmless: six
nutrition units go unmet while awaiting the replacement harvest. This fixture has
no physiological condition feedback, starvation death, emergency food market or
lender relief. Its deterministic fixed bid does not test competitive price discovery.

The temporary case harvests in month 12. Month 13 Due still observes zero cash
and 41.16 debt. Acquire then sells two grain for 24 coins; those proceeds first
service debt in month 14. Another sale funds final repayment in month 15, releasing
the pledge while the borrower retains title. Interest is paid before principal.
Maturity is month 13; the grace rule permits this later cure.

The persistent case first misses in month 8. Month 16 Due reaches the eight-month
grace boundary and settles against the agreed **60-coin fixed land value**:
42.33 debt credit and 17.67 cash surplus to the borrower. Cash interest paid before
enforcement was 4.20; another 3.13 interest is cleared through collateral, not cash.
The crop contributes nothing to valuation. Both debt views become zero and the
pledge clears; the state now owns the plot valued at 60 coins in this scoped view.

**A terminal loan status of `Repaid` is insufficient evidence of cash repayment.**
The existing status means debt is cleared, including through enforcement. The
`Enforced` event and ownership history identify the route. This experiment keeps
that domain behavior and exposes its receipts rather than changing status semantics.

## Crop and settlement boundaries

The third crop starts in month 13. At month 16 Due, title, operator and output
beneficiary transfer together to the lender. Stage, elapsed work, occupancy and
active status remain intact, and the old personal goal is cleared. No harvest is
created at transfer and the borrower's labor does not move. The lender supplies
its own work during Productive; the crop completes in month 18 with twelve grain
and one seed belonging to the lender.

Open supplies dated capacity; Due accrues, collects and enforces; Acquire purchases
and sells stocks; Productive does work; consumption and Close follow. Receipts
remain at their actual boundary. No sale retroactively pays an earlier installment,
and no scheduler or allocation priority changed.

## Inspection and validation

From `exp/economics`, use a fresh ignored directory:

```sh
TELEMETRY_DIR=../../output/economics/credit-stress-demo \
  cargo +1.92.0 run --locked --example credit_stress
cargo +1.92.0 test --locked --test credit_stress
```

The example runs all three cases, prints monthly finalized journal equity, operational cash, debt and title diagnostics and credit
events, and writes JSONL through the ordinary observer. `settlement: true` now
exports `loan_state`, `loan_event` (accrual, payment, arrears and fixed enforcement),
`credit_stock_sale` with limiting quantities, and `collateral_process_transfer`.
These read committed domain receipts. They are not a complete observer for resale,
all credit rejection reasons or full balance sheets. Agent filters retain both
sides of a selected loan or sale; log limits still apply.

Tests check symmetric principal/interest claims and total coins at every monthly
boundary, conservation of non-Open transfers, collateral release, enforcement
versus cash clearance, Due-before-sale timing, and intact crop transfer followed
by actual lender production. All three controls compare reference batched runs
against CPU monthly runs with checkpoint reconstruction before Due and reordered
participants/catalogs. Observer execution must match unobserved state and ledger.

All 36 focused tests passed: five new stress/observer tests, nine credit, five
collateral-crop, seven stock-sale and ten existing telemetry tests. Strict
all-target Clippy and formatting checks passed. The full crate suite was not run;
raw CPU results and JSONL remain under ignored `output/economics/`.

The state still combines lender, land seller and grain buyer. This is one secured
claim, not general insolvency, multi-creditor priority, negotiated refinancing or
an independent bank model. A useful follow-up would compare grace durations or
separate the grain buyer from the lender while keeping these controls unchanged.


The former state-derived balance-sheet fallback has been removed. All three cases
now use the double-entry adapter through audited telemetry. Opening land cost is
explicitly 10,000 reporting ticks and opening stock cost is one tick per unit;
grain and seed outputs share production cost equally. These are reporting
conventions, not planner prices. Crop WIP follows validated attachment transfers
at historical cost without changing collateral credit or remaining debt. Monthly
reports are finalized only after completion; no snapshot-equity fallback exists.
