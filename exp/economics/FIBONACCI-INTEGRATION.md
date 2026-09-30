# Fibonacci integration follow-up

Scope: finish bounded autonomous household hiring and direct prepaid-delivery
recovery, then demonstrate that they compose with the existing economic loop.
The batches contain logical changes, not a prescribed count of files or tests.
The completed sequence is **1, 1, 2, 3, 5, 8, 13, 21, 34, 55** (batch 89 in progress). Batch sizes count distinct
implementation and integration changes, not test cases. Person self-directed policy changes remain explicitly deferred.

## Batch 1 — one recovery change

Direct prepaid delivery contracts now enter the existing authorized recovery model.
A household can wind down with an outstanding delivery, extend its date, then apply
an explicitly accepted partial write-off. Remaining prepaid assets/deferred revenue
survive until delivery or relief; relief creates loss/income without fake goods.
New prepaid agreements reject either counterparty in active recovery. Custody agents
cannot become forward counterparties. Existing performance remains serviceable.

The test delivers two of four goods, extends the other two, blocks premature
household dissolution, then writes off only the residual and closes the case.
CPU/reference execution and checkpoint continuation agree. See
[household forwards](HOUSEHOLD-FORWARDS.md).

## Batch 2 — one hiring change

[Household labor offers](HOUSEHOLD-HIRING-OFFERS.md) reuse the employment terms
catalog, capacity transfer, earned wage claims and accounting. Supplied offers are
worker consent, not automatic household acceptance. The household uses its current
governor policy and work allocator to select useful hours within its static budget.
Existing jobs reserve first; offer ties use rank then ID. Earlier acceptances enter
later projections. Unaccepted offers create neither debt nor a dissolution blocker.

The controls distinguish useful work from absent work, insufficient grants, own-labor
sufficiency, prohibited work and excessive wage cost. Buying two hours from a
three-hour offer leaves no purchased hour to expire unused. Rejected offers retain
an observable reason. Altered acceptance receipts are rejected atomically.

## Batch 3 — two integration changes

1. **Recovery with town trading.** Direct loans and forward proceedings can coexist
   with the monthly town book. Ordinary spot trading is disabled while a proceeding
   is active. Orders recheck this after Due, even if the agent was admitted at Open.
   Native deliveries still settle. Custody agents cannot be traders or employees.
   The shared Acquire budget still resolves recovery asset sales/credit before
   forward settlement and spot exchange.
2. **Hiring within the household finance loop.** The work preview includes actual
   acquired-stock pooling, collective material allocation and consented support.
   A direct prepayment received in month one cannot fund that same Acquire's hiring;
   it funds two useful hours in month two, production pools the output, and month
   three delivers the promised goods. Separate worker/member/household/buyer books
   reconcile through the existing journal. A town-income case repeatedly hires two
   hours for three months after forecasting payroll and next-book sales. Household
   cash rises from four to seven, worker cash rises by six and buyer cash falls by
   nine. Raising wages above projected proceeds rejects hiring.

These are bounded acceptance and integration results. Worker offers, price limits,
forward terms and recovery authorization are supplied. There is no general labor
matching, ZIP wage negotiation, autonomous underwriting, wage insolvency or automatic
death estate. The income test spends a finite buyer endowment; it does not establish
an indefinitely self-sustaining economy. Hiring previews consider current earned
payroll and disable further household hiring in their hypothetical continuation.
The broader long-term roadmap remains in [GOALS.md](GOALS.md).

## Verification

Focused tests cover real output and cash, double-entry positions, partial claims,
receipt rejection, table-order controls, replay and checkpoint continuation, on both
the reference implementation and CubeCL CPU backend. Generated logs stay under
ignored `output/economics/fibonacci-*.log`; only source, tests and Markdown are tracked.

Final regression selection: **355 tests passed across 29 suites**, with
1 pre-existing slow annual accounting test ignored. All eight newly added tests
passed, as did the existing ten-year household income and full-horizon observer
checks. The full crate suite was not run. Formatting, strict all-target Clippy,
whitespace checks and the repository artifact-policy check passed.


## Continued batch 3: wages and recovery

1. Admit earned wages, keep native claims, stop new employer delivery during a
   proceeding, and stay coin collection to prevent bypassing the estate window.
2. Share actual estate cash with other creditors under explicit ranks; update the
   existing employment book and separate statements, including household pooling.
3. Exercise funded liquidation, household wage receipt, physical-denomination
   limits, forged records, replay and observer evidence together.

See [wage recovery](WAGE-RECOVERY.md). These three changes close the bounded wage
admission/payment gap, not the entire long-term financial stress-test roadmap.
The next batch size is 5; explicit non-loan disposition is the next consolidation
work. Fibonacci has no exhaustion point. Person self-directed policy changes remain
excluded, and static constitutions/charters stay static.


## Continued batch 5: explicit non-loan disposition

