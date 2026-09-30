# Fibonacci integration follow-up

Scope: finish bounded autonomous household hiring and direct prepaid-delivery
recovery, then demonstrate that they compose with the existing economic loop.
The batches contain logical changes, not a prescribed count of files or tests.
The completed sequence is **1, 1, 2, 3, 5, 8, 13** (batch 21 in final verification). Batch sizes count distinct
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

## Next batch 34 — acceptance and continuing integration (in progress)

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

See [acquisition adapters](ACQUISITION-ADAPTERS.md) and
[guarantee admission](GUARANTEE-ADMISSION.md). This is an unfinished batch; no
claim is made that the broader consolidation or financial stress-test roadmap is
complete. Fibonacci batch sizes do not supply a finite stopping point.
