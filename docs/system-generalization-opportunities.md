# System generalization opportunities

Status: proposal for discussion only. These are options, not planned work, an
implementation commitment or authorization to change simulation behavior.

The opportunity is to share mechanisms that already recur across the simulation:
how assets wear, journeys advance, services meet needs, people learn, organizations
retain roles, information reaches decision-makers and parties negotiate credit.
A useful generalization reduces duplicated rules while preserving the differences
that give each system its behavior. Fewer types alone would not establish a simpler
or better model.

This proposal covers seven opportunities, including general credit contracts. It
describes current foundations, possible generalized models and small comparisons
that could establish whether each abstraction is worthwhile. Source links identify
the inspected implementation; older linked experiment results apply to their
recorded revisions.

## 1. Physical assets, construction and maintenance

### State today

[Institutional facilities](material-objects-and-facilities.md) already contain rooms
with material components, condition, useful capacity and unfinished construction.
Their [implementation](../src/facilities.rs) keeps components in the owning
artifact's material ledger rather than introducing another inventory.

Other infrastructure uses its own representations and construction rules: housing,
waterworks, storage, [road upkeep](../src/road_upkeep.rs) and fleet assets.
[Named vessels](../src/vessels.rs), for example, subdivide installed port assets;
their hull materials remain in the port ledger. There are shared concerns, but no
single asset lifecycle covering all these systems.

### Possible generalized model

An asset could expose:

- Stable identity, physical location, owner and authoritative inventory reference.
- Components with material quantities, condition and remaining useful life.
- Construction, repair, expansion and dismantling projects with bounded inputs
  and work remaining.
- Installed capacity and actually available service, with an asset-specific rule
  connecting condition, staffing, inputs and environment to output.

The common layer would handle project progress, component replacement, wear,
ownership and receipts. A waterworks adapter would still require water; a vessel
would still require a crew and a usable route. Installation would not automatically
mean operation. Material substitution would remain limited to supported methods.

### Why explore it, and a small first comparison

New infrastructure could reuse construction and maintenance mechanics instead of
acquiring another independent lifecycle. Start by comparing institutional rooms
with one other stationary asset, such as storage. Extract only matching component
and project responsibilities, preserving its current service equations and monthly
visibility. Check construction, partial completion, repair, destruction and waste.

Do not migrate every asset into individual objects merely for uniformity. Aggregate
GPU infrastructure may need packed data and aggregate component quantities. Shared
semantics need not mean a common in-memory layout or additional GPU readbacks.

## 2. Journeys and transport

### State today

[Household relocation](../src/relocation.rs),
[institutional relocation](../src/institution_relocation.rs),
[expeditions](../src/expeditions.rs), commercial cargo and relief have distinct
journey records and purpose-specific rules. Household journeys account for people,
provisions, blockage, return and loss. Institutional relocation preserves the
organization while leaving its buildings behind.

Some reuse already exists: [religious relief](religious-relief.md) funds shipments
through existing relief transport, and [participation](../src/participation.rs)
tracks named absence and travel duties. These are foundations to extend rather
than evidence that all movement already follows one model.

### Possible generalized model

A journey could describe its itinerary, departure boundary, travelers, cargo and
custody, provisions, reserved capacity, progress and arrival conditions. Common
operations could advance travel, apply route delays, consume provisions, record
losses and deliver surviving contents exactly once.

Purpose-specific behavior would select destinations, authorize departure and
interpret arrival. Migration changes residence; delivery transfers cargo;
exploration records encounters; military movement preserves its own supply and
combat rules. Transport modes would define speed, eligible routes and capacity.

### Why explore it, and a small first comparison

Shared movement mechanics could reduce inconsistent absence, arrival and loss
accounting, and provide consistent hooks for infection and witnessed information.
Start with two existing noncombat flows whose route and delivery semantics actually
match, such as ordinary relief and another compatible goods shipment. Share a
journey clock and delivery receipt before attempting a universal passenger manifest.

Test closure, destination abandonment, partial loss, in-transit saves and disabled
new departures. Existing travelers and cargo must still finish or fail under their
contracts. A generic journey must not create free freight, make all route networks
equivalent, or introduce a second population or inventory owner. Migration and
military adoption would require their own later accounting review.

## 3. Needs and service provision

### State today

Food access, shelter, water service, care and institutional operation define their
own demand, eligibility and responses. Some competition is already explicit. The
[essential-service recovery policy](essential-service-recovery.md) distinguishes
urgent shelter, eligible water repairs and optional housing headroom within a
bounded construction allowance. Research and culture have their own
[service allocation window](service-allocation.md).