1. Shared dated wage write-off terms with consent/history and loss accounting.
2. Land-bill adapter; all funding and collection readers subtract accepted relief.
3. Wage date extensions with effective-date collection and closure validation.
4. Land extensions preserving original annual dates and later annual bills.
5. Combined household assistance, wages, land and forward recovery controls.

The combined case retains three coins of actual wage payment and zero grain
payments/deliveries. Accepted relief closes the estate in month 18, recognizing
seven coins of total claim losses; without that consent all three claims remain.
CPU/reference, reordered terms, replay and checkpoint continuation agree. This
completes this batch, not the proposed ten-stage financial stress-test program.


## Continued batch 8: contingent guarantees

1. Identify covered obligations through a shared typed selector.
2. Cover earned wage claims with actual payment and same-book recourse.
3. Cover dated land bills without renewing rights or changing annual billing.
4. Configure stable/proportional allocation against remaining opening funds.
5. Pool member wage receipts once, preserving household recourse as material debt.
6. Integrate accepted extensions/write-offs with call amounts, delays and expiry.
7. Date every recourse advance; prevent same-boundary collection/closure of additions.
8. Combine household guarantee funding, a loan, wages and land dues with statements,
   observer filters, CPU/reference, checkpoint and forged-record controls.

See [guaranteed claims](GUARANTEED-CLAIMS.md). The preceding complete crate run
passed **801 tests**, with **one existing ignored test**, across 99 Cargo test
outputs (including empty binary/doc targets). It covers the batch-5 snapshot;
the subsequently added physical-wage relief test passed separately. Batch-8
validation is recorded in its own document rather than attributed to that older run.


## Batch 13 — physical issuance in the shared contract economy

1. Expose mint packages to the shared resource window.
2. Compose direct lending with later mint purchases and financial statements.
3. Compose direct forward admission and delivery with the same opening budget.
4. Pass remaining budgets through generated fixed/provisioning order matching.
5. Enforce authorized recovery and custody restrictions on mint counterparties.
6. Collect annual coin/native land dues without fictitious issuance.
7. Verify employment and mint purchases compete for actual current hours.
8. Base household labor entitlement on the common monthly own-capacity endowment.
9. Reserve member labor and pool actual mint-market receipts once.
10. Verify shared storage before accepting household-member purchases.
11. Align public order previews with household reservation rules.
12. Compare provision policies in a continuing person/household/land/loan/forward
    economy, including actual arrears and reconciled statements.
13. Document supported compositions, regressions, economic outcomes and limits.

See [mint finance](MINT-FINANCE.md). The composed six-month scenario repays its loan
and delivers its forward under both policies. Full-buffer planning leaves the
annual two-coin bill unpaid and performs no issuance; incremental planning pays
that bill and completes physical issuance. This leaves the default unchanged.

Validation: 153 tests passed across 14 suites, with one existing ignored test.
Strict all-target Clippy, formatting, whitespace and repository artifact checks
passed. Raw logs are ignored local artifacts.


## Batch 21 — allocation, native finance and household composition

1. Extract shared ranked native/alternative-tender allocation.
2. Apply proportional policy to standalone dated land claims.
3. Share land grant execution and observable collection receipts.
4. Add opt-in concurrent prepaid admission with bounded funds/storage.
5. Allocate mature forward deliveries against one seller stock budget.
6. Record and validate direct-forward attempts, including zero payment.
7. Respect whole useful grants for indivisible single-resource claims.
8. Compose posted household hiring with scarce physical-mint inputs.
9. Share explicit native-claim reporting valuation.
10. Account for commodity advances, accrual and repayments.
11. Reconcile physical loan guarantees and native recourse.
12. Admit and account for physical wage guarantees.
13. Reserve mandatory household pooling space alongside other guarantee calls.
14. Settle physical land guarantees without fictitious debtor inventory.
15. Admit direct-forward guarantees after the first ordinary delivery window.
16. Preserve historical prepaid basis alongside native recourse valuation.
17. Verify overlapping deliveries and cumulative cost rounding.
18. Compose extension/expiry with recovery, keeping native recourse outside coin custody.
19. Identify native units in guarantee telemetry.
20. Update supported combinations and remaining roadmap boundaries.
21. Run the complete crate and required repository checks.

See [collection adapters](COLLECTION-ADAPTERS.md) and
[commodity finance](COMMODITY-FINANCE.md). This batch extends the active
consolidation roadmap; it does not complete the proposed ten-stage stress-test
program. Person self-directed policy changes remain deferred.


Validation: the complete batch-21 baseline passed **846 tests**, with **zero failures**
and **one existing ignored test**, across 103 Cargo test outputs (including empty
binary/doc targets). Later capacity, plot-credit and posted-guarantee refinements
passed their affected suites separately; those overlapping counts are not added to
the baseline total. Strict all-target Clippy, formatting, whitespace and repository
artifact checks passed before their commits.

