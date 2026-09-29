# Five household and consolidation passes

This records the first batch. The [second batch](INTEGRATION-PASSES-2.md) extends
the physical barter, member buying and accepted-process input limits below.

Work requested 2026-09-29: alternate household/organizational behavior with shared
gap consolidation, demonstrating formerly isolated combinations. Person
self-directed policy changes are explicitly excluded. No scheduler rewrite.

| Pass | Household/organizational work | Shared consolidation and evidence | Status |
| --- | --- | --- | --- |
| 1 | Useful voluntary support under needs-first governance without a market | Shared stock protection for consumption, commitments and voluntary offers | Complete |
| 2 | Member wage income and bounded household labor contributions | Employment + pooling + separate financial statements | Complete |
| 3 | Private member surplus sales alongside collective purchases | Town trade collection without duplicated needs | Complete |
| 4 | Collective cash reserve decisions | Direct lending + town acquisition sharing finite opening resources | Complete |
| 5 | Collective adaptation to income and membership changes | Combined scenario, adverse controls, replay and continuation | Complete |

Each pass records its actual scope, validation and limits below. Configuration and
accepted agreements may remain supplied; these passes do not implement autonomous
personal policy revision or all forms of organizational governance.

## Pass 1 — voluntary support and shared stock protection

NeedsFirst households now accept authorized private surplus when it improves
member need fulfillment without harming the donor. This works without a market;
NeedsThenIncome retains its secondary cash objective. Market sell orders and
support use one commitment/consumption protection calculation.

Evidence: two integration tests cover food sharing without a town market,
private food protection, forged receipt rejection, CPU/reference parity and
ledger replay. Existing need-order and household-income controls also pass.
Voluntary mandates remain supplied, not autonomously renegotiated.

## Pass 2 — earned wages, household labor and paid-income pooling

An independent employer can now employ a household member. Acquire protects the
household's percentage of own labor; Productive retains that entitlement after
external work. Close pools half of actual paid wages with fractional carry,
leaving wage receivables outside the cash budget. Existing monthly phases remain.

Evidence: CPU/reference audited integration earns four of five available hours,
retains one promised hour, pools paid wages and purchases household food the
following month. An unfunded employer pays only three coins, shares one with the
household, carries 77 in arrears and suspends subsequent delivery. The nine
employment controls remain green.

Limits: household/member employers and onward allocation of purchased labor are
still rejected pending a cost-basis adapter. Member wages currently use a
storage-free stock denomination; physical wage pooling needs storage reservation.

## Pass 3 — private surplus and a collective town account

Members may now submit fixed-side sell orders while their household buys for
member needs. Consumption and contract reserves protect private stocks. Actual
town sale proceeds participate in half-income pooling, using town transaction
receipts rather than classifying all untyped transfers as income. Loans therefore
remain outside the contribution base.

Evidence: audited CPU/reference runs sell member grain to an independent buyer,
retain private food and pool coins. The collective cannot spend those incoming
coins in the same Acquire boundary. Duplicate private buying/adaptive orders and
physical payment are rejected. Existing household market and accounting tests pass.

Scope: member sales currently require a storage-free payment resource. General
barter needs pooling-aware storage reservations before that restriction can lift.

## Pass 4 — bounded cash objectives and shared loan/market acquisition

The static charter may set a cash target in the town payment denomination.
NeedsThenIncome values extra cash only up to that projected next-book balance;
needs still take precedence. Both voluntary support and labor use the same
comparison. Omitted targets preserve the existing income objective.

Direct accepted loans now compose with household town books. Acquire reserves
loan transfers first, then clears markets against remaining opening budgets.
Incoming loan principal cannot finance another transfer in the same batch.
Receipt validation checks the combined result atomically. Due still services
loans before the later market boundary.

Evidence: buffer controls release unnecessary labor and resume it below target;
audited CPU/reference loan-plus-market scenarios check loan principal, subsequent
repayment and purchases, lender budget exhaustion and forged combined settlement.
Loan discovery remains supplied. Town recovery proceedings, mortgage purchase
configuration and joint production planning remain outside this composition.

## Pass 5 — changing income/membership and combined boundaries

The collective now exercises the preceding mechanisms together: private grain
sales, donated fuel, contributed work, external member wages, grain/fuel town
books, a static cash target and an accepted direct loan. The eight-month control
removes one member's productive capacity in month 3, restores it in month 4,
accepts their exit at month 5 and their consensual accession at month 7.

Observed assertions: no wage is earned for absent capacity; earnings resume when
capacity returns. The outside worker supplies five hours while unaffiliated and
four after rejoining, restoring one household hour. Paid wages stop pooling during
exit and resume on accession. Voluntary support, private sales and directed work
all occur; the loan repays completely. CPU and reference execution agree on state,
ledger and separate financial books, including continuation from month 5 Open
and batched monthly execution. Total coins are conserved.
These controls demonstrate reaction to supplied changes, not autonomous migration,
recruitment or politically chosen membership.

Consolidation adds earned wage arrears to the shared protected-stock calculation,
without reserving wages for work not yet delivered. An employer's 40 opening coins
remain protected against a 77-coin earned claim instead of buying discretionary
food, and later reduce that claim to 37 at Close. The household buffer limit now
counts employment's sidecar transfers plus contribution effects together. A limit
that fits payroll alone rejects the larger combined batch without publication.
A town-production regression also caught an overly broad lending guard; it now
applies only when a lending/recovery driver is enabled.

## Interpretation and remaining scope

All five requested pairs are implemented and exercised. The test fixtures supply
contracts, work opportunities, price limits, support mandates, cash targets and
membership decisions. They demonstrate correct composition and adaptation; they
do not establish a calibrated, indefinitely sustainable economy for arbitrary
terms or automatically provide market registration after exit. Prior standalone
income-loop sustainability remains a separate ten-year control.

No personal self-directed policy changes were added. Further work includes
household hiring with purchased-labor cost allocation, physical wage/barter
pooling reservations, member buy-order coordination, autonomous recruitment,
producer-input planning, town recovery and broader institutional formation.

## Verification

Final focused run: **288 tests passed across 23 suites**, including all 12 new
`household_integration` tests and the existing ten-year CPU/reference household
income control. One existing slow annual 32-person test remained ignored. The full
crate suite was not run. The final integration rerun also covers batched execution,
coin conservation and pooling changes at exit/accession. Strict all-target Clippy,
formatting, diff whitespace and repository artifact checks passed.

From `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_integration
cargo +1.92.0 test --locked --test household_integration --test household_income \
  --test households --test household_accounting --test household_credit \
  --test household_dissolution --test household_equipment_disposal \
  --test household_property_package --test household_market --test town_market \
  --test need_orders --test agreement_laws --test laws --test telemetry \
  --test inventory_accounting --test process_accounting --test employment \
  --test acquisition --test lending --test credit --test finance \
  --test service_accounting --test reporting_coverage
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Generated logs stay under ignored `output/household-*.log`; no generated outputs
are committed. This document and the tests record the reproducible evidence.
