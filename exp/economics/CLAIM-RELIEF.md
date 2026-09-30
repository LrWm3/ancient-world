# Explicit relief for dated non-loan claims

`recovery.claim_relief` supplies dated terms accepted by the debtor and creditor
inside an authorized proceeding. It initially covers earned wages; land bills and
date extensions follow as adapters to this same mechanism. Relief is never
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
