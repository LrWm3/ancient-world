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

## Funded assignment of loan claims

An authorized estate can post `recovery.receivable_listings` naming an entire
existing direct-loan or authorized-liquidation mortgage asset. Dated `receivable_bids` provide explicit buyer
consent and cash price. Listings specify positive custody coins per native claim
unit. Coin claims retain a one-to-one quote; unsecured commodity claims use an
explicit fixed quote. The required price is current principal plus accrued
interest, multiplied by that quote. A stale or discounted price is rejected; the
borrower's amount owed and denomination never change.

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
valuation. It excludes fixed-value/resale security, nontransferable guarantees, secured native-commodity, partial
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
loan cannot be listed for assignment. Discovery exposes accepted, unexpired
coverage with its original terms, acceptance date and remaining native cap.
Unaccepted posted offers, exhausted caps and expired terms are absent from that
coverage list. Preconfigured future coverage still shows its future start date;
it is not callable early. Remaining cap is not reserved cash or a funding forecast:
original term, delay, current claim and actual guarantor resources still constrain
performance. Assignment is not an underwriting promise.
The guarantor cannot buy its own covered claim through this adapter.

The original claim ID remains unchanged. Calls, common agreement inspection and
settlement observers identify the current creditor. Actual payment reaches that
holder and creates recourse against the original debtor, never the seller.
Person and household seller controls exercise installments, private guarantor
funding, estate distributions, separate statements and CPU/checkpoint parity.

A household estate inspection control distinguishes preconfigured, posted but
unaccepted, and posted accepted guarantees. Partial calls leave only the unused
cap visible; full calls exhaust it and expiry removes it. The loan and any
recourse survive those discovery changes. Inspection does not mutate state, and
CPU/reference statements and reconstructed continuation agree.

The secured household variant now combines assignment with lien subrogation.
A six-coin guarantee of a ten-coin loan and four actual property-sale coins
follow the existing timing and stable lien priority: a previously reserved lien
can transfer to recourse; a later sale allocates against the surviving claims.
Neither route duplicates the four coins. Funded/unfunded assignment and retained/
discharged deficiency controls preserve separate investor, guarantor, household,
member and custodian books. Household dissolution depends on its own remaining
exposure, not whether the unrelated guarantor has recovered in full.

Common productive requests can now include a receivable bid alongside seed
purchase, land prerequisites and planting. The combined regression checks
separate purchase costs, estate custody, later borrower payments to the investor,
harvest and unfunded rejection. CPU/reference and reconstructed checkpoints
reconcile. The claim's face amount never becomes immediate spendable cash.

The secured assignment and guarantee continuation also runs with one shared
custodian for the winding household and its debtor. Separate estate cash and
reserved liens produce the same recoveries as dedicated custodians, including
unfunded claim bids and retained deficiencies. Shared custody neither consolidates
their financial statements nor authorizes either estate to spend the other's
proceeds.

## Commodity claims sold for coins

An unsecured seed loan can be sold for custody coins without converting its debt
into coins. Discovery exposes the native claim, payment resource and unit quote.
Assignment moves only creditor identity and actual purchase cash; neither seeds
nor a second loan are created. Later borrower payments deliver seed to the buyer
and still require storage. A buyer can acquire a claim with no storage, but then
its goods remain uncollected and the original borrower retains the debt.

The first valuation model requires the quote to equal the fixed reporting value
of a native claim unit. Inconsistent reporting configurations fail early. This
keeps assignment at carrying value; negotiated discounts, impairment and amortized
acquisition cost still require an explicit extension. Native secured claims remain
excluded.

`tests/native_receivables.rs` compares person and winding-household sellers, funded
and unfunded buyers, available and absent receiving storage, stale prices and
invalid quotes. A two-seed asset sells for four real coins; later delivery either
repays those same seed units or remains due. Loan interest, estate discharge,
private member funds, separate statements and CPU/checkpoint continuation agree.

The same matrix now includes transferable native guarantees and actual employment.
The borrower pays a worker two seed units for completed labor; that worker later
covers the missed seed installments as guarantor. Delivery goes to the current
claim holder and creates seed recourse against the original borrower. Absent buyer
storage prevents both delivery and premature recourse. Household wind-down,
private balances and estate proceeds retain their separate accounting. The
four-target gate passed 42 tests; strict all-target Clippy passed.

## Mortgage claims held by households

Financed purchases now use the same configured-loan inspection adapter as direct
advances for listings, guarantees, lien subrogation and reporting. Accepted loans
still own their servicing terms and current creditor; the original offer remains
unchanged when a claim is assigned.

`tests/mortgage_receivables.rs` gives a household a real six-coin mortgage asset
and a separate coin liability. During wind-down its authorized estate offers the
mortgage for six coins. A funded investor receives the claim, preserving borrower,
collateral and collection rank. A later four-coin property sale pays that investor
and leaves two due. The household's estate handles its own deficiency and can
close; an unfunded claim bid leaves the asset with the household and blocks exit.
The member purchases the property with private money and maintains the transferred
crop. Separate books, actual custody, CPU/reference and reconstructed continuation
agree. Fixed-value/resale collateral remains outside receivable assignment.

Mortgage assignment also retains explicit transferable guarantees and authorized
lien subrogation. The matrix calls two units of coverage before or after the
four-unit collateral sale, with dedicated or shared custodians. Under the selected
inherited-lien terms, a pre-sale call leaves the original creditor six recovered
and the guarantor two in recourse; a post-sale call transfers two reserved proceeds
to recourse, leaving the creditor four recovered and two still due. These are
explicit subrogation semantics, not a promise of identical recovery across timing.
New recourse cannot collect in its creation month. Original claim holders follow
the same rule when assignment is unfunded, and household exit depends on remaining
assets. Separate books and CPU/checkpoint continuation agree.
