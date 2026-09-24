# Shared agreements and explicit consequences

Implemented in `src/agreements.rs`: citizenship, land access and production expose
one accepted-agreement structure with identity, grantor, holder, acceptance date,
optional expiration, grants, recurring payment terms and dated obligations.
Production also exposes staged performance terms and conditional outputs; see
[MARKET-AGREEMENTS.md](MARKET-AGREEMENTS.md) for its common offer interface and planning.

These are derived views of existing authoritative membership, access and payment
receipts. There is no second mutable agreement balance or lifecycle table. The
shared evaluator is used by membership authorization and land-use checks in both
planning and settlement. Land billing reads the shared recurring payment terms.
Offer eligibility and atomic acceptance remain in their domain resolvers.

`agreements::for_agent(world, state, agent)` now provides a shared inspection entry
point for accepted membership, land, process, loan, forward and guarantee agreements.
The participant filter includes holders/grantors and all three guarantee parties.
It excludes unaccepted catalog offers and retains terminal agreements. It groups domains and orders by stable IDs, independent of catalog row
order. It is a read-only query over validated state, not an acceptance or payment
interface. A separate output beneficiary who is neither holder nor grantor is not
included by this participant filter.

| Agreement | Grant | Obligation | Consequence |
| --- | --- | --- | --- |
| Citizenship | Membership in the issuing state with a role; state policy maps the role to actions | None after acceptance | None: there is no upkeep obligation to breach |
| Land access | Use of a particular right, within its dated duration | Payment annually, first due 12 months after acceptance | Unpaid due amounts suspend **new use of that grant** until fully settled |
| Loan | Accepted financing in its denomination, identified by `Identity::Loan` | Scheduled principal and accrued interest; collectible deficiency after enforcement | Optional collateral terms identify asset, creditor, grace and settlement; unsecured debt retains arrears |
| Prepaid forward | Funding already advanced | Recorded undelivered commodity claim | Outstanding delivery restricts new advances under the existing policy |
| Guarantee | Capped protection of an original loan | Callable guarantor-to-creditor payment | Payment reduces the original claim and creates matching debtor-to-guarantor recourse |

## Read-only loan adapter

The inspection result is a typed `View`: existing domains expose `Agreement`,
while loans expose `LoanView` and guarantees expose `GuaranteeView`. The shared
view provides identity, grantor, holder and acceptance month. A loan's grantor is its creditor, not necessarily the asset seller.

`LoanView::record()` borrows the authoritative `credit::Loan`. Original principal,
denomination, rate, duration, grace, collateral priority and settlement terms come
from the accepted loan, not today's offer catalog. Current principal, interest,
pledge and accrual fields come from that same record; no second mutable balance
or lifecycle registry is introduced. Changing an unaccepted catalog offer cannot
rewrite the inspected accepted terms.

`LoanView::state()` preserves domain distinctions:

| State | Meaning |
| --- | --- |
| Current | Active loan with no recorded failed collection |
| Overdue | Active loan with a committed `first_unpaid` month |
| PendingSale | Repossessed collateral awaiting realization, with listing month |
| Deficiency | Enforced loan with remaining collectible debt |
| Repaid | Principal and interest cleared, including repayment through collateral proceeds |
| Stayed | Authorized proceeding pauses ordinary collection/enforcement and freezes interest; estate recovery handles the claim |
| Discharged | Explicitly written-off deficiency, distinct from repayment |

`outstanding()` returns total debt in its denomination. `claim()` returns the
current shared financial claim, or none when no amount is due, the debt is repaid,
or pending resale pauses collection. **No claim does not necessarily mean no
debt.** In pending sale, full debt remains on both balance sheets, the creditor
holds title/custody and the borrower retains the restricted financial asset.
`title_holder()` reports current title; it must not be interpreted as the owner
of every financial interest or historical crop output. A stayed loan can expose
its accelerated claim through inspection; that is not authorization for ordinary
collection. The proceeding controls actual recovery.

The view's `boundary()` records the current month and next phase to execute.
Inspection does not accrue interest or run collection. Before Due, a scheduled
installment may be visible without recorded arrears or that month's uncommitted
interest. After Due, the same query reflects committed accrual and payment.
`on_default()` describes accepted enforcement terms; it neither predicts an
automatic seizure nor bypasses grace, funding or settlement checks.

