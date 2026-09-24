# Contract recovery using the existing credit book

This implements a bounded first recovery lifecycle in the existing `Simulation`.
It adds no separate scenario engine, scheduler, cash ledger, or duplicate debt
balances. General advances, mortgage servicing, guarantees and subrogated loans
use `State.credit.loans`; actual payments use `finance::Execution` and the normal
validated CPU/reference batch commit.

## Acceptance and scope

`World.recovery` contains explicit accepted guarantees, authorized proceeding
terms, and dated consenting asset buyers. These are configuration-driven consent
and authorization, not autonomous underwriting, court adjudication, negotiation,
or discovery. When a state transaction policy exists, the proceeding authority
must match its authority. Asset buyers must have `Action::AssetTrade` permission.

A proceeding admits one debtor's loan portfolio in one storage-free cash
denomination, materialized land dues, accepted forward deliveries, cash, and an
explicit inventory of titled physical assets. All still-pledged collateral must
be included. [Land/forward admission](LAND-FORWARD-ADMISSION.md) preserves native
performance and maturity: eligible land cash payments share the estate pool;
native goods still collect at their existing boundaries. Unfulfilled non-loan
claims prevent closure unless explicitly released under
[accepted forward relief](DELIVERY-RELIEF.md); no implicit conversion occurs. Mortgages using
the older scenario configuration remain excluded. This is bounded contractual
recovery, not yet universal insolvency.

Each proceeding has a dedicated non-operating estate agent for custody. It cannot
borrow, guarantee, work, or trade through unrelated configured markets. Its cash
balance must reconcile with the proceeding's recorded undistributed funds.
Balance-sheet views exclude held cash from the custodian's equity and attribute
it to the debtor's estate interest. Asset ownership remains with the debtor until
a funded sale; authorization controls disposal, not a fictitious title transfer.

## Monthly boundaries

| Boundary | Work and visibility |
| --- | --- |
| Due: opening | A dated authorized proceeding opens only if previous loan/land arrears or an overdue accepted forward exists. Otherwise it records rejection. Loans accelerate, interest freezes and ordinary loan collection/enforcement is stayed; non-loan dates remain unchanged. |
| Due: ordinary servicing | Non-stayed loans and native land performance collect with existing policies. Estate-eligible land cash claims wait for estate allocation. |
| Due: guarantees | Eligible guarantees pay residual due claims from remaining opening guarantor resources, after ordinary servicing/collateral enforcement. Recourse is recorded but cannot be collected in this same boundary. |
| Due: estate collection/distribution | Non-protected debtor coins enter custody. Only opening estate funds can be distributed. Newly received deposits and current-month sale proceeds wait for a later boundary. |
| Acquire | Existing admitted forwards deliver at maturity from finite goods/storage. Funded bids purchase listed assets. Price-descending order within each case/asset, then stable bid ID, is explicit. An unfunded bid leaves the asset available for the next eligible bid. |
| Productive and later phases | Existing work and essential-need behavior continue. The purchaser receives future attached work/output rights; completed work and sunk inputs are preserved. |

An active proceeding blocks new direct borrowing, lending and land acceptance by its debtor.
It does not terminate the agent or cancel production, and an unsold asset does
not disappear. A failed bid is recorded without partial money or title changes.

## Guarantees and recourse

A guarantee records the covered original loan, guarantor, cap, validity interval,
missed-payment delay, collection priority, and a reserved recourse-loan identity.
The configuration represents the parties' consent to these terms.

Calls run in priority/ID order against the same finite opening execution budget
used for ordinary servicing. Each successful payment reduces the original loan
interest-first and adds the same amount to an unsecured, zero-interest recourse
loan owed by the debtor to the guarantor. Borrower total debt does not disappear;
its creditor changes. Balance sheets derive both sides from those records.
The common `agreements::for_agent` adapter exposes the accepted guarantee to the
borrower, creditor and guarantor, including paid-to-date and any currently callable
claim. Inspection and servicing share the same trigger calculation. A contingent
call is not additional borrower principal and inspection does not settle it.
No cash is credited to the borrower. Repayments received by the guarantor cannot
fund another call in the same boundary.

