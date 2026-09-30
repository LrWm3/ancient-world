# Voluntary member support for household payments

Implemented integration pass 27, September 2026. A household can accept a member's
authorized surplus to fund its own earned wages or current loan payments, even
without a consumption improvement or a useful market sale. This connects existing
voluntary support, employment, lending and separate accounting paths.

## Authorization and policy

Static `Charter.accept_payment_support` defaults to `false`. Under `NeedsFirst` or
`NeedsThenIncome`, it extends the existing support resolver:

1. Require a current member's live, personally signed surplus mandate. Its dates,
   monthly limit, private reserve, consumption horizon and collective stock target
   still apply. A governor cannot authorize somebody else's donation.
2. Protect the donor's needs, claims and accepted process inputs. Only surplus
   available at the opening reservation boundary can be offered; incoming household
   allocations cannot be donated back at that same boundary.
3. Keep the existing consumption/income comparison. If it accepts, its result is
   unchanged.
4. Otherwise, inspect the household's own earned wages and current collectible loan
   dues in the offered resource. Subtract actual holdings and cap a second candidate
   at that shortage, the feasible offer and available storage.
5. Recheck this smaller candidate against collective and individual need projections.
   Accept only if collective needs do not worsen under their existing ordering and
   no protected donor need worsens. Later donors see the remaining shortage.

This is a fallback funding criterion, not a new priority over existing consumption
or income choices. It changes neither governor selection nor labor allocation.
`support_member_wages` and `support_member_loans` govern the opposite transfer
direction and remain separate choices.

The payment reader reuses `employment::claims` and `credit::current_dues`, combining
only identical denomination units. Future undelivered work, payroll outlooks, future
loan installments, member debts, land dues and forwards do not create this target.
Loan claims under an active estate or stay retain the existing reader's exclusions.
This is not yet a universal obligation adapter.

## Execution and records

Support runs at the existing before-Productive household boundary. The donation
changes ownership once. It neither pays a creditor nor gives the donor a new claim,
ownership stake, guarantee or governance power. Replay verifies consent, quantities
and comparisons before publishing the transfer.

Wages collect at ordinary Close. Loan funding supplied after Due waits for a later
Due; there is no backdated collection or extra payment phase. Storage and later
competing uses can still prevent payment. Support is not creditor-specific escrow
and changes no creditor priority.

`support::Receipt.payment_funding` records native units due, observed shortage and
candidate shortage after funding. `accepted` records the actual transfer; a rejected
candidate may still have a projected improvement. The existing `household_support`
observer exports this optional record. Under `NeedsThenIncome`, the income forecast
is recomputed for the smaller candidate so its receipt does not describe an
unaccepted larger transfer.

Existing double-entry adapters record member transfer expense and household transfer
income. Physical stock moves at carrying cost. Wage/loan liabilities stay with the
household until settlement; a donation is not a second wage expense, loan advance
or repayment. Member and household books remain separate.

## Combined checks

| Opening situation | Result |
| --- | --- |
| Six household wages owed; member has ten coins, reserves two; feature disabled | No donation or wage payment |
| Same opening, feature enabled | Eight offered, six accepted; member keeps four; Close pays six |
| Member has five coins and reserves two | Three accepted and paid; three remain owed |
| Two consenting donors can supply three and eight | Stable member-ID policy accepts three each; total support is capped at six |
| Donor also owes six personal wages out of ten coins | Six protected; household gets four; personal wages clear and two collective wages remain owed |
| Donor has ten grain and needs six; household owes six grain wages; worker has room for two | Four donated, donor fed, two paid, two retained by household and four still owed |
| Household owes six wages and a six-coin loan, has no cash; member has twenty | Twelve donated; Close pays wages; six stay for next Due, which clears the loan |
| No mandate, withdrawn mandate, departed donor, or only unearned employment | No payment-support transfer |

The loan comparison originates a permitted advance, then supplies a pre-audit loss
of its proceeds to establish the distressed opening. It does not model how the loss
occurred. Wage fixtures import explicit earlier earnings. Donors are not recruited
automatically; finite support does not establish business viability.

CPU/reference checks compare states, ledgers and audits, including reordered member
and mandate tables under stable policy. Continuation and full replay agree. Forged
funding receipts reject atomically. An income-policy case checks the capped forecast
and observer output against actual settlement.

## Verification and remaining work

The selected 12-suite regression run passed 117 tests, including the ten-year
household income CPU/reference comparison; one existing slow test remains ignored.
This includes seven new payment-support tests. Formatting and strict all-target
Clippy passed. The full crate suite was not run. Generated logs stay under ignored
`output/economics/payment-support-*.log`.

Consent terms remain supplied. Payment support is voluntary assistance rather than
an enforceable promise of future capital. General claim coverage, wage estates,
guarantees and long-horizon funding decisions remain open. Person self-directed
policy changes remain deferred.
