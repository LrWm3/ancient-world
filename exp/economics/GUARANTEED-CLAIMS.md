# Guarantees across authoritative claims

Implemented in Fibonacci batch 8. Accepted guarantee terms identify an original
loan, a direct prepaid delivery, one earned wage `(agreement, earning month)`, or one annual land bill
`(agreement, original due date)`. No second wage or rent debt ledger is introduced.
The guarantee retains its own guarantor, lifetime paid cap, validity interval,
missed-payment delay, priority and reserved recourse identity. Terms are supplied
consent, not autonomous discovery, underwriting or a new legal formation flow.

## Execution and allocation

At the existing Due boundary, ordinary collections and collateral enforcement run
first. Calls read the resulting authoritative claim. An unearned wage, unissued
bill, fully settled claim or unaccepted land offer creates no payable call.
Existing relief changes the claim's remaining quantity and effective date; an
extension does not extend the guarantee's validity. Delay for a dated non-loan
claim runs from its effective due date. Same-boundary guarantees precede relief,
so consent referring to an amount already paid is rejected as stale.

`recovery.guarantee_policy` is independent of ordinary creditor collection policy.
`Stable` remains the default: lower priority then stable guarantee ID. Opt-in
`Proportional` inventories calls and applies the shared finite-budget allocator to
equal-priority requests. Guarantee identity distinguishes multiple calls covering
the same underlying contract. Calls recheck residual coverage before payment;
unused grants from overlapping coverage are redistributed against remaining
opening funds. Receipts distinguish requested, allocated and paid quantities.
Receiving a payment or household contribution cannot fund another outgoing call in
that same reservation window. Existing resource protection and storage limits apply.

Actual payment reduces the original claim and creates equal, normally unsecured,
zero-interest recourse in the existing loan book. It creates no debtor cash and no
new wage/rent income. Land payments retain actual native receipt quantities; later
annual bills and land rights are unchanged. Loan payments retain interest-first
application. Earned wage receipts enter the worker's current household pooling
once, including when the household itself is the guarantor. Membership does not
eliminate recourse from separate financial statements.

## Dated recourse and recovery

`credit.recovery.guarantee_advances` records actual amounts by guarantee and month.
Its totals reconcile to paid caps and original recourse principal. Covered wage
and land settlement must explain those payments. Removing dated evidence or
reversing covered settlement invalidates a checkpoint.

Each addition becomes collectible at a subsequent Due, even when it increases an
older recourse loan. Estate distribution and closure exclude fresh additions;
inspection and telemetry distinguish current collectible principal from deferred
recourse. This fixes a reproduced timing bug in which a later guarantee advance
could immediately receive estate cash because its loan's original opening date
was in the past. A continued test now pays old recourse in month 4, keeps new
recourse and estate cash outstanding, then settles and closes in month 5.

## Combined verification

The mixed scenario contains a person employer, a household member worker, a state
landlord, a lender and a household guarantor. A real loan funds the first wage.
Pooling raises household funds from four to six coins. In month 13 a loan, a new
wage and annual rent each request four coins. Proportional guarantees pay two to
each; two remains on each original claim and three two-coin recourse assets remain
with the household. One coin of wage pooling arrives after the reservation window
and is not respent. Separate books balance without consolidating away liabilities.

Controls compare stable 4/2 and proportional 3/3 division of identical eight-coin
requests against six coins, priority overrides, overlapping coverage, unavailable
work, employer payment, expiry, extensions and stale relief. CPU/reference,
reordered catalogs, checkpoints, actual balances, statements, filtered observer
records and atomic rejection of forged settlement/reporting are checked.

## Remaining boundaries

[Commodity finance](COMMODITY-FINANCE.md) now covers physical wages, native land
and direct prepaid deliveries, with explicit valuation and inventory accounting.
A forward first becomes callable after its original Acquire delivery window;
extensions move eligibility without extending coverage. Physical wage payments
reserve household pooling space before committing. Guarantee observers identify
the native resource alongside requested, allocated and paid quantities.

Accepted land coin terms and explicitly agreed loan coin payments now have
tender adapters. Broader tender routes, conversion damages and dynamically
underwritten tool-forward coverage remain extensions. Fixed resource valuations
do not establish general FX or noncash collateral/estate accounting.

Guarantees of pending-resale mortgages remain rejected. Secured recourse now has
the explicit inherited-lien chain adapter described below. General security
subrogation, guarantee markets, pricing, premiums, legal formation requirements,
autonomous household guarantee selection and cyclic contingent-credit networks
remain extensions. Dedicated custody agents cannot guarantee obligations.

## Earlier batch-8 checks

