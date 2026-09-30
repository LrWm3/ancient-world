# Economics integration and planning interfaces

Latest consolidation: [physical minting and finance](MINT-FINANCE.md) now connects
direct loans, prepaid deliveries, annual dues, employment, household pooling and
authorized recovery through the shared acquisition budget. Its mixed scenario
retains unpaid claims and compares provision policies with reconciled statements.
This supersedes earlier blanket exclusions for those combinations; other driver
and admission limits remain explicit.


Current financial work extends [contract consolidation](CONTRACT-CONSOLIDATION.md).
Direct consented loans reuse the mortgage book with optional collateral, share Due
reservations/ranks with land claims, and compose with legacy exchange or bilateral
negotiation or a local town book at Acquire. Land, forward and loan collections use one claim executor.
Optional proportional Due allocation includes accepted coin alternatives.

[Contract recovery](CONTRACT-RECOVERY.md) now adds original-loan guarantees and
recourse, authorized single-denomination loan estates, custody and funded asset
liquidation. This is implemented within the existing book and scheduler.
Land/forward admission now preserves native performance and blocks premature
closure. General discharge and recovery with search acquisition remain
unfinished; town and mint recovery now have bounded adapters, and supporting one combination does not remove another driver's limits.


Implemented: a shared acquisition boundary for secured credit, its finite state
stock bid, and one bilateral negotiated exchange. Borrowing, sale-only and joint
production/sale forecasts also share need-constraint accounting. The experiment
still contains several separately tested pilots; this is not a universal economy.

## Latest household consolidation — 2026-09-29

The [five alternating passes](INTEGRATION-PASSES.md) supersede earlier blanket
employment/town-lending exclusions in the historical progress entries below.
Members can earn external wages and sell protected private surplus while their
household buys for their needs. Earned wage arrears join stock protection;
actual wage and town sale receipts pool once, with fractional carry. Direct loans
and town settlement share outgoing opening budgets. Static cash targets bound
income work and support. Need-first support also works without a market.

[Five further passes](INTEGRATION-PASSES-2.md) add useful partial support,
physical barter with exact pooled-storage reservations, and static charter routing
between collective and private consumption buyers. Current membership controls
buy eligibility, including adaptive orders; pre-registered former members can buy
privately and accession restores their charter's route. Opt-in collective buying
also covers missing inputs of active member processes. Speculative work targets
do not qualify. [The third batch](INTEGRATION-PASSES-3.md) adds physical wage
storage and native-claim accounting, opt-in member loan assistance, collective
market demand for current loan payments and explicit assistance priority.
Budgeted household employers and costed direction to member work now compose in
[the fourth batch](INTEGRATION-PASSES-4.md). [The fifth batch](INTEGRATION-PASSES-5.md)
also admits outside member employment, earned-wage assistance and claim-funded
collective purchases. Internal household employment remains excluded.
[Pass 26](PAYROLL-OUTLOOK.md) adds opt-in current-month payroll estimates for
collective funding; future wages remain outside authoritative claims and accounts.
Direct town lending
does not enable mortgage purchase configuration, recovery proceedings or joint
production planners. Adult accession/exit changes contribution and consumption
scope without rewriting accepted wages or personal debt. Personal self-directed
policy changes are explicitly deferred.

## Financial reporting coverage

The opt-in [double-entry adapter](FINANCIAL-STATEMENTS.md) observes validated batches
and reconciles its journal to authoritative cash, asset, loan and estate positions.
It supports cash lending, valued mortgages, fixed enforcement/resale, guarantees
and loan estates, plus costed posted stock bids and bilateral negotiated/ZIP trades.
Opt-in owner-operated production adds material work-in-progress, joint-product
cost shares, consumption expense and aborted-work loss. Work ownership transfers,
household pooling, opt-in completed-output transfers and opt-in paid-capacity
capitalization are supported. [Preaccepted employment agreements](EMPLOYMENT.md) add capacity delivery, earned wage claims, partial payment and optional suspension. Outside member employment now composes with household labor contributions and paid-wage pooling. Internal household employment, negotiated hiring, wage insolvency and priced third-party contract production remain unsupported. Dated land dues now compose with lending and
material production on existing boundaries, including native goods and accepted
coin alternatives. Estate-paid native/accepted-coin dues now reconcile to restricted debtor cash and neutral custody positions; collection-linked issuance has an explicit opt-in convention. Storage blockage uses existing process failure;
there is no stored-goods spoilage event to recognize. The complete report set is not universal
transaction coverage. Execution and existing acquisition priority are unchanged.

## Shared acquisition boundary

