# Consolidate contracts before adding more financial scenarios

Latest integration: [common acquisition adapters](ACQUISITION-ADAPTERS.md) now
prepare dated loans, prepayments, guarantees, household hiring and estate bids
through ordinary settlement. Direct and tool-backed forwards share collection,
funding and storage; mortgages compose with prepayments and negotiated exchange. Fixed-reserve and
bounded-forecast mortgage stock sales now share direct-delivery acquisition,
including household contribution storage. A household now carries
its shared-input, labor and output-pooling receipts in a dated joint production
plan; configured negotiated seed purchases also feed joint work. Bounded joint
forecasts preserve other participants' ordinary decisions and inspect their need
deficits separately, with a two-member farming/warmth integration control.
Town production/purchase planning now also carries ordinary coin loans, debt and
arrears forecasts, including households whose charter delegates buying to members.
Its rollouts preserve the complete opening acquisition budget; configured cooperative deliveries also share that budget. Collective town
buying and household conditional credit forecasts still require adapters.
Household mortgages now connect repayment to solvent disposal and dissolution.
Independent leases (including fixed-reserve and bounded-forecast stock income)
and physical mint procurement compose with financed purchases;
[portable equipment](EQUIPMENT-LIQUIDATION.md) uses funded estate liquidation.
[Estate receivables](ESTATE-RECEIVABLES.md) preserve recoverable assets and newly
collected cash before deficient closure. These are tested combinations; broader
admission, planning, custody and liquidation work remains.

This is active implementation work. The next financial work should extend the
existing contract, claim and settlement model, rather than introduce another
isolated economy or a second authoritative debt ledger.

## Target execution model

```text
available offer -> consent and legal/physical feasibility -> accepted terms
    -> dated obligations and grants
    -> collect competing requests at an existing boundary
    -> rank, reserve and validate actual performance
    -> transaction effects + authoritative contract receipt changes
    -> atomic gather/commit
    -> shortfalls and the agreed consequences
```

One execution model does not require every contract to be a loan. Citizenship
creates membership; a cultivation agreement consumes inputs and services and
produces outputs; a loan transfers principal and creates repayment claims.
These remain typed terms/adapters. They must share identity, inspection,
reservation, validated effects and commit, with no independent copy of balances.
Production and minting explicitly transform/create resources; they must not be
misrepresented as conserved transfers between fictional counterparties.

## Implemented consolidation

- General advances and financed asset purchases use **the same loan records** in
  `State.credit`, monthly accrual, scheduled repayment, arrears, collateral
  enforcement, balance-sheet views and CPU batch settlement.
- `World.lending` holds explicitly consented, dated advances between distinct
  agent IDs. Lender type is not hardcoded. Unsecured advances can use a stock
  commodity; collateral is optional. Secured direct advances use a storage-free denomination and explicit fixed-value,
  creditor-resale or authorized-estate-liquidation terms. Compatible shared liens
  now reserve actual proceeds under the last option.
- General loan acceptance checks the `Loan` legal form, borrower `Borrow`
  permission, creditor `Lend` permission, the interest ceiling, actual funding,
  collateral ownership/exclusivity and receiving storage. Permission withdrawal
  does not erase or stop servicing an already accepted loan.
- `finance::Execution` owns opening spendable resources and current storage for
  a reservation window. It executes partial dated claims or atomic exchange
  packages. Receipts cannot fund another outgoing leg within the same window;
  a failed package leaves its reservations unchanged.
- Loans, annual land payments and prepaid-forward deliveries now use that claim
  executor. Their authoritative balances remain in their existing records.
- Due loans and land claims share an execution budget. `World.claim_priorities`
  supplies explicit lower-first collection ranks; absent overrides use a loan's
  accepted priority or rank zero for land/forwards. Loan/land type and stable ID
  break equal-rank collection ties. Within each land agreement, older bills come
  first. Existing essential-stock protection applies to both loan and land
  collection. `World.collection_policy` defaults to `Stable`, preserving that
  ordering. Opt-in `Proportional` inventories loan and land dues before collection,
  including accepted coin alternatives, and shares scarce opening resources among
  equal-rank claims. Native-first tender allocation preserves whole claim units.
  Receipts retain rank, requested units, optional allocated units and actual
  payment. See [creditor allocation](CREDITOR-ALLOCATION.md) for scope and tests.
