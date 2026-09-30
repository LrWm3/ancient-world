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