`acquisition::evaluate` reads one immutable Acquire boundary and returns a dated
batch. The explicit allocation rule is **credit first, negotiated exchange
or town exchange second**. Within credit, the existing purchase/resale/state-bid order is preserved.
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
| General loans/recovery + households | Shared servicing and separate statements; explicit last-member wind-down before household recovery; member loans are not eliminated |
| Mortgage purchase/negotiation + households | Dedicated purchase driver remains excluded; negotiated collective purchase budgets need explicit receipts |
| Legacy equipment/forward exchange, competing-access or pool-market drivers + credit/negotiation | Still rejected |
| Need-generated marketplace orders | [Bounded consumption/surplus policy](NEED-ORDERS.md) implemented; bilateral parties, lot and reservation prices remain supplied |
| Monthly town book + household accounts | [Bounded collective adapter](HOUSEHOLD-MARKET.md): locality, governor policy, real member demand, shared money/stock/storage and fixed/ZIP quotes; private sales/barter, charter-delegated buying, active-process input funding, paid outside wages and direct loans now compose; budgeted external hiring and costed member allocation now compose; posted labor acceptance and authorized direct-loan/forward recovery also compose; joint production-market planning and mortgage drivers remain excluded |
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
Terms are preaccepted; negotiation and wage estate priority remain outstanding.
Budgeted household hiring and costed member delegation are now supported.

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
Autonomous voting, broader institutional formation, longer-horizon collective planning and
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
succession or household employment/credit integration. Household legal founding
has since been added; see
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


### Integrated household governance basics

Household founding now checks agreement recognition, each founder's action grants
and legal requirements, plus ceilings on constitutional choices and charter terms.
Historical admission receipts preserve the founding law without retroactively
voiding existing households. Current process law still governs delegated work.
Governors can atomically schedule an operating objective and labor tie-break;
constitution and charter remain static.

The six-month lawful election fixture runs through policy activation, pooled
production/consumption and finalized separate financial statements, with identical
CPU/reference and resumed results. The executable example and completion evidence
are in [Household governance basics](HOUSEHOLD-BASICS.md). Autonomous politics,
longer-horizon optimization, market recruitment/exit settlements and household employment/borrowing
remain separate extensions.


Verification on 2026-09-28: 75 focused checks plus the explicitly run 13-month,
32-person/eight-household CPU accounting test passed. The latter covers annual dues
and final separate statements. Strict all-target Clippy and the executable small
CPU/reference scenario passed. The core governance checklist is complete within
its adult-only, supplied-political-choice scope; this is not completion of the
broader institutional or finance roadmap.


### Adult household accession and exit

[Membership changes](HOUSEHOLD-MEMBERSHIP.md) now apply before Open work using
explicit consent, current legal admission and preserved founding limits. Dated
rosters drive labor, pooling, storage, elections and observer selection. Exit leaves
property and debts unchanged and rejects unsupported storage. Ordinary last-member
exit remains separate from the solvent dissolution path described below. Household-specific fractional carry cannot leak across moves. Open
receipts validate membership evidence; CPU/reference and audited separate-book
checks cover actual join/exit execution. Recruitment markets, negotiated exits,
estates and general institutional membership remain outstanding.


### Solvent household dissolution

The opt-in [dissolution path](HOUSEHOLD-DISSOLUTION.md) now separates wind-down,
verified residual-stock distribution and final membership release. Constitution
and static charter select permission and recipient; admitted legal limits persist.
The integrated CPU/reference case pays annual household land dues before releasing
surplus and closes with balanced separate statements. Explicit funded sales now
clear unencumbered catalog assets and usable portable equipment at Open, retain
proceeds until the next clearance
check, and recognize disposal gains/losses in separate statements. CPU/reference
and checkpoint cases agree. Explicit retirement now clears exhausted portable
equipment while retaining its permanent provenance. Explicit plot/equipment package
sales retain attachments and account for each component separately. The subsequent
[wind-down completion](HOUSEHOLD-WIND-DOWN.md) adds explicit title-following crop/right
transfers, salvage/write-off and general household loan/recovery composition.
Automatic death estates remain outside this slice.


