# Economics structure review and standardization opportunities

Status: source review and proposals only, not an implementation plan. Reviewed
against the collateral-resale implementation at `3925abe`. No simulation behavior
changes accompany this document. The scope is the standalone experiment, not the
main civilization scheduler. Findings describe interface gaps, not demonstrated
accounting failures.

Follow-up: the first increment below is now implemented as read-only
`agreements::for_agent` / `LoanView` inspection. See [shared agreements](AGREEMENTS.md)
for its lifecycle distinctions, boundary semantics and tests. Other standardization
options in this review remain proposals; the original findings below describe
the reviewed baseline.

The next increment adds a [financed-purchase offer adapter](SECURED-CREDIT.md#common-offer-adapter).
It retains the scripted application and existing credit settlement. General
borrowing search remains a proposal; acquisition composition now has the scoped
credit/exchange implementation described below.
A bounded [borrowing comparison](BORROWING-DECISIONS.md) can now accept or decline
one configured offer; general borrowing search remains outside that pilot.
Shared observation construction is now implemented for search/planning, work
choice and resale; see [forecast context](FORECAST-CONTEXT.md).

Current follow-up: [integration status](INTEGRATION-STATUS.md) records the shared
credit/negotiation acquisition boundary, membership-based credit permission and
common need-constraint helpers. Its compatibility matrix supersedes baseline
claims below about missing interfaces. The remaining suggestions remain proposals.

## Reviewed baseline

The foundation already separates decisions from publication. Agents and policies
propose work; domain resolvers produce effects and receipts; settlement validates
a dated batch, stages changes and publishes the resulting state. Stable IDs and
integer resource balances support reference/CPU comparisons.

| Responsibility | Current implementation |
| --- | --- |
| Tables and boundary records | `model.rs`: `World`, `State`, `Batch`, `Transaction`, process and resource tables |
| Monthly execution | `simulation.rs`: boundary dispatch; `model::Phase::next` and `settlement::commit_core`: phase progression |
| Discovery and acceptance | `opportunities.rs` and `offers.rs`: membership, land access, production and collection |
| Accepted agreement views | `agreements.rs`: membership, land and process identities, grants, payments and consequences; domain records remain authoritative |
| Transfers and claims | `finance.rs`: resource-denominated transfers, obligations and payment feasibility |
| Allocation | `allocation.rs`: ranking; `resolution.rs`: immediate versus whole-bundle prerequisite reservation |
| Decisions | `search.rs` and `planning.rs`: need-directed candidates/evaluation; `work_choice.rs`: bounded remaining-value decisions |
| Exchange and finance pilots | `exchange.rs`, `forward.rs`, `negotiation.rs`, `marketplace.rs`, `zip.rs`, `credit.rs`, `resale.rs` |
| Publication | `settlement.rs`: domain receipt validation, staged state changes and gathered account effects |

These are several controlled experiments sharing primitives. They are not yet one
fully composable economy. For example, `credit::validate` rejects combination with
negotiation, households, transaction permissions and several other acquisition
drivers. Those guards should stay until joint budgets and timing are defined.

## Where standardization would help

### 1. Extend the common offer and agreement views

Today `offers::Id` covers membership, land and processes. Financed purchases use
`credit::discover` and a configured application; resale uses a configured buyer.
Neither is exposed through the common offer interface or the marketplace agent.
Likewise, `agreements::Identity` has no loan identity, although `credit::Loan::claim`
already exposes a shared `finance::Obligation`.

Proposed direction: add domain adapters with stable offer/agreement identities,
participants, eligibility, terms, claims, grants and consequences. Keep loan
balances in `credit::Book`; derive common views from that book rather than copying
debt into another mutable registry. Credit enforcement needs a typed consequence
extension: it must not be disguised as the existing `SuspendNewUse` or process
abort rule. Pending sale, custody, deficiency and completed repayment must remain
visible without squeezing them into a misleading generic status.

A common offer interface should eventually expose discovery, feasibility,
proposal and acceptance for these domains. It need not use one matching algorithm
or send environment production through a bilateral pricing session. ZIP remains
a pricing policy for eligible exchange offers, separate from contract semantics.

### 2. Give forecasts one explicit observation boundary

`SearchContext::new` hides future capacity shocks and scripted starts.
`work_choice::forecast` separately sanitizes those inputs, future credit transfers
and resale attempts. `resale::bid` creates hypothetical ownership and process
changes before invoking the remaining-value evaluator. These are related forecast
operations with separately maintained assumptions.

Proposed direction: share construction of an observed snapshot, with explicit
horizon, permitted hypothetical changes, known future commitments and excluded
fixture events. Keep candidate generation and scoring swappable. Need reduction,
subjective stock value and a coin-denominated bid are different outputs; one
unlabelled score would obscure their meaning. Known contractual due dates should
remain visible even when undisclosed future endowments are hidden.

The resale buyer currently values alternatives using `work_choice`, but executes
after purchase through ordinary continuing-work behavior. Expose this policy
assumption in forecast receipts. Eventually let valuation and execution select
the same policy where intended; do not silently claim that they already do.

### 3. Standardize competing action requests before composing pilots

`Simulation::step_core` selects alternative acquisition drivers through branches;
credit takes over Open/Due/Acquire when enabled. That is an explicit pilot boundary,
not a joint resource resolver. Removing validation guards could allow separate
evaluators to promise the same opening coins or asset.

Proposed direction: collect eligible requests at an existing boundary, identify
their shared accounts/assets and use explicit allocation and resolution policies.
Reserve complete required bundles before producing one atomic batch. Reuse the
ranking/resolution separation already implemented; unique title and loan-state
preconditions still need domain validation alongside divisible resource limits.

The [possible redemption extension](COLLATERAL-RESALE.md#possible-extension-borrower-repayment-before-resale)
would be a bounded example: repayment and resale both claim the same pending plot.
Priority chooses among feasible actions; it does not change when the month runs.
Do not change the scheduler to express borrower preference.

### 4. Separate asset title, control and beneficial interest consistently

`model::Asset.owner` provides initial ownership; `credit::Book.owners` records
changes and `credit::owner` resolves the current title. Ownership-following rights
and crop transfers already use this path. Pending resale also distinguishes
lender custody from the borrower's restricted economic asset in balance sheets.

Proposed direction: make asset queries and transfer receipts a shared interface
as a second transferable-asset domain needs them. Distinguish title, use/output
rights, security interests and custody explicitly. Preserve attachment transfer
rules and historical completed-process ownership. Do not flatten these into one
owner field or revalue a crop merely because its operator changes.

### 5. Standardize receipt metadata and quantity meaning

`Batch` and `Transaction` accumulate optional domain fields. Some boundaries also
carry their own transaction lists, checked for exact equality with the batch.
This works under current validators but increases the combinations each new
feature must validate. Several IDs are `u32` aliases, while amounts and prices
often use bare `i32`; resale values explicitly mean denomination ticks, whereas
work-choice values are subjective.

Proposed direction: introduce a common receipt envelope with dated boundary,
actor, domain-qualified offer/agreement ID and outcome. Keep typed domain payloads
and exact validation. Incrementally distinguish quantities, denomination-bearing
prices and subjective scores at public interfaces. Document rounding and scale;
do not change calibrated arithmetic as part of an interface cleanup. Shared
failure codes can coexist with domain detail and human-readable explanations.

## Recommended first increment

Start with **read-only loan agreement views**, before another financial behavior:

1. Add a loan identity and a typed view of accepted terms, current claims,
   collateral and enforcement state, backed solely by `credit::Book`.
2. Expose that view alongside existing membership/land/process views for reporting
   and inspection; leave offer acceptance and monthly settlement unchanged.
3. Verify active, overdue, fixed-value enforced, pending-sale, deficient and repaid
   loans against authoritative records, including matched receivable/payable claims.

Success means one way to inspect what an agent has agreed to, with no duplicated
balances or changed outcomes. After that, consider a credit offer adapter and a
shared forecast context as separate increments. Defer a universal contract
language, broad module reorganization and general market composition until a
specific second use makes each extraction necessary.

For later behavior changes, retain forged/stale receipt rejection, conservation,
CPU/reference equality, monthly/batched execution and checkpoint controls. For
allocation changes, compare identical opening claims and inspect actual completed
actions, not just grants. No tests were rerun for this documentation-only review.
