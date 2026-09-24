# Consolidate contracts before adding more financial scenarios

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
  commodity; collateral is optional. Secured direct advances currently use a
  storage-free denomination and the existing fixed-value repossession rule.
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
  ordering. Opt-in `Proportional` inventories native loan and land dues before
  collection and shares scarce opening resources among equal-rank claims.
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
  views. `View::claims` exposes their recorded collection claims without
  creating a second ledger. A collection claim is not a total balance sheet or
  a forecast of all future obligations.

## What is deliberately not claimed yet

Direct advances are configured consent, not an autonomous credit offer search or
underwriter. Existing acquisition drivers do not all compose: town markets,
physical minting, household delegation, competing-access/pool allocation,
consequence-search acquisition and market-driven plot expansion still need
adapters. Unsupported direct-loan combinations fail validation explicitly.
Mortgage-specific compatibility restrictions also remain until their ownership
and planning assumptions are migrated.

The shared claim executor is a substantive consolidation, but **not yet a
universal contract interpreter**. Domain code still materializes dates, chooses
terms, translates alternative payments and applies its own consequence. The
annual arrears retry, forward collection and loan servicing retain their existing
visibility boundaries. Autonomous formation/negotiation of every arrangement is
not established by financial settlement tests.

## Ordered remaining work

1. **Finish execution and acceptance adapters.** Route the remaining market,
   institutional and process arrangements through common acceptance and dated
   performance interfaces. Keep domain-specific terms, remove exclusive-driver
   branches only when a mixed regression proves shared funding, storage, rights,
   labor and settlement. Include a person holding land, owing a loan/forward and
   trading to meet needs in one continuing scenario. Do this before declaring
   the model unified.
2. **Extend creditor allocation coverage.** Equal-rank proportional allocation
   now covers divisible native loan and land claims at Due. Extend it to the
   other collection adapters, alternative-denomination payments and explicit
   minimum-useful/indivisible rules. Distinguish claim priority from collateral
   lien priority. Inventory all
   claims, protect only explicitly exempt resources, and preserve claims in
   their denomination unless an actual conversion transaction occurs. Compare
   policies against identical opening requests and budgets.
3. **Insolvency as a lifecycle.** Distinguish an overdue installment, temporary
   illiquidity and an authorized insolvency proceeding. Record who initiates it,
   the accepted/legal trigger, acceleration, any collection stay, control of
   assets and work, and permitted ongoing essential activity. Being short of
   cash must not silently delete debts or declare every agent insolvent.
4. **Guarantees as contingent claims.** Record guarantor consent, covered claim,
   cap, trigger, term and recourse. A successful guarantee payment reduces the
   original creditor's claim and creates the guarantor's corresponding recourse
   claim; it must not pay the creditor twice. Reserve guarantor resources across
   multiple calls using the same allocation window. Cycles and chains need
   bounded execution and dated visibility, not recursive unbounded collection.
5. **Liquidation through actual exchanges.** Discover/list realizable assets,
   accept funded buyers, transfer permitted title/attached responsibilities,
   and distribute actual proceeds by the accepted/legal waterfall. Unsold assets
   remain unsold; appraisals do not create coins. Retain surplus, deficiencies,
   explicit discharge/write-offs and final receipts. The existing fixed-value
   repossession and realized-proceeds sale become adapters to this lifecycle.

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

### Verification result, 2026-09-23

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
fairness, or the unimplemented insolvency/guarantee/liquidation lifecycle.
