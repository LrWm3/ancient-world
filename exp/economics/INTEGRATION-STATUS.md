# Economics integration and planning interfaces

Current financial work extends [contract consolidation](CONTRACT-CONSOLIDATION.md).
Direct consented loans reuse the mortgage book with optional collateral, share Due
reservations/ranks with land claims, and compose with legacy exchange or bilateral
negotiation at Acquire. Land, forward and loan collections use one claim executor.
Optional proportional Due allocation includes accepted coin alternatives.

[Contract recovery](CONTRACT-RECOVERY.md) now adds original-loan guarantees and
recourse, authorized single-denomination loan estates, custody and funded asset
liquidation. This is implemented within the existing book and scheduler.
Land/forward admission now preserves native performance and blocks premature
closure. General discharge and town/minting/household/search acquisition adapters remain
unfinished; supporting one combination does not remove another driver's limits.


Implemented: a shared acquisition boundary for secured credit, its finite state
stock bid, and one bilateral negotiated exchange. Borrowing, sale-only and joint
production/sale forecasts also share need-constraint accounting. The experiment
still contains several separately tested pilots; this is not a universal economy.

## Financial reporting coverage

The opt-in [double-entry adapter](FINANCIAL-STATEMENTS.md) observes validated batches
and reconciles its journal to authoritative cash, asset, loan and estate positions.
It supports cash lending, valued mortgages, fixed enforcement/resale, guarantees
and loan estates, plus costed posted stock bids and bilateral negotiated/ZIP trades.
Opt-in owner-operated production adds material work-in-progress, joint-product
cost shares, consumption expense and aborted-work loss. Work ownership transfers,
household pooling, opt-in completed-output transfers and opt-in paid-capacity
capitalization are supported. [Preaccepted employment agreements](EMPLOYMENT.md) add capacity delivery, earned wage claims, partial payment and optional suspension. Negotiated hiring, household employment, wage insolvency and priced third-party contract production remain unsupported. Dated land dues now compose with lending and
material production on existing boundaries, including native goods and accepted
coin alternatives. Estate-paid native/accepted-coin dues now reconcile to restricted debtor cash and neutral custody positions; collection-linked issuance has an explicit opt-in convention. Storage blockage uses existing process failure;
there is no stored-goods spoilage event to recognize. The complete report set is not universal
transaction coverage. Execution and existing acquisition priority are unchanged.

## Shared acquisition boundary

`acquisition::evaluate` reads one immutable Acquire boundary and returns a dated
batch. The explicit allocation rule is **credit first, negotiated exchange
second**. Within credit, the existing purchase/resale/state-bid order is preserved.
This changes neither monthly phase order nor when installments fall due.

1. Credit emits its transactions and receipts from opening balances.
2. The shared resource view subtracts every outgoing leg from spendable balances.
   Incoming cash or goods cannot finance another leg in this batch. Storage tracks
   net reserved effects, so a confirmed outgoing stock transfer frees room.
3. Bilateral quote discovery uses opening permissions and pricing memory, with the
   remaining money, stock and storage limits. Optional need-generated orders use net
   reserved holdings; crossed quotes can still fail funding.
4. Settlement recomputes both components together and checks the exact combined
   transaction list before publishing balances, loans, title or ZIP memory.

A 2,000-tick downpayment uses all of a buyer's 2,000 opening ticks; an otherwise
acceptable 40-tick grain purchase fails. With 2,050 opening ticks both settle and
10 ticks remain. A land seller receiving 10,000 ticks cannot spend those proceeds
inside the same batch. Rejected exchange does not undo a valid financed purchase;
a forged batch or buffer overflow publishes neither component.

The resource view is deliberately small: outgoing balance reservations, net holdings and net
storage usage. It is not a universal resource auction. Credit still owns title,
collateral and loan rules; negotiation owns quotes and learning. Individual domain
previews do not authorize the combined batch. Use the shared resolver for that.

## Permissions

`Action::FinancedPurchase` controls the buyer's discovery and origination. An
existing citizenship agreement can grant this action through the ordinary
membership permission table. An unpermitted configured application records an
ineligible rejection. Revoking permission after acceptance does not erase debt or
prevent due settlement. Credit stock bids require `StockTrade` permission for
both parties; venue exchange additionally retains its configured participant type.

