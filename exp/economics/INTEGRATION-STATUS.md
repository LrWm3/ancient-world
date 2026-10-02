# Economics integration and planning interfaces

Verified economics v1 is candidate `a1b99fa`; subsequent opt-in planner coverage
is recorded separately in [the comparison](PLANNER-COMPARISON.md) and its
[calibration/market follow-up](PLANNER-CALIBRATION.md), plus the
[continuation-policy comparison](PLANNER-CONTINUATION.md) and
[independent multi-person admission](PLANNER-PERSONS.md) and
[active exchange comparison](PLANNER-EXCHANGE.md), followed by
[observed counterparty expectations](PLANNER-EXPECTATIONS.md).
[Release results](V1-RESULTS.md) record the completed bounded gates.
This page describes supported combinations and present exclusions. The
[v1 release checklist](V1-RELEASE.md) defines release requirements and exclusions;
the [consolidation roadmap](CONTRACT-CONSOLIDATION.md) orders the broader backlog.
[Fibonacci integration](FIBONACCI-INTEGRATION.md) and the linked subsystem reports
retain historical changes, scenario settings and verification results. Historical
test counts are snapshots, not additional tests to sum or universal coverage claims.

## Current scope

The initial [person–household economic loop](PERSON-HOUSEHOLD-LOOP.md) is complete
within its bounded adult/town-market scenario. The coordinated 120-month
CPU/reference comparison meets household food needs, preserves finite coins and
reconciles separate statements. Market-interruption controls produce actual
shortages and later recovery. Opportunities, consent and several planning limits
remain supplied; this is not general autonomous economic coordination.

Persons retain their own needs, holdings, rights and debts. Households have static
constitution/charter terms, governed contributed labor, resource/storage pooling,
collective trading, external hiring, member support and explicit wind-down.
Membership does not imply ownership or financial consolidation. Person
self-directed policy changes remain deferred. Autonomous state governance,
children, recruitment and automatic death estates remain extensions.

## Supported combinations

Support is specific to the listed adapter and fixture. A supported pair does not
prove that every combination of those systems works together.