## Batch 34 — acceptance and continuing integration

Completed additions so far:

1. Preserve the observed own-capacity endowment in household forecasts.
2. Compose direct lending with additional-plot review and its dated projection.
3. Distinguish posted guarantee terms from accepted contingent commitments.
4. Apply state form recognition and guarantor permission at admission.
5. Expose guarantee discovery and dated preparation through the common offer interface.
6. Integrate accepted exposure with household wind-down and normal performance.
7. Observe admission outcomes and verify timing, replay and checkpoint boundaries.
8. Expose consented advances and prepaid deliveries through common discovery.
9. Prepare financial bundles through the normal household/settlement boundary.
10. Verify shared funding, failed bundles, continued service and wind-down consent.
11. Inventory existing financial receivables before deficient estate closure.
12. Preserve the custody delay for cash collected during the closing boundary.
13. Compose receivable collection with household wind-down and separate accounts.
14. Resolve linked estates through actual counterparty discharge and dated visibility.
15. Keep earned wage assets and later post-closure work distinct.
16. Explain deferred assets through receipts, counterparty filters and checkpoint checks.
17. Expose posted household hiring through common offer discovery and preparation.
18. Compose hiring requests with prepaid funding while preserving policy and timing.
19. Expose active unsold estate listings with denomination and reserve terms.
20. Prepare liquidation bids through common acceptance and actual funded priority.
21. Preserve competing bid identities and reject duplicate-sale interpretations.
22. Apply household lifecycle eligibility to optional estate purchases.
23. Compose direct prepayments and legacy tool/state exchange in one reservation window.
24. Collect both forward origins once with explicit stable/proportional allocation.
25. Preserve forward identity and future storage across both admission adapters.
26. Exercise household prepaid buying, stock targets and common inventory accounting.
27. Compose mortgage and direct prepaid admissions with negotiated acquisition.
28. Keep crop-control transfer separate from personal forward delivery and accounting.
29. Admit household financed purchases through the normal wrapper and common offers.
30. Recheck household lifecycle and active recovery before optional mortgage acceptance.
31. Date mortgage catalog encumbrances independently of accepted liens.
32. Compose household repayment, solvent property disposal and residual distribution.
33. Reconcile current README/integration coverage with the supported adapter matrix.
34. Complete the full crate regression gate for these integrations.

See [acquisition adapters](ACQUISITION-ADAPTERS.md) and
[guarantee admission](GUARANTEE-ADMISSION.md). The full run passed **876 tests,
zero failures, one ignored**, across 105 Cargo target results (including empty
targets). It was launched after `445da09`; subsequent equipment/mortgage/mint
changes were developed while it ran and have separate focused regression gates.
Do not treat this as a full run of the eventual batch-55 source tree. Strict
all-target Clippy, formatting, whitespace and artifact checks passed for each
subsequent implementation chunk.

## Batch 55 — wider asset and execution composition

Completed integrations:

1. Admit portable equipment to funded estate listing discovery and settlement.
2. Carry actual acquisition cost and existing condition into buyer depreciation.
3. Release sold estate restrictions for household equipment retirement and dissolution.
4. Recheck durable eligibility and preserve atomic title/condition/payment evidence.
5. Compose financed ownership with independent accepted land leases.
6. Exercise mortgage/lease collection through household budgets and alternative tender.
7. Compose state financed purchases with physical mint input procurement and issuance.
8. Compose direct commodity lending with recurring environmental pool collection.
9. Reconcile collection, native repayment and separate inventory/financial statements.
10. Expose dated repayment demand through read-only accepted-loan projections.
11. Generate collection requests for repayments even when consumption is buffered.
12. Compose household governance and resource pooling with environmental allocation.
13. Compare need-first and output-value labor direction in the same constrained household.
14. Compose direct prepaid admissions and delivery with environmental collection.
15. Plan collection for accepted performance claims alongside loan installments and needs.
16. Unify current-debt protection and projected installments with dated guarantee recourse.
17. Preserve native collection and repayment during authorized coin insolvency.
18. Keep household assets and membership distinct from a member’s environmental-work estate.
19. Admit posted household hiring against useful work and actual shared environmental stock.
20. Compose negotiated buying/selling with authorized stays and post-closure eligibility.
21. Reserve household contributions and storage for negotiated member purchases.
22. Respect collective/member purchasing while keeping concurrent loan proceeds unpooled.
23. Exchange collected output through financed need orders and the same monthly budgets.
24. Compose person/household mortgages and repossession with independent environmental work.
25. Verify individual/household native guarantees and dated recourse against real collected stocks.
26. Reconcile the main integration matrix and recovery documentation with these combinations.
27. Fund useful household hiring from negotiated revenue at the next eligible boundary.
28. Include eligible collection techniques without multiplying exclusive tool capacity.
29. Share accepted credit-record application between forecasts and committed settlement.
30. Compose estate tool purchase, household hiring and actual environmental production.
31. Admit compatible shared liens under an explicit authorized-liquidation agreement.
32. Allocate each asset’s actual proceeds by collateral rank and equal-rank policy.
33. Preserve separate asset reservations, arrears authority and residual deficiencies.
34. Carry household/member secured claims through recovery and permitted dissolution.
35. Route opted-in mortgages through authorized estate custody and funded liquidation.
36. Transfer unfinished crop obligations through mortgage estate sales and reconcile outcomes.
37. Include funded direct advances in prerequisite search and explicit productive bundles.
38. Preserve dated work, atomic rejection and checkpoint continuation across credit/search acceptance.
39. Allocate competing land bundles against the same funded lending boundary.
40. Preserve independently consented credit after allocation losses and empty rounds.
41. Share the financial acquisition base across credit, prepaid deliveries and prerequisite search.
42. Carry prepaid consent through competitive and empty land allocation rounds.
43. Compare feasible delivery and retained shortfalls through production and double-entry statements.
44. Accept named credit/prepaid terms and ordered productive prerequisites through one common bundle.
45. Liquidate explicit inventory lots through shared funding, storage, exemptions and estate custody.
46. Reconcile household inventory purchases and separate buyer/debtor/custodian statements.
47. Discover and accept dated inventory bids through the common financial offer interface.
48. Preserve receiving-space reservations from estate sales into subsequent commodity advances.
49. Pool member inventory purchases with fractional carry and reserve collective space before later lending.
50. Carry inventory contribution reservations into negotiated market matching without resetting fractional carry.
51. Carry household inventory liquidation through retained debt, explicit discharge and permitted dissolution.
52. Carry estate contribution reservations into need-generated town order settlement.
53. Pool actual direct-forward delivery with prior inventory purchases and later market reservations.
54. Pool guarantor delivery under the same receiving-space limits and preserve native recourse.
55. Preserve fractional prepaid-cost release without publishing zero-valued accounting lines.

