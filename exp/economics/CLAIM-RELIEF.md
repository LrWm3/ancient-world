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

## Unsecured loan write-offs

`ContractId::Loan` now uses the same dated consent interface for partial or full write-off of
an unsecured loan inside an active authorized proceeding. Terms name the current
creditor, debtor, original first-collection date (`opened + 1`) and exact current
debt. They are checked after ordinary collection and guarantee payments; stale
quantities are rejected. Native loans and guarantee recourse can use this route
without conversion into the estate's custody currency.

A fully forgiven loan becomes discharged, with original principal retained.
Partial relief leaves the remaining debt and collection stay in place. Relief
reduces accrued interest first, then principal, matching collection order without
representing repayment. The existing proceeding freezes further interest accrual;
partial relief does not change that timing or the original installment schedule.
`loan_writeoffs` holds accepted terms and the disposed principal/interest as
provenance, not another balance. A native discharge without accepted disposition
is invalid at checkpoint. Reporting values the creditor loss and debtor relief
in the loan's denomination; no goods, coins, repayment or interest income are
invented. An unresolved native claim continues to block closure even when the
estate permits ordinary coin deficiencies to be discharged.

Rescheduling of amortizing loans, full secured releases and autonomous negotiation
remain open. A bounded partial secured adapter follows below. The control compares absent, exact and stale consent against the
same actual advance and repayment, including CPU/reference, checkpoint and
forged receipt/history rejection. The six-target gate passed 104 tests and strict
all-target Clippy passed.

The household continuation separately accepts forgiveness of a member's native
guarantee recourse. It resolves the loan after the external prepaid delivery has
been settled/forgiven, records the member's loss, closes the proceeding and
releases residual coins before dissolution. Without this acceptance, the same
internal claim remains material and blocks exit. CPU/reference and checkpoint
statements agree; the five-target integration gate passed 74 tests.

A full write-off does not cancel unused guarantee coverage. Later calls can add
new, dated recourse to the same loan identity. Its ordered disposition history
preserves prior losses and requires new principal to be backed by advances after
the latest write-off. A subsequent full write-off needs fresh exact consent.
The regression reproduced a rejected valid later call before this change; it now
retains both accepted losses, rejects unsupported principal and reordered history,
and agrees across CPU/reference and checkpoint continuation. The five-target gate
passed 100 tests, followed by the final focused history controls and strict Clippy.

The renewed-recourse control also runs with a household guarantor and later
member lending. Two accepted native write-offs retain the household's losses
while its separate coin debt to the member remains. Actual wage pooling finances
only later calls, including fractional carry; it never creates native inventory.
CPU/reference and checkpoint statements agree. The four-target gate passed
58 tests and strict all-target Clippy passed.

Partial-loan controls retain the post-disposition principal, interest and fractional
interest carry as validation provenance. Later relief needs fresh consent to the
exact current balance. Actual collections may lower it; only dated guarantee
advances may add principal, and the insolvency stay cannot invent new interest.
One test forgives a native loan in three steps. Another first spends borrowed
stock delivering a prepaid order, then separately forgives unpaid interest and
principal. Physical delivery, earlier repayments, earned interest and historical
losses remain distinct; closure waits for the final resolution. CPU/reference,
checkpoint and tampered history/batch controls pass. The five-target gate passed
90 tests, followed by the interest/delivery control and strict all-target Clippy.

Household recovery uses the same partial adapter. Forgiving one of two native
units owed to a member leaves that member's remaining receivable material and
keeps dissolution blocked. Fresh consent for the final unit permits closure;
residual household cash is distributed at the later lifecycle boundary. The
composed prepaid delivery, substitute guarantee, partial relief and wind-down
control agrees across CPU/reference and checkpoints on either side of relief.
The four-target gate passed 77 tests and strict all-target Clippy passed.

A loan sold after partial relief retains the seller's accepted loss history.
Disposition validation uses the creditor at the original Due boundary: a sale at
Acquire later that month cannot rewrite that consent. Subsequent relief must name
the new creditor; the former holder cannot forgive the buyer's remaining claim.
The composed two-estate control rejects rewritten historical consent and preserves
separate losses, actual purchase proceeds and CPU/checkpoint continuation. Its
five-target regression gate passed 101 tests and strict all-target Clippy passed.

## Partial secured relief and liquidation

A loan using authorized liquidation can now accept partial relief while its
collateral is still pledged and unsold. Consent must match current debt and leave
a positive balance. The original asset and lien remain attached; relief transfers
neither title nor goods, and causes no payment. The subsequent actual sale ranks
the reduced claim with competing liens against its realized proceeds.

The history records the retained collateral so loss provenance remains valid after
sale and collection. Partial relief after sale now reruns the accepted lien
waterfall over only the amount released from unspent reservations for that same
asset, preserving other existing reservations. Junior liens
retain priority over unsecured collection; other assets' proceeds and completed
payments are untouched. Full secured forgiveness remains unsupported.
The control compares relief before sale, before distribution and after distribution,
as well as the unsupported full-forgiveness scope. A seven-coin senior loss leaves a three-coin lien; an eight-coin sale
then pays three senior and five junior coins at the later Due boundary. Forged
collateral history is rejected, and CPU/reference/checkpoint statements agree.

The five-target gate passed 103 tests and strict all-target Clippy passed.

The same scenario now includes a household borrower and junior member lender.
The member receives five actual coins after senior relief and retains a five-coin
claim. Repayment does not pool as income, and the internal payable still blocks
household dissolution after estate closure. The four-target gate passed 75 tests,
with CPU/reference, checkpoint continuation and strict all-target Clippy.

Post-sale relief uses the existing allocation policy and beneficial custody limits.
An unsecured creditor with higher ordinary collection priority receives none of the
released proceeds while a junior lien still claims them. A later reduction of the
remaining deficiency does not recover money already paid. Person and household
controls agree across CPU/reference and checkpoints. The five-target gate passed
117 tests and strict all-target Clippy passed.

Guarantee subrogation remains binding through relief. A regression reproduced a
six-coin transferred reservation falling to five when the entire waterfall was
rerun. The allocator now keeps each existing reservation up to its remaining claim
and allocates only released amounts against unsatisfied liens. Stable/proportional
controls retain all six guarantor coins and prevent same-month recourse collection.
The three-target gate passed 89 tests, with CPU/reference, checkpoint continuation
and strict all-target Clippy.