This does not automatically acquire citizenship before applying for a mortgage.
The combined fixture can start with accepted membership; prerequisite search and
acceptance remain their existing process/access pilot. Permission-governed
collateral resale remains rejected pending explicit buyer/seller rules.

## Supported combinations

| Combination | Current status |
| --- | --- |
| Scripted secured purchase + bilateral fixed/concession/ZIP exchange | Shared Acquire reservations and CPU controls |
| Finite state stock bid + bilateral exchange | Shared stock, money and net storage; state bid retains its posted price |
| Credit origination + existing citizenship/type permissions | Supported; due enforcement remains independent of permission |
| Borrowing/sale-only forecast + configured negotiation | Uses the same resolver in hypothetical branches; optional bounded consumption orders |
| Joint dated production plan + negotiation | Explicitly rejected; future work reservations need their own shared budget contract |
| Credit/negotiation + households | Still rejected; pooled purchase resources and loan support need explicit receipts |
| Legacy equipment/forward exchange, competing-access or pool-market drivers + credit/negotiation | Still rejected |
| Need-generated marketplace orders | [Bounded consumption/surplus policy](NEED-ORDERS.md) implemented; bilateral parties, lot and reservation prices remain supplied |
| Four-person monthly town book | [Implemented separately](TOWN-MARKET.md): locality, generated orders, multiple counterparties, fixed/ZIP quotes; credit and households remain excluded |
| Physical minting + scripted or generated dated stock/capacity orders | [Isolated CPU pilot](MINTING.md); excludes other acquisition drivers and collection-linked issuance |
| State posted bids learning ZIP prices | Not implemented; co-settlement does not change the price-setting policy |
| Direct loans + native/alternative-tender land dues | Shared Due collection; opt-in proportional policy with whole claim units and protected opening funds |
| Original-loan guarantees + servicing | Capped calls from remaining opening resources; same-book unsecured recourse, collectible at a later boundary; chains and pending-resale guarantees rejected |
| Authorized direct-loan estate + configured asset buyers | Single storage-free denomination, dedicated custody, funded sales and loan waterfall; collateral resale shares the asset-transfer helper |
| Estate + land/forward claims | [Admitted](LAND-FORWARD-ADMISSION.md): eligible land cash shares the waterfall; native performance retains its boundary; unresolved claims block closure |
| Estate + existing prepaid-delivery market | Servicing-only composition; retained accepted contracts, no new tool purchase, stock sellers or plot expansion |
| Estate + legacy mortgage driver or active market/negotiation | Rejected; broader acquisition adapters remain outstanding |
| Death/household dissolution + estate | Not integrated; configured arrears proceedings are not automatic lifecycle administration |

These exclusions are intentional validation boundaries, not claims that every
agent system can now be combined. Extend one boundary at a time with the same
opening-resource, forgery and continuation controls.

## Standardized planning contracts

`ForecastContext` remains the shared observation constructor. `forecast::needs`
now supplies common operations for borrowing, sale-only and joint planning:

- Validate nonnegative cumulative limits against positive catalog needs.
- Accumulate monthly deficits in integer units, separately for each provision.
- Check absolute limits and report the first violation in priority/resource order.
- Produce a stable priority-ordered deficit vector for policy-specific scoring.

The public policy configuration and decision receipts stay domain-specific.
Borrowing permits an empty limit map; sale and joint policies require limits.
Horizon bounds, candidate enumeration, payment checks, economic scores and fallback
rules remain with their policies. This preserves their different questions:

| Planner | Objective retained |
| --- | --- |
| Borrowing | Accept only a repayable, admissible improvement over declining |
| Sale-only | Choose the largest admissible current sale; otherwise sell zero |
| Joint work/sale | Compare needs, failures, buffer and net wealth across bounded work/sale alternatives; explicit no-sale scarcity fallback |

No common simulator loop or universal utility score was introduced. The rollouts
make different hypotheses, and merging those loops would hide their policy choices.
These limits constrain decisions; they do not add missing deprivation consequences
to fixtures that have no condition rules.