These mechanisms do not form a general representation of a beneficiary's need
and the alternative ways to meet it. They also operate at different boundaries and
over different resource pools.

### Possible generalized model

A need could identify its beneficiary, service and units, required quantity,
urgency, duration, acceptable substitutes and consequences of remaining unmet.
A provision option would specify prerequisites, provider, inputs, cost, delivery
time and expected service. An explicit policy would choose among feasible options.

For example, unmet shelter might support a construction or repair proposal where
those alternatives are implemented. A food need could be met through existing
purchase or assistance paths. New alternatives would be model additions, not
automatic consequences of declaring a common need type.

### Why explore it, and a small first comparison

It would become easier to distinguish absent supply, unaffordable supply, failed
delivery and service that arrived too late. Gross-health diagnostics could identify
persistent needs and the unavailable response rather than infer them from balances.

Start with a read-only description of shelter and water shortfalls and their
existing provision options. Verify that it explains current decisions before
allowing it to generate requests. Preserve resource-specific allocation boundaries
and minimum useful work. Do not combine food and shelter into interchangeable
well-being points or implement a global optimizer as a side effect.

## 4. Skills, knowledge and learning

### State today

[Practical knowledge](knowledge-continuity.md) controls access to techniques and
depends on available local holders. Paid lessons and informal exposure already
share [study progress](informal-learning.md) in
[`culture/learning.rs`](../src/culture/learning.rs).

[Production experience](production-experience.md) tracks completed named work in
farming, forestry, mining and construction. Workshops and merchant crews retain
their own experience models. Knowledge of a topic, accumulated practice and
effective productive work are related but distinct representations.

### Possible generalized model

A capability could expose knowledge prerequisites, proficiency, completed practice
and learning provenance. Tasks would declare required capabilities and use their
own mapping from proficiency to effectiveness. Instruction, practice and observation
would contribute through explicit learning channels, with configured transfer
between related capabilities where supported.

Knowing a technique would remain distinct from performing it efficiently or owning
the necessary tools. Informal exposure would not automatically consume paid teaching
time, and reserved idle work would not earn completed-work experience.

### Why explore it, and a small first comparison

Teaching, recruitment and succession could refer to the same capability interface
without introducing another skill counter for each occupation. Start with a common
query and completed-practice receipt for production and workshop work, preserving
their existing learning curves and arithmetic.

Test unrelated-skill controls, absent teachers, partial learning, idle reservations
and continuation. Forgetting, cross-sector transfer and skill-dependent quality are
optional behavior experiments, not necessary parts of a structural extraction.

## 5. Organizations, roles and succession

### State today

Cultural institutions combine membership, facilities, treasuries, operating
[capacity](../src/institution_capacity.rs) and
[service duties](../src/institution_services.rs). Local
[offices](../src/offices.rs), institutional succession and political
[leadership](../src/leadership.rs) have different selection and tenure mechanisms.
Presence, eligibility, vacancy, duties and continuity recur across these systems.

### Possible generalized model

An organization could hold memberships, assets and roles. A role would specify
authority, qualifications, obligations, location requirements, tenure and its
selection/vacancy rules. Appointment, election and hereditary selection would be
separate policies operating on that common role lifecycle.

Organization-specific behavior would remain responsible for its decisions and
services. Holding an office would not automatically grant ownership of assets,
qualification for every task or personal liability for organizational obligations.

### Why explore it, and a small first comparison

The model could support new organization types with less duplicated vacancy and
succession handling. Start by comparing vacancy and holder-presence records for
one local office and one institutional role. Preserve their current selection,
funding, authority and action cadence.

Test death, sustained absence, relocation, replacement and unfinished duties.
A replacement must not inherit completed work as if they performed it. This is
broader than an identity refactor: merging whole organizations too early could
erase useful distinctions between a household, guild, temple and government.

## 6. Information, observations and memory

### State today

[Route warnings](../src/route_warnings.rs) describe encountered closures.
[Witnessed relief appeals](witnessed-relief.md) transmit dated hardship through
actual arrivals. [Social indicators](social-indicators.md) retain pressure memory,
while learning, trade and credit consume other forms of evidence. Freshness,
visibility and response rules are implemented in their respective systems.

### Possible generalized model

An observation could identify its subject, content, source, observed month,
received month, audience, provenance and uncertainty. Common operations could
deliver, deduplicate, retain and age evidence. Decision systems would query what
the actor knows at their decision boundary rather than the current global state.

Keep facts, testimony, beliefs and responses distinct. A remembered hunger pressure
is derived state, not the original hunger observation; a closure report can become
stale without proving that the route has reopened. Confidence and decay rules
would remain appropriate to the evidence type.