[Equipment liquidation](EQUIPMENT-LIQUIDATION.md) and
[acquisition adapters](ACQUISITION-ADAPTERS.md) record the tests and boundaries.
Focused gates passed 74 equipment/recovery tests, 53 mortgage/lease tests, and 57
mint/finance tests; these selections overlap and are not distinct-test totals.
The broader consolidation roadmap remains active. Fibonacci batch sizes do not
supply a finite stopping point.

An interim full run completed with **898 passed, zero failures, one ignored**
across 110 Cargo target results. It started from `3eb0a20`; later targeted builds
ran while it was executing, so this is mixed-tree regression evidence, not a
full gate for the final batch-55 revision. Subsequent tool-aware collection and
acquired-state changes have their own focused gates.

Inventory liquidation: [scope and evidence](INVENTORY-LIQUIDATION.md). Its focused
seven-target gate passed 63 tests; all five final inventory tests and strict
all-target Clippy passed. The final target covers the
explicit current-essential exemption and inventory cost/revenue assertions too.

The sale/advance storage regression and affected lending/credit/offer targets passed
37 tests; strict all-target Clippy passed. Newly purchased goods still cannot fund
same-boundary lending, and unfundable requests leave accepted purchases intact.

Member inventory pooling passed 35 tests across five affected targets, followed by
all eight final inventory checks and strict all-target Clippy. The scoped driver
explicitly rejects member bids if later spot/forward matching is not adapted.

The fixed-revision full crate run at `a5974d2` passed **913 tests, zero failures,
one ignored**, across 112 Cargo target results. Newer inventory development used a
separate target directory, preserving that run's compiled executables. Its results
do not cover subsequent inventory changes, which have their own focused gates.
Inventory/negotiation composition passed 22 tests across five targets and strict
all-target Clippy, including CPU/checkpoint/accounting comparisons with full and
available household storage.

Household inventory wind-down and affected estate/disposal targets passed 35 tests
with strict all-target Clippy. Unfunded lots retain the estate, residual debt blocks
dissolution, and only cleared claims allow remaining household goods to reach the
member. Private member cash remains separate throughout.

Town-order integration passed 41 tests across four affected targets and strict
all-target Clippy. The same need submits an order in both storage controls; only
the funded, storable outcome trades, retaining distinct submission/settlement evidence.

Direct-delivery pooling passed two affected selections (36 and 60 tests) plus
strict all-target Clippy. Prepayments remain unpooled; receiving-space shortfalls
retain seller stock and the original outstanding claim.

Final guarantee/pooling selection: **85 tests passed across five targets**, with
strict all-target Clippy. The full crate gate at completed batch-55 revision `3509dd5` subsequently passed
**925 tests, zero failures, one ignored**, across 113 Cargo target results.
That exact revision was compiled and run in an isolated target directory; later
batch-89 edits did not replace its executables. The preceding 913-test full run
remains explicitly tied to `a5974d2`.