## Verification

[Explicit forward relief](DELIVERY-RELIEF.md) adds accepted overdue-date extensions
and quantity write-offs, separate from actual performance. Land-bill disposition
and autonomous renegotiation remain outstanding.

Latest forward-relief validation passed 105 distinct tests across 11 affected
suites in scoped runs; the final recovery/telemetry run passed all 36 tests.
See [completed validation](DELIVERY-RELIEF.md#completed-validation).

Earlier land/forward admission passed 99 tests across 11 affected suites, including all
20 recovery tests. See [the admission record](LAND-FORWARD-ADMISSION.md#completed-validation)
for tested behavior and the limits of this scoped rerun.

Earlier loan-estate recovery/consolidation evidence: 451 full-suite tests passed,
followed by 61 overlapping final focused tests; formatting, strict Clippy and artifact checks
passed. [The recovery record](CONTRACT-RECOVERY.md#earlier-loan-estate-validation) identifies
the final-edit coverage. These counts supersede the overview's earlier totals,
without claiming that every combination in the matrix is supported.

Run from `exp/economics` with Rust 1.92.0:

```sh
cargo +1.92.0 test --locked --test acquisition
cargo +1.92.0 test --locked --test forecast_needs --test borrowing --test sale_plan --test joint_plan
cargo +1.92.0 test --locked --test credit --test credit_offers --test stock_sale --test negotiation --test zip
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Historical acquisition-interface validation: the then-current full crate run
passed 290 tests. After the final common-offer dispatch refinement, all 35 focused acquisition, offer, need-accounting, venue,
membership and permission tests passed (including the additional eighth
acquisition test). All-target Clippy passed with warnings denied. Generated logs
remain under ignored `output/economics/`.

Acquisition controls cover cash contention, incoming-proceeds exclusion, state-bid
stock contention, net storage, both-party permissions, citizenship, continued debt
service after permission removal, forged component receipts, buffer exhaustion,
ZIP state, CPU/reference equality, monthly/batched/resumed execution and reordered
catalogs. Existing planner controls retain funded/scarce outcomes and repeated
harvests. These establish settlement compatibility, not economic calibration or
realistic emergent prices.

The other review work remains separate: deprivation consequences in the credit
fixtures, controlled planner ablations, uncertainty and competing sellers. The
broader architecture and speculative extensions remain proposals unless a linked
implementation report says otherwise.

## Production and market planning pilot

[Production-market planning](PRODUCTION-MARKET.md) adds an opt-in four-person
comparison of ordinary work, producer preferences, waiting and buying. Decisions
use bounded reference rollouts and preceding market observations; live clearing
reserves actual stocks, money and storage, and productive execution honors ongoing
work before new requests. Adaptive market sides use fixed supplied quotes. This
does not integrate household budgets, credit or legacy state trading into the town
book. The original fixed-side ZIP pilot remains available.

## Reciprocal-market extension

[Grain and wood exchange](RECIPROCAL-MARKET.md) adds a second listing with shared
opening money, stocks and storage, per-good observations and purchase choices,
and selectable market clearing priority. Historical unfilled bids can signal
possible future demand; actual and forecast settlement still require finite
resources. This remains a four-person fixed-quote experiment. Household budgets,
credit and adaptive ZIP are not integrated by this extension.

The directed complementary-work control sustains grain/wood exchange and keeps
all four people funded. The autonomous planner has not demonstrated that result;
its forecasts and monthly replanning can fail to provide anticipated supply.
Use the linked results to distinguish settlement support from emergent behavior.

## Static law constraints

[Named laws](LAWS.md) constrain the existing single-authority transaction policy.
Supported permission call sites share prohibition and required-membership checks;
legacy exchange/household combinations remain unsupported. This is not yet
jurisdictional law or organizational founding law. An opt-in recognition catalog
now permits or refuses new land-use leases and financed asset purchases; existing
agreements retain servicing and use-right semantics. Separate optional ceilings
now bound remaining lease duration and monthly loan interest at new entry.
These controls do not integrate lease-versus-purchase planning or negotiated terms.

## Household governance: first slice

The household fixture now uses static 20% contribution charters and a named member
governor. Authorized future policy instructions select net-output or
committed-work-preserving allocation within the founding constitution. Contribution
receipts, explicit ties, unused-hour return and settlement logs are implemented.
The legacy spare-labor option remains available. This does not remove existing
household/credit/town-market driver exclusions. See [Households](HOUSEHOLDS.md).


Equipment reporting now recognizes posted coin purchases at actual cost,
seller disposal gain/loss, and validated use-based wear as production cost.
See [financial statements](FINANCIAL-STATEMENTS.md#coin-equipment-acquisition-and-use-based-cost).
Prepaid-forward/tool bundles now recognize creditor prepayments and producer
deferred revenue, releasing those balances on actual delivery or accepted write-off.
Extensions preserve carrying value; spot and forward deliveries share opening
inventory costing. Equipment and posted commodity barter accept explicit reporting
values. Royalties and non-posted barter without payment valuation remain outside
the reporting adapter. See [forward recognition](FINANCIAL-STATEMENTS.md#prepaid-forwards-delivery-and-accepted-relief).


Physical minting reporting now reconciles funded stock/service purchases, actual
material consumption and authorized currency creation. The opt-in non-redeemable
convention adds issuer equity separately from income and separates self-created
money from external cash flows. Monthly paid capacity is expensed upon delivery;
future labor capitalization and redeemable currency need distinct policies.
See [recognition and verification](FINANCIAL-STATEMENTS.md#physical-minting-and-collection-linked-issuance).


The estate-dues reporting increment passed 58 focused tests and strict all-target
Clippy, including CPU/reference and checkpoint comparisons. Ranked and proportional
payments preserve their existing physical results; no change was made to estate
eligibility, allocation or closure. See [the recognition record](FINANCIAL-STATEMENTS.md#estate-paid-land-dues).


Owner-operated equipment manufacture now capitalizes material and productive wear
costs through WIP into the completed asset. Repairs expense their inputs without
revaluing restored capacity; idle monthly decay is depreciation. Joint durable/stock
outputs now use explicit typed cost shares.
See [equipment manufacture accounting](FINANCIAL-STATEMENTS.md#equipment-manufacture-repair-and-decay).


Financial journals now support versioned JSON save/load with validated balance
reconstruction and completed-period locks. The Audit only finalizes completed
simulation months; provisional reports remain available. This does not yet save
or restore the full simulation and accounting subledgers. See
[journal persistence](FINANCIAL-STATEMENTS.md#journal-persistence-and-completed-periods).


The state-derived credit balance-sheet implementation has been removed. Financial
reports now use only the double-entry Book/Audit pipeline, with explicit opening
valuations and validated events. Operational contract state, treasury balances and
settlement receipts remain available for simulation diagnostics. Scenarios awaiting
accounting adapters report those diagnostics without substituting snapshot equity.


Reporting coverage now includes town-market stock trades, title-following crop WIP
transfers, configured inventory expiration, explicit historical WIP at opening,
and equipment barter with supplied exchange values. Credit-stress scenarios now
produce journal reports through audited telemetry. See
[coverage expansion](FINANCIAL-STATEMENTS.md#coverage-expansion-and-remaining-adapters)
for the verified cases and remaining gaps.


### Household financial reporting

The double-entry adapter now observes household allocation, core execution and
collection in their actual order. Pooled stock retains carrying cost, cash support
has classified operating flows, and member dues retain the member's liability.
Unpaid shared labor remains nonfinancial. Authorized environmental pool inputs
carry their historical cost into production; regeneration adds zero-cost quantity.
See [financial statements](FINANCIAL-STATEMENTS.md#household-pooling-and-shared-resource-inputs)
for conventions and checks. This does not add dissolution, ownership consolidation,
paid-labor capitalization or unrestricted third-party production accounting.


The financial adapter now supports an explicit
[earned-only tool royalty policy](FINANCIAL-STATEMENTS.md#earned-only-tool-royalties).
It expenses supplier tool basis at delivery and values actual output shares as
noncash consideration when earned, with no forecast royalty debt or receivable.
Production cost is split between retained and delivered output, and both parties'
statements reconcile. This does not implement capitalization or valuation of
estimated contingent consideration, or establish full specialist-scenario coverage.


Completed output can now transfer to a distinct beneficiary under an explicit
[carrying-cost policy](FINANCIAL-STATEMENTS.md#completed-output-for-a-distinct-beneficiary).
This follows existing production rights and stock/durable settlement. Unfinished
costs and failed-work losses stay with the operator; recipient depreciation begins
only after ownership transfers. No planner, allocation rule or physical execution
phase changed. Priced production and three-party royalty consideration remain open.


[Negotiated and town-market barter](FINANCIAL-STATEMENTS.md#negotiated-and-town-market-barter)
now uses accepted payment-resource terms and actual settled quantities for financial
recognition, including ZIP-priced matches. Both sides receive costed inventory and
recognize noncash sales against their own opening basis. Quotes and unsuccessful
orders create no revenue; payment-stock valuation remains explicit.


[Actual-use paid-capacity accounting](FINANCIAL-STATEMENTS.md#paid-capacity-and-actual-use-capitalization)
now supports accepted period-service purchases in the minting acquisition driver,
including their use in ordinary stock or durable production. Used cost enters WIP,
output, or process expense/loss; unused cost expires at the existing monthly reset.
The default immediate-expense policy remains available. This is financial cost
recognition, not a new labor market or employment-contract implementation.

## Employment and wage arrears

[Employment agreements](EMPLOYMENT.md) now deliver available hours at Acquire and
collect earned wages at Close through shared financial primitives. Dated claims
persist after suspension or expiry and reconcile to both parties’ financial
statements. Paid-capacity costing accepts earned wages as well as cash purchases.
Settlement metrics/logs include the verified transfers and contract receipts.
Terms are preaccepted; negotiation, household delegation and wage estate priority
are still outstanding.

## Explicit financial reporting scope

Statements now carry an explicit separate-agent scope and exports label that scope.
Membership and ownership do not implicitly consolidate books. Household/member
claims remain visible. Explicit consolidated requests currently reject until an
elimination adapter exists; any future eliminations will affect only the report,
not obligations or per-agent books. See [reporting scope](FINANCIAL-STATEMENTS.md#explicit-reporting-scope).

## Household rotating governance

An opt-in rotating constitution now uses static charter terms and a founding
member as its starting governor. Deterministic stable-ID rotation and next-Open
succession are separate from labor tie-breaks and operational policy. Fixed-founder
governance remains the default and has no automatic succession. Accepted policy
instructions retain their issue month and historical authority; later governors
can supersede pending policies without deleting those records. Open receipts and
settlement logs expose current authority, term and policy, and replay validates
them. See [rotating governance](HOUSEHOLDS.md#rotating-governance-and-succession).

This changes no ownership, financial reporting scope or settlement obligations.
Autonomous voting, lawful institutional formation, longer-horizon collective planning and
household market/credit/employment integration remain outstanding.


### Household election governance

Added opt-in elected terms alongside fixed-founder and rotating governance.
Static charter parameters define turnout and tie resolution; accepted ballots
carry their issue month and target term. Existing Open household receipts expose
the electorate, tally and winner, and the settlement observer exports that evidence.
Historical results survive subsequent deaths; vacancies do not erase operating
policy. Election selection remains separate from labor allocation and separate
agent financial reporting.

This first slice uses supplied ballots and the fixed adult founding roster. It
does not implement endogenous voting, election work costs, by-elections, hereditary
succession, legal formation or household employment/credit integration. See
[household election semantics and validation](HOUSEHOLDS.md#elected-governance).


### Household needs-first allocation

An opt-in operational policy now compares actual same-month settlement previews
through consumption before comparing output value. It uses the existing contribution
pool, respects mandates and continuing commitments, and records baseline/projected
need deficits in replay-validated receipts and settlement observer logs. Default
policy and phase ordering are unchanged. Preview transactions remain private.

This covers current-month needs across existing member plans. Autonomous governance
choices, longer-horizon collective planning, multiple simultaneous recipient search,
and household employment/credit integration remain outstanding.
