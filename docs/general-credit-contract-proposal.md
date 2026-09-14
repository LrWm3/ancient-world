# General credit contracts — proposal only

Status: proposal for discussion, not planned work or an implementation commitment.
No credit rules, feature defaults or existing contracts change through this document.
The possible increments below describe how the idea could be evaluated if adopted.

## Proposed simplification

Represent credit as a contract between two identifiable parties. Give the contract
a creditor, debtor, principal, commodity, maturity, interest, optional collateral
and/or guarantor, and priority. Let agents propose and negotiate these terms using
their resources and observed circumstances. Keep permission to originate a contract
in a separate policy, so the same mechanism can later be restricted to selected
party types, relationships, commodities or locations.

The simplification is one contract lifecycle and one negotiation interface. It does
not mean that all parties own the same assets, know the same facts or accept the
same risks. Council taxes, workshop earnings and expected harvests become evidence
supplied to negotiation rather than separate kinds of debt with separate lifecycles.

## What already exists

The current foundation is already partly general:

- [`src/credit.rs`](../src/credit.rs) defines `Account`, `Terms`, `Loan`, simple
  interest, maturity, arrears, repayment, restructuring and write-off. Accounts
  include councils, institutions, towns, operators and households. Household
  accounts currently receive settlements but cannot originate loans.
- [`accounts.rs`](../src/credit/accounts.rs) transfers existing cash between
  account representations, preserving exact debit/credit agreement. A claim is
  not spendable money, and borrowing is tracked separately from operating income.
- [`underwriting.rs`](../src/credit/underwriting.rs) already separates requests,
  voluntary offers, dated evidence and bounded allocation. It considers lender,
  borrower and repayment-source capacity before actual transfers.
- [`councils.rs`](../src/credit/councils.rs) and
  [`commercial.rs`](../src/credit/commercial.rs) construct specialized requests.
  `RepaymentSource` currently requires annual taxes, export proceeds or a service
  order; this makes particular financing purposes part of the contract schema.
- Servicing, claim ownership, estates, restructuring and recovery preserve debts
  across closure and succession. These are useful lifecycle responsibilities,
  not reasons to create another parallel ledger.

The current model does not provide general commodity lending, pledged physical
collateral, guarantor settlement or negotiated claim seniority. Nor does it have
one general negotiation model using all the signals described here. Those would
be behavior additions, separately evaluated from structural simplification.

The [earlier credit design](credit-and-currencies-design.md) describes the existing
pilots and their evidence gates. This proposal offers a different possible shape
for general credit; it does not declare those experiments successful or authorize
their wider activation. Issuance and currency exchange remain separate mechanisms.

## Contract, evidence and permission

### A small common contract

| Field | Proposed meaning |
| --- | --- |
| Contract ID and opening month | Stable identity and actual origination boundary |
| Creditor and debtor | Stable legal/account identities; officeholders act for an account rather than personally inheriting its debts |
| Principal | Quantity actually advanced; requested and agreed quantities remain in proposal receipts |
| Commodity | A typed settlement unit, initially shared currency; later an explicitly supported catalog good with its unit |
| Maturity | Absolute due month, plus an explicit grace/default rule |
| Interest | Initially the existing annual simple rate, accruing in the principal's commodity; no implicit compounding |
| Collateral | Optional identified assets or receivables, pledged quantity/share, custody and recovery terms |
| Guarantor | Optional consenting party, maximum liability, covered components and payment trigger |
| Priority | Agreed rank within a defined debtor/commodity repayment pool; collateral rights apply only to the pledged assets |

Outstanding principal, accrued interest, status and receipts remain lifecycle
state derived from actual actions. Retain original terms and dated amendments;
claim assignment changes the current creditor through the existing ownership
history, without overwriting origination provenance.

Use one typed identity interface backed by the existing account variants. “Any two
parties” means the engine does not encode allowed pairings: both parties still need
stable identity and adapters for the assets and settlement operations they support.
A future person or other entity can gain an adapter without another loan type.
Do not manufacture personal ownership of town stores or household money merely
because an individual negotiates on behalf of that group. Self-contracts remain
invalid, and an estate can settle without regaining operating eligibility.