Caps are lifetime paid amounts. Expiry prevents new calls without deleting
existing recourse. Recourse starts collecting at a subsequent Due boundary.
Multiple calls can add to the same recourse record. Guarantees cannot cover other
recourse loans, so cycles and recursive same-month chains are explicitly rejected.
Existing pending-resale mortgage loans are also excluded until lien subrogation
is defined. Fixed-value collateral settles before guarantee calls; guarantees
cover the remaining due exposure. This version does not transfer collateral liens
to guarantors or implement proportional sharing between guarantee calls.

## Realization, priority and closure

`asset_exchange::settle` is now shared by existing collateral resale and estate
sales. It validates seller title, buyer permission, finite funding, complete
payment legs, receiving storage, and atomic transfer of title and attachments.
Existing collateral resale pays creditor/surplus recipients directly; estate
sales pay custody. Their agreed timing differs; their physical exchange primitive
is the same. Neither creates coins from an appraisal.

At a later Due boundary, actual proceeds reserved for an asset's existing lien
pay that claim first. Any remaining estate cash funds the loan and eligible land-cash waterfall:
lower collection ranks first, with proportional sharing among equal ranks. The
current loan book permits one active pledge per asset, not multiple competing
liens. Collateral priority is separate from ordinary collection priority.

After the earliest authorized closing month, all listed assets must be sold and
secured reservations resolved. Unpaid creditors cannot be discharged while
undistributed cash remains. Closure either preserves accelerated deficiencies or,
if explicitly authorized, records per-loan principal/interest write-offs and marks
those loans `Discharged`, distinct from `Repaid`. Surplus goes back to the debtor.
New same-month recourse must get a later collection boundary before closure.
Unpaid land bills or undelivered accepted forwards also prevent closure, including
future forwards. The discharge flag does not forgive these performance claims.
A separate accepted forward amendment can extend an overdue delivery or release
a quantity at Due before closure; actual delivery counters remain unchanged.
Closed receipts retain their date so subsequent annual bills remain distinguishable
from claims that should have blocked closing.

Remaining general claims, durable equipment inventories, multicurrency estates,
shared operating custodians, multiple liens, automatic asset discovery, auctions,
contested authorization, autonomous restructuring and general non-loan discharge
are not implemented. Constitution/charter and state-law machinery will eventually
supply these terms; configured authorization is the current integration point.

## Verification controls

`tests/recovery.rs` runs through the real simulation and settlement:

- Two six-coin guarantees with eight guarantor coins pay six and two, retaining
  total borrower debt and creating matching recourse receivables.
- Recourse repayments cannot finance further guarantee calls in the same window;
  expired guarantees do not create claims.
- A pledged asset sold for eight coins against two ten-coin loans pays eight to
  its lien first; twelve remain as deficiencies or explicit recorded write-offs.
- An unpledged asset's eight-coin proceeds split four/four at equal rank; thirty
  coins repay both loans and return ten as surplus.
- Nine coins deposited into custody cannot pay creditors until the next Due;
  the subsequent equal-rank allocation is five/four.
- An unfunded top bid releases the opportunity; an asset without a funded buyer
  remains unsold, with its proceeding and claims still open.
- Arrears alone do not open proceedings; an authorized but solvent case rejects.
- Altered estate cash receipts reject the whole batch without money/title changes;
  unknown lien references reject malformed checkpoints before execution.
- Interest freezes from the authorized opening month and due collateral
  enforcement is stayed without discarding its pledge.
- Shared contract inspection exposes contingent guarantee calls to their three
  parties without mutating debt; unrelated agents do not receive that view.
- CPU, reference, monthly and checkpoint continuation agree in the secured case.

Settlement observers expose guarantee calls, sales/rejections, distributions,
closure and per-loan write-offs. Distribution receipts retain requested,
allocated and paid quantities, including zero allocations under scarcity.
Raw output belongs in ignored `output/economics/`.
These are accounting/continuation controls, not evidence of calibrated credit
supply, a viable economy, or a complete legal system.

### Earlier loan-estate validation

The crate-wide `cargo +1.92.0 test --locked` run passed 451 tests with no
failures. After the final checkpoint guard, guarantee inspection and distribution
receipt changes, six affected suites passed 61 tests, including all 14 recovery
tests. These counts overlap; the final focused run covers the last edits.

Formatting, all-target Clippy with warnings denied, and the repository artifact
check passed. CPU/reference and continuation checks are part of these suites;
no CUDA validation or economic calibration is claimed.

Subsequent land/forward admission and its validation are recorded in
[Land/forward admission](LAND-FORWARD-ADMISSION.md).