| Combination | Implemented coverage | Boundary still present |
| --- | --- | --- |
| Direct lending + mortgages | One authoritative loan book, accrual, repayment, arrears and collateral terms | Direct advances use configured consent; general underwriting/discovery is absent |
| Credit + negotiated/ZIP or town exchange | Shared opening cash, goods and storage; legal/venue checks; forged-batch rejection | Each market retains its own quote and matching policy; state posted bids do not learn ZIP prices |
| Direct + tool-backed prepayments | Shared collection/admission resources and historical-cost reporting | Tool underwriting remains specialized; general negotiated forward formation is absent |
| Person prerequisite search + finance | Bounded citizenship/land search, competitive access, dated farming and explicit loan/prepayment/guarantee/estate-purchase bundles | Finance terms are supplied; the search is not universal |
| Household + town production/purchase planning | Member-directed or collective buying with ordinary coin loans, active-input funding and accepted work preserved in forecasts | Collective buying under cooperative discovery or posted cooperative agreements remains excluded |
| Household + joint work/sale planning | Dated shared-input, contributed-labor and output-pooling receipts; bounded comparison with up to four participants | Changes one participant's new work while peers keep ordinary decisions; no collective horizon optimizer |
| Household + common acceptance | Financial requests and explicit citizenship/land/process bundles prepare through ordinary household boundaries; member claims stay private while harvests pool | Mixed financial/productive search bundles remain excluded |
| Experimental offer composition | Beam/best-first discover bounded person or explicitly member-consented household packages; ordinary acceptance, accounting and land allocation remain authoritative | Current membership/land/process lots; separately, person need-order subsets with passive town counterparties. No household market/finance composition, general dated schedule optimization or default-policy replacement |
| Independent person composition + shared resources | Same-opening personal searches, equal-score alternatives, explicit whole-package admission, fallback work and separate review schedules; two/four-person repeated harvests and 32-person opening contention | One new plot per reviewed package; ordered admission, not optimal joint matching; forecasts do not predict competitors; trade is covered separately below; no households or finance in this adapter |
| Independent person composition + town exchange | Every person chooses work and order submissions; common clearing, actual-fill work checks, finite money, expected/actual receipts, CPU/restart/separate accounting controls | Monthly review only; preexisting rights, fixed supplied listings/quotes in the comparison; peer forecasts omit new production; optional recent-submission hypotheses can suppress trade and worsen survival. Beam and higher-cash controls fail |
| Experimental plan continuation | Isolated crop/household scopes compare monthly search, retain/repair and horizon reviews; fresh dated work, observation-triggered reconsideration, CPU/reference/accounting and cloned checkpoint checks | All household participants must consent. Exact full-context deviation detection; retained market plans, optimized local repair and durable controller serialization remain absent; monthly active exchange is covered separately above |
| Household + employment | External member wages, household/member employers, budgeted acceptance of useful posted labor, costed hour allocation, wage support and arrears | No internal household employment or general negotiated wage matching |
| Environmental collection + finance | Direct loans, forwards, household labor/hiring, native guarantees, financed purchases and coin recovery have mixed controls | Specialized mortgage stock-sale planning is outside this collection adapter |
| Physical minting + finance | Finite coin/input/hour reservations compose with loans, prepayments, dues, employment, households and authorized recovery | Issuance follows a configured policy, not an autonomous state objective |
| Due claims + creditor allocation | Ranked or opt-in proportional loan/land allocation, accepted coin tender, whole conversion lots and single-resource indivisible claims | Forward collection retains Acquire timing; joint multi-resource minima and further tender routes remain open |
| Guarantees + servicing/recovery | Native loan, wage, land and direct-delivery coverage; posted admission; selected coin alternatives; dated recourse | Autonomous acceptance and arbitrary tender/security combinations remain open |
| Guarantee chains + liens | Finite rooted chains, explicit authorized-liquidation lien/proceeds inheritance and no same-boundary recourse cascade | Cycles, broken inheritance, pending-resale guarantees and substitute secured tenders remain rejected |
| Estate + ongoing work/markets | Native performance, permitted cultivation/collection and essential behavior continue; stays block specified new contracts/trades | Admission is explicitly authorized, not automatic from shortage or death |
| Estate + liquidation | Funded property/crop, portable-equipment, inventory and whole-loan sales; actual proceeds and explicit secured/general waterfalls | Listings, valuations and bids remain supplied; no universal asset market |
| Estate + shared custody | Several proceedings can share a non-operating custodian with separate beneficial balances and opening budgets | Multiple custody currencies are not supported |
| Receivable assignment + guarantees/collateral | Compatible authorized-liquidation security and explicitly transferable guarantees follow the unchanged loan | No partial/onward assignment, fixed-value/resale security assignment or secured native-commodity assignment |
| Priced receivable assignment + accounting | Opt-in floors, highest-funded bids, discount/premium principal cost, subsequent interest, collection and relief; native unsecured claims retain their goods | Priced purchases require no unpaid interest at acquisition; no effective-yield amortization or market revaluation |
| Household wind-down + recovery | Separate member/household claims, explicit last-member wind-down, asset disposal, loss and residual distribution | Automatic death estates and compulsory operating-household proceedings are not implemented |

See [acquisition adapters](ACQUISITION-ADAPTERS.md),
[commodity finance](COMMODITY-FINANCE.md), [guaranteed claims](GUARANTEED-CLAIMS.md),
[estate receivables](ESTATE-RECEIVABLES.md), and
[production-funded credit](PRODUCTION-FUNDED-CREDIT.md) for individual controls.

## Shared acquisition boundary

`acquisition::evaluate` reads an immutable Acquire boundary and returns a dated
batch. Credit and recovery acquisitions, forward collection/admission, and the
selected exchange/mint driver retain their explicit ordering. That ordering is
not a universal fairness policy and does not move Due claims into Acquire.

All outgoing legs share opening spendable balances. Incoming cash or goods cannot
fund another outgoing leg in the same window. Confirmed outgoing stock can free
receiving storage, including the household contribution share. A financed purchase
may explicitly pay the seller directly as part of its package; that does not make
an unrelated incoming loan spendable again. Forecasts must use the same remaining
budget rather than restart from the untouched opening state.

Common financial preparation is read-only and evaluates ordinary settlement on a
candidate state. Acceptance revalidates dated terms and resources before atomic
publication. A failed requested package publishes no partial transfers, rights,
claims or work. An unfunded independent bid can leave an asset available for the
next eligible bid; that allocation rule is distinct from package atomicity.