This adapter deliberately does not run loans through the land/process status
evaluator. Their enforcement and custody states retain their own typed meaning.
Credit offer acceptance, repayment scheduling, resale and balance-sheet calculation
are unchanged. Borrower redemption remains a
[possible extension](COLLATERAL-RESALE.md#possible-extension-borrower-repayment-before-resale).

## Guarantee inspection and current recovery coverage

`GuaranteeView` exposes configured accepted terms, original debtor/creditor,
paid-to-date and any currently callable claim. The borrower, original creditor
and guarantor can inspect it once the underlying loan exists. Inspection and
servicing use the same trigger calculation; the view does not reserve funds,
promise a full payment or add another borrower liability. Actual payment creates
recourse in the authoritative loan book.

The latest [recovery checks](CONTRACT-RECOVERY.md#completed-validation) include
all-party filtering, contingent exposure without debt mutation, stays, discharge,
custody and CPU/checkpoint continuation. Configured consent and authority remain
distinct from autonomous offer discovery and general legal proceedings.

## Consequences are agreement terms

`PaymentTerms.on_unpaid` and each dated obligation identify a
`Consequence::SuspendNewUse(grant)`. This is distinct from cancelling the agreement
or revoking every permission held by the person. Evaluation returns breach records
with the source obligation index, outstanding amount and applicable consequence.
The claim retains its due-date condition, parties, commodity and settled amount.

`Agreement::permits(month, grant, usage)` distinguishes starting new work from
continuing existing work. A land breach prevents new cultivation using that right;
it does not interrupt an existing crop, revoke citizenship, block an unrelated
right, erase debt, or impose another penalty. Partial payment leaves the restriction
in place. Paying all due claims clears it automatically. These are the existing
behavioral rules, now represented and evaluated through shared agreement machinery.

The derived statuses are Pending, Active, Restricted, Expired, Completed and Failed.
Completed and Failed describe production outcomes. Grant expiration
is inclusive of the final month and does not erase unpaid claims. An expired
agreement can therefore retain breach information while permitting no further use.
Existing production feasibility checks still enforce the right's full duration.

## Monthly boundary

Acceptance remains atomic in Acquire, including citizenship-plus-land bundles.
Annual billing and payment remain in Due. The shared evaluator reads committed
receipts: a newly due bill is not fabricated during a permission query. Once Due
commits, unpaid claims affect subsequent new-work checks. Existing work executes
normally; ClearArrears can use the resulting harvest to pay the claim. A successful
payment restores new-use eligibility for later decisions, without changing already
completed production. Forecasts use these same boundaries and checks.

No additional scheduler phase, automatic revocation, or independently stored
consequence event was added. Replay reconstructs restrictions from committed
agreement and payment receipts.

## Scope

This is a common accepted-agreement and consequence interface, not yet a universal
contract interpreter. The adapter methods are `membership::Agreement::contract`,
`commitments::Agreement::contract` and `agreements::process`. A common offer
interface dispatches to existing domain catalogs. The secured-credit pilot now
exposes its accepted terms and collateral consequences through inspection, but
financed-purchase acceptance now uses the common offer dispatcher for the
configured application. Resale acceptance remains separate.
Arbitrary negotiated grants, general selectable penalties and termination are
not implemented by a universal interpreter.
Citizenship's action permissions still come from the state's role policy.

## Validation

Focused tests cover common terms, namespaced identities, due versus future claims,
partial payment, restriction scope, continuing work, expiration with outstanding
debt, and restoration after CPU harvest settlement. Existing commitment and
membership tests check atomic acceptance, replay, monthly/batched and checkpoint
continuation, and CPU/reference agreement.

Historical initial-adapter results: 35 focused and integration tests passed.
Four 18-month reference regression controls retained identical state, reports and committed batches
(excluding planner diagnostics). Formatting, Clippy with warnings denied and
repository artifact checks passed. Local test outputs are under ignored `output/`.

Loan inspection adds five tests covering accepted terms versus edited offers,
namespaced identities and participant filtering, pre/post-Due observations,
fixed-value enforcement, pending-sale custody, realized deficiency and repayment.
They compare inspected claims with both balance sheets and verify unchanged
state, ledger and reports across reference/CPU execution. The new query does not
change monthly scheduling or checkpoint data.

Loan-view, agreement, credit, resale, collateral-crop, finance, membership and
commitment suites passed **45 tests**. After boxing the larger inspection variant
to keep the enum compact, the eight loan-view/agreement tests passed again.
All-target Clippy passed with warnings denied. Run the focused additions with
`cargo +1.92.0 test --locked --test loan_views` from this directory.