For the initial form, lend and repay the same commodity. A loan of 100 kg of grain
at 10% annual simple interest for twelve months owes 110 kg if no principal is
repaid early. A currency loan used to buy grain remains a currency loan. Repaying
grain debt in cash would require separately agreed conversion and an actual trade;
prices alone cannot settle it. Unlike commodity quantities must never be summed.

### Evidence is not a compulsory contract type

Move required `RepaymentSource` out of universal terms into optional, dated
underwriting evidence. Preserve its stable references for existing tax, export and
service-order loans and their recovery rules. A reference used only to forecast
income is distinct from an enforceable pledge of that income.

The common evidence view could contain liquid resources, obligations, protected
operating needs, observed repayment history, expected net receipts, uncertainty,
location and known political exposure. Specialized providers still calculate tax
or harvest forecasts correctly. They do not need separate origination, accrual or
default paths. Unsecured lending can be permitted by policy without inventing a
pledged source; it still requires a willing creditor and a feasible transfer.

### Permission is a separate filter

A dated origination policy would answer whether these parties may negotiate this
commodity and security arrangement. It could filter party classes, specific IDs,
same-town relationships, political jurisdiction or supported asset capabilities.
Keep the existing council/commercial permissions as a compatibility profile; an
explicit experimental profile could allow all supported distinct account pairs.
Broad capability would not silently enable every pairing in ordinary histories.

Permission does not compel consent or guarantee funding. Apply it both before
negotiation and at commitment. A later restriction blocks new contracts without
canceling existing debt, preventing collection or rewriting agreed terms. Permission
to guarantee and pledge assets also needs explicit authority; a borrower cannot
name an unwilling guarantor or pledge someone else's property.

## Bounded agent negotiation

Use a small offer/counteroffer procedure, with a fixed bound on proposals, rather
than an open-ended bargaining simulation. A borrower proposes a useful quantity,
acceptable maturity and maximum financing cost; potential creditors quote amounts,
rates and security requirements. Both can decline. An agreed small loan that cannot
fund an indivisible purpose should be rejected or counteroffered, not celebrated
as activation.

| Signal | Possible effect on willingness and terms | Boundary to preserve |
| --- | --- | --- |
| Wealth | Available principal, exposure tolerance and borrower reserves | Spendable assets differ from illiquid wealth and existing claims |
| Reputation | Expected loss, willingness to accept unsecured terms, required security | Use observed payment/default history; unknown is not perfect, and agents need not know every distant default |
| Scarcity | Value of retaining a commodity, amount offered, borrower's urgency | Need alone does not establish repayment capacity; essential reserves remain explicit |
| Distance | Counterparty discovery, delay, transport cost and recovery difficulty | Use known routes/contact and actual delivery rules, not instantaneous remote goods transfers |
| Harvest expectations | Repayment quantity, due date and uncertainty margin | Use dated forecasts of net harvest after needs and prior claims, never future realized weather |
| Political risk | Exposure ceiling, security preference, willingness and rate | Use known war, route or jurisdiction conditions, not omniscient future outcomes |

Start with transparent bounded rules and a small number of terms. Avoid putting
every signal into one unexplained score: receipts should show which constraint
reduced the amount or caused refusal. High risk may produce no acceptable offer,
not simply a higher rate. The borrower compares repayment burden and expected
benefit, so lender willingness alone does not determine acceptance.

Counterparty discovery should use existing local/contact relationships and a
bounded candidate set. Generic pair support does not require an all-world pairwise
search every month. If negotiation uses randomness, use a dedicated deterministic
stream and persist the decision boundary.

Negotiated proposals are provisional until competing commitments are resolved.
Collect feasible claims on each lender's commodity pool and shared borrower,
collateral and guarantor limits before any claimant spends them. Reuse the existing
snapshot/allocation pattern, recheck minimum useful amounts and joint constraints,
then obtain acceptance of any changed terms. Start with no redistribution of an
unfillable grant, matching the existing funding-check approach; record unused
capacity. A later redistribution policy would be a separate comparison.

## Settlement and security semantics

