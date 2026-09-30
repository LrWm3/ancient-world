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
interest, multiplied by that quote by default. An opt-in whole-claim price floor
now permits other prices for coin or unsecured commodity claims without unpaid interest at purchase (below). The
borrower's amount owed and denomination never change.

Common offer discovery exposes the current loan terms, security, debtor and claim amount.
`ReceivableLiquidationBid` preparation uses ordinary Acquire settlement. Physical
asset sales run first, receivable bids next (listing, highest funded price, then stable bid ID), then
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

This is a bounded assignment adapter, not general debt trading or autonomous
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
keeps native assignment at carrying value. The coin-claim acquisition-cost adapter
below permits discounts and premiums; native discounts, interest-bearing purchase
cost and estimated impairment remain open. Native secured claims remain excluded.

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

## Assignment after accepted partial relief

The existing whole-loan sale can transfer the remaining balance after a partial
write-off. The old creditor keeps its recognized loss; the buyer acquires only
the surviving receivable. Due relief precedes Acquire assignment, so same-month
loss provenance still belongs to the seller. Fresh relief after assignment needs
the buyer's consent. A two-estate regression reproduced rejection of this valid
sale, then verifies stale-former-creditor refusal, immutable history, separate
statements and CPU/reference/checkpoint agreement. No discount or onward-sale
adapter is introduced by this change.

The same sequence now includes household wind-down. Its old loss remains a
household result; sale proceeds pay its estate creditors without becoming private
member income. After its own liabilities and receivable ownership resolve, the
household can dissolve while the buyer's separate claim survives. Former-holder
consent cannot cancel that claim, even after household exit. CPU/reference and
checkpoint financial statements agree; 49 tests passed across the four affected
suites, followed by strict all-target Clippy.

## Agreed price floors and acquisition cost

`recovery.receivable_price_floors` optionally supplies a minimum whole-claim price
by listing ID. It admits claims with no unpaid interest at purchase, either in
the custody coin or unsecured commodities with an explicit fixed reporting quote. Sale payment uses custody
coins; borrowers still owe native units and collection requires real storage.
Otherwise the existing exact unit quote remains required.
Highest funded bids clear first; an unaffordable bid leaves the claim available
to a lower eligible bid. A below-floor bid cannot buy it. Prices and consent are
still supplied, with no autonomous valuation or negotiation implied.

The loan book keeps the full contractual principal. Reporting separately records
`LoanBasisAdjustment`: reported face principal plus this adjustment equals the buyer's
remaining acquisition cost. Sellers recognize their actual disposal gain/loss;
buyers recognize no immediate profit merely from purchasing below face value.
Remaining cost follows the proportion of original acquired principal still owed,
rounded down to integer reporting ticks. Collection realizes the corresponding
gain/loss; accepted relief charges remaining cost, releasing the matching
adjustment rather than expensing the full face amount again. Mixed dispositions
at one boundary split adjustment release proportionally between loss and actual
collection. Full disposal always releases all remaining basis.

Checks cover discounts, par and premiums followed by collection or full write-off;
competing funded/unfunded bids, price floors, reversed inputs, forged prices, and
invalid carrying values. CPU/reference and checkpoint continuation agree. The
six-target gate passed 130 tests and strict all-target Clippy passed. Partial and
onward assignments, purchase-cost allocation to already unpaid interest and market valuation remain
outstanding.

A winding household also uses priced assignment with separate disposal results.
Custody and creditor payment boundaries still delay exit; selling the claim does
not make newly swept cash immediately distributable. After clearance, the
household dissolves while the investor still owns a remaining installment. Later
collection and basis release continue independently. The member inherits neither
the household's sale loss nor the buyer's claim. Discount/par/premium and borrower
relief controls agree on CPU/reference and checkpoints. The four-target gate
passed 58 tests and strict all-target Clippy passed.

Priced claims also compose with transferable guarantees. Discounted and premium
buyers receive the native amount guaranteed, releasing their own acquisition
cost into settlement gain/loss. The guarantor's new recourse equals actual
performance; it never inherits the investor's discount or premium. Person and
household sellers retain their own disposal result, separately from both claims.

