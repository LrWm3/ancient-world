# Receivables during recovery and household wind-down

An estate with a loan deficiency now checks existing receivables before closing.
The view includes outstanding loan principal/interest, materialized land bills,
accepted prepaid deliveries and actually earned wages owed **to** the debtor.
It excludes hypothetical future rent, payroll and production. Native quantities
remain in their denominations; the view creates no coins or converted claims.

Ordinary contract servicing continues to collect these assets. Receipts go to
their existing legal creditor, then eligible cash enters the existing estate
custody window. New receipts cannot be spent again in the same boundary. A
closure check also examines actual post-transaction debtor cash, so collecting
the last receivable at Due cannot cause discharge before that cash is swept.
`AssetsPending` receipts report both uncollected cash and outstanding receivables;
settlement logs can be filtered by the receivable's counterparty.

The check applies while loan deficiencies remain. A fully paid estate can close
without liquidating unrelated financial property. Existing claims must be
performed or explicitly disposed of before deficient closure. A counterparty's
authorized discharge resolves the corresponding loan asset and records its loss
through the same double-entry adapter. There is no automatic bad-debt write-off
based solely on how long a receivable has remained unpaid.

Closure checkpoint validation rejects unresolved receivables predating a recorded
deficient closure. Recognition respects the phase boundary: wages earned at Close
or agreements admitted at Acquire after that month's Due closure are new assets,
not retroactive reasons to invalidate the earlier closure.

## Verification

Six focused tests cover:

- One coin collected in the closure month: wait for custody and distribution.
- Three coins collected over installments: recover all three before writing off
  the remaining seven of a ten-coin loan.
- An unfunded counterparty: retain the full receivable and defer closure.
- Two authorized estates: counterparty discharge recognizes the asset loss;
  the creditor estate then closes through its next ordinary boundary.
- A winding-down household: retain membership/property until collection and
  recovery finish, then permit dissolution without distributing creditor funds
  to its member.
- Earned wages: collect actual employer payment before deficient closure; later
  work remains a new, material receivable. Counterparty-filtered observer output
  explains the deferred closure.

CPU/reference, checkpoint and separate-account checks accompany the composed
cases. Fixtures explicitly start reporting from a distressed opening snapshot;
their initial cash depletion is not presented as a simulated expenditure. The
affected commodity, wage, mixed-claim, guarantee, household-dissolution and
recovery suites pass alongside these tests. Strict all-target Clippy passes.

## Funded assignment of coin loans

An authorized estate can post `recovery.receivable_listings` naming an entire
existing direct-loan asset. Dated `receivable_bids` provide explicit buyer
consent and cash price. The first adapter requires the current principal plus
accrued interest, in the estate's custody denomination. A stale or discounted
price is rejected; it never changes the borrower's amount owed.

Common offer discovery exposes the current loan terms, security, debtor and claim amount.
`ReceivableLiquidationBid` preparation uses ordinary Acquire settlement. Physical
asset sales run first, receivable bids next (listing then stable bid ID), then
inventory lots and new advances. All share opening cash; a purchased claim or
new custody receipt cannot fund another purchase inside the same window.

Successful settlement changes the existing loan's creditor and records immutable
assignment evidence. The borrower, principal, interest, maturity and collection
rank remain unchanged. Future Due collections pay the new creditor. The estate
receives actual cash in custody, distributable only at a later Due. No secondary
debt book, debt cancellation or fictitious repayment is created.

Reporting moves the receivable and actual cash at equal carrying value; neither
party recognizes sale income at par. Both investor payment and restricted estate
proceeds are investing flows. Household and member statements remain separate.
Configured listings, bids and current creditor must reconcile to assignment
history; changing ownership in an altered batch fails atomic validation.

The composed tests include a person or winding household selling a claim, funded
and unfunded buyers, wrong prices, read-only common preparation, duplicate
applications, future repayments, estate discharge, separate books and CPU/checkpoint
agreement. A receivable and inventory lot also compete for the same buyer money:
two coins fund only the claim; three fund both, with no duplicated purchasing power.

## Remaining boundaries

This is a bounded assignment adapter, not general debt trading or discount
valuation. It excludes fixed-value/resale security, nontransferable guarantees, mortgage, native-commodity, partial
and onward assignments, borrower buybacks, netting and impairment estimation.
Accepted estate authorization supplies assignment authority; autonomous listing,
pricing and consent remain future work. Unresolved assets still retain their
normal collection and closure protections. Inventory sales have their own
[funded liquidation adapter](INVENTORY-LIQUIDATION.md).

Authorized-liquidation security can now accompany the assigned claim. Neither
the lien rank nor the loan ID changes, so the same loan receives its asset's
reserved proceeds after assignment. A two-estate continuation has a winding
household sell its claim against a person whose collateral is independently
liquidated, either before or after claim assignment. Funded investors receive
actual proceeds and retain the agreed deficiency risk; unfunded bids leave the
household's exposure and exit blockers intact. Explicit debtor discharge clears
the remaining receivable in the owning party's statements. The four-target gate
passed 50 tests, including CPU/reference and checkpoint comparisons.

## Explicitly transferable guarantees

A guarantee may consent to `follows_assignment`. Without that term, its covered
loan cannot be listed for assignment. Discovery exposes the attached terms; a
configured future guarantee is still subject to its original admission, term,
cap, delay and available funding. Assignment is not an underwriting promise.
The guarantor cannot buy its own covered claim through this adapter.

The original claim ID remains unchanged. Calls, common agreement inspection and
settlement observers identify the current creditor. Actual payment reaches that
holder and creates recourse against the original debtor, never the seller.
Person and household seller controls exercise installments, private guarantor
funding, estate distributions, separate statements and CPU/checkpoint parity.

The secured household variant now combines assignment with lien subrogation.
A six-coin guarantee of a ten-coin loan and four actual property-sale coins
follow the existing timing and stable lien priority: a previously reserved lien
can transfer to recourse; a later sale allocates against the surviving claims.
Neither route duplicates the four coins. Funded/unfunded assignment and retained/
discharged deficiency controls preserve separate investor, guarantor, household,
member and custodian books. Household dissolution depends on its own remaining
exposure, not whether the unrelated guarantor has recovered in full.