### Why explore it, and a small first comparison

Consistent information boundaries could prevent distant omniscience and retrospective
decisions, while making behavior easier to explain. Start with the shared envelope
for route warnings and witnessed appeals, preserving their existing expiry and
delivery rules. Check that reports cannot influence departures that preceded receipt.

Test stale information, repeated delivery, contradictory later observations and
missing provenance. Use bounded local records or indexes; a universal event scan
every month would be a poor trade for cleaner types. Do not replace the historical
event record with an actor's beliefs, or fabricate old knowledge when loading saves.

## 7. General credit contracts

This section incorporates the full general credit contract proposal. No credit
rules, feature defaults or existing contracts change through this document. The
possible increments describe evaluation options only, not planned work.

### Proposed simplification

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

### Current credit foundations

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

### Contract, evidence and permission

#### A small common contract

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

#### Evidence is not a compulsory contract type

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

#### Permission is a separate filter

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

### Bounded agent negotiation

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

### Settlement and security semantics

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

### Integration and possible increments

The [monthly schedule](monthly-schedule.md) remains authoritative. The inspected
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

### Shared mechanisms around contracts

These are additional proposals, not planned extensions or prerequisites for the
credit experiment. They connect credit to the broader opportunities above: party
authority overlaps [organizations and roles](#5-organizations-roles-and-succession),
asset identity supports [physical assets](#1-physical-assets-construction-and-maintenance),
and counterparty evidence builds on [information and memory](#6-information-observations-and-memory).
The shared mechanisms below could prevent separate rules for each party or activity.

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

### Evaluating general credit

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

## Relative value and adoption criteria

| Opportunity | Strongest reason to explore | Main risk |
| --- | --- | --- |
| Physical assets | Existing component model and recurring construction/repair lifecycle | Duplicate inventories or expensive conversion of aggregate GPU state |
| Information and memory | Existing witnessed evidence with clear timing contracts | Accidental omniscience or treating belief as fact |
| Journeys | Repeated movement, absence, delay and arrival accounting | Population/cargo duplication and changed travel semantics |
| Skills and learning | Existing partial sharing, reusable capability queries | Flattening distinct learning mechanisms or rewarding idle work |
| Needs and provision | Better explanations of unmet service and feasible responses | Implicit global allocation or incomparable services collapsed into a score |
| Organizations and roles | Repeated vacancy, eligibility and continuity handling | Erasing authority and selection differences |
| General credit contracts | Existing shared ledger with specialized request/evidence paths | Changing liabilities, settlement priority or permissions during structural refactoring |

Physical assets and information/memory are the strongest initial candidates for
discussion: both have concrete existing foundations and small extraction boundaries.
Journeys offer substantial reuse but carry more accounting and timing risk. Needs
and organizations offer broader behavior possibilities and warrant a narrower
two-system demonstration before choosing an abstraction. This ordering is a
recommendation for evaluation, not a work schedule. Credit has a concrete narrower
entry point: unify the existing cash-contract interface under compatibility behavior
before experimenting with negotiation, commodities or security. The broader
opportunities are useful connections, not prerequisites for that extraction.

For any candidate, a possible adoption process would be:

1. Identify two concrete consumers and list matching responsibilities, differences,
   authoritative state and monthly boundaries.
2. Introduce the smallest shared interface or receipt while retaining existing
   behavior. Reuse the [resolution framework](resolution-framework.md) where its
   dated comparison/commit semantics apply; it is not a universal transaction engine.
3. Verify unchanged outcomes under compatibility settings, including monthly,
   batched and checkpoint-resumed execution where timing or persistent state changes.
4. Remove duplicated responsibility only once both consumers use the shared path.
   Keep specialized equations and policies in their owning subsystems.
5. Evaluate new behavior separately with controlled cases and matched seed runs.
   Use the [run-health proposal](run-health-evaluation.md) to distinguish actual
   improvement from merely increased activity or newly missing observations.

The [monthly schedule](monthly-schedule.md) and explicit allocation boundaries
remain authoritative. Generalization would not justify changing execution order
to express a sharing preference, moving event creation to Close, or releasing work
backward into completed production. New archive fields must preserve old identities
and quantities without inventing past observations, skill or ownership.

An extraction would be worth keeping if it makes a second consumer easier to
understand, prevents a demonstrated inconsistency or enables a concrete experiment
with less duplicated code. If it mostly adds adapters while leaving every operation
special-cased, retaining separate components may be simpler. Generated comparisons,
logs and trajectories belong under ignored `output/`; committed evidence should be
source, tests and curated Markdown findings with their limitations.
