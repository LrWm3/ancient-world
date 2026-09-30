# Explicit accepted claim relief

`recovery.claim_relief` supplies dated terms accepted by the debtor and creditor
inside an authorized proceeding. Earned wages and annual land bills share dated
extensions and quantity write-offs through this mechanism. Relief is never
inferred from insolvency or inability to deliver.

Terms identify the contract, original due date, parties, proceeding, application
month, expected current date and exact remaining claim. A stale agreement is
rejected. At Due, after ordinary collection and before estate distribution, accepted
write-offs reduce entitlement and append history recording actual payments at that
point. They leave delivered work, earned income and actual cash settlement unchanged.
The reporting adapter recognizes creditor loss and debtor relief separately.

The original earned wage is reconstructible from delivered hours and the wage rate.
Validated relief history explains any reduction in the claim; checkpoint changes
without that history are rejected. An authorized loan-deficiency discharge still
does not automatically forgive wages. Configured consent is not autonomous
negotiation, legal adjudication or a general debt cancellation policy.

First change in Fibonacci batch 5: 29 tests passed across wage recovery, employment
and household payment support. A partially paid four-coin wage retains one actual
coin paid and two hours delivered after three coins are explicitly forgiven in two
steps. CPU/reference and checkpoint continuations agree, and the estate closes only
after the remaining claim is resolved. Stale relief and unexplained balance changes
are rejected. The preceding batch's broader run passed 125 tests across 11 suites.


## Land-bill adapter

The same dated terms now support annual land bills. Original charges, actual
payments and native paid quantities remain intact; separate relief history supplies
the waived quantity. Collection, projections, household support, dissolution and
recovery use the resulting outstanding claim. Forgiving a bill does not renew or
transfer a land right, pay a tax, issue currency or cancel later annual bills.

The dues adapter values the loss at its explicit native-unit reporting valuation,
including when coins are an accepted tender. Partial waivers retain collectible
claims; stale consent after ordinary native payment is rejected. Six estate-dues
tests pass, including CPU/reference, continuation, partial relief and forged history.

## Wage date extensions

An accepted `Extend` action moves the effective due date of one earned wage while
retaining its original identity and full unpaid amount. Collection eligibility and
recovery admission read that effective date. An extension does not create escrow,
new wages, forgiveness or a receivable valuation change. Active proceedings still
block new employer work and cannot close around the deferred claim.

The third change passed 25 tests across employment, wage recovery and estate-dues
accounting. A funded extension from month 2 to month 5 leaves money with the debtor
until month 5, then observes the existing custody delay before paying at month 6.
CPU/reference and resumed states and books agree. Forged closure around the future
claim is rejected. The land-write-off affected run also passed 59 tests in six suites.


## Land date extensions

Land bills now use the same `Extend` action. Their original annual date remains the
stable bill identity; collection, estate cash requests, need projections and
household payment support use the effective due date. Extending one bill does not
renew access, change the annual calendar or remove future bills. Closing a case
around an unresolved deferred bill is invalid.

The fourth change's integrated control extends the month-13 bill to month 16,
reserves no early land payment at month 15, then uses actual asset-sale proceeds
for accepted coin tender at month 16. Native receipts and issuance remain zero.
The next annual bill still arrives at month 25. CPU/reference and checkpoint books
agree; wage extension, land funding and member payment-support regressions pass.


## Composed recovery

The fifth change combines one household member's employment, household wage
assistance, annual rent and direct prepaid delivery in one authorized proceeding.
One household coin and two advance coins pay three of six earned wage coins.
No grain is produced: rent and delivery genuinely fail. The control preserves all
three claims indefinitely; the consented case writes off wage arrears, extends then
waives the land bill, and extends then waives the forward. Only the last resolution
allows closure, at month 18. Seven reporting coins of creditor losses match debtor
relief; three actual wage coins and zero grain payments/deliveries remain unchanged.
CPU/reference, reordered terms, replay and checkpoint continuation agree.

Final focused verification: all 11 wage-recovery tests and the mixed recovery test
pass. A physical wage write-off is valued from its native units without transferring
coins or inventory. The affected land-extension run passed 34 tests across four
suites. Formatting, strict all-target Clippy and artifact checks pass; the broader
crate regression completed with 801 passing tests and one existing ignored test.
That snapshot predates the separately passing physical-wage test.

## Full unsecured loan write-off

`ContractId::Loan` now uses the same dated consent interface for full write-off of
an unsecured loan inside an active authorized proceeding. Terms name the current
creditor, debtor, original first-collection date (`opened + 1`) and exact current
debt. They are checked after ordinary collection and guarantee payments; stale
quantities are rejected. Native loans and guarantee recourse can use this route
without conversion into the estate's custody currency.

The authoritative loan becomes discharged, with original principal retained.
`loan_writeoffs` holds accepted terms and the disposed principal/interest as
provenance, not another balance. A native discharge without accepted disposition
is invalid at checkpoint. Reporting values the creditor loss and debtor relief
in the loan's denomination; no goods, coins, repayment or interest income are
invented. An unresolved native claim continues to block closure even when the
estate permits ordinary coin deficiencies to be discharged.

This adapter deliberately supports full unsecured write-offs. Partial reductions,
rescheduling of amortizing loans, secured releases and autonomous negotiation
remain open. The control compares absent, exact and stale consent against the
same actual advance and repayment, including CPU/reference, checkpoint and
forged receipt/history rejection. The six-target gate passed 104 tests and strict
all-target Clippy passed.
