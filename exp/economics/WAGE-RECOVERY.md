# Earned wages in authorized recovery

## Admission and stays

Earned, unpaid employment claims now appear in the shared recovery inventory as
`ContractId::Wages(agreement)`, retaining each earning month's due date, creditor,
native denomination and remaining quantity. Future work is not a debt and is not
accelerated. Already-overdue wages can trigger a configured proceeding without
requiring a loan. Authorization, custody and household wind-down requirements
remain unchanged.

At Due, a proceeding opens from previously observed arrears. While it is active,
Acquire stops new employment delivery to that employer. Coin wages in the estate's
denomination leave ordinary Close collection and await estate allocation. Other
native wage denominations retain their ordinary Close servicing; admission does
not invent an exchange rate or cash buyout. Unpaid wages block closure even when
the configuration authorizes loan-deficiency discharge.

The first part of Fibonacci batch 3 covers admission and stays. Its focused run
passed 37 tests across employment, recovery and wage recovery. Controls reject
opening on future/unearned payroll and compare CPU/reference execution and
separate wage receivable/payable statements. Estate cash distribution follows in
the next change; there is no wage forgiveness or automatic death estate here.

## Shared cash distribution

The second part sends due wages in the estate's coin denomination through the same
finite ranked/proportional window as loan and eligible land claims. A configured
`claim_priorities[Wages(id)]` overrides that employment agreement's rank; equal-rank
contracts share proportionally. Within a wage agreement, older earnings receive
its grant first. Collateral liens retain their separate claim on actual sale proceeds.

Funds deposited at Due become distributable at the following Due. Each payment
updates the original employment claim and publishes requested/allocated/paid
receipts with its earning month. The worker's cash and receivable, employer's
restricted cash and payable, and custody books reconcile independently. Household
members share actually received estate wages under their existing pooling terms.
Unpaid wages are not covered by the existing loan-only discharge option.

Validation: 39 tests passed across employment, recovery and wage recovery. New
controls compare wages-first, loans-first and equal-rank sharing against identical
six-coin resources and eight-coin claims; reordered catalogs agree. A four-coin
payment runs from later earned income through custody and full wage payment to
closure, with CPU/reference and checkpoint continuation agreement.