Retain one lifecycle: proposal, accepted commitment, actual advance, performing
debt, arrears, repayment/default and any recovery. An accepted proposal is not
principal disbursed. Commit actual transfers and their receipts together so failed
preflight cannot leave a debit without a credit or a pledge without a contract.

Commodity adapters would need ownership, available quantity, reservation, transfer
and ledger operations. Physical loans must use actual storage and transport. A
remote advance may be committed and in transit before it becomes usable principal;
the contract must state when debt begins and who bears transit loss. For a first
goods experiment, use local delivery and start debt on receipt. Defer remote goods
loans until cargo ownership, arrival, loss and repayment transport are explicit.
Borrowed goods can then be consumed normally; the debt remains a claim, not a
second copy of the inventory. Interest creates a claim, not grain or currency.

Collateral needs an encumbrance record with stable asset identity and bounded
quantity. Existing source-pledge checks are a starting point for preventing double
pledges, but an expected harvest is not inventory already in custody. Begin with
one supported collateral class, such as escrowed existing goods or an identified
receivable; reject unsupported pledges. Define spoilage/loss and release on payment.
Seizure is an actual ownership transfer. Valuation does not create repayment cash;
conversion requires a sale, with residual debt and any surplus reported separately.

A guarantee is a capped contingent obligation, not cash. Reserve exposure across
all guarantees, require recorded consent, and call it only for the uncovered amount
under the agreed trigger. First scope could exclude guarantee chains and cycles.
Payment reduces the creditor's outstanding claim once and creates a corresponding
recovery claim for the guarantor against the debtor; it does not duplicate debtor
liability. Borrower, collateral and guarantor recovery together cannot exceed the
covered debt. A guarantor can lack funds and fail too.

Priority would rank claims only within their defined settlement scope. Protect
agreed operating/subsistence allowances first, settle higher ranks next, and share
insufficient allowance proportionally among equal ranks. Do not use contract
iteration order as seniority. Secured recovery uses its pledged asset pool; only
the remaining eligible claim joins general collection. Existing loans should
retain equal rank under the compatibility profile. New senior claims must respect
existing covenants/consents; changing rank must not silently subordinate old debt.

Preserve arrears, bounded consensual restructuring, explicit losses, exact-transfer
precision handling, estate retention and dated claim succession. Closing an operator
must not silently make its owner personally liable. Where legacy loans have special
export or estate recovery behavior, retain a versioned recovery policy until an
equivalent common path has been verified.

## Integration and possible increments

The [monthly schedule](monthly-schedule.md) remains authoritative. Current
[`civilization.rs`](../src/civilization.rs) services debt in Open, runs council
credit early in Reserve, and commercial credit after production planning in
Reserve. Annual tax evidence arrives later in Respond. Consolidating types does
not require merging these decision windows or changing which claim sees cash first.
Keep dated plans and the existing settlement hooks; new same-month income cannot
retroactively enlarge a completed collection snapshot.

If this proposal were adopted, these would be possible independently reviewable
increments, not a committed sequence:

1. **Common cash-contract interface with compatibility behavior.** Separate account
   existence/asset capability from origination permission. Introduce common terms,
   evidence and policy interfaces around the current ledger. Migrate old currency,
   source references, rates, dates, balances and receipts without changing behavior.
   Old loans get no new collateral/guarantor and retain current equal-rank collection.
   Verify existing fixtures and matched checkpoint histories before deleting any
   redundant paths.
2. **Shared negotiation with restricted participants.** Make council and commercial
   code evidence/request providers. Route them through the same bounded proposal
   interface while keeping legacy terms as a selectable profile. Add the observed
   risk/wealth signals incrementally. Demonstrate different supported party pairs
   through the same engine and policy-only permission changes in controlled tests.
3. **One local goods loan.** Add one real commodity adapter and same-commodity
   repayment. Test delivery, consumption, interest claims, shortage and default.
   Keep cross-commodity settlement and remote physical loans outside that experiment.
4. **Security and priority, one feature at a time.** Evaluate a single collateral
   class, then capped guarantees, then ranked collection. Each needs contention,
   failure, recovery and estate tests before combining them.
