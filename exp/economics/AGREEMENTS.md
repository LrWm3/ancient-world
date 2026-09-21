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

| Agreement | Grant | Obligation | Consequence |
| --- | --- | --- | --- |
| Citizenship | Membership in the issuing state with a role; state policy maps the role to actions | None after acceptance | None: there is no upkeep obligation to breach |
| Land access | Use of a particular right, within its dated duration | Payment annually, first due 12 months after acceptance | Unpaid due amounts suspend **new use of that grant** until fully settled |

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
interface dispatches to existing domain catalogs; arbitrary negotiated grants,
selectable penalties, grace periods,
termination, collateral seizure and other contract types are not implemented.
Citizenship's action permissions still come from the state's role policy.

## Validation

Focused tests cover common terms, namespaced identities, due versus future claims,
partial payment, restriction scope, continuing work, expiration with outstanding
debt, and restoration after CPU harvest settlement. Existing commitment and
membership tests check atomic acceptance, replay, monthly/batched and checkpoint
continuation, and CPU/reference agreement.

Results: 35 focused and integration tests passed. Four 18-month reference
regression controls retained identical state, reports and committed batches
(excluding planner diagnostics). Formatting, Clippy with warnings denied and
repository artifact checks passed. Local test outputs are under ignored `output/`.