Asset-disposal verification (2026-09-28): **137 focused tests passed**: household
dissolution 18, households 55, household accounting 6, credit 9, resale 7, recovery
26 and accounting 16. Strict all-target Clippy passed. The slow 32-person accounting
test remains ignored; the full crate suite was not run. Generated logs are under
ignored `output/economics/household-disposal-*.log`. Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked --test household_dissolution --test households --test household_accounting --test credit --test resale --test recovery --test accounting
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```


Portable equipment now uses that same disposal boundary and opening budget.
Ownership changes preserve condition and last-use history; buyer cost is established
before ordinary monthly depreciation. Existing offers/delivery agreements and
attached property prevent sale. The new checks also prevent independently disposing
of a plot carrying attached equipment. No scrap removal or agreement novation is
implied. See [equipment rules and verification](HOUSEHOLD-DISSOLUTION.md#portable-equipment).

Equipment-disposal verification (2026-09-28): **164 tests passed** across household,
equipment, manufacture, activities, recovery/resale and accounting suites, including
eight new focused checks. Strict all-target Clippy passed. The slow 32-person test
remains ignored and the full crate suite was not run.

Exhausted portable equipment now has an explicit, permission-checked retirement
instruction at Open. Verified receipts move the zero-use, zero-basis asset into a
permanent provenance archive; it cannot be repaired, sold or manufactured again
under the same ID. Clearance and residual payout still wait for the next Open.
The subsequent wind-down extension adds immediate material recovery, write-offs
and explicit title-following crop transfers. Original retirement verification (2026-09-28): **188 distinct tests passed**,
including seven new retirement tests and law controls; strict all-target Clippy
passed. One slow accounting test remains ignored; the full crate suite was not run.
See [verification details](HOUSEHOLD-DISSOLUTION.md#exhausted-equipment-retirement-verification-2026-09-28).

Explicit plot/equipment package sales now extend the same household disposal
boundary. All usable attachments must be named and owned by the seller; one funded
payment transfers the complete package. Supplied component allocations determine
buyer cost and seller gain/loss; ordinary buyer depreciation follows at Open.
The plot retains a positive allocated price under the current catalog registry.
With no explicit control-transfer terms, live rights/crops remain blocked. Mixed
ownership, exhausted attachments and pledged property still prevent package sale;
exhausted attachments can instead be explicitly decommissioned at a separate
boundary. No new use right is inferred.

Property-package verification (2026-09-28): **207 distinct tests passed** across 15
suites, including nine new package checks. After the zero-plot-value admission
check, all 42 package/disposal/retirement tests were rerun successfully. Strict
all-target Clippy passed. One slow accounting test remains ignored and the full
crate suite was not run. See [package rules](HOUSEHOLD-DISSOLUTION.md#explicit-plot-and-equipment-package-sales)
and the verification record in that document for settings and limitations.


Wind-down completion verification (2026-09-29): **252 distinct tests passed** across
20 household, property, lending/recovery, accounting, law and telemetry suites.
Strict all-target Clippy, formatting and artifact checks passed. One slow annual
32-person accounting check remained ignored; the full crate suite was not run.
See [completed behaviors, controls and limits](HOUSEHOLD-WIND-DOWN.md).


### Household town-market integration

Households can now register as collective town traders under explicit venue and
legal permission. `NeedsFirst` and `NeedsThenIncome` use member consumption projections for purchases;
other current objectives offer only protected surplus. Current private member stock
reduces collective demand without becoming collective funding. Each listing shares
real opening money, stock and storage; current governor authority is recorded and
replayed with order decisions. Existing resource pooling, productive labor and
separate double-entry statements compose with this adapter.

This supersedes earlier blanket household/town exclusions above. It does not remove
the joint production-market forecast planner or legacy exchange exclusions.
Later passes above add direct credit, member trader routing and active-process
input funding. Longer-horizon investment planning and autonomous hiring discovery still
need adapters. The CPU example runs short of money in month three after feeding
two adults for two months. See [settings, controls and limits](HOUSEHOLD-MARKET.md).

Verification (2026-09-29): **195 distinct tests passed** across 16 selected suites,
including the final 12-test household-market run, plus the CPU example and strict
all-target Clippy. One slow annual accounting check remained ignored; the full crate
suite was not run. Formatting and artifact checks passed.

### Household income-aware labor allocation

[`NeedsThenIncome`](HOUSEHOLD-INCOME.md) adds an opt-in secondary work objective:
after current needs and commitment protection, compare collective cash through
the next town book. Existing work candidates, contribution receipts, market budgets,
settlement and financial reporting are reused. Expected sales remain hypotheses,
and no projected income becomes current spending power.

The reciprocal grain/fuel fixture keeps two household members fed for 12 months
with 60 closing coin ticks after the first purchase. A 36-month CPU check exposes
the original boundary: a private fuel target stops work in month 25; food deficits
begin in month 27. That behavior remains a control; voluntary surplus support now
closes the coordination gap in the [completed initial loop](PERSON-HOUSEHOLD-LOOP.md).
The coordinated 120-month CPU/reference run meets household food needs, conserves
coins and reconciles separate statements.
This does not enable the joint production-market planner or autonomous hiring;
later passes above add direct town credit and bounded input funding. The linked
document records assumptions, controls and full results.

Verification (2026-09-29): **162 distinct tests passed** across 11 selected suites,
including all 13 final income-policy tests. The 12- and 36-month CPU examples,
strict all-target Clippy, formatting and artifact checks passed. One slow annual
accounting test remained ignored; the full crate suite was not run.


### Initial person–household loop complete

Voluntary member surplus support now connects private stock targets to collective
income without changing those targets or extending governor authority. Consent,
reserves, obligations, storage, demand and policy bound transfers; dated withdrawal
preserves historical replay. The 120-month CPU/reference comparison keeps both
household members fed, conserves coins and reconciles every separate statement.
Temporary market closure produces real shortages and subsequent recovery. See
[completed scope and subsequent work](PERSON-HOUSEHOLD-LOOP.md).

Verification (2026-09-29): **215 distinct tests passed** across 15 selected suites,
including the long audit and final support controls. The 36-month CPU example,
strict all-target Clippy, formatting and artifact checks passed. One slow annual
32-person test remained ignored; the full crate suite was not run.

## Fourth household/shared batch

[Passes 16–20](INTEGRATION-PASSES-4.md) support preaccepted outside household
hiring behind a static affordability budget, productive member allocation with
paid-cost transfer, shared earned-payroll reserves and collective purchases for
wage arrears. The fifth batch below adds member employers; internal hires and
wage estate treatment remain excluded. Timing is unchanged: Acquire delivers, Productive directs and
executes, Close pays, next Open expires unused hours.

The combined scenario runs an eight-month hiring/output/sale/payroll loop on
CPU/reference with separate books, reversed tables, ledger replay and checkpoint
continuation. A supplied month-three market interruption changes the worker's
self-production and later purchases; it does not establish automatic business
recovery or sustainable autonomous hiring. Person self-directed policy changes
remain deferred.

## Fifth household/shared batch

[Passes 21–25](INTEGRATION-PASSES-5.md) remove the blanket member-employer
exclusion. Own-hour contributions exclude bought hours; historical cost survives
onward allocation. Physical payroll can connect two distinct households with exact
pooled storage reservations and separate claims. Static optional wage assistance
and claim-funded market orders support a member's payment without assuming debt.

Combined live employment/production/pooling/market/payroll tests expose the cost of
funding only earned wages: delivery pauses until arrears clear. A dated exit stops
assistance while the claim remains personal; rotating governance keeps the remaining
household operational. CPU/reference, replay and checkpoint accounting agree.
Preaccepted terms, supplied market limits and finite counterparties remain scenario
configuration. This adds no wage insolvency, internal household employment,
autonomous recruitment or person self-directed policy changes.

## Current payroll funding outlook

[Pass 26](PAYROLL-OUTLOOK.md) reuses employment delivery rules to estimate this
month's payroll at Acquire. Static charter policy can include it in collective
orders and market retention; actual delivery alone earns wages and authorizes Close
support. The six-month member-employer comparison produces six paid working months
with adequate counterparty coins, or five working months followed by suspension
when coins run out. Earned-only demand remains the default and produces four
working months in both controls. No scheduler, liability or hiring-admission rule
changes; estimates are not escrow or guaranteed resource reservations.

## Voluntary funding of household payments

[Pass 27](HOUSEHOLD-PAYMENT-SUPPORT.md) adds `accept_payment_support` (default
false). Existing signed surplus mandates can fund a household's own earned wages
and current loan dues when the existing need/income comparison finds no benefit.
The fallback accepts only the remaining native payment shortfall, preserving donor
reserves and claims and rechecking need effects. Successive donors share one gap.

Combined checks retain separate books, physical carrying costs and worker storage
limits. Productive donations can pay wages at Close; loan funds wait until the next
Due. No creditor ranking, escrow, debt assumption, future-employment funding or
consent inference is introduced.

## Land dues and voluntary funding

[Pass 28](HOUSEHOLD-LAND-SUPPORT.md) extends optional payment support to current
native land bills. Proportional collection and funding share the land claim reader;
ordinary collection shares its estate/terminal eligibility gate. Neither future
annual rent nor a coin alternative is counted as an additional current obligation.

The combined case funds four rent and six wages from ten donated grain. Land settles
at the existing after-Productive ClearArrears pass, wages at Close. Five donated
grain leaves five wages unpaid. Creditor storage can retain funded stock and arrears
without requesting another donation. Separate accounting, CPU/reference, checkpoint
and replay checks agree.

[Pass 29](HOUSEHOLD-LAND-FUNDING.md) now connects collective market orders to own
current land bills. Static native-first/accepted-alternative-first preference is
shared by funding, protection and collection; only one funding denomination is
counted per bill. Proportional collection shares currency with same-rank loans
before native fallback. Town markets admit accepted land agreements while retaining
land-offer search and mortgage/recovery exclusions. Autonomous land/hiring discovery
remains open.

## Direct prepaid deliveries and household funding

[Pass 30](HOUSEHOLD-FORWARDS.md) separates prepaid commodity admission from the
person/tool-underwriting path. Explicit terms create the same forward records,
claims, delivery receipts and financial positions. Both parties need stock-trade
permission; new terms require recognized `PrepaidDelivery` form, opening funds and
prospective storage, including shared household space. Accepted performance survives
later law changes. Unfilled deliveries block another advance by that seller.

Opt-in collective orders and signed surplus support target own current deliveries.
The shared Acquire resolver reserves credit first, direct delivery/admission next,
then spot trades. Purchases or Productive support cover delivery at next Acquire;
they do not reopen an earlier settlement. Combined rent/forward and credit/prepayment
checks reconcile separate statements and preserve cash/storage bounds.

Direct terms support plain, town and bilateral acquisition. Direct/tool admission
coexistence, autonomous underwriting and negotiated hiring remain open. Direct-forward
recovery and bounded labor-offer acceptance are covered by the follow-up below. This does not lift legacy mortgage, minting, search or joint-production
composition limits. Person self-directed policy changes remain deferred.

## Fibonacci follow-up: batches 1, 1, 2

[Fibonacci integration](FIBONACCI-INTEGRATION.md) closes the selected direct-forward
recovery and household hiring gaps. Direct deliveries retain partial claims through
authorized recovery, extensions and explicit write-offs. Town admission is rechecked
after Due opens a proceeding; normal trading is stayed while existing deliveries
continue. Custody remains outside ordinary contracts and trading.

Households now select quantities from posted, worker-consented labor terms using
their existing policy and member work allocator. Preaccepted jobs reserve first;
optional offers follow explicit rank/ID order and one finite budget. The preview
includes acquired stock pooling, collective input allocation and consented support.
The integrated cases exercise prepayment → later hiring → production → delivery,
and income-based hiring with actual payroll and finite buyer money. Wage negotiation,
wage insolvency and general labor matching remain extensions. Historical pass notes
above describe their original boundaries; this follow-up supersedes their blanket
statements that hiring is always preaccepted or town recovery is unavailable.


### Earned-wage recovery integration — Fibonacci batch 3

[Wage recovery](WAGE-RECOVERY.md) now admits earned claims, pauses new employer
work during an authorized proceeding, and allocates same-denomination estate cash
with loan/land creditors. Actual payment updates the original employment book and
separate worker/employer/custodian statements; member wages pool once on receipt.
Native physical wages retain their existing service path and block closure if
unpaid. This supersedes the earlier blanket wage-insolvency exclusion. Accepted wage/land relief now follows in [claim relief](CLAIM-RELIEF.md).
Coin guarantees for those claims now follow in [guaranteed claims](GUARANTEED-CLAIMS.md).
Automatic estates and general employment discovery remain open; person self-directed policy changes stay deferred.


### Accepted non-loan disposition — Fibonacci batch 5

[Claim relief](CLAIM-RELIEF.md) now supplies accepted write-offs and date extensions
for earned wages and individual annual land bills. The common terms/history adapter
preserves original identities, actual work, paid quantities, issuance and annual
billing. Funding, projections, estate allocation, reporting and closure read the
adjusted claim. A composed person/household wage-support, rent and prepaid-delivery
scenario compares accepted relief with an otherwise identical unresolved estate.
This is configured consent, not autonomous renegotiation or automatic insolvency.


### Guaranteed claims — Fibonacci batch 8

[Loan, wage and land guarantees](GUARANTEED-CLAIMS.md) now share typed claim
inspection, funded Due settlement, explicit call allocation and dated same-book
recourse. Actual member wage receipts pool once; employer, worker, guarantor and
household retain separate statements. A combined household/loan/wage/rent control
uses one scarce cash pool, and a reproduced estate timing bug is fixed for later
advances on an older recourse loan. Coin claim coverage is implemented; physical
and delivery coverage, lien transfer, autonomous underwriting and guarantee
formation/discovery remain outstanding. Person self-directed policy changes stay
deferred, and constitutions/charters remain static.