## Next batch 89 — further shared-contract composition (in progress)

Continue the active consolidation roadmap: extend remaining acquisition adapters,
creditor allocation, custody, guarantees and real liquidation. Household/member
separation, static founding terms and deferred person self-policy changes remain
in force. A Fibonacci count does not expand the roadmap into speculative systems
or justify declaring the remaining work complete.

1. **Inventory recovery in the physical-mint economy.** Member estate purchases
   now retain their fractional household contribution through later mint-market
   trades. A funded state mints coins from purchased metal and labor; after a
   borrower defaults, an estate wheat purchase and a state wheat sale compete for
   the same shared storage. Full storage rejects only the later trade. Actual
   issuance, custody, unchanged debt until the next Due boundary, CPU/reference
   results, checkpoint continuation and separate books are checked together.

Batch 89 first integration gate: 40 tests across mint finance, inventory
liquidation and household forwards passed; strict all-target Clippy passed.

2. **Legacy stock-market reservation adapter.** Multiple posted lots and earlier
   estate purchases now share exact contribution carry. Available, partly full
   and full household storage controls admit three, two and zero later lots,
   preserving funds, inventory and separate books on CPU and checkpoint replay.
   Recovery no longer excludes unrelated posted stock sellers. Ordinary trading
   by the active debtor stays blocked, and custody remains non-operating. The
   affected six-target gate passed 93 tests; strict all-target Clippy passed.

3. **Explicit guarantee lien subrogation.** Accepted guarantee terms can carry
   authorized-liquidation security into the existing recourse loan. Before sale,
   the active lien transfers even on full original repayment. After sale, its
   actual custody reservation transfers without duplication. New recourse stays
   uncollectible until a later Due, including secured distributions; reserved
   cash remains protected meanwhile. Partial/full guarantees, both allocation
   policies, CPU/checkpoint parity and separate statements pass. The seven-target
   affected gate passed 94 tests; strict all-target Clippy passed.

4. **Household guarantor recovery and wind-down.** An organization guarantees
   a person's secured loan from its own cash; member coins remain private. Its
   recourse receives actual collateral proceeds. Retained deficiency blocks the
   household's exit; explicit discharge clears the receivable, then normal Open
   distribution and dissolution release remaining household cash. CPU/reference,
   checkpoint and separate books agree. The three-target gate passed 59 tests
   and strict all-target Clippy.

5. **Funded receivable assignment through common acceptance.** An authorized
   estate can sell an entire unsecured coin loan at remaining principal plus
   accrued interest. The existing loan changes creditor; borrower terms and
   future collection timing remain intact. Actual price enters custody, with
   separate investor/estate statements and no invented sale gain. Person and
   winding-household cases compare funded, unfunded and wrong-price bids,
   tampered/duplicate applications, subsequent collection and discharge. Claim
   and inventory purchases also share one opening budget. The five-target gate
   passed 77 tests; strict all-target Clippy passed.

6. **Secured claim assignment across organizational estates.** A household can
   assign a direct coin loan with authorized-liquidation security. The original
   lien identity and reserved proceeds follow the existing loan, both before and
   after its debtor's property sale. Two estates, private member cash, investor
   funding, retained/discharged deficiency and household dissolution now run
   together. Offer inspection exposes the complete loan terms and security.
   The four-target gate passed 50 tests; strict all-target Clippy passed.

7. **Guarantee benefits follow explicitly consented claim assignment.** Whole-loan
   buyers can receive an attached guarantee when its terms permit transfer.
   Calls, common inspection and observers use the new creditor; cap, term and
   recourse debtor remain unchanged. Person and household estates exercise later
   installment calls, separate statements and CPU/checkpoint replay. Unconsented
   transfers and guarantor self-purchases are rejected. The seven-target affected
   gate passed 103 tests; strict all-target Clippy passed.

8. **Assigned household loans retain guarantee and collateral consequences.**
   Two estates now compose claim sale, a third-party guarantee, inherited liens,
   custody distributions and household dissolution. Before/after-sale timing,
   funded/unfunded investors and retained/discharged deficiencies preserve
   actual proceeds, original debtors and private member funds. CPU and checkpoint
   replay agree with separate statements throughout.
   The four-target gate passed 84 tests; strict all-target Clippy passed.

9. **Accepted land-coin guarantee tender.** Explicit terms select the existing
   land coin rate. Shared stable/proportional execution reserves actual cash in
   whole conversion lots; original claim units remain authoritative for caps,
   settlement and recourse. Reporting distinguishes cash outlay from native claim
   value without inventing goods or debtor income. Controls cover competing
   rates, unaffordable lots, forged receipts and CPU/checkpoint/catalog parity.
   The eight-target gate passed 124 tests; strict all-target Clippy passed.