- Forward collections retain their Acquire boundary and use rank, due date and
  stable ID there. Ranking does not backdate a later claim into Due or change
  monthly scheduling.
- Direct advances compose with the existing bilateral and legacy stock/tool
  acquisition paths through the common opening-resource reservation window.
  A loan issued at Acquire cannot be spent again in that same batch; the
  existing financed-purchase package can pay the seller directly.
- Ownership-following rights can be declared independently of purchase offers
  through `World.ownership_rights`. Repossession changes control and future
  output, not crop progress, elapsed labor or already consumed inputs.
- `agreements::for_agent` includes membership, land, process, loan and forward
  and guarantee views, plus accepted cooperative exchange schedules/outcomes and household
  founding, membership, governance and lifecycle records.
  `View::parties` avoids assigning an artificial creditor role to reciprocal trade.
  Atomic exchange packages remain conditional, not independently collectible legs.
  `View::claims` exposes their recorded or callable claims without
  creating a second ledger. A collection claim is not a total balance sheet or
  a forecast of all future obligations.

- Shared alternative-tender execution records real coin legs while keeping dues
  and native-linked issuance in their proper claim units.
- Existing collateral resale and estate liquidation now share funded asset-sale
  settlement, including atomic title and attached-process transfer.
- Configured capped guarantees pay residual loan and dated wage/land claims and create zero-interest
  recourse in the same loan book. Authorized loan-estate proceedings add stays,
  interest freezing, custody, actual asset sales, ranked/proportional distributions,
  retained deficiencies or explicit per-loan discharge. See
  [contract recovery](CONTRACT-RECOVERY.md) for exact scope and verification.

## What is deliberately not claimed yet

Direct advances are configured consent, not an autonomous credit offer search or
underwriter. Direct lending now composes with productive pool collection.
Authorized coin recovery and posted household hiring also compose with productive
pool allocation. Direct lending also composes with citizenship/land consequence search and
dated productive work, including competing land applications. Authorized recovery
now composes with this bounded person search: existing cultivation/native rent
continue while new land agreements are stayed. Broader
search/market combinations still need adapters. Legacy plot expansion now
observes direct-loan liabilities; mortgage expansion remains unsupported. Unsupported direct-loan combinations fail validation explicitly.
Mortgage-specific compatibility restrictions also remain until their ownership
and planning assumptions are migrated.

Explicit productive bundles now include dated posted guarantee admission and
estate inventory purchases alongside seed credit and land prerequisites. Financed
property without the specialized stock-sale planner also supports explicit
purchase/planting bundles and bounded process search. Existing caps, call timing and recourse apply
after failed work; selection of those financial offers remains supplied consent.

The shared claim executor is a substantive consolidation, but **not yet a
universal contract interpreter**. Domain code still materializes dates, chooses
terms, supplies accepted alternative-tender rates and applies its own consequence. The
annual arrears retry, forward collection and loan servicing retain their existing
visibility boundaries. Autonomous formation/negotiation of every arrangement is
not established by financial settlement tests.

## Current verification and next integration target