The final affected regression selection passed **153 tests across 14 suites**,
with one existing ignored test, including all 11 new guaranteed-claim tests. Formatting, strict all-target Clippy,
whitespace and repository artifact checks passed. Raw logs remain ignored under
`output/economics/guarantee-*.log`. These checks demonstrate the specified bounded
compositions; they do not establish autonomous financial viability or the proposed
long-term stress-test institutions.

Authorized-liquidation loans may explicitly select inherited security; see
[lien subrogation](LIEN-PRIORITY.md#explicit-guarantee-subrogation). The common
offer exposes this term. It transfers the lien or reserved proceeds without
changing the next-month collectibility of actual guarantee payments.

Whole-loan estate assignment can explicitly carry the benefit of a guarantee
with `follows_assignment`. This does not extend its cap, term or acceptance.
Inspection, payments and observers use the live loan creditor; the unchanged
claim identity continues to control calls and recourse. Non-loan guarantees
cannot select this term. See [receivable assignment](ESTATE-RECEIVABLES.md).

## Accepted coin tender for land guarantees

`GuaranteeTender::Native` preserves existing performance. An explicitly selected
`AcceptedLandCoins` route pays the original land agreement's accepted coin rate,
without a native fallback or a new exchange rate. Missing, invalid or non-land
routes reject during validation. The selected route is visible in common terms.

The shared executor allocates actual tender quantities in whole conversion lots,
under the guarantee's existing stable or proportional policy. Native caps,
requested/allocated/paid claim quantities and recourse remain in land-dues units.
Receipts and observers also expose the actual tender resource and quantity.
Cash payment cannot count as harvested goods or support native-linked issuance.

Separate reporting uses the existing dues and native-recourse valuations. The
debtor substitutes equal native recourse for its dues liability; the guarantor
bears any difference between its cash outlay and recorded receivable value. This
is an explicit reporting convention, not evidence that the recourse can be sold
for that value. The creditor records its ordinary agreed-tender settlement.

Controls compare native-only and coin performance, rates above/below reporting
value, two competing conversion lots, an unaffordable lot, reversed catalogs,
CPU/checkpoint continuation and atomic rejection of forged tender receipts.
Other guarantee kinds retain native performance; general FX is still absent.

The household mixed-claim scenario exercises the alternative alongside native
coin loan and wage calls. Six collective coins pay two to each recipient, while
the land claim falls by one native unit at its two-coin rate. Member wage pooling
adds one collective coin only after settlement; it cannot fund another call.
Recourse records two coin claims and one native claim, with separate valuation
and statements. Reversed catalogs, CPU/checkpoint replay and observer quantities
agree.

## Agreed alternative payment for loan and wage guarantees

`GuaranteeTender::AgreedCoins` records the explicitly consented payment
resource and whole payment units per native claim unit. It applies to loan, earned wage and prepaid-delivery claims with unsecured recourse. The payment resource must be distinct, a stock
resource and storage-free; the rate must be positive. This does not change the
underlying contract's ordinary performance terms or infer creditor consent from a market quote.

The lifetime guarantee cap, original debt reduction and resulting recourse all
remain in native units. Funding and proportional allocation use payment units
and whole conversion lots. Less than one lot pays nothing. Overlapping guarantees
recheck the outstanding claim and cannot discharge it twice. Actual receipts
record the tender separately; paying coins for a grain debt never creates grain.

Reporting uses actual tender quantity for cash flow and explicit fixed reporting
values for the discharged claim and recourse. A difference is settlement gain or
loss for the creditor and guarantor. If the tender is not the reporting currency,
its stock cost basis moves through the common inventory adapter; it is not
misclassified as reporting cash. This is fixed contract conversion, not FX discovery.

The five-target gate passed 44 tests and strict all-target Clippy passed. New
controls cover underfunding, rates below/equal/above reporting value, a second
currency with distinct historical cost, overlapping coverage, stable/proportional
allocation, reordered terms, CPU/checkpoint equality and forged tender rejection.

The alternative loan tender also composes with household membership and solvent
wind-down. A household can pay two of its own coins for one native grain unit
owed by its member, leaving a separately reported grain receivable/payable between
them. The member's private ten coins cannot fund the household call. An unpaid
recourse asset prevents dissolution; an underfunded, expired guarantee permits
household residual distribution without erasing the member's original private
loan. CPU/checkpoint and audited statements agree. The four-target household
integration gate passed 55 tests with one ignored; strict Clippy passed.

Earned native wage claims now use this same agreed coin route. Actual coins can
settle only whole wage units; the worker's wage receivable and employer's wage
payable fall in native units and the guarantor receives native loan recourse.
No physical wage commodity is delivered or invented. Household contribution
classification uses actual tender, rather than the nominal wage commodity.

The wage control compares underfunded lots and rates below/above claim reporting
value, including a payment currency distinct from the reporting currency with
historical inventory cost. CPU/reference, reconstructed statements and forged
payment receipts agree. Unpaid wage units remain owed. This does not add a secured wage-recourse route;
accepted land conversion retains its separate existing terms.

A household may also guarantee a member's native wage using agreed coins. Actual
coin receipts pool once; those returned contributions cannot finance another call
at the same Due boundary, and private member coins do not fund the household's
guarantee. Native recourse remains a separate household asset that blocks solvent
closure until recovered or disposed of. With an unfunded expired guarantee,
residual household cash can be distributed without deleting the member's unpaid
wage claim. CPU/reference and checkpoint statements agree.

## Substitute tender for prepaid deliveries

A prepaid-delivery guarantee can now use the same explicitly agreed coin route.
`Contract.delivered` counts only actual goods; `substituted` counts native units
discharged by substitute tender. Outstanding claims and historical prepayment
release include both, while physical stock, storage and delivered goods include
only actual transfer. Dated guarantee advances reconcile substitute performance
at checkpoints. This is accepted guarantee performance, not unilateral conversion
of the original forward or a new spot sale.

The buyer recognizes actual payment against released historical prepaid cost.
The guarantor compares payment value with native recourse value; the seller
recognizes the released prepayment and native recourse cost. A second payment
currency retains its inventory cost and is not reporting cash. Mixed coin/native
guarantees consume prepayment basis in receipt order, preserving integer rounding
even when a small partial performance releases zero cost.

Relief at the same Due boundary now reads already accepted guarantee performance.
Consent based on the old outstanding quantity is rejected; valid residual relief
records both physical and substitute performance and never overwrites a payment.
Unrecovered native recourse continues to block estate closure. The six-target
regression gate passed 102 tests; strict all-target Clippy passed. CPU/reference,
checkpoint accounting and forged delivery/tender controls are included.

A member-guaranteed household delivery now composes with explicit wind-down and
recovery. The member's coins discharge two native delivery units; accepted relief
forgives only the remaining two. The resulting grain recourse remains a material
member receivable and household payable, blocking closure despite household coins.
With an unfunded expired guarantee and accepted full delivery relief, the estate
closes and residual cash passes to the member. Neither case creates grain or pools
the external buyer's receipt. CPU/reference and checkpoint statements agree; the
four-target gate passed 65 tests and strict all-target Clippy passed.

## Bounded guarantees of unsecured recourse

A configured guarantee may cover another guarantee's unsecured recourse loan.
Terms resolve iteratively to an original loan, wage, land or prepaid obligation,
retaining the original debtor and denomination and the immediate recourse creditor.
Cycles and missing roots fail validation; this does not recursively execute calls.
Secured recourse now also supports the explicit inherited-lien adapter below.

Each call still needs accepted coverage, actual arrears, cap and opening funds.
Every addition to recourse is dated, and downstream calls exclude units created
that month even when an older balance on that loan is already overdue. Incoming
guarantee payments cannot fund another outgoing call in the same window.
Unaccepted posted coverage remains inactive.

The composed control uses earned wages, two guarantees and later direct lending.
It leaves newly added recourse for the next month despite funded downstream
coverage, then settles it once. CPU/reference, reversed guarantee catalogs and
checkpoint continuation agree; cyclic/unidentified roots are rejected. The
five-target gate passed 93 tests and strict all-target Clippy passed.

Household/member chains also compose with pooling and solvent wind-down. Half of
actual wage receipts pool using the existing carried rounding; repayment of the
household's guarantee recourse does not create another income contribution. The
household can dissolve after its own contingent duties and claims clear, while a
member's separate recourse against the original employer remains owed. The
four-target gate passed 65 tests with CPU/reference and checkpoint equality.

The household chain also supports distinct accepted coin rates at each link.
A native wage paid at two coins per unit can create recourse covered at one coin
per unit. Neither payment rate converts the native principal or its reporting
valuation. Eight actual wage coins contribute four pooled coins; downstream
recourse collections contribute none. Private recourse survives solvent household
exit. CPU/reference and checkpoint controls pass alongside the original coin case.

## Bounded chains of inherited liquidation liens

A guarantee can now cover secured recourse when every link explicitly inherits
the original authorized-liquidation lien. Terms resolve iteratively to one rooted
collateral definition; a cycle, broken inheritance or forged collateral rank is
rejected. Payment transfers the live pledge or remaining reserved proceeds, never
duplicating them. Existing same-denomination and native-tender restrictions apply.

Recourse created during an authorized stay becomes callable at its first maturity.
A collection stay does not cancel a separate accepted guarantee, but new advances
cannot cascade or be collected in their creation month. Stable/proportional
controls cover both a live pledge and already realized proceeds through two links,
with actual creditor receipts, retained deficiencies and separate statements.
CPU/reference, reversed catalogs and checkpoint continuation agree. The five-target
gate passed 102 tests and strict all-target Clippy passed.