10. **Household guarantees mix native and alternative claims.** One collective
    cash pool now covers a loan, member wages and native land dues paid in coins.
    The member's actual wage pooling arrives after allocation and is not recycled
    into another call. Native land recourse, creditor cash, valuation differences
    and filtered tender telemetry reconcile in separate statements. The four-target
    gate passed 55 tests; strict all-target Clippy passed.

11. **Authorized recovery shares bounded farming search.** Existing cultivation
    and annual native rent continue under an explicit coin-estate stay, while
    new land acceptance rejects. The same forecast and dated production plan
    carries the authoritative financial boundary. Seed credit, a serviced coin
    loan with a real residual deficiency, later harvest/rent and retained versus
    discharged debt run together with separate statements and CPU/checkpoint
    parity. The six-target gate passed 70 tests; strict all-target Clippy passed.

12. **Posted guarantees join common productive acceptance.** Explicit bundles
    can name seed credit, a posted guarantee, citizenship, land and planting.
    Funding and legal controls reject the entire requested bundle. A later labor
    interruption fails the crop, calls accepted coverage and leaves native seed
    recourse against the original farmer, even after formation permission is
    withdrawn. Tampered admission and CPU/checkpoint controls pass. The five-target
    gate passed 39 tests; strict all-target Clippy passed.

13. **Estate seed purchases join cultivation acceptance.** Common requests can
    combine a funded inventory bid with citizenship, land and planting. The
    purchased seed supports dated work; the seller's actual proceeds stay in
    custody until later recovery. Unfunded and tampered requests publish nothing.
    Continuing harvest, CPU/reference and reconstructed checkpoints agree with
    separate statements. The five-target gate passed 26 tests; strict all-target
    Clippy passed.

14. **Financed property joins explicit productive acceptance.** The common
    request can fund a mortgage and reserve planting after ownership transfers,
    with atomic rejection for absent downpayment or seed. Later repossession
    transfers crop control; creditor labor determines completion versus failure
    without changing the fixed-value deficiency. Separate books, CPU/reference
    and reconstructed checkpoints agree. The eight-target gate passed 36 tests;
    strict all-target Clippy passed. Specialized mortgage stock-sale planning
    and household prerequisite search retain their separate guards.

15. **Property and claim bids join productive acceptance.** A funded estate-land
    bid can reserve cultivation under its acquired right. A separate combined
    request purchases seed and a whole loan claim; only actual seed supports
    planting, while borrower installments later fund the investor. Custody sale
    proceeds wait for their existing recovery window. Unfunded requests,
    CPU/reference and reconstructed checkpoints preserve separate books. The
    six-target gate passed 37 tests; strict all-target Clippy passed.

16. **Shared custody preserves separate estate budgets.** Multiple authorized
    estates can name one non-operating custodian. Per-case opening funds bound
    distributions; new deposits cannot borrow another estate's liquidity. Loan,
    rent, inventory-sale, wage and surplus flows reconcile to separate beneficial
    owners, including a winding household and private member property. Tampering,
    catalog order and CPU/checkpoint controls pass. The seven-target gate passed
    85 tests; strict all-target Clippy passed.

### Full-suite checkpoint during batch 89

The isolated source snapshot at **`2b9d40a`** (through item 10) passed
`cargo +1.92.0 test --locked`: **938 passed, 0 failed, 1 ignored**, across
113 Cargo result targets including empty unit/doc targets. Later items have
their separately recorded affected gates; this full result does not cover them.

17. **Shared custody composes with household claim sales and inherited liens.**
    A winding household and its debtor use one custodian while a whole secured
    receivable changes holder. Optional guarantees transfer liens or reserved
    proceeds to recourse without crossing beneficial balances. Before/after-sale,
    funded/unfunded and retained/discharged controls preserve investor and
    guarantor recoveries, private member money and household exit. CPU/checkpoint
    and separate books agree. The four-target gate passed 54 tests; strict
    all-target Clippy passed.

18. **Native loan claims can be sold for custody coins.** An explicit fixed
    unit quote prices an entire unsecured commodity receivable. Assignment
    changes the creditor and transfers real coins without converting the debt.
    Later goods still require receiving storage. Person/household sellers,
    unfunded bids, stale prices, absent storage, valuation mismatch and altered
    denomination controls reconcile with CPU/checkpoint and separate books.
    The five-target gate passed 20 tests; strict all-target Clippy passed.
    Discounted acquisition basis and native collateral remain outside this adapter.

19. **Assigned native guarantees use resources earned through employment.**
    A worker receives seed wages, then covers a sold seed loan under explicit
    transferable coverage. The current holder receives the goods; the original
    borrower owes native recourse. Person/household, funded/unfunded and storage
    controls preserve actual wages, custody, wind-down and separate statements.
    CPU/checkpoint continuation agrees. The four-target gate passed 42 tests;
    strict all-target Clippy passed.

