# Fibonacci integration follow-up

Scope: finish bounded autonomous household hiring and direct prepaid-delivery
recovery, then demonstrate that they compose with the existing economic loop.
The batches contain logical changes, not a prescribed count of files or tests.
The completed sequence is **1, 1, 2**. No extra changes were added to fill a larger
batch. Person self-directed policy changes remain explicitly deferred.

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