When one boundary both collects and forgives principal, the change in acquisition
basis is divided in proportion to forgiven versus total disposed units. Integer
rounding is explicit: remaining cost rounds down, the waiver's adjustment truncates
toward zero and collection gets the remainder. A mixed guarantee/partial-waiver
control checks discounted, par and premium claims, remaining carrying value,
later full relief, native recourse and separate household/member statements.

Priced commodity controls exercise person and household sellers, discounts and
premiums, unfunded bids, and buyers with no delivery space. Uncollected goods
remain a claim at purchase cost; actual goods delivery realizes the purchase
difference. The unit quote still must equal the fixed reporting value, so price
negotiation does not silently revalue every holding of that commodity.

Priced native claims also retain transferable guarantees after a household seller
winds down. The guarantor delivers real goods only when the buyer has space and
receives native recourse valued under the original reporting quote, without the
buyer's purchase adjustment. Household exit neither releases the buyer's storage
constraint nor transfers disposal results to its member.

Partial native relief releases only the corresponding purchased cost. A further
control starts with blocked receiving storage, opens the debtor estate after an
actual missed installment, and forgives one unit. The remaining unit can later
be delivered when storage becomes available, or waived by separately accepted
terms. Those paths preserve actual goods, recognize different loss/return amounts,
and never substitute an invented coin payment. Both person and household sellers
use the same execution and reporting adapters.

Interest-bearing loans with no unpaid interest at assignment now use the same
principal cost adjustment. Later interest accrues in the ordinary interest
receivable/income accounts, independently of cost released on principal. Full
later relief expenses remaining principal cost plus accrued unpaid interest.
This is proportional principal-cost release, not an effective-interest-yield
model. Purchasing an already accrued interest balance remains excluded until
there is an explicit rule allocating purchase cost between the acquired claims.

An interest-bearing claim sold by a household now has an exit control too. The
household retains pre-sale interest and disposal results, finishes after its own
estate distributions, and closes while the buyer still owns future principal.
Later collection or debtor relief affects only the buyer's claim and accounts;
it does not reopen the seller or pass its income/losses onto a member.

Native interest-bearing controls now exercise the same acquisition rule with
actual goods-denominated interest and principal. Discount/par/premium cases either
collect both in goods or lose principal plus later unpaid interest in an admitted
coin-custody estate. No native debt converts to coins; fixed reporting value,
purchase basis and interest remain separate for person and household sellers.

Discovery now carries explicit whole-claim `Pricing::Exact` or `Pricing::Minimum`
terms in both native receivable inspection and the common offer interface. The
unit reporting quote remains separate. A priced listing with unpaid interest is
not currently eligible and is omitted until it satisfies the acquisition rule;
its debt and listing are not deleted. Both discovery and actual sale use the same
price-rule calculation. Discovery still does not promise funding or future terms.

Priced receivable bids now have a combined acquisition control with estate seed,
land prerequisites and dated cultivation. Discounted/premium purchases spend
actual coin prices from the same opening budget as seed. Having enough for either
purchase but not both rejects the requested package without partial publication.
Successful packages later harvest and collect the unchanged face claim, releasing
purchase cost independently from production and custody proceeds.

A household-member control combines common financial acceptance of priced claims
and seed with subsequent cultivation under existing land rights. Actual harvests
pool into the household; claims, principal collection and purchase gains/losses
remain private. It uses supported fixed individual priorities and scheduled work.
This does not establish household acceptance of a combined prerequisite/process
bundle: that still needs the household allocation/collection envelope, and the
consequence-aware individual search remains excluded with households.

Priced mortgage assignments now compose with active cultivation and authorized
property liquidation. A household sells its six-coin claim at a discount or
premium, while the member's later four-coin property purchase transfers the plot
and crop responsibility. Guarantees can act before sale or against reserved
proceeds afterward, including shared custody. The investor's unpaid deficiency
keeps proportional purchased cost; the price neither changes the debt nor creates
additional collateral recovery. Unfunded assignments retain the original holder.

The same priced mortgage control can now finish with authorized deficiency
discharge. After actual collateral and guarantee recovery, the investor expenses
only remaining purchase cost. A guarantor's residual recourse is discharged at its
own native principal, independently of the purchase price. Neither loss removes
previous cash receipts or reverses plot/crop transfer. Unfunded assignments also
retain their original household exposure until performance or discharge permits
its own wind-down to finish.