5. **Optional broader participation experiments.** Compare permission profiles and
   negotiation settings on matched runs. Only then consider additional party
   adapters, goods or remote delivery. None requires another contract lifecycle.

Every increment would preserve monthly/batched/checkpoint consistency and old-save
loading. Structural refactoring would require unchanged outcomes under the legacy
profile; new bargaining/security policies would require separately labeled outcome
comparisons. Archive migration must not fabricate past reputation observations,
consent, collateral or guarantees. Meaningful constants belong in their owning
subsystem modules, following repository guidance.

## Related generalizations worth exploring

These are additional proposals, not planned extensions or prerequisites for the
credit experiment. The strongest candidates are mechanisms surrounding contracts
that otherwise risk acquiring separate rules for each party or activity.

| Candidate | Common responsibility | Distinctions to preserve |
| --- | --- | --- |
| Party identity, ownership and authority | Identify asset owners, authorized representatives and the parties liable after death, closure or leadership change; useful across credit, trade, employment and estates | Representing an organization does not confer ownership or personal liability; account existence differs from operating permission |
| Promises and delivery obligations | Share stable obligation IDs, deadlines, partial fulfillment and breach receipts across loans, procurement orders and export agreements | A debt repayment, a goods delivery and paid employment have different completion and settlement rules |
| Asset reservations and competing claims | Reuse a pattern for reserving money, goods, receivables or capacity, with explicit priority, expiry and release | Each resource retains its own scope, eligibility, minimum useful grant and allocation policy; there is no implied global resource auction |
| Counterparty knowledge and reputation | Record observed payments, deliveries and broken agreements for use by multiple decision systems | Knowledge depends on contact and observation; financial reliability, delivery capability and political trust should not collapse into one universal score |
| Offers and negotiation | Share proposal, counteroffer, acceptance and commitment handling across lending, purchases and service hiring | Each activity supplies its own terms, feasibility checks and bounded search; acceptance alone does not transfer assets or complete work |
| Decision receipts | Consistently identify opportunities, requests, refusals, commitments and actual outcomes | Preserve subsystem-specific reasons, units, timing and missing evidence rather than treating every action as the same funnel |

Party identity/authority and decision receipts would be the most useful starting
points if this direction were adopted. They address existing ownership boundaries
and make the proposed subsystem health checks easier to support without changing
economic behavior. Credit could consume those interfaces while retaining its
current accounting and settlement rules.

Shared negotiation and obligation infrastructure would be better assessed against
a second concrete use case, such as service procurement, before extracting common
code. Demonstrate which responsibilities actually match, retain specialized
settlement adapters, and compare behavior before and after extraction. A universal
contract engine designed in advance could add more complexity than it removes.

## How the idea could be evaluated

Controlled checks would cover any supported pair using the common engine; policy
denial versus voluntary refusal; insufficient lender funds; joint offers exceeding
one lender's budget; minimum useful financing; commodity conservation; double
pledges; competing guarantee calls; equal-rank order independence; senior/junior
shortfalls; and unchanged servicing after new lending is disabled. Closure,
succession, stale forecasts and repeated monthly calls remain required boundaries.

Use the [run-health proposal](run-health-evaluation.md) to distinguish no opportunity,
no willing counterparty, denied permission, failed negotiation, unfunded agreement
and disbursed credit without useful completion. Retain dated inputs, proposed and
accepted terms, binding constraints, actual advances, work/delivery outcomes,
repayment, lender losses and guarantor/collateral outcomes.

Compare the legacy policy with negotiated contracts from the same opening states.
Measure borrower benefit and lender operating shortfalls, food access, completed
production/services, debt concentration and defaults. More contracts or higher
interest receipts alone do not establish improvement. Use controlled changes in
scarcity, reputation evidence, distance, harvest forecasts and political risk to
check their immediate effects before interpreting long-run seed differences.

The design choice still open is how much negotiation adds useful behavior relative
to simple fixed offers. The first experiment could answer that with currency and
existing parties; full collateral, guarantees and goods lending are not prerequisites
for assessing the common interface. Generated evidence would remain under ignored
`output/`, with only source and curated Markdown summaries committed.