20. **Mortgage stock sales respect authorized recovery.** Fixed-reserve grain
    sales now pause for an active seller or buyer proceeding, with an explicit
    stayed-party receipt and log field. Funded property liquidation clears the
    four-unit deficiency and restores ordinary sales; unfunded liquidation leaves
    both debt and stay. Custodians cannot trade. Continuing crop control, separate
    books, forged receipts and CPU/checkpoint controls pass. The four-target gate
    passed 56 tests, followed by the expanded five-test mortgage-recovery suite;
    strict all-target Clippy passed. Forecast/joint sale planning retains its guard.

21. **Member mortgage sales compose with household income pooling.** Fixed-reserve
    sales now enter the existing household receipt path. Half the actual income
    belongs to the collective and stays outside the member's loan/estate budget;
    the resulting deficiency is five rather than four. Funded liquidation cures
    it, while unfunded recovery retains it. Separate books, crop continuation,
    forged receipts and CPU/checkpoint checks pass. The three-target gate passed
    18 tests; strict all-target Clippy passed. Collective specialized sale planning
    and forecast/joint household policies retain explicit limits.

22. **Bounded sale forecasts project authorized recovery.** The sale planner
    now uses the ordinary recovery stay, actual property bids and later closure
    while evaluating nutrition. An active stay leaves only the zero-sale
    candidate; a feasible food forecast neither erases debt nor grants exchange
    permission. Funded/unfunded continuation, tampering, separate books and
    CPU/checkpoint controls pass. The four-target gate passed 21 tests;
    strict all-target Clippy passed. Joint work/sale planning remains guarded.

23. **Member sale forecasts retain collective and private boundaries.** The
    bounded forecast now composes with household contribution carry, private food
    consumption, member mortgage debt and an authorized estate. Funded and
    unfunded property bids retain distinct closures; household cash never becomes
    assumed private financing. Separate statements and CPU/reconstructed
    continuation agree. The four-target gate passed 24 tests; strict all-target
    Clippy passed. Joint household work/sale reservations remain guarded.

24. **Joint work/sale plans retain recovery timing.** The single-participant
    planner now projects authorized recovery, continues an existing crop during
    the ordinary-trade stay, and commits a dated Productive plan. Funded sale
    after harvest closes the estate; an unfunded bid leaves the deficiency.
    Altered plan dates fail atomically. Food, custody, separate books and
    CPU/reconstructed continuation agree. The four-target gate passed 22 tests;
    strict all-target Clippy passed. Household joint work allocation stays guarded.

25. **Household mortgage receivables share loan assignment.** One configured-loan
    inspection adapter now serves direct advances and financed purchases in
    listings, guarantees, lien subrogation and reporting. A winding household
    sells its six-coin mortgage; four actual property-sale coins pay its investor
    while two remain due. An unfunded bid retains the household asset and blocks
    exit. Private member property/crop control, separate estates and CPU/checkpoint
    books agree. The six-target gate passed 57 tests; strict all-target Clippy
    passed. Fixed-value/resale security remains excluded.

26. **Assigned mortgages retain guarantees across shared custody.** Transferable
    coverage and inherited liens now run with a winding household's mortgage
    sale, another estate's property sale and dedicated/shared custodians. Calls
    before and after collateral sale retain their distinct reserved-proceeds
    recoveries. New recourse never collects in its creation month. Unfunded bids,
    household exit, separate books and CPU/checkpoint controls pass. The
    four-target gate passed 50 tests; strict all-target Clippy passed.

### Full-suite checkpoint through item 18

The isolated source snapshot at **`37a6208`** passed
`cargo +1.92.0 test --locked`: **947 passed, 0 failed, 1 ignored**, across
118 Cargo result targets including empty unit/doc targets. Items 19 onward have
the separately recorded affected gates above; this full result does not cover them.

27. **Specialized stock sales retain pooled storage reservations.** A regression
    reproduced a failed monthly commit after an estate purchase and later stock
    lots overfilled collective storage. The existing contribution budget now
    flows through financing and specialized sales. Bounded forecasts use the same
    feasible lot ceiling; receipts/logs distinguish it from raw room. Identical
    openings allow three, two or zero lots with reconciled household/custody books
    and CPU/checkpoint equality. The five-target gate passed 39 tests; strict
    all-target Clippy passed.

28. **Earlier native lending frees physical space for later stock purchases.**
    Specialized sale quoting and final trade validation now observe preceding
    accepted transfers. A full household can lend grain and then receive a funded
    stock lot; without the outgoing transfer it cannot. Opening spending limits
    remain separate: a control cannot resell newly borrowed grain in the same
    boundary. Reference/CPU and reconstructed continuation preserve contribution
    carry, native debt and separate books. The seven-target gate passed 69 tests;
    strict all-target Clippy passed.