The preceding loan-estate full crate run passed 451 tests; the final affected
suites passed 61 overlapping tests after inspection, checkpoint and receipt refinements.
See [contract recovery](CONTRACT-RECOVERY.md#earlier-loan-estate-validation) for the evidence
and boundaries. Older totals below describe earlier consolidation snapshots.

[Land/forward admission](LAND-FORWARD-ADMISSION.md) is now implemented: native
performance retains its denomination, timing and receipts; eligible land cash
shares estate allocation; incomplete non-loan claims block closure. [Explicit forward relief](DELIVERY-RELIEF.md) now supports accepted date extensions
and quantity write-offs. [Land and wage relief](CLAIM-RELIEF.md) now share accepted dated extensions and
write-offs. Conversion, damages and autonomous renegotiation remain outstanding. Broader
acquisition adapters remain necessary before the mixed continuing scenario below
can establish that all these arrangements compose.

## Ordered remaining work

1. **Finish execution and acceptance adapters.** Route the remaining market,
   institutional and process arrangements through common acceptance and dated
   performance interfaces. Keep domain-specific terms, remove exclusive-driver
   branches only when a mixed regression proves shared funding, storage, rights,
   labor and settlement. Include a person holding land, owing a loan/forward and
   trading to meet needs in one continuing scenario. Do this before declaring
   the model unified.
2. **Extend creditor allocation coverage.** Equal-rank proportional allocation
   now covers native and accepted coin-tender loan/land claims at Due, including
   whole conversion lots. Standalone land/forwards and indivisible single-resource
   claims now have adapters. Extend additional tender routes and joint
   minimum-useful/multi-resource rules. Distinguish claim
   priority from collateral lien priority. Inventory all claims, protect only explicitly exempt resources, and preserve claims in
   their denomination unless an actual conversion transaction occurs. Compare
   policies against identical opening requests and budgets.
3. **Broaden insolvency admission.** The authorized single-custody-denomination estate
   lifecycle now distinguishes arrears from a proceeding and admits land/forward
   performance claims. Accepted land/wage/forward disposition now exists, plus explicit full unsecured
   loan write-offs in the native denomination. Several
   estates may now share a non-operating custodian with separate beneficial cash
   and opening spending limits. Add further custody-denomination/lifecycle combinations and market compositions before describing it as general insolvency. Record who initiates it,
   the accepted/legal trigger, acceleration, any collection stay, control of
   assets and work, and permitted ongoing essential activity. Being short of
   cash must not silently delete debts or declare every agent insolvent.
4. **Extend contingent guarantees.** Configured loan and dated wage/land guarantees now record
   consent, cap, trigger, term and dated recourse, with explicit stable/proportional
   allocation. Physical and direct-delivery claims now have native adapters and statements.
   Posted guarantee discovery and dated consented admission now exist. Extend
   autonomous acceptance and further alternative tenders. Explicitly agreed loan, earned-wage and prepaid-delivery
   coin payments now preserve native debt/recourse with whole conversion lots and
   settlement gain/loss accounting. Forward substitute performance is separate from
   actual goods delivery and releases historical prepaid cost. Same-boundary
   relief preserves guarantee payments and checks updated outstanding units. Land guarantees now
   select original accepted coin terms with whole-unit allocation and native recourse. Explicit authorized-liquidation
   lien subrogation now preserves both active pledges and already reserved proceeds;
   broader security/denomination combinations remain open. A successful
   guarantee payment reduces the original creditor's claim and creates the guarantor's corresponding recourse
   claim; it must not pay the creditor twice. Reserve guarantor resources across
   multiple calls using the same allocation window. Unsecured guarantee chains now use finite rooted terms and dated exposure;
   current-month additions cannot cascade. Cycles and secured chains remain
   excluded, with no recursive collection.
5. **Broaden actual liquidation.** Configured asset lists and funded bids now
   transfer permitted title/attached responsibilities and distribute actual
   proceeds through the loan waterfall. Eligible listing discovery and funded bid preparation now exist. Add autonomous
   listing/valuation, further asset kinds beyond portable equipment and general claims.
   [Compatible competing liens](LIEN-PRIORITY.md) now reserve actual per-asset
   proceeds under explicit authorized-liquidation terms; cross-currency priority
   and broader security assignment remain open. Explicit same-denomination
   guarantee subrogation now follows that lifecycle. [Receivable collection](ESTATE-RECEIVABLES.md)
   now blocks deficient closure until existing assets are performed or disposed of;
   [inventory liquidation](INVENTORY-LIQUIDATION.md) now sells configured stock lots
   into the same custody and waterfall. Whole-loan assignment now transfers direct or mortgage coin claims with compatible security and
   transferable guarantees, or unsecured commodity claims with explicit fixed
   coin quotes, through funded estate bids; broader receivable pricing and
   security/denomination combinations remain open. Unsold assets
   remain unsold; appraisals do not create coins. Retain surplus, deficiencies,
   explicit discharge/write-offs and final receipts. Mortgages can now explicitly select the authorized-liquidation lifecycle,
   including crop-control transfer, custody and actual proceeds. Legacy fixed-value
   and creditor-resale choices remain distinct. Fixed-reserve, bounded-forecast and single-participant joint mortgage stock
sales now respect counterparty recovery stays. Household labor now accompanies
dated joint work; collective horizon optimization and broader market composition
still need integration.

Every step above extends the same book, claim executor and committed ledger.
There should not be separate guarantees/insolvency/liquidation scenario engines.

## Verification gates

- Every debit has its authorized counter-leg, transformation or issuance reason.
- Borrower and creditor exposures derive from the same authoritative claim.
- Failed acceptance and tampered/replayed batches publish no partial money,
  debt, ownership or process-control changes.
- Shared budgets prevent duplicate spending across contracts and counterparties;
  receiving storage is a real limit even on repayment.
- Existing mortgages, crop transfers, forwards, land collection and exchanges
  retain their tested consequences. Priority changes have explicit receipts.
- CPU and reference execution, month-at-a-time and batched execution, checkpoint
  continuation and reordered input catalogs agree where policy says they should.
- Recovery tests distinguish allocated amounts, actual payments, unsold assets,
  remaining claims and losses. Conservation alone is not economic viability.

Focused implementation checks are in `tests/lending.rs`, alongside the existing
credit, forward, commitments, agreement-view and acquisition regressions. Raw run
output belongs under ignored `output/economics/`; repository summaries should
report only checks actually completed.

### Earlier consolidation verification, 2026-09-23

The crate-wide `cargo +1.92.0 test --locked` run completed with 432 passing tests
and no failures. Final focused reruns covered the changed acquisition, loan,
collateral, forward, land, legal and observer paths (75 passing tests), followed
by the final essential-stock change (12 lending and four payment-policy tests).
These counts overlap and must not be summed as distinct tests.

The 12 lending checks exercise unsecured and secured advances, commodity storage,
shared lender funds, rejection without partial publication, legal permissions,
priority changes against identical resources, loans competing with annual land
claims, coexistence with stock exchange, essential-stock protection, optional
collateral inspection, crop-control transfer and CPU/checkpoint consistency.
All use existing `Simulation` and settlement paths; no new standalone financial
runtime was introduced. Formatting, Clippy across all targets with warnings denied,
and the repository artifact-policy check passed.

This verifies the supported composition and accounting boundaries. It does not
establish economic calibration, general loan demand/underwriting, equal-rank
fairness, or a universal insolvency/guarantee/liquidation lifecycle. Subsequent bounded
recovery work and its additional checks are documented in [Contract recovery](CONTRACT-RECOVERY.md).

## Direct household prepaid deliveries

[Pass 30](HOUSEHOLD-FORWARDS.md) adds an explicit bilateral admission adapter for
prepaid commodity deliveries, independent of tool purchases. It writes the existing
forward book and uses its common claim inspection, stock delivery, arrears and
accounting. Household orders/support consume a shared current-forward claim view.
Credit, direct forward settlement/admission and town trades reserve one opening
budget; prospective delivery space respects household shared storage.

Direct and tool-underwritten configurations now share collection and admission
resources. Direct prepayments also compose with bounded citizenship/land consequence
search and competitive access; the legacy tool-market search remains separate.
Direct-forward recovery now uses the existing proceeding and delivery-relief adapters,
including town-market stays; see [Fibonacci integration](FIBONACCI-INTEGRATION.md). Consent and pricing are supplied terms, not autonomous discovery
or underwriting.


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

Configured cooperative deliveries now share direct-loan acquisition reservations
and retain cancellation after failed performance; see [cooperative lending](COOPERATION.md#configured-agreements-with-ordinary-lending).
Individual cooperative discovery now checks cash less accepted debt and projected arrears.
Household conditional credit forecasts remain excluded pending their governance adapter.
