# Current-month payroll funding outlook

Implemented household/shared integration pass 26, September 2026. This extends
the [earned-only funding comparison](INTEGRATION-PASSES-5.md) without changing
monthly execution or the authoritative wage book.

## Static charter choice

`Charter.payroll_outlook` selects `employment::PayrollOutlook`:

- `EarnedOnly` (default): collective wage demand covers existing earned claims.
- `CurrentDelivery`: also estimate wages for this month's deliverable contracted
  hours at the market's Acquire snapshot.

Both retain the `fund_earned_wages` opt-in and collective purchasing route.
Member employers additionally require `support_member_wages`. Needs-first buying,
market admission, legal permissions, finite payment stock and matching offers
remain necessary. Leaving membership ends subsequent collective funding demand;
it neither cancels employment nor transfers personal debt to the household.

The public read-only `employment::projected_payroll` reuses the delivery evaluator.
It checks contract dates, active parties, permissions, suspended arrears, own-hour
household contributions, shared worker capacity and household hiring ceilings and
affordability. Competing contracts use the same rank/ID ordering. It returns wage
denomination totals by employer, not hypothetical worker income or spendable money.
There is no estimate outside Acquire.

## Boundaries and accounting

At Acquire, enabled estimates join collective order targets and market stock
protection. Actual earned claims and estimated new wages remain separate inputs;
private stock offsets each member's combined requirements once, then collective
stock reduces the remaining shortage. Existing order receipts expose the before
and after deficits. A whole market lot can exceed the shortage.

The estimate does not commit its hypothetical employment book or transactions.
Only actual Acquire delivery creates receivables, payables and paid-service cost.
Before Close, household assistance still requests only **earned** member shortfalls.
Ordinary Close settles the real claims and records actual payment; household books
retain assistance expense rather than assuming the member's wage liability.

This is a funding estimate, not a dated labor reservation or payment escrow.
Later acquisition spending or production reservations can reduce actual delivery;
subsequent obligations, allocation and worker storage can reduce payment. Market
forecasts use their observed snapshot, while execution still enforces remaining
opening resources. Any excess purchased payment stock remains inventory/cash.
The actual claim readers used by accounting, collection, hiring affordability and
household assistance continue to return earned claims only.

Household employers still require opening affordability. This outlook does not
allow a household without hiring cash to buy coins and hire in the same Acquire.
The improved loop below uses an individual member employer, whose ordinary
preaccepted contract permits earning wages on credit. Forecasts do not establish
that this hiring is profitable or sustainable indefinitely.

## Six-month comparison

Identical opening scenarios differ only in the payroll outlook within each pair.
The household starts with four coins. A member buys two labor units for four coins,
produces four grain and contributes half. A supplied counterparty sells four coins
for two collective grain per monthly lot. There are no consumption needs in this
controlled payroll scenario. Work targets, employment terms and prices are supplied.

| Counterparty opening coins | Outlook | Working months | Produced grain | Wages paid | Ending unpaid wages |
| --- | --- | ---: | ---: | ---: | ---: |
| 12 | EarnedOnly | 4 | 16 | 12 | 4 |
| 12 | CurrentDelivery | 5 | 20 | 16 | 4 |
| 20 | EarnedOnly | 4 | 16 | 12 | 4 |
| 20 | CurrentDelivery | 6 | 24 | 24 | 0 |

Earned-only demand waits until month three to fund month-two arrears; work has
already suspended. Current-delivery demand buys payment stock in month two before
that shortfall. With twenty external coins, all six months deliver and settle.
With twelve, the counterparty runs out of coins: month-five wages remain unpaid
and month-six delivery suspends. No currency is issued or recycled from future
worker receipts. Total coins remain sixteen or twenty-four respectively.

CPU/reference runs agree under reversed participant/resource order. State, full
ledger and Audit agree with continuation from Open three and full replay. Separate
financial statements reconcile; no member payable appears on the household book.
Additional controls cover private-stock offsets, disabled funding/support, accepted
member exit, competing employers, dates, law permissions, arrears, contribution
limits, household affordability and whole-hour hiring ceilings. Preview calls leave
state and financial books untouched.

A `Continue` control combines six old wages with ten expected new wages and two
private coins. Demand is fourteen coins, not twelve: private funds offset once.
The available four-coin lot lets Close clear the old six; ten newly earned wages
remain unpaid. A projection therefore neither pays a claim nor inflates available
stock, and carried debt is not counted again as new payroll.

## Limits and verification

This is one current-month outlook, not employment discovery, long-horizon financing,
a cash-flow optimizer or simultaneous resolution of all markets and work plans.
Wage insolvency and guarantees, internal household hiring and household-as-worker
delivery remain separate gaps. Person self-directed policy changes remain deferred.

Verification: **113 distinct tests passed across 12 selected suites**. These cover
member employment, household hiring, employment, need orders, household markets,
household finance loops, town markets, service accounting, household accounting,
household credit, acquisition and household composition. The existing slow annual
32-person accounting test remained ignored; the full crate suite was not run.
The final member-employment run includes one additional old/new-claim control;
overlapping focused runs are not counted twice.

Formatting, strict all-target Clippy, whitespace checks and the repository artifact
policy passed. Generated logs stay under ignored
`output/economics/payroll-outlook-*.log`; only source, tests and Markdown are committed.