Due servicing, Acquire delivery/sales, Productive work and later arrears/payroll
retain their visibility boundaries. Estate sale proceeds wait for a later Due;
new recourse cannot collect or cascade in its creation boundary. Forecast income
is never current spending power.

## Permissions

Existing type/membership grants, named prohibitions and recognized agreement forms
bound new acceptance. Loans check borrower/lender permissions and accepted term
limits; financed purchases and asset trades have their own permissions. Market
participation adds venue eligibility. Supported person search can acquire ordered
citizenship/land prerequisites, but a generic financial request does not invent
missing membership or consent.

Withdrawal of permission does not erase an accepted debt or stop ordinary servicing.
An authorized estate stay blocks specified new actions while preserving accepted
performance and permitted ongoing work. Household membership does not transfer
personal debt to the organization, and custody never authorizes ordinary trading.

## Household planning gaps

V1 includes the bounded common-acceptance adapter below. Conditional cooperative
credit, collective cooperative purchasing and general consequence-aware household
search are deferred; their absence does not block that release.

Explicit citizenship/land/process bundles now use the ordinary household
allocation, labor and collection envelope. Preparation is read-only; acceptance
previews the dated Productive boundary before publishing prerequisites. Scarce
land retains the existing allocation policy and fixed-priority household fallback.
[Acceptance verification](V1-ACCEPTANCE.md) covers repeated harvests, rejected
packages and continuation. The subsequent opt-in `composition` experiment can
now discover such packages under an explicit member-consent mandate. Its
[comparison](PLANNER-COMPARISON.md) exposes budget/horizon failures and excludes
financial/market drivers; it is not universal household planning. The current
legacy guards remain intentional:

- `acquisition::search_composition` excludes households.
- `households::validate` requires fixed individual priorities; it rejects
  `ConsequenceAware` with households.
- Household credit with `production_market::Policy::Cooperate` requires a
  conditional projection adapter.
- Collective household purchases under `Cooperate` or `Agreement` require a
  cooperative delivery adapter; member-directed purchasing is distinct.

Removing these guards is not integration. A mixed regression must first preserve
shared labor, storage, opening money, rights, dated commitments and separate books.

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

## Financial reporting coverage

The double-entry journal is the only financial reporting system. The strict
`financial_reporting::Audit` reconciles supported transactions against authoritative
state and rejects missing valuations or unsupported activity without publication.
Contract records remain execution data, not a competing financial report.

Supported adapters include lending/mortgages, costed stock trade and barter,
production/WIP and losses, equipment, dues, forwards, issuance, household pooling
and distributions, employment/arrears, guarantees, custody, liquidation and priced
claim assignment. Explicit `Separate` scopes retain member and household claims.
`Consolidated` requests still fail pending an elimination adapter. Journal JSON
round-trips preserve finalized reports; full durable Simulation/Audit restoration
is not implemented. See [financial statements](FINANCIAL-STATEMENTS.md) for policies
and outstanding valuation/coverage work.

## Verification

| Recorded boundary | Result | Scope |
| --- | --- | --- |
| V1 candidate `a1b99fa` | 1,033 passed, zero failed, one ignored; ignored population test passed explicitly | Six release families, full suite, strict Clippy, format and artifact gates; [results](V1-RESULTS.md) |
| Full crate snapshot `30870e5`, batch 89 through item 84 | 1,023 passed, zero failed, one ignored | 123 Cargo result targets, including empty unit/doc targets |
| Items 85–88 | Separate affected-suite gates passed | Tests/documentation added after that snapshot; details in the batch record |
| Final item 89, committed in `30c74e6` | 54 passed; strict all-target Clippy, formatting and artifact checks passed | Accounting, estate/native/mortgage receivables and recovery search |

These are overlapping runs, not counts to add together. The v1 candidate has a
complete full-suite and explicit annual population run. CUDA and other GPU backends
remain unverified; CPU/reference equality is the tested execution boundary.

Run from `exp/economics`:

```sh
cargo +1.92.0 test --locked
cargo +1.92.0 fmt --check
cargo +1.92.0 clippy --locked --all-targets -- -D warnings
```

Targeted changes should run their affected integration suites. Preserve atomic
failure, actual versus forecast performance, CPU/reference and relevant checkpoint,
monthly/batched and catalog-order controls. Raw outputs belong under ignored
`output/economics/`. Balanced books and deterministic settlement establish neither
economic calibration nor sustainable autonomous behavior.