29. **Mortgage stock income composes with independent rent.** Fixed-reserve and
    bounded-forecast sales now run alongside a separate land lease. Real sale
    income funds later mortgage/rent collection; stable and proportional policies
    retain distinct scarce-cash outcomes. A household member pools actual income,
    and the existing rent-support rule records any later collective contribution.
    Native rent preserves its denomination and leaves pooled coin untouched.
    Separate books, altered receipts and CPU/reconstructed continuation agree.
    The five-target gate passed 29 tests; strict all-target Clippy passed.
    Joint work/sale planning with independent leases remains guarded.

30. **Specialized stock sales share acquisition with prepaid deliveries.**
    Fixed-reserve and bounded-forecast sales now compose with direct prepayments,
    a mortgage, independent rent and household pooling. Earlier sales reserve
    buyer funds; an incoming downpayment cannot fund the remaining prepayment.
    Accepted deliveries settle later and retain separate financial positions.
    A second regression reproduced collective-storage overflow: stock receipts
    now carry their exact fractional share into later forward collection. Full
    space leaves the delivery owed, rather than rejecting the whole month.
    CPU/checkpoint and separate books agree. The seven-target gate passed 64
    tests; strict all-target Clippy passed. Joint work/sale prepayments stay guarded.

31. **Repeated farming exercises the combined person/household financial loop.**
    A 24-month scenario combines seed-returning harvests, nutrition, a mortgage,
    independent native rent, two prepaid deliveries and finite posted stock sales.
    Bounded sale forecasts meet food needs and repay/perform the claims with and
    without household pooling. Private loan payments do not assume household
    balances. Removing sale funding leads to repossession; a fixed food-buffer
    control performs the financial claims but misses a meal. Seed conservation,
    separate books and CPU/reconstructed continuation agree. The four-target gate
    passed 21 tests; strict all-target Clippy passed. This uses calibrated yields
    and supplied terms, not autonomous or indefinitely sustainable underwriting.

### Full-suite checkpoint through item 27

The isolated source snapshot at **`aea53e8`** passed
`cargo +1.92.0 test --locked`: **958 passed, 0 failed, 1 ignored**, across
119 Cargo result targets including empty unit/doc targets. Items 28–31 have the
separately recorded affected gates above and are not covered by that snapshot.

32. **Joint work/sale plans observe independent leases and forward performance.**
    The single-participant planner now prepares its dated Productive batch after
    shared prepayment/collection. A normal grain-forward continuation is feasible;
    delivering the only seed makes the continuation infeasible and creates no
    planting transaction. Accepted plans execute once, altered dates fail
    atomically, and separate books plus CPU/checkpoint continuation agree. The
    five-target gate passed 30 tests; strict all-target Clippy passed. Household
    and negotiated joint planning retain their separate compatibility guards.

33. **Household and marketplace classifications no longer collide.** Both had
    built-in type ID 3, leaking household borrowing permission to marketplace
    agents. Shared built-in IDs now keep household 3 and assign marketplace 4;
    the public marketplace constant re-exports that definition. A formed
    household, two venues and real funded traders demonstrate separate grants
    and rejection at a household-only venue on reference and CPU execution.
    Custom numeric marketplace definitions require the documented migration.
    The five-target gate passed 52 tests; strict all-target Clippy passed.

34. **Negotiated seed trades feed dated joint production plans.** Configured
    concession and ZIP sessions can buy the missing seed before planting. A
    mortgage-downpayment-only control cannot spend those coins again, submits no
    executable seed purchase and starts no crop. Both executions preserve exact
    prepared work, reconstructed continuation and atomic receipt rejection.
    The six-target gate passed 39 tests; strict all-target Clippy passed.
    Producer-input order generation and household joint allocation remain separate.

35. **Collateral resale composes with current laws.** Both counterparties need
    asset-trade permission. Denial records a legal decision while preserving
    accepted enforcement, pending custody, crop, debt and cash. Later permission
    restoration admits a later sale. Granted/buyer-denied/seller-denied controls
    agree across CPU and reconstructed continuation; forged denial receipts fail
    atomically. The four-target gate passed 35 tests and strict Clippy passed.

### Full-suite checkpoint through item 31

The isolated source snapshot at **`0325e4f`** passed
`cargo +1.92.0 test --locked`: **964 passed, 0 failed, 1 ignored**, across
120 Cargo result targets including empty unit/doc targets. Later items have
separately recorded affected gates and are not covered by that snapshot.

36. **Single-member household joint work retains the complete allocation boundary.**
    Candidate work carries governed labor, shared-input reservations and output
    collection alongside its core transactions. Execution verifies and consumes
    that full dated plan once. A six-month mortgage/farming control reserves the
    household's only seed and pools the harvest with separate statements;
    missing seed prevents planting. The control uses farming as its sole
    productive option and a twelve-month need horizon. CPU and reconstructed
    continuation agree; altered envelopes and dates fail atomically. The
    eight-target gate passed 124 tests with one ignored; the additional final
    checkpoint-validation regression and strict all-target Clippy passed. The
    one-participant joint bound remains; multi-person forecasts are outstanding.
